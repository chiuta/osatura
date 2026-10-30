<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->

# Formatul de proiect OSATURA

Aceasta este referința pentru fișierul de proiect OSATURA: un singur document JSON care
conține scena, rig-ul, desenul și animația. Este scrisă pornind de la structurile
serde din `osatura-core/src/project.rs` și de la modulele la care acestea fac referire,
iar fiecare exemplu din secțiunile 13-15 a fost verificat cu binarul `osatura`
(`osatura check`, `osatura render`, `osatura explain`).

Cuprins

1. [Convenții și tipuri primitive](#1-convenții-și-tipuri-primitive)
2. [Încărcare, versionare și câmpuri necunoscute](#2-încărcare-versionare-și-câmpuri-necunoscute)
3. [Câmpuri de nivel superior](#3-câmpuri-de-nivel-superior)
4. [Rig: bones, ik, constraints, springs](#4-rig-bones-ik-constraints-springs)
5. [Straturi](#5-straturi)
6. [Căi, ancore, paint, stroke](#6-căi-ancore-paint-stroke)
7. [Deformatoare, efecte, normal map](#7-deformatoare-efecte-normal-map)
8. [Canale, piste, easing](#8-canale-piste-easing)
9. [Expresii](#9-expresii)
10. [Cameră, shot-uri, audio, metadate](#10-cameră-shot-uri-audio-metadate)
11. [Clipuri, skin-uri, legături, interacțiune, lod-uri](#11-clipuri-skin-uri-legături-interacțiune-lod-uri)
12. [Reguli de validare, limite și mesaje de eroare](#12-reguli-de-validare-limite-și-mesaje-de-eroare)
13. [Exemplul 1: minge care sare](#13-exemplul-1-minge-care-sare)
14. [Exemplul 2: braț cu rig și IK](#14-exemplul-2-braț-cu-rig-și-ik)
15. [Exemplul 3: gură cu strat switch](#15-exemplul-3-gură-cu-strat-switch)
16. [Ordinea de evaluare](#16-ordinea-de-evaluare)
17. [Ordinea de randare](#17-ordinea-de-randare)
18. [Resurse și căi](#18-asset-uri-și-căi)
19. [Politica de versionare](#19-politica-de-versionare)
20. [Neconcordanțe cunoscute între cod și comentarii](#20-lacune-cunoscute-între-cod-și-comentarii)

## 1. Convenții și tipuri primitive

| Tip | Formă JSON | Note |
| --- | --- | --- |
| number | număr JSON | Stocat ca `f32`, dacă în câmp nu se spune că este întreg. Orice float trebuie să respecte `abs(x) <= 1e7` (secțiunea 12). |
| `Vec2` | `[x, y]` | Două numere. Unitățile scenei sunt pixeli, +x spre dreapta, +y în jos. |
| `Rgba` | string | `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa` sau unul dintre `black`, `white`, `red`, `green` (0,128,0), `blue`, `yellow`, `transparent`. La ieșire este întotdeauna hex cu litere mici. |
| `Paint` | string sau obiect | Un șir de culoare este un paint plin. `{"from": [x,y], "to": [x,y], "stops": [...]}` este un gradient liniar. `{"center": [x,y], "radius": r, "stops": [...]}` este un gradient radial. Tipul este un enum fără etichetă (untagged). |
| `Stop` | `{"offset": 0..1, "color": Rgba}` | Stop-urile sunt sortate (stabil) la desenare, iar offseturile sunt limitate la 0..1 în rasterizator. Un gradient degenerat pictează ultimul stop. |
| angle | număr | Grade peste tot în fișier (unghiurile oaselor, rotația stratului, skew, rotația camerei, `angle_limits`). |
| time | număr | Cadre în chei și markere, secunde în sunete, mașini și emitere. Fiecare câmp precizează care. |
| names | string | Nume de os, strat, cale, deformator, ik, constrângere. Secțiunile 5 și 12 listează restricțiile de caractere. |

O culoare greșită într-un context de paint produce mesajul serde
`data did not match any variant of untagged enum Paint`, deoarece `Paint` nu are etichetă, iar serde
nu poate spune care ramură era vizată.

Tabelele de câmpuri de mai jos folosesc această notație: **Req** înseamnă că în câmp nu există valoare implicită și trebuie să fie prezent;
în caz contrar este afișată valoarea implicită. „Range" este ceea ce impune încărcătorul sau motorul. Un interval scris
„clamped" nu este o eroare: motorul limitează valoarea în tăcere.

## 2. Încărcare, versionare și câmpuri necunoscute

`Project::from_json(text)` este singurul punct de intrare folosit de CLI, de API-ul C, de modulul WASM
și de Studio. Acesta face, în ordine:

1. Parsează JSON-ul cu serde. Serde ignoră câmpurile pe care nu le cunoaște.
2. Rulează verificarea de magnitudine: orice literal float a cărui valoare absolută depășește `1e7` face încărcarea
   să eșueze cu `invalid JSON in <file>: value ... is outside the supported range of +-1e7 ...`. Literalii întregi
   sunt prinși mai târziu, de reverificarea din `Engine::new` (vezi secțiunea 12).
3. Aplică bariera de format: un `format` mai mare decât `FORMAT_VERSION` al build-ului (în prezent `1`) este
   respins cu `project format 2 is newer than this build supports (1); update OSATURA`.

`Engine::new` (sau `Engine::with_asset_dir`) validează apoi întregul proiect și returnează toate
problemele găsite sub formă de listă. CLI le afișează astfel:

```text
<file> is not a valid project:
  - first problem
  - second problem
```

Textele de eroare sunt destinate oamenilor. Nu sunt un contract stabil; potrivește textul doar în teste.

### Câmpuri necunoscute

Serde elimină cheile necunoscute la încărcare, deci o greșeală de tastare precum `fil` în loc de `fill` s-ar pierde în
tăcere. Modulul `unknownfields` compară JSON-ul de intrare cu proiectul re-serializat și raportează cheile care
nu au supraviețuit:

```text
warning: unknown field layers[0].paths[0].fil (ignored)
```

`osatura check` afișează aceste avertismente. Valorile egale cu cea implicită nu sunt raportate (scriitorul
omite valorile implicite, deci un `"visible": true` explicit este în regulă). Câteva structuri sunt stricte în loc de
permisive și resping cheile necunoscute cu o eroare gravă: `Transition` și `Condition` ale mașinilor, configurația CI, configurația
live-input, overlay-urile și configurația parametrilor feței; fișierele de configurare sunt citite cu `parse_strict`.

### Parcursul dus-întors al metadatelor editorului

Proiectul se serializează cu `skip_serializing_if` pentru listele opționale. Un fișier scris de mână poate
fi mai scurt decât ce scrie `osatura` înapoi; ambele sunt echivalente.

## 3. Câmpuri de nivel superior

| Nume JSON | Tip | Implicit | Interval / regulă | Semnificație |
| --- | --- | --- | --- | --- |
| `format` | integer | `FORMAT_VERSION` (1) | trebuie să fie `<=` versiunea build-ului | Versiunea formatului, secțiunea 19. |
| `name` | string | **Req** | oricare | Nume afișat. Tipărit de `check`. |
| `width` | number | **Req** | `> 0`, `<= 32768` | Lățimea scenei în pixeli. |
| `height` | number | **Req** | `> 0`, `<= 32768` | Înălțimea scenei în pixeli. |
| `fps` | number | **Req** | `(0, 1000]` | Cadre pe secundă. |
| `frames` | integer (`u32`) | **Req** | `1..=200000` | Lungimea timeline-ului. Indicii cadrelor sunt `0..frames-1`. Un literal float precum `48.0` este respins: `invalid type: floating point `48.0`, expected u32`. |
| `background` | `Rgba` sau null | null (transparent) | | Culoarea scenei din spatele tuturor. |
| `color_space` | string | `"srgb"` (nu se scrie) | `srgb`, `linear` | Spațiul în care compozitoarele raster și float amestecă. `linear`: culorile sunt convertite în lumină liniară, compuse în f32 și convertite înapoi o singură dată (un negru 50 % peste alb este 188/255, nu 128/255). Se setează cu `osatura color-space`. SVG, Lottie, randorul GPU, `dof` și `fx` îl ignoră. |
| `camera` | object | valori implicite | secțiunea 10 | Camera globală. |
| `bones` | array | `[]` | cel mult 4096 | Scheletul, secțiunea 4. |
| `ik` | array | `[]` | | Lanțuri IK. |
| `constraints` | array | `[]` | | Aim, copy-rotation, pin. |
| `springs` | array | `[]` | | Mișcare secundară, coaptă la încărcare. |
| `layers` | array | `[]` | | Straturi rădăcină, desenate în ordinea listei. |
| `channels` | array | `[]` | | Canale de animație, evaluate în ordinea listei. |
| `markers`, `comps`, `events`, `bone_vis`, `channel_meta`, `shots` | arrays | omise când sunt goale | nevalidate | Metadate de timeline și de editor, secțiunea 10. |
| `sounds` | array | omis când este gol | validat | Piste audio, secțiunea 10. |
| `skins`, `clips`, `clip_baked`, `lods` | arrays | omise când sunt goale | nevalidate la încărcare | Secțiunea 11. |
| `mesh_locks`, `glues` | arrays | omise când sunt goale | validate | Legături de mesh între straturi, secțiunea 11. |
| `machines`, `bindings`, `containers`, `scrolls` | arrays | omise când sunt goale | validate | Interacțiune și layout, secțiunea 11. |

Proiect valid minimal:

```json
{"name": "empty", "width": 100, "height": 100, "fps": 24, "frames": 1}
```

Magnitudini: orice float din fișier trebuie să respecte `abs(x) <= 1e7`. Aceasta se aplică coordonatelor,
scalelor, offseturilor și valorilor cheilor deopotrivă. Lățimea și înălțimea sunt limitate în plus la 32768, iar ieșirea
raster la 16384 pe latură și 100 de megapixeli (secțiunea 18).

## 4. Rig: bones, ik, constraints, springs

### 4.1 `bones[]`

Oasele trebuie listate cu părinții primii. Părintele unui os trebuie să apară mai devreme în listă.

| Nume JSON | Tip | Implicit | Interval | Semnificație |
| --- | --- | --- | --- | --- |
| `name` | string | **Req** | unic | Referit de greutăți, ik, canale, `Layer.bone`. |
| `parent` | string sau null | null | un os anterior | Osul părinte. Null înseamnă rădăcină. |
| `length` | number | **Req** | finit, `>= 0` | Lungimea osului în pixeli, de-a lungul axei +x a osului. |
| `rest_offset` | `Vec2` | `[0,0]` | | Poziția capului în spațiul părintelui (spațiul scenei pentru rădăcini). |
| `rest_angle` | number | 0 | | Rotația de repaus în grade, relativ la părinte. |
| `angle_limits` | `[min, max]` sau null | null | `min <= max` | Limită a deltei unghiului animat, în grade. |
| `scale_limits` | `[min, max]` sau null | null | `0 < min <= max` | Limită a scalei animate. |

Interacțiuni: canalele `bone_angle` se adună la `rest_angle`; `bone_offset_x/y` se adună la `rest_offset`;
`bone_scale` înmulțește. `clamp_limits` rulează după toate canalele și înainte de FK, și din nou după fiecare
lanț IK.

```json
{"name": "upper", "parent": null, "length": 90, "rest_offset": [60, 120],
 "rest_angle": 0, "angle_limits": [-120, 120]}
```

### 4.2 `ik[]`

| Nume JSON | Tip | Implicit | Interval | Semnificație |
| --- | --- | --- | --- | --- |
| `name` | string | **Req** | unic | Ținta canalelor `ik_mix`. |
| `bones` | array of strings | **Req** | cel puțin 2; fiecare copil direct al celui anterior | Lanț de la rădăcină la vârf. |
| `target` | string | **Req** | un os, care nu face parte din lanț | Osul spre al cărui cap tinde vârful. |
| `bend` | number | 1 | +1 sau -1 | În ce parte se pliază un lanț cu două oase. |
| `mix` | number | 1 | limitat la 0..1 | Amestecul inițial FK-IK; animează cu `ik_mix`. `<= 0` omite lanțul. |
| `iterations` | integer | 16 | | Treceri FABRIK pentru lanțuri mai lungi de două oase. Este un prag minim: solverul iese devreme când vârful este la mai puțin de 0.01 px, altfel continuă până la 256 de treceri. |
| `soft` | number | 0 | `0..0.9` | Fracțiunea din rază pe care extensia se atenuează. Cu `stretch > 1`, zona este ultima fracțiune a razei întinse (`reach x stretch`). |
| `stretch` | number | 1 | `1..4` | Factorul maxim de întindere; peste 1, scala osului rădăcină crește pentru a ajunge la țintă. |

Un lanț cu două oase folosește solverul analitic bazat pe legea cosinusurilor; lanțurile mai lungi folosesc FABRIK. După rezolvare,
unghiurile sunt amestecate în poziție ca `old + wrap_pi(new - old) * mix`.

```json
{"name": "reach", "bones": ["upper", "lower"], "target": "target", "bend": 1, "mix": 1}
```

### 4.3 `constraints[]`

Tipurile de constrângeri folosesc o etichetă internă `"type"`, aplatizată lângă `name`, `bone` și `weight`.

| Nume JSON | Tip | Implicit | Semnificație |
| --- | --- | --- | --- |
| `name` | string | **Req** | Ținta canalelor `constraint_weight`. |
| `bone` | string | **Req** | Osul constrâns. |
| `type` | `aim`, `copy_rotation`, `pin` | **Req** | Tipul. |
| `target` | string | **Req** pentru `aim`, `pin` | Osul spre care se țintește sau pe care se alunecă. |
| `offset` | number | 0 | doar `aim`: grade suplimentare. |
| `source` | string | **Req** pentru `copy_rotation` | Osul a cărui rotație globală este copiată. |
| `weight` | number | 1 | Influența, limitată la 0..1. `<= 0` omite constrângerea. |

O constrângere nu își poate lua ca țintă pe ea însăși. Constrângerile rulează în ordinea listei, după toate lanțurile IK.

```json
{"name": "aim", "bone": "root", "type": "aim", "target": "look", "offset": 0, "weight": 0.5}
```

### 4.4 `springs[]`

Un arc amortizat care face ca rotația globală a unui os să rămână în urma rotației sale animate. Este
simulat o dată la încărcare și copt, deci orice cadru poate fi evaluat în orice ordine.

| Nume JSON | Tip | Implicit | Interval | Semnificație |
| --- | --- | --- | --- | --- |
| `bone` | string | **Req** | un arc pe os | Osul comandat. |
| `frequency` | number | 3 | `(0, 30]` Hz | Rigiditate. |
| `damping` | number | 0.35 | `>= 0` | Raport de amortizare; sub 1 oscilează, 1 este critic. |
| `warmup` | bool | true | | Rulează în prealabil o trecere, astfel încât buclele să pornească în regim staționar. |

Unghiurile coapte înlocuiesc unghiul osului după constrângeri (pasul 8 al evaluării).

## 5. Straturi

Un strat este `{"name": ..., "type": ..., <kind fields>, <common fields>}`. `type` este eticheta lui
`LayerKind`; câmpurile sale sunt aplatizate în obiectul stratului.

### 5.1 Câmpuri comune

| Nume JSON | Tip | Implicit | Interval | Semnificație |
| --- | --- | --- | --- | --- |
| `name` | string | **Req** | nevid, fără `/`, fără `::`; calea completă trebuie să fie unică | Numele stratului. Copiii sunt adresați ca `parent/child`. |
| `type` | string | **Req** | `vector`, `group`, `switch`, `liquid`, `boolean`, `emitter`, `image` | Tipul. |
| `transform` | object | identitate | | `position` `[0,0]`, `rotation` 0, `scale` `[1,1]`, `skew` 0, `anchor` `[0,0]`. |
| `opacity` | number | 1 | limitat la 0..1 | Se înmulțește în compoziția stratului. |
| `visible` | bool | true | | Straturile ascunse sunt omise la desenare, dar pot fi în continuare surse de mască. |
| `blend` | string | `normal` | vezi mai jos | Modul de amestecare. |
| `mask` | `{layer, invert}` | null | fără auto-mască, fără cicluri | Folosește alfa altui strat ca mască. |
| `bone` | string | null | un os cunoscut | Atașează la un os: matricea de skin a osului se aplică între matricea de scenă a părintelui și transformarea locală. |
| `parallax` | number | 1 | | Factor de adâncime pentru cameră. Contează doar valoarea stratului de nivel superior. |
| `deformers` | array | omis când este gol | doar straturi vector și image | Secțiunea 7. |
| `wobble` | object | null | | `amp`, `scale` (0.03), `rate` (8), `seed` (0): deplasare determinista prin zgomot. Animează `amp` cu `layer_wobble`. |
| `reference` | bool | false | | Strat de referință: ascuns, cu excepția cazului în care editorul afișează referințele. |
| `locked` | bool | false | | Blocare în editor, vezi mai jos. |
| `normal_map` | object | null | | Secțiunea 7. |
| `effects` | array | omis când este gol | cel mult 16 | Secțiunea 7. |

Matricea locală: `T(position) * R(rotation) * Shear(skew) * S(scale) * T(-anchor)`.

Moduri de amestecare: `normal`, `multiply`, `screen`, `overlay`, `darken`, `lighten`, `color_dodge`,
`color_burn`, `hard_light`, `soft_light`, `difference`, `exclusion`, `add` (CSS plus-lighter).

`locked` blochează doar operațiile din editor: selectarea pe canvas, gizmo-urile, deplasările fine (nudge), ștergerile, editările de timeline și
editările de canale (prin `assert_unlocked`), pentru strat și descendenții săi. Randarea și
evaluarea îl ignoră, iar `Op`-urile simple (secțiunea „Editor ops" din ARCHITECTURE.md) se aplică în continuare.

Câmpurile comune sunt scrise după câmpurile specifice tipului în ieșire, dar ordinea nu contează la intrare.

### 5.2 `vector`

| Câmp | Tip | Implicit | Semnificație |
| --- | --- | --- | --- |
| `paths` | array of path | `[]` | Secțiunea 6. |

### 5.3 `group`

| Câmp | Tip | Implicit | Semnificație |
| --- | --- | --- | --- |
| `layers` | array of layers | `[]` | Copiii moștenesc transformarea grupului. |

### 5.4 `switch`

| Câmp | Tip | Implicit | Semnificație |
| --- | --- | --- | --- |
| `layers` | array of layers | `[]` | Desene candidate. |
| `selected` | integer | 0 | Indicele inițial al copilului. Animează cu `layer_switch` (chei step). |

Se afișează exact un copil. Valoarea este rotunjită; cea negativă devine 0; valorile egale sau mai mari decât numărul de
copii devin ultimul copil. Validarea nu respinge cheile din afara intervalului. Vezi secțiunea 20 pentru o
neconcordanță de formulare în lint despre aceasta.

### 5.5 `liquid`

Lichid de tip metaball. Câmpul este suma `r^2 / d^2` peste blob-uri; conturul este izocontur,
extras prin marching squares pe o grilă limitată la 600 pe 600 de celule și netezit cu Catmull-Rom.

| Câmp | Tip | Implicit | Interval | Semnificație |
| --- | --- | --- | --- | --- |
| `blobs` | array of `{pos, radius, bone?}` | **Req** | cel puțin 1; `radius > 0` | Un blob. `bone` îl atașează unui os. |
| `threshold` | number | 1 | `> 0` | Nivelul izo; 1 face dintr-un blob izolat un cerc de raza sa. |
| `cell` | number | 3 | `>= 0.25` | Celula grilei în unități de strat. |
| `fill` | `Paint` | null | | |
| `stroke` | stroke object | null | | |

### 5.6 `boolean`

| Câmp | Tip | Implicit | Semnificație |
| --- | --- | --- | --- |
| `op` | `union`, `intersect`, `difference`, `xor` | **Req** | Operația. |
| `paths` | array of path | **Req**, cel puțin 2 | `paths[0]` este subiectul; decuparea (clip) este reuniunea celorlalte. |

Rezultatul preia fill-ul și stroke-ul subiectului; toleranța de aplatizare este 0.25. Căile rezultate sunt
numite `<first path name><index>`.

### 5.7 `emitter`

`{"type": "emitter", "spec": {...}}`. Particulele sunt o funcție în formă închisă de timp, deci orice cadru
poate fi evaluat. Timpii sunt în secunde.

| Câmp `spec` | Implicit | Interval | Semnificație |
| --- | --- | --- | --- |
| `pos` | `[0,0]` | | Poziția emițătorului (spațiul stratului). |
| `bone` | null | | Atașează la un os. |
| `rate` | 20 | `(0, 2000]` | Particule pe secundă. |
| `lifetime` | 1 | `> 0` | Secunde. |
| `lifetime_jitter` | 0 | 0..1 | |
| `speed` | 60 | | Pixeli pe secundă. |
| `speed_jitter` | 0 | 0..1 | |
| `direction` | -90 | | Grade; -90 este în sus. |
| `spread` | 30 | | Grade de lățime a conului. |
| `gravity` | `[0,0]` | | Accelerație, pixeli pe secundă la pătrat. |
| `size_start`, `size_end` | 3, 3 | | Raza pe durata vieții. |
| `color_start`, `color_end` | `#fff`, `#fff0` | | Culoarea pe durata vieții. |
| `seed` | 0 | | Seed determinist. |
| `start` | 0 | | Momentul de start în secunde. |
| `duration` | null | | Emisia se oprește după atâtea secunde; null înseamnă la nesfârșit. |
| `max_particles` | 2000 | | Limită maximă de particule vii. |

### 5.8 `image`

| Câmp | Tip | Implicit | Interval | Semnificație |
| --- | --- | --- | --- | --- |
| `source` | `{file?, png_base64?}` | **Req** | unul din cele două | `file` este o cale relativă la directorul proiectului (secțiunea 18); `png_base64` este autonom. Niciunul: `image has neither 'png_base64' nor 'file'`. |
| `origin` | `Vec2` | `[0,0]` | | Colțul stânga-sus în spațiul stratului. |
| `size` | `Vec2` sau null | null (dimensiunea în pixeli) | | Dimensiunea afișată. |
| `mesh` | object sau null | null | | Mesh de deformare. |

`mesh`: `cols`, `rows` (mesh de tip grilă, fiecare cel puțin 1), `weights` (greutăți de os per vertex; gol
înseamnă static) și, opțional, `free` = `{vertices, triangles, uvs}`. Un mesh liber (free) are nevoie de cel puțin 3
vertecși, cel puțin 1 triunghi, cel mult 200000 de vertecși și 400000 de triunghiuri, coordonate finite,
indici de triunghi valizi, iar `uvs` fie gol, fie de aceeași lungime ca `vertices`. Cu deformatoare, dar
fără mesh, se folosește o grilă de 16 pe 16.

```json
{"name": "img2", "type": "image", "source": {"png_base64": "iVBOR..."},
 "origin": [100, 0], "size": [80, 80],
 "mesh": {"free": {"vertices": [[100,0],[180,0],[180,80],[100,80]],
                   "triangles": [[0,1,2],[0,2,3]],
                   "uvs": [[0,0],[8,0],[8,8],[0,8]]}, "weights": []}}
```

## 6. Căi, ancore, paint, stroke

### 6.1 Calea (`paths[]`)

| Nume JSON | Tip | Implicit | Semnificație |
| --- | --- | --- | --- |
| `name` | string | `""` | Un nume gol este înlocuit cu `path{index}` pentru adresare. Nu trebuie să conțină `::`. Unic în cadrul stratului. |
| `closed` | bool | false | |
| `anchors` | array | **Req** | Ancore Bezier. |
| `fill` | `Paint` | null | |
| `stroke` | stroke | null | |
| `morphs` | array of `{name, anchors}` | omis când este gol | Forme de blend aditive. Numărul de ancore trebuie să fie egal cu cel al căii de bază. Ponderea prin `path_morph`. |

### 6.2 Ancora

| Nume JSON | Tip | Implicit | Semnificație |
| --- | --- | --- | --- |
| `pos` | `Vec2` | **Req** | Poziția. |
| `handle_in`, `handle_out` | `Vec2` | null | Puncte de control în același spațiu ca `pos` (absolute, nu relative). Un handle lipsă este egal cu `pos` (un colț). |
| `weights` | array of `{bone, weight}` | omis când este gol | Skinning. Fiecare os trebuie să existe, iar fiecare greutate este finită și `>= 0`. Greutățile sunt normalizate la evaluare; gol înseamnă static. Handle-urile primesc și ele skin. |

### 6.3 Stroke

| Nume JSON | Tip | Implicit | Semnificație |
| --- | --- | --- | --- |
| `paint` | `Paint` | **Req** | |
| `width` | number | 1 | |
| `cap` | `butt`, `round`, `square` | `round` | |
| `join` | `miter`, `round`, `bevel` | `round` | O valoare necunoscută dă ``unknown variant `foo`, expected one of `miter`, `round`, `bevel` ``. |
| `dash` | array | omis | Folosit doar dacă lungimea sa este pară, cel puțin 2, iar fiecare element este `> 0`. |
| `dash_offset` | number | 0 | |
| `width_profile` | array of `[t, factor]` | omis | Multiplicator de lățime liniar pe porțiuni de-a lungul lungimii (`t` în 0..1), constant dincolo de capete, limită miter 2. Un stroke cu lățime variabilă ignoră `dash` și `join`. |

Exemplu de umplere cu gradient radial și stroke (din exemplul cu minge):

```json
"fill": {"center": [-8, -10], "radius": 44,
         "stops": [{"offset": 0, "color": "#ffd9a0"}, {"offset": 1, "color": "#d6336c"}]},
"stroke": {"paint": "#3d2c2e", "width": 2}
```

Linterul (`osatura lint`) avertizează despre probleme de gradient (L011), chei de switch (L012) și probleme de date de editor (L013); vezi
`osatura-core/src/lint.rs`.

## 7. Deformatoare, efecte, normal map

### 7.1 `deformers[]`

Permise doar pe straturile `vector` și `image`. Fiecare are o etichetă `"type"` și un `name` (unic în cadrul
stratului, fără `::`). Rulează în ordinea listei pe geometria din spațiul de repaus, înainte de skinning și înainte de
`wobble`. Offseturile sunt statice; animează-le cu canalele de sub-țintă.

| `type` | Câmpuri | Constrângeri | Animează cu (țintă) |
| --- | --- | --- | --- |
| `lattice` | `cols`, `rows`, `rect` `[x,y,w,h]`, `offsets` | `cols, rows >= 1`; `w, h > 0`; `offsets` gol sau `(cols+1)*(rows+1)` în ordine row-major | `lattice_dx/dy` (`layer::name::c,r`) |
| `spine` | `rest` (puncte de curbă), `offsets`, `radius` (margine soft opțională) | cel puțin 2 puncte | `spine_dx/dy` (`layer::name::i`) |
| `pin` | `pins`, `offsets`, `radius` | 1 până la 64 de pini, `radius > 0`, pini necoincidenți | `pin_dx/dy` (`layer::name::i`) |
| `cage` | `rest` (poligon), `offsets` | 3 până la 256 de vertecși, poligon simplu, arie nenulă | `cage_dx/dy` (`layer::name::i`) |
| `cloth` | ca `lattice` | ca `lattice` | `lattice_dx/dy` (`layer::name::c,r`) |

Lattice amestecă offseturile cu ponderi Bezier-Bernstein (neted); cloth deplasează fiecare vertex al grilei
exact (biliniar). Pin folosește o funcție de bază radială cu suport compact Wendland. Cage folosește
coordonatele cu valoare medie Floater: conținutul din interiorul poligonului îl urmează, conținutul din exterior este lăsat
în pace.

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

Cel mult 16 pe strat. Fiecare efect are `"type"`, `enabled` (implicit true) și câmpurile de mai jos.
Ținta de canal `fxN` indexează array-ul, inclusiv efectele dezactivate.

| `type` | Câmpuri | Interval |
| --- | --- | --- |
| `blur` | `radius` | 0..64 |
| `drop_shadow` | `dx`, `dy`, `blur`, `color`, `opacity` (1) | offseturi de cel mult 1024; blur 0..64; opacity 0..1 |
| `tint` | `black`, `white`, `amount` (1) | amount 0..1 |
| `fill` | `color`, `opacity` (1) | opacity 0..1 |

Animează cu `fx_blur`, `fx_dx`, `fx_dy`, `fx_soft`, `fx_amount` pe ținta `layer::fxN`.

### 7.3 `normal_map`

`{"data": base64-PNG | "path": file, "fit": "stretch" | "tile" | "exact" (implicit stretch),
"strength": 0..8 (implicit 1), "green_down": bool (implicit false)}`. Are nevoie de `data` sau `path`. Alimentează
post-efectul `relight`; randările simple nu îl folosesc. Ca `Layer.normal_map`, este potrivit la caseta de încadrare în pixeli a acelui strat. `fit` `exact` cere o hartă exact de dimensiunea cadrului și este eroare în caz contrar.

## 8. Canale, piste, easing

### 8.1 Canalul

```json
{"target": "ball", "prop": "layer_y", "source": "keys",
 "track": {"keys": [{"frame": 0, "value": 40}, {"frame": 24, "value": 170}]},
 "mode": "replace", "weight": 1}
```

| Nume JSON | Tip | Implicit | Semnificație |
| --- | --- | --- | --- |
| `target` | string | `""` | Nume sau forma `layer::sub`. Ignorat pentru proprietățile camerei. |
| `prop` | string | **Req** | Una dintre proprietățile snake_case de mai jos. |
| `source` | `keys`, `expr`, `noise`, `driven` | **Req** | Etichetă aplatizată, cu câmpurile acelei surse alături. |
| `mode` | `replace`, `add`, `multiply` | `replace` | Cum se combină valoarea cu cea curentă. |
| `weight` | number | 1 | Influența, trebuie să fie finită; amestecare spre rezultat. |

Combinare: replace este `cur + (v - cur) * w`; add este `cur + v * w`; multiply este
`cur * (1 + (v - 1) * w)`. Valorile nefinite sunt omise. Canalele rulează în ordinea listei, deci un canal
ulterior vede rezultatele celor anterioare (prin `get(...)` și `driven`).

Surse:

| `source` | Câmpuri |
| --- | --- |
| `keys` | `track`: `{keys: [{frame, value, ease?}], extrapolate?}` |
| `expr` | `expr`: string (secțiunea 9) |
| `noise` | `amp`, `freq` (cicluri pe secundă), `seed` (0), `octaves` (1). Ieșire în `[-amp, amp]`; bazat pe hash, determinist. |
| `driven` | `input` (țintă), `input_prop`, `curve`: o pistă ale cărei „cadre" sunt valori de intrare (de exemplu unghiul cotului la cantitatea de bombare). |

### 8.2 Proprietăți

Există 39 de proprietăți. Proprietatea decide ce înseamnă `target`.

| Grup | Proprietăți | `target` |
| --- | --- | --- |
| Os | `bone_angle`, `bone_scale`, `bone_offset_x`, `bone_offset_y` | numele osului |
| Strat | `layer_opacity`, `layer_x`, `layer_y`, `layer_rotation`, `layer_scale_x`, `layer_scale_y`, `layer_skew`, `layer_switch`, `layer_wobble` | calea stratului (`parent/child`) |
| Cameră | `camera_x`, `camera_y`, `camera_zoom`, `camera_rotation`, `camera_focus`, `camera_blur` | ignorat |
| Rig | `ik_mix`, `constraint_weight` | numele ik-ului sau al constrângerii |
| Cale | `path_morph`, `point_dx`, `point_dy`, `grad_dx`, `grad_dy` | `layer::path::morphName`, `layer::path::anchorIndex`, `layer::path::fill.from` / `fill.to` / `stroke.from` / `stroke.to` |
| Deformator | `lattice_dx/dy`, `spine_dx/dy`, `pin_dx/dy`, `cage_dx/dy` | `layer::deformer::c,r` (lattice, cloth) sau `layer::deformer::i` |
| Efect | `fx_blur`, `fx_dx`, `fx_dy`, `fx_soft`, `fx_amount` | `layer::fxN` |

Semantica handle-urilor de gradient: la gradienții liniari, `.from` și `.to` mută punctul de start și pe cel de sfârșit cu
`(dx, dy)`. La gradienții radiali, `.from` mută centrul, iar `.to` schimbă raza cu `dx`.
Doar căile al căror fill sau stroke este un gradient au aceste ținte. `layer_switch` cere un
strat `switch`. O proprietate necunoscută listează în eroare toate variantele valide.

Limitări la scriere: opacity 0..1, camera zoom cel puțin 0.01, camera blur cel puțin 0, `ik_mix` și
`constraint_weight` 0..1.

### 8.3 Pista și cheia

`Track` = `{"keys": [...], "extrapolate": "hold"}`. O cheie este `{"frame": number, "value": number,
"ease": {...}}`. Cheile trebuie să fie finite și strict crescătoare ca cadru:
`channel N ('target'): keys must be finite and strictly increasing in frame`. Easing-ul unei chei
modelează segmentul care începe la acea cheie.

Extrapolare (în afara primei și ultimei chei): `hold` (implicit), `linear`, `loop`, `ping_pong`,
`loop_offset`.

### 8.4 Easing

`ease` este un obiect cu o etichetă `type`. Implicit este `{"type": "linear"}`.

| `type` | Parametri | Curba |
| --- | --- | --- |
| `linear` | | `u` |
| `step` | | Ține valoarea cheii până la următoarea cheie. |
| `ease_in` | | `u^2` |
| `ease_out` | | `1 - (1-u)^2` |
| `ease_in_out` | | Smoothstep. |
| `cubic_bezier` | `x1, y1, x2, y2` | Curbă în stil CSS prin (0,0) și (1,1). |
| `smooth` | | Hermite Catmull-Rom neuniform prin cheile învecinate. |
| `spring` | `bounces`, `decay` | `1 - e^(-decay*u) * cos(2*pi*bounces*u) * window`, window = `1 - smooth01((u-0.85)/0.15)`; aterizează exact pe țintă. |
| `back` | `overshoot` | Cubic cu panta de start `3 + overshoot` (limitat la 0 sau mai mult). Overshoot 1 atinge vârful 1.037 (u = 2/3); overshoot 2 atinge vârful 1.1317 (u = 0.5556). |
| `bounce` | | Restituție 0.6, trei reveniri, durată totală 3.352 unități de timp. Prima lovitură la u = 0.2983; vârfurile reveniri 0.64, 0.8704, 0.953344; se termină la 1. |

`back` și `bounce` sunt derivări originale (vezi `CLEANROOM.md`).

## 9. Expresii

Un canal cu `"source": "expr"` evaluează o formulă la fiecare cadru. Sursa este limitată la 4096 de octeți
și la adâncime de imbricare 64; erorile sunt raportate la validare cu o poziție, de exemplu
`channel 2 ('upper'): expression error: unexpected end of expression`.

Variabile: `frame`, `time` (= frame / fps), `fps`, `frames`, `pi`, `t` (= frame / frames).

Operatori (de la precedență mică la mare): `?:`, `||`, `&&`, comparații, `+ -`, `* / %`, unari `- !`,
`^` (asociativ la dreapta). Împărțirea la zero dă 0; `%` este euclidian (`rem_euclid`).

Funcții: `sin cos tan asin acos atan atan2 abs sqrt pow min max clamp lerp smoothstep floor ceil
round fract sign mod exp ln noise` (1 sau 2 argumente) `fbm` (2 sau 3) `wiggle` (2 sau 3) `saw tri
square deg rad` și `get("target", "prop")`. Un rezultat nefinit al unei funcții devine 0.

`get("target", "prop")` citește valoarea curentă a oricărei proprietăți, inclusiv rezultatele canalelor
anterioare. Ambele șiruri trebuie să se rezolve la validare (`channel 2: get("q", "bone_angle"): unknown
target`). Un canal care citește o valoare trebuie să vină după canalul care o scrie.

Exemplu din proiectul cu minge:

```json
{"target": "shadow", "prop": "layer_opacity", "source": "expr",
 "expr": "clamp((get(\"ball\", \"layer_y\") - 40) / 130, 0.15, 1)"}
```

Expresiile de legare (secțiunea 11) folosesc o formă restrânsă: variabila este `value`, iar variabilele de timp
și `get()` sunt interzise.

## 10. Cameră, shot-uri, audio, metadate

### 10.1 `camera`

| Nume JSON | Tip | Implicit | Interval | Semnificație |
| --- | --- | --- | --- | --- |
| `position` | `Vec2` sau null | null (centrul scenei) | | Punctul din lume aflat în centrul ecranului. |
| `zoom` | număr | 1 | limitat la cel puțin 0.01 | |
| `rotation` | număr | 0 | grade | |
| `focus` | număr | 1 | | Adâncimea `parallax` a stratului care este clară (folosită de `dof` și `dof-export`). |
| `blur` | număr | 12 | finit, `>= 0` | Raza de estompare în pixeli pe unitate de distanță de parallax. |

Parallax: un strat cu `parallax = p` vede `zoom = 1 + (zoom - 1) * p`,
`pos = centre + (pos - centre) * p`, `rotation = -rotation * p`. Un `focus` sau `blur` nefinit
produce `camera focus must be finite and camera blur a finite number >= 0`.

### 10.2 Array-uri de metadate (nevalidate la încărcare)

Acestea conțin date de editor și de instrumente. Motorul le acceptă ca date; operațiile care le
folosesc le verifică atunci când rulează.

| Array | Câmpurile unui element |
| --- | --- |
| `markers` | `name`, `frame`, `end` (opțional; o regiune când este setat) |
| `comps` | `name`, `layers`: hartă de la calea stratului la `{visible, opacity, selected?}` |
| `events` | `name`, `frame`, `data` (orice JSON) |
| `bone_vis` | `bone`, `from`, `to` (intervalul de cadre în care osul este afișat) |
| `channel_meta` | `target`, `prop`, `color`, `shy`, `solo`, `lock` (indicatori de editor; `lock` blochează set-key) |
| `shots` | `name`, `x`, `y`, `zoom`, `rotation` (vizualizări de cameră salvate) |

```json
"markers": [{"name": "intro", "frame": 0, "end": 12}, {"name": "hit", "frame": 20}],
"events": [{"name": "step", "frame": 12, "data": {"sound": "step.wav"}}],
"shots": [{"name": "wide", "x": 200, "y": 150, "zoom": 1, "rotation": 0}]
```

### 10.3 `sounds[]`

| Nume JSON | Tip | Interval | Semnificație |
| --- | --- | --- | --- |
| `name` | șir | unic | |
| `file` | șir | o cale de asset (secțiunea 18) | WAV sau alt format audio acceptat. |
| `start` | număr | secunde | Poziționarea pe timeline. |
| `gain` | număr | 0..4 | |
| `trim_in`, `trim_out` | număr | `trim <= 86400` | Secundele tăiate de la începutul și de la sfârșitul fișierului. |
| `fade_in`, `fade_out` | număr | încape în clipul tăiat | Secunde. |

`check` (nu `Engine::new`) raportează un fișier lipsă: `sound 's': cannot read sound file ./nofile.wav: ...`.

## 11. Clipuri, skin-uri, legături, interacțiune, lod-uri

### 11.1 `clips[]` și `clip_baked`

Un clip plasează pe timeline un `take` reutilizabil. Bake-ul clipurilor adaugă canale obișnuite (după
cele făcute manual) și stochează un instantaneu în `clip_baked`, astfel încât re-bake-ul este idempotent, iar munca
manuală nu este niciodată ștearsă.

| Câmp al clipului | Semnificație |
| --- | --- |
| `name`, `start`, `speed`, `loops` | Poziționare. |
| `take` | `{name, length, tracks: [{target, prop, keys}]}` |
| `mode` | `replace`, `additive`, `multiply` |
| `weight`, `weight_keys` | Influență; pistă animată opțională. |
| `include`, `exclude` | Liste de tipare care selectează pistele. |
| `track_weights` | `[{pattern, prop, weight}]` |
| `fade_in`, `fade_out` | Cadre. |
| `rename` | `[[from, to]]` redenumiri de ținte. |

### 11.2 `skins[]`

`{name, group, layers, constraints, iks, bones}`. Skin-urile din același `group` sunt variante exclusive.
Scalarea oaselor pe skin este comprimată, la bake, într-un canal-pas `bone_scale` de tip multiply. Regulile skin-urilor
se verifică doar când rulează operațiile de skin.

### 11.3 `mesh_locks[]` și `glues[]`

O pereche glue folosește `VertexRef` = `{layer, path (default 0), index}`; un mesh lock denumește căi întregi cu `{layer, path}` și o listă opțională de indici ai vertecșilor urmăritori (goală înseamnă fiecare vertex).

```json
"mesh_locks": [{"name": "lock", "host": {"layer": "hostm", "path": 0},
                "follower": {"layer": "follm", "path": 0}, "vertices": [], "strength": 1}],
"glues": [{"name": "g", "mode": "rigid", "strength": 1,
           "pairs": [{"a": {"layer": "hostm", "path": 0, "index": 0},
                      "b": {"layer": "s1", "path": 0, "index": 0}}]}]
```

`mode` al unui glue este `snap`, `rigid` sau `offset`. Ambele sunt validate de `links::resolve` și aplicate în
spațiul scenei în ordine topologică (pasul 12 al evaluării).

### 11.4 `machines[]` și `bindings[]` (doar la runtime)

Mașinile de stări și legăturile de date (data bindings) sunt conduse de Player în timpul rulării. Pentru export și bake
devin canale `replace` constante, adăugate la proiect. Fiecare mașină are nevoie de cel puțin un strat.

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

`kind` al unui input este `bool`, `number` sau `trigger`. `source.kind` al unei stări este `clip`, `segment`, `region`
sau `blend`; `mode` este `loop`, `once` sau `ping_pong`. `Transition` și `Condition` resping cheile
necunoscute.

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

`weight` al unui binding trebuie să fie în intervalul 0..1 (`binding 'b': weight must be within 0..1`). Țintele mașinilor și ale binding-urilor
sunt verificate printr-o compilare de probă.

### 11.5 `containers[]` și `scrolls[]`

Layout 1-D în stil flexbox. Câmpurile unui container: `id`, `members`, `axis` (`row`, `column`), `gap`,
`cross_gap`, `padding` (4 numere), `justify`, `align`, `wrap`, `rect`. Un `rect` are un `kind`:
`fixed` (`x,y,w,h`), `pin`, `aspect` sau `hug`. Un membru are exact unul dintre `layer`, `container` sau
`spacer`, plus `grow`, `shrink`, `width`, `height`, `align`, `resize`. Limite: 100000 de membri,
adâncime de imbricare 64. Un scroll are `id`, `viewport`, `content`, `axis`, `offset`, `elastic`, `snap_x`,
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

Offseturile containerelor se adaugă la poziția stratului și se înmulțesc în scală (pasul 7 al evaluării).
`apply_clip_masks` adaugă referințe de mască în proiectul primit, deci modifică copia motorului.

### 11.6 `lods[]`

O înregistrare a unei treceri de nivel de detaliu (level-of-detail) scrise de `osatura lod`: `level`, `tolerance_px`, `scale`,
`declared_px`, `bound_stage`, `path_px`, `mesh_px`, `keys_px`, `cull_px`, `stats`
(`path_anchors`, `mesh_vertices`, `keys`, `layers`, `bytes`) și `notes`. Este documentația a ceea ce a
făcut o trecere; randarea nu o citește niciodată.


## 12. Reguli de validare, limite și mesaje de eroare

`Engine::new` validează într-o singură trecere și returnează toate problemele. Tabelul listează ce este impus;
mesajele între ghilimele au fost produse de `osatura check` pe fișiere stricate intenționat.

### 12.1 Limite globale

| Regulă | Limită | Mesaj |
| --- | --- | --- |
| fps | `(0, 1000]` | `fps must be within (0, 1000]` |
| frames | `1..=200000` | `frames must be within 1..=200000` |
| width, height | `> 0` | `width and height must be > 0` |
| width, height | `<= 32768` | `width and height must be at most 32768` |
| orice float | `abs <= 1e7` | `value 100000000.0 at /bones/0/length is outside the supported range of +-1e7 (coordinates, scales, offsets and key values must stay below it)` |
| oase | cel mult 4096 | `at most 4096 bones` |
| format | `<= 1` | `project format 2 is newer than this build supports (1); update OSATURA` |
| dimensiune JSON | CLI `--max-input-bytes`, implicit 256 MiB; API `MAX_JSON_BYTES` 256 MiB | |

Limita 1e7 se aplică de două ori. La încărcare, un literal float care o depășește eșuează la parsare, iar mesajul
este învelit ca `invalid JSON in <file>: value ...`. Un literal fără punct zecimal (un întreg)
trece de parsare și este prins de re-verificarea din `Engine::new`, unde apare ca un element obișnuit
de listă. Limita există pentru ca o înmulțire ulterioară să nu poată depăși `f32` până la infinit.

### 12.2 Rig

| Regulă | Mesaj |
| --- | --- |
| părintele unui os trebuie să existe mai devreme | `bone 'lower' has parent 'nope' which is missing or listed after it` |
| lungimea osului finită, `>= 0` | `bone 'lower' has invalid length` |
| `angle_limits` `min <= max`; `scale_limits` `0 < min <= max` | |
| ik are nevoie de 2 oase | `ik 'reach': needs at least two bones` |
| lanțul ik legat prin părinte | `ik 'reach': each bone must be a direct child of the previous one` |
| ținta ik nu este în lanț | |
| soft, stretch | `ik 'reach': soft must be within 0..0.9 and stretch within 1..4` |
| o constrângere nu se poate ținti pe sine | |
| spring | `spring on 'upper': frequency must be in (0, 30] Hz and damping >= 0`; unul pe os |

### 12.3 Straturi, căi, geometrie

| Regulă | Mesaj |
| --- | --- |
| numele stratului nevid, fără `/`, fără `::` | `layer name 'a/b' must be non-empty and contain neither '/' nor '::'` |
| cale duplicată | `duplicate layer path 'arm'` (și linii pentru țintele animabile duplicate) |
| numele căilor și ale deformatoarelor nu conțin `::` și sunt unice în cadrul stratului | |
| numărul de ancore morph este egal cu cel al căii de bază | |
| greutățile de skin denumesc oase cunoscute; greutate finită, `>= 0` | |
| mesh liber | cel puțin 3 vertecși, cel puțin 1 triunghi, cel mult 200000 / 400000, finit, indici valizi, `uvs` gol sau cu același număr |
| mesh de tip grilă | `cols`, `rows >= 1` |
| mască | fără mască proprie; `mask cycle involving layer 'arm'` |
| sursă de imagine | `layer 'im': image has neither 'png_base64' nor 'file'` |
| liquid | cel puțin 1 blob; `threshold > 0`; `cell >= 0.25`; blob `radius > 0` |
| boolean | cel puțin 2 căi |
| emitter | `rate` în `(0, 2000]`; `lifetime > 0`; jitter-e 0..1 |
| deformatoare | doar pe straturi vectoriale și de imagine; vezi secțiunea 7.1 pentru fiecare tip |
| efecte | cel mult 16; blur și softness 0..64; offseturi cel mult 1024; cantități 0..1 |
| normal map | `strength` 0..8; are nevoie de `data` sau `path` |
| cameră | `camera focus must be finite and camera blur a finite number >= 0` |

### 12.4 Canale

| Regulă | Mesaj |
| --- | --- |
| ținta se rezolvă | `channel 0: unknown target 'zzz' for BoneOffsetX` |
| `layer_switch` pe un strat switch | `channel 2: layer_switch needs a switch layer, 'arm' is not one` |
| greutate finită | |
| chei finite și strict crescătoare | `channel 0 ('target'): keys must be finite and strictly increasing in frame` |
| expresia se parsează | `channel 2 ('upper'): expression error: unexpected end of expression` |
| țintele și proprietățile `get()` se rezolvă | `channel 2: get("q", "bone_angle"): unknown target` |
| inputul condus se rezolvă | |

Numele necunoscute de `prop` sau `ease` produc erori serde care listează fiecare variantă validă, de exemplu
``unknown variant `foo`, expected one of ...``. Tipurile întregi greșite produc
``invalid type: floating point `48.0`, expected u32``. Un câmp obligatoriu lipsă produce
``missing field `frames` ``.

### 12.5 Alte secțiuni validate

- Sunete: `gain` 0..4, `trim` cel mult 86400 s, fade-urile încap în clipul tăiat, nume unice.
  Existența fișierului este verificată de `check`, nu de `Engine::new`.
- Mașini: cel puțin un strat (`machine 'm': needs at least one layer`); tranzițiile și
  condițiile resping cheile necunoscute. Binding-uri: `binding 'b': weight must be within 0..1`. Țintele lor
  sunt verificate compilându-le o dată.
- Containere și scroll-uri: membri unici, cel mult 100000, adâncime de imbricare 64, reguli pentru rect.
- Mesh lock-uri și glue-uri: referințe rezolvate de `links::resolve`.
- Fișiere de imagine și audio: secțiunea 18.

### 12.6 Date de editor: avertismente, și erori doar pentru numere nefinite

`Engine::new` și `check` examinează `skins`, `comps`, `markers`, `events`, `shots`, `channel_meta`, `bone_vis`, `clips` și
`lods` (modulul `editordata`). Pentru ca orice proiect salvat anterior să se încarce în continuare, **doar valorile care nu pot exista într-un fișier salvat
sunt erori**: NaN sau infinit în markers, events, shots, `bone_vis`, opacitatea comp, lods și opacitatea stratului. Orice altceva este
un **avertisment** afișat de `check` (și raportat de `lint` ca L013): zoom-ul camerei `<= 0` (motorul îl limitează la 0.01), `opacity`
al stratului în afara 0..1 (limitat), `ik_mix` și `constraint_weight` în afara 0..1 (limitate), valori cheie `layer_opacity` și
`camera_zoom` în afara intervalului lor, referințe necunoscute la strat/os/canal, nume duplicate, momente din afara
animației, skin-uri invalide (`skins::validate`), clipuri (`clipstack::validate`, și ținte de pistă din take care nu se rezolvă,
ținând cont de `rename`), `channel_meta` și `lods`. Fixture-urile din `out/` se încarcă fără avertismente.

`osatura check` afișează avertismentele de lint de acest fel; linter-ul complet (`osatura lint`) are mai multe reguli (de la L001 în sus).

## 13. Exemplul 1: minge care sare

Verificat cu `osatura check` (`ok: 'bouncing ball' 320×240 @ 24 fps, 48 frames, 0 bones, 0 ik,
3 layers, 5 channels`) și `osatura render ball.json 7 out.png`. Ce demonstrează:

- `layer_y` cu chei și easing `bounce` pe primul segment: mingea cade de la y=40 până la
  podea, la y=170, și sare de trei ori. Prima lovitură ajunge la aproximativ cadrul 7.2 (24 / 3.352).
  Segmentul de la 24 la 47 urcă înapoi cu `ease_out`.
- O umbră care urmărește mingea: `layer_x` și `layer_opacity` sunt expresii care citesc
  mingea cu `get()`. Sunt listate după canalele proprii ale mingii, deoarece canalele rulează în ordine.
- Squash la contact: o expresie returnează 0.8 pentru `layer_scale_y` când mingea este sub y=160.
- Cerc din patru ancore Bezier cu lungimea mânerului 16.57 (0.5523 ori raza 30).

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

## 14. Exemplul 2: braț cu rig și IK

Verificat cu `osatura check` (3 bones, 1 ik, 3 layers, 2 channels). `osatura explain arm.json
24` raportează vârful la (170, 190), egal cu osul `target`, iar `osatura render arm.json 24
out.png --bones` desenează scheletul. Ce demonstrează:

- `upper` este o rădăcină cu o limită de unghi; `lower` este copilul ei (`rest_offset` `[90, 0]` este egal cu
  lungimea părintelui); `target` este un os rădăcină liber spre care se întinde lanțul IK.
- `reach` este un lanț IK cu două oase, deci se folosește solverul analitic.
- `sleeve` are skin: fiecare ancoră listează greutăți de os. `hand` și `goal` sunt atașate cu
  `bone`.
- Mutarea osului `target` cu canale `bone_offset_x` și `bone_offset_y` face brațul să se întindă.

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

## 15. Exemplul 3: gură cu strat switch

Verificat cu `osatura check` și prin randarea cadrelor 0, 5 și 9, care arată trei guri
diferite. Cheile din afara intervalului sunt limitate: o valoare de cheie 7 arată copilul 2, iar -3 arată copilul 0.

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

## 16. Ordinea de evaluare

`Engine::evaluate(frame)` este o funcție pură de numărul cadrului și de proiect. Spring-urile sunt
simulate o singură dată, la construirea motorului; nimic altceva nu păstrează stare între cadre. Pașii:

1. **Starea inițială** (`initial_state`): transformările statice ale straturilor, opacitatea limitată la 0..1, `selected`
   al switch-ului, amplitudinea wobble, camera, mix-ul `ik`, greutățile constrângerilor și offseturile inițiale ale
   sloturilor de parametri (offseturi de oase, offseturi de deformatoare, greutăți morph și așa mai departe).
2. **Canale** (`apply_channels`) în ordinea listei, fiecare combinându-se cu valoarea curentă prin `mode` și
   `weight`. Rezultatele nefinite sunt sărite. Scrierile se limitează așa cum este listat în secțiunea 8.2.
3. **Limitare** (`clamp_limits`): `angle_limits` și `scale_limits` pe fiecare os.
4. **Cinematică directă (FK)**: matricile de lume ale tuturor oaselor, începând cu părinții.
5. **Lanțuri IK** în ordinea listei, sărind lanțurile cu `mix <= 0`. După fiecare lanț, poza este limitată
   și FK recalculat, astfel încât următorul lanț vede rezultatul.
6. **Constrângeri** în ordinea listei, sărind `weight <= 0`: aim, copy_rotation, pin.
7. **Containere de layout**: offseturile se adaugă la x și y ale stratului și se înmulțesc în scală.
8. **Spring-uri bake-uite**: unghiurile bake-uite înlocuiesc unghiul osului, apoi FK este recalculat.
9. **Matrici de skin**: `skin_mat = world * inverse(rest world)` pe fiecare os.
10. **Camera**: o matrice de cameră pe strat rădăcină, folosind `parallax` al stratului rădăcină (secțiunea 10.1).
11. **Noduri** în pre-ordine. Matricea de scenă a unui nod este `parentStage * boneAttach(skin_mat) * local`.
    Geometria vectorială se construiește în această ordine:
    1. morph-uri (blend shape-uri aditive);
    2. offseturile `point_dx/dy` pe ancoră;
    3. deplasările mânerelor de gradient (`grad_dx/dy`);
    4. deformatoarele în ordinea listei, apoi `wobble`;
    5. skinning cu greutăți normalizate (greutăți goale înseamnă static; mânerele de gradient sunt și ele
       supuse skinning-ului).
    Geometria cu skin se află în coordonatele de repaus ale scenei, iar transformarea stratului se aplică după ea.
    Un switch arată un singur copil (rotunjit, limitat). Geometria liquid, boolean și emitter este
    sintetizată la fiecare cadru. O imagine primește un mesh grilă sau liber; cu deformatoare, dar fără mesh, primește o
    grilă 16 pe 16.
12. **Mesh lock-uri și glue-uri** în spațiul scenei, în ordine topologică.
13. **Reparare nefinită** (`repair_nonfinite`): un nod a cărui geometrie conține NaN sau infinit
    este ascuns și se setează `Frame.nonfinite`. `evaluate_checked` și `check_all_frames_finite` transformă asta
    într-o eroare.

Consecințe pe care te poți baza:

- Un canal vede doar canalele dinaintea lui în listă (prin `get` și `driven`).
- IK suprascrie FK pentru oasele din lanț, proporțional cu `mix` (amestecul este
  `old + wrap_pi(new - old) * mix`). Cu `mix` 0, lanțul este FK pur.
- Constrângerile rulează după IK, deci o constrângere aim pe un os IK câștigă.
- Spring-urile acționează ultimele asupra unghiului osului, după IK și constrângeri.

## 17. Ordinea de randare

- Straturile rădăcină se desenează în ordinea listei, copiii în ordinea listei în interiorul părintelui lor.
- Un nod care nu este `visible` este sărit (poate fi totuși sursă de mască).
- Un nod este randat pe un strat offscreen când are `opacity < 1`, efecte, un mod de blend altul decât
  `normal`, sau o mască. Pași: conținutul; efectele stratului în ordinea listei (pe un canvas cu margine suplimentară); masca
  (stratul-mască este desenat cu propria transformare; alpha-ul lui, inclusiv opacitatea, se înmulțește;
  `invert` îl inversează); compunerea cu opacitatea și modul de blend.
- Adâncimea de câmp (depth of field) este aplicată doar de `dof` și `dof-export`: raza = `abs(parallax - focus) *
  blur`, cel mult 24 de pixeli implicit.
- Suprapunerea oaselor (`--bones`) se desenează ultima.

## 18. Asset-uri și căi

Proiectele pot referi fișiere: `image.source.file`, `normal_map.path`, `sounds[].file`.

- Căile trebuie să fie relative, nu trebuie să conțină `..` și, după urmărirea legăturilor simbolice, trebuie să rămână în interiorul
  directorului proiectului: `layer 'im': asset '/etc/passwd': absolute paths are not allowed (a project
  may only reference files inside its own directory; use --allow-external-assets to override)`, și
  `layer 'im': asset '../x.png': '..' is not allowed (...)`.
- `--allow-external-assets` ridică această restricție pentru o singură comandă. Se deschid doar fișiere obișnuite.
- Plafoane de dimensiune: imagini 64 MB pe disc, audio 256 MB. Imagini decodate: fiecare latură cel mult 16384 și
  cel mult 64 de megapixeli.
- Plafoane de ieșire: raster cel mult 16384 pe latură și 100 de megapixeli; secvențe cel mult 2 GiB; bugetul de
  export, cadre înmulțite cu ancore, cel mult 40 de milioane.
- Când CLI scrie un proiect, îl re-validează prin `Engine::new` și rescrie căile asset-urilor
  relativ la locația de ieșire.

Preferă `png_base64` pentru un fișier autonom.

## 19. Politica de versionare

- `format` este un întreg, în prezent `1` (`FORMAT_VERSION`).
- Un fișier cu un `format` mai mare decât cel acceptat de build este refuzat cu mesajul din secțiunea 12.1;
  motorul nu ghicește.
- Adăugarea unui câmp opțional cu valoare implicită nu reprezintă o schimbare de versiune. Build-urile mai vechi îl ignoră (iar
  `check` avertizează despre el ca despre un câmp necunoscut).
- Eliminarea sau reinterpretarea unui câmp necesită o incrementare, iar incrementarea necesită un pas de migrare în
  `Project::from_json`, care actualizează documentele mai vechi înainte de validare.
- API-ul bibliotecii Rust are propria versiune, `API_VERSION` 1.4.0, păzită de testul de snapshot
  `api_surface.txt`; ABI-ul C are `ABI_VERSION` 1. Vezi `docs/API.md`.

## 20. Lacune cunoscute între cod și comentarii

Acestea au fost găsite în timpul scrierii acestei referințe.

1. **Remediat.** Regula cheii switch (`lint::switch_key_warnings`, L012) spune acum ce face motorul: o cheie sub 0 arată copilul 0,
   o cheie egală cu numărul de copii sau mai mare arată ultimul copil (un switch gol nu arată nimic). Test: testele unitare `lint`, verificate
   față de motor.
2. Comentariul din `project.rs` spune că fișierele fără `format` sunt versiunea 1, dar valoarea implicită serde este
   `FORMAT_VERSION`. Cele două coincid astăzi doar pentru că constanta este 1. Când constanta va fi crescută, un
   fișier fără `format` ar fi citit ca cea mai nouă versiune.
3. **Remediat.** `unknownfields` nu mai raportează o structură goală scrisă explicit (`clips[].weight_keys: {"keys": []}`);
   `default_like` este recursiv, iar un test fixează mulțimea structurilor opționale pe care writer-ul le omite când sunt goale.
4. **Remediat (ca avertismente).** Câmpurile de editor sunt validate; vezi secțiunea 12.6.
5. **Remediat în wave 7 (L-71).** Geometria unui strat cu skin se află în spațiul de repaus al scenei, iar transformarea părintelui se aplică după
   ea. `Layer.bone` se atașează acum cu `world = bone_skin * parent_stage * local`: matricea de skin (lumea osului x inversul repausului) este
   o mișcare în spațiul scenei, deci se aplică după întregul lanț de părinți. În repaus, matricea de skin este identitatea, deci nimic
   nu se mișcă; când osul este **în poză**, un grup părinte cu scala 2 nu mai dublează deplasarea unui strat atașat. Un strat
   fără un os propriu urmează osul celui mai apropiat strat strămoș atașat la un os; osul propriu al unui strat îl înlocuiește pe cel al strămoșului
   (cele două mișcări nu se cumulează). Teste: `bone_attach_under_a_transformed_group_composes_bone_after_the_group`,
   `bone_attach_is_inherited_and_a_child_bone_replaces_the_parents`. Proiectele care se bazau pe vechea deplasare dublată (un os
   în poză sub un grup scalat sau rotit) se randează diferit; proiectele fără această combinație rămân neschimbate.
