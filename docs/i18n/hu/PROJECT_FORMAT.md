<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->

# OSATURA projektformátum

Ez az OSATURA projektfájl referenciája: egyetlen JSON-dokumentum, amely
a színpadot, a rigt, a rajzokat és az animációt tartalmazza. Az `osatura-core/src/project.rs`
fájlban lévő serde struktúrákból és az általuk hivatkozott modulokból készült,
a 13–15. szakasz minden példáját pedig ellenőriztük az `osatura` binárissal
(`osatura check`, `osatura render`, `osatura explain`).

Tartalom

1. [Konvenciók és elemi típusok](#1-konvenciók-és-elemi-típusok)
2. [Betöltés, verziózás és ismeretlen mezők](#2-betöltés-verziózás-és-ismeretlen-mezők)
3. [Legfelső szintű mezők](#3-legfelső-szintű-mezők)
4. [Rig: bones, ik, constraints, springs](#4-rig-bones-ik-constraints-springs)
5. [Rétegek](#5-rétegek)
6. [Útvonalak, horgonypontok, paint, stroke](#6-útvonalak-horgonypontok-paint-stroke)
7. [Deformerek, effektek, normáltérkép](#7-deformerek-effektek-normáltérkép)
8. [Csatornák, sávok, easing](#8-csatornák-sávok-easing)
9. [Kifejezések](#9-kifejezések)
10. [Kamera, snittek, hang, metaadatok](#10-kamera-felvételek-hang-metaadatok)
11. [Klipek, skinek, linkek, interakció, lodok](#11-klipek-skinek-kapcsolatok-interakció-lod-ok)
12. [Validációs szabályok, korlátok és hibaüzenetek](#12-validációs-szabályok-korlátok-és-hibaüzenetek)
13. [1. példa: pattogó labda](#13-1-példa-pattogó-labda)
14. [2. példa: riggelt kar IK-val](#14-2-példa-ik-s-riggelt-kar)
15. [3. példa: switch-réteges száj](#15-3-példa-switch-réteges-száj)
16. [Kiértékelési sorrend](#16-kiértékelési-sorrend)
17. [Renderelési sorrend](#17-renderelési-sorrend)
18. [Assetek és útvonalak](#18-assetek-és-útvonalak)
19. [Verziózási szabályzat](#19-verziókezelési-szabályzat)
20. [Ismert eltérések a kód és a megjegyzések között](#20-ismert-eltérések-a-kód-és-a-megjegyzések-között)

## 1. Konvenciók és elemi típusok

| Típus | JSON-alak | Megjegyzések |
| --- | --- | --- |
| number | JSON szám | `f32`-ként tárolódik, hacsak a mező nem mondja, hogy egész. Minden lebegőpontos számra teljesülnie kell: `abs(x) <= 1e7` (12. szakasz). |
| `Vec2` | `[x, y]` | Két szám. A színpad egysége a pixel, +x jobbra, +y lefelé. |
| `Rgba` | string | `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, vagy a `black`, `white`, `red`, `green` (0,128,0), `blue`, `yellow`, `transparent` egyike. A kimenet mindig kisbetűs hexadecimális. |
| `Paint` | string vagy objektum | Egy színstring egyszínű kitöltés. A `{"from": [x,y], "to": [x,y], "stops": [...]}` lineáris színátmenet. A `{"center": [x,y], "radius": r, "stops": [...]}` sugaras színátmenet. A típus címke nélküli enum. |
| `Stop` | `{"offset": 0..1, "color": Rgba}` | A stopokat rajzoláskor (stabilan) rendezi, az offseteket pedig a rasztermotor 0..1 közé szorítja. Elfajult színátmenet az utolsó stop színével fest. |
| angle | number | A fájlban mindenhol fok (csontszögek, rétegforgatás, skew, kameraforgatás, `angle_limits`). |
| time | number | Kulcsokban és jelölőkben képkocka, hangokban, gépekben és emitterekben másodperc. Minden mező megmondja, melyik. |
| names | string | Csont-, réteg-, útvonal-, deformer-, ik- és constraint-nevek. Az 5. és a 12. szakasz felsorolja a karakterkorlátozásokat. |

Egy rossz szín egy paint-kontextusban a következő serde-üzenetet adja:
`data did not match any variant of untagged enum Paint`, mert a `Paint` címke nélküli, és a serde
nem tudja megmondani, melyik ágra gondoltak.

Az alábbi mezőtáblák ezt a jelölést használják: a **Kötelező** azt jelenti, hogy a mezőnek nincs alapértéke, és jelen kell lennie;
egyébként az alapértelmezés látható. A „Tartomány” azt jelenti, amit a betöltő vagy a motor kikényszerít. A „clamped” (szorított) tartomány
nem hiba: a motor csendben a tartományba szorítja az értéket.

## 2. Betöltés, verziózás és ismeretlen mezők

A `Project::from_json(text)` az egyetlen belépési pont, amelyet a CLI, a C API, a WASM modul
és a Studio használ. Sorrendben ezt teszi:

1. Elemzi a JSON-t a serde-del. A serde figyelmen kívül hagyja az általa nem ismert mezőket.
2. Lefuttatja a nagyságrend-ellenőrzést: minden olyan lebegőpontos literál, amelynek abszolút értéke `1e7` fölött van, megbuktatja a betöltést
   ezzel: `invalid JSON in <file>: value ... is outside the supported range of +-1e7 ...`. Az egész
   literálokat később kapja el az `Engine::new` belső újraellenőrzése (lásd 12. szakasz).
3. Alkalmazza a formátumkaput: a build `FORMAT_VERSION` értékénél (jelenleg `1`) nagyobb `format` értéket
   elutasítja ezzel: `project format 2 is newer than this build supports (1); update OSATURA`.

Az `Engine::new` (vagy az `Engine::with_asset_dir`) ezután az egész projektet validálja, és minden talált
problémát listaként ad vissza. A CLI így írja ki őket:

```text
<file> is not a valid project:
  - first problem
  - second problem
```

A hibaüzenetek emberek számára készültek. Nem stabil szerződés; szövegre csak tesztekben illeszkedj.

### Ismeretlen mezők

A serde betöltéskor eldobja az ismeretlen kulcsokat, ezért egy elírás, például `fil` a `fill` helyett, csendben elveszne. Az
`unknownfields` modul összeveti a bemeneti JSON-t az újraszerializált projekttel, és jelenti azokat a kulcsokat, amelyek
nem élték túl:

```text
warning: unknown field layers[0].paths[0].fil (ignored)
```

Az `osatura check` kiírja ezeket a figyelmeztetéseket. Az alapértékkel egyező értékeket nem jelenti (az író
kihagyja az alapértékeket, így egy explicit `"visible": true` rendben van). Néhány struktúra engedékeny helyett szigorú, és
az ismeretlen kulcsokat kemény hibával utasítja el: az állapotgép `Transition` és `Condition` elemei, a CI-konfiguráció,
az élő bemenet konfigurációja, az átfedések és az arcparaméter-konfiguráció; a konfigurációs fájlokat `parse_strict` olvassa.

### Szerkesztői metaadatok körforgása

A projekt az opcionális listákat `skip_serializing_if` szabállyal szerializálja. A kézzel írt fájl
rövidebb lehet annál, amit az `osatura` visszaír; a kettő egyenértékű.

## 3. Legfelső szintű mezők

| JSON-név | Típus | Alapérték | Tartomány / szabály | Jelentés |
| --- | --- | --- | --- | --- |
| `format` | integer | `FORMAT_VERSION` (1) | `<=` a build verziója | Formátumverzió, 19. szakasz. |
| `name` | string | **Kötelező** | bármi | Megjelenített név. A `check` kiírja. |
| `width` | number | **Kötelező** | `> 0`, `<= 32768` | A színpad szélessége pixelben. |
| `height` | number | **Kötelező** | `> 0`, `<= 32768` | A színpad magassága pixelben. |
| `fps` | number | **Kötelező** | `(0, 1000]` | Képkocka másodpercenként. |
| `frames` | integer (`u32`) | **Kötelező** | `1..=200000` | Az idővonal hossza. A képkockaindexek `0..frames-1`. Az olyan lebegőpontos literált, mint a `48.0`, elutasítja: `invalid type: floating point `48.0`, expected u32`. |
| `background` | `Rgba` vagy null | null (átlátszó) | | A színpad színe mindennek a mögött. |
| `color_space` | string | `"srgb"` (nem íródik ki) | `srgb`, `linear` | Az a tér, amelyben a raszteres és a float kompozitor kever. `linear`: a színek lineáris fénybe konvertálódnak, f32-ben kompozitálódnak, majd egyszer visszakonvertálódnak (egy 50%-os fekete fehér fölött 188/255, nem 128/255). Beállítása: `osatura color-space`. Az SVG, a Lottie, a GPU renderelő, a `dof` és az `fx` figyelmen kívül hagyja. |
| `camera` | object | alapértékek | 10. szakasz | Globális kamera. |
| `bones` | array | `[]` | legfeljebb 4096 | Csontváz, 4. szakasz. |
| `ik` | array | `[]` | | IK-láncok. |
| `constraints` | array | `[]` | | Aim, copy-rotation, pin. |
| `springs` | array | `[]` | | Másodlagos mozgás, betöltéskor beledolgozva. |
| `layers` | array | `[]` | | Gyökérrétegek, a lista sorrendjében rajzolva. |
| `channels` | array | `[]` | | Animációs csatornák, a lista sorrendjében kiértékelve. |
| `markers`, `comps`, `events`, `bone_vis`, `channel_meta`, `shots` | tömbök | üresen kimarad | nem validált | Idővonal- és szerkesztői metaadatok, 10. szakasz. |
| `sounds` | array | üresen kimarad | validált | Hangsávok, 10. szakasz. |
| `skins`, `clips`, `clip_baked`, `lods` | tömbök | üresen kimarad | betöltéskor nem validált | 11. szakasz. |
| `mesh_locks`, `glues` | tömbök | üresen kimarad | validált | Rétegek közötti hálókapcsolatok, 11. szakasz. |
| `machines`, `bindings`, `containers`, `scrolls` | tömbök | üresen kimarad | validált | Interakció és elrendezés, 11. szakasz. |

Minimális érvényes projekt:

```json
{"name": "empty", "width": 100, "height": 100, "fps": 24, "frames": 1}
```

Nagyságrendek: a fájl minden lebegőpontos számára teljesülnie kell: `abs(x) <= 1e7`. Ez a koordinátákra,
skálákra, eltolásokra és kulcsértékekre egyaránt vonatkozik. A szélességet és a magasságot ezen felül 32768 korlátozza, a raszteres
kimenetet pedig oldalanként 16384 és 100 megapixel (18. szakasz).

## 4. Rig: bones, ik, constraints, springs

### 4.1 `bones[]`

A csontokat szülők-előbb sorrendben kell felsorolni. Egy csont szülőjének korábban kell szerepelnie a listában.

| JSON-név | Típus | Alapérték | Tartomány | Jelentés |
| --- | --- | --- | --- | --- |
| `name` | string | **Kötelező** | egyedi | A súlyok, az ik, a csatornák és a `Layer.bone` hivatkozik rá. |
| `parent` | string vagy null | null | egy korábbi csont | Szülőcsont. A null gyökeret jelent. |
| `length` | number | **Kötelező** | véges, `>= 0` | A csont hossza pixelben, a csont +x tengelye mentén. |
| `rest_offset` | `Vec2` | `[0,0]` | | A fej pozíciója a szülő terében (gyökereknél a színpad terében). |
| `rest_angle` | number | 0 | | Nyugalmi forgatás fokban a szülőhöz képest. |
| `angle_limits` | `[min, max]` vagy null | null | `min <= max` | Korlát az animált szögdeltára, fokban. |
| `scale_limits` | `[min, max]` vagy null | null | `0 < min <= max` | Korlát az animált skálára. |

Kölcsönhatások: a `bone_angle` csatornák hozzáadódnak a `rest_angle` értékhez; a `bone_offset_x/y` hozzáadódik a `rest_offset` értékhez;
a `bone_scale` szoroz. A `clamp_limits` minden csatorna után és az FK előtt fut, majd minden
IK-lánc után újra.

```json
{"name": "upper", "parent": null, "length": 90, "rest_offset": [60, 120],
 "rest_angle": 0, "angle_limits": [-120, 120]}
```

### 4.2 `ik[]`

| JSON-név | Típus | Alapérték | Tartomány | Jelentés |
| --- | --- | --- | --- | --- |
| `name` | string | **Kötelező** | egyedi | Az `ik_mix` csatornák célja. |
| `bones` | stringek tömbje | **Kötelező** | legalább 2; mindegyik az előző közvetlen gyermeke | Lánc a gyökértől a csúcsig. |
| `target` | string | **Kötelező** | egy csont, nincs a láncban | Az a csont, amelynek a fejét a csúcs el akarja érni. |
| `bend` | number | 1 | +1 vagy -1 | Merre hajlik a kétcsontos lánc. |
| `mix` | number | 1 | szorított 0..1 | Kezdeti FK-IK keverés; animáld az `ik_mix` tulajdonsággal. A `<= 0` kihagyja a láncot. |
| `iterations` | integer | 16 | | FABRIK-körök a kettőnél hosszabb láncokhoz. Ez alsó korlát: a megoldó korán kilép, ha a csúcs 0.01 px-en belül van, egyébként legfeljebb 256 körig folytatja. |
| `soft` | number | 0 | `0..0.9` | Az elérés azon hányada, amelyen át a kinyúlás lecseng. `stretch > 1` esetén a zóna a megnyújtott elérés (`reach x stretch`) utolsó hányada. |
| `stretch` | number | 1 | `1..4` | Maximális nyújtási tényező; 1 fölött a gyökércsont skálája nő, hogy elérjen. |

A kétcsontos lánc analitikus koszinusztételes megoldót használ; a hosszabb láncok FABRIK-ot. A megoldás után
a szögek `old + wrap_pi(new - old) * mix` képlettel keverednek a pózba.

```json
{"name": "reach", "bones": ["upper", "lower"], "target": "target", "bend": 1, "mix": 1}
```

### 4.3 `constraints[]`

A kényszertípusok belső `"type"` címkét használnak, amely a `name`, `bone` és `weight` mellé van lapítva.

| JSON-név | Típus | Alapérték | Jelentés |
| --- | --- | --- | --- |
| `name` | string | **Kötelező** | A `constraint_weight` csatornák célja. |
| `bone` | string | **Kötelező** | A kényszerített csont. |
| `type` | `aim`, `copy_rotation`, `pin` | **Kötelező** | Fajta. |
| `target` | string | **Kötelező** az `aim`, `pin` esetén | A csont, amelyre céloz vagy amelyre csúszik. |
| `offset` | number | 0 | Csak `aim`: többletfokok. |
| `source` | string | **Kötelező** a `copy_rotation` esetén | A csont, amelynek világforgatását másolja. |
| `weight` | number | 1 | Befolyás, 0..1 közé szorítva. A `<= 0` kihagyja a kényszert. |

Egy kényszer nem célozhatja önmagát. A kényszerek a lista sorrendjében, az összes IK-lánc után futnak.

```json
{"name": "aim", "bone": "root", "type": "aim", "target": "look", "offset": 0, "weight": 0.5}
```

### 4.4 `springs[]`

Csillapított rugó, amely egy csont világforgatását az animált forgatása mögé lemaradni készteti. Betöltéskor
egyszer szimulálódik és beledolgozódik, így bármely képkocka tetszőleges sorrendben kiértékelhető.

| JSON-név | Típus | Alapérték | Tartomány | Jelentés |
| --- | --- | --- | --- | --- |
| `bone` | string | **Kötelező** | csontonként egy rugó | A hajtott csont. |
| `frequency` | number | 3 | `(0, 30]` Hz | Merevség. |
| `damping` | number | 0.35 | `>= 0` | Csillapítási arány; 1 alatt rezeg, az 1 kritikus. |
| `warmup` | bool | true | | Előre lefuttat egy menetet, hogy a hurkok egyensúlyi állapotban induljanak. |

A beledolgozott szögek a kényszerek után lecserélik a csontszöget (a kiértékelés 8. lépése).

## 5. Rétegek

A réteg alakja `{"name": ..., "type": ..., <fajta mezői>, <közös mezők>}`. A `type` a `LayerKind`
címkéje; a mezői a réteg objektumába vannak lapítva.

### 5.1 Közös mezők

| JSON-név | Típus | Alapérték | Tartomány | Jelentés |
| --- | --- | --- | --- | --- |
| `name` | string | **Kötelező** | nem üres, nincs benne `/` és `::`; a teljes útvonalnak egyedinek kell lennie | A réteg neve. A gyermekekre `parent/child` alakban hivatkoznak. |
| `type` | string | **Kötelező** | `vector`, `group`, `switch`, `liquid`, `boolean`, `emitter`, `image` | Fajta. |
| `transform` | object | identitás | | `position` `[0,0]`, `rotation` 0, `scale` `[1,1]`, `skew` 0, `anchor` `[0,0]`. |
| `opacity` | number | 1 | szorított 0..1 | Beleszorzódik a réteg kompozitjába. |
| `visible` | bool | true | | A rejtett rétegeket rajzoláskor kihagyja, de maszkforrás még lehetnek. |
| `blend` | string | `normal` | lásd lent | Keverési mód. |
| `mask` | `{layer, invert}` | null | nincs önmaszk, nincs ciklus | Egy másik réteg alfáját használja maszkként. |
| `bone` | string | null | ismert csont | Csonthoz rögzít: a csont skin-mátrixa a szülő színpadmátrixa és a helyi transzformáció között érvényesül. |
| `parallax` | number | 1 | | Kamera-mélységi tényező. Csak a legfelső szintű réteg értéke számít. |
| `deformers` | array | üresen kimarad | csak vektor- és képrétegeken | 7. szakasz. |
| `wobble` | object | null | | `amp`, `scale` (0.03), `rate` (8), `seed` (0): determinisztikus zajeltolás. Az `amp` animálható a `layer_wobble` tulajdonsággal. |
| `reference` | bool | false | | Referenciaréteg: rejtett, hacsak a szerkesztő nem mutatja a referenciákat. |
| `locked` | bool | false | | Szerkesztői zár, lásd lent. |
| `normal_map` | object | null | | 7. szakasz. |
| `effects` | array | üresen kimarad | legfeljebb 16 | 7. szakasz. |

Helyi mátrix: `T(position) * R(rotation) * Shear(skew) * S(scale) * T(-anchor)`.

Keverési módok: `normal`, `multiply`, `screen`, `overlay`, `darken`, `lighten`, `color_dodge`,
`color_burn`, `hard_light`, `soft_light`, `difference`, `exclusion`, `add` (CSS plus-lighter).

A `locked` csak a szerkesztői műveleteket blokkolja: vászonválasztást, gizmókat, nudge-okat, törléseket, idővonal-szerkesztéseket és
csatornaszerkesztéseket (az `assert_unlocked` révén), a rétegre és a leszármazottaira. A renderelés és
a kiértékelés figyelmen kívül hagyja, a sima `Op`-ok (az „Editor ops” szakasz az ARCHITECTURE.md fájlban) pedig továbbra is érvényesülnek.

A közös mezők a kimenetben a fajta mezői után íródnak, de bemeneten a sorrend lényegtelen.

### 5.2 `vector`

| Mező | Típus | Alapérték | Jelentés |
| --- | --- | --- | --- |
| `paths` | útvonalak tömbje | `[]` | 6. szakasz. |

### 5.3 `group`

| Mező | Típus | Alapérték | Jelentés |
| --- | --- | --- | --- |
| `layers` | rétegek tömbje | `[]` | A gyermekek öröklik a csoport transzformációját. |

### 5.4 `switch`

| Mező | Típus | Alapérték | Jelentés |
| --- | --- | --- | --- |
| `layers` | rétegek tömbje | `[]` | Jelölt rajzok. |
| `selected` | integer | 0 | Kezdeti gyermekindex. Animáld a `layer_switch` tulajdonsággal (lépcsős kulcsok). |

Pontosan egy gyermek látszik. Az értéket kerekíti; a negatív 0 lesz; a gyermekszámmal egyenlő vagy annál nagyobb értékek
az utolsó gyermeket adják. A validáció nem utasítja el a tartományon kívüli kulcsokat. A lint ezzel kapcsolatos megfogalmazási
eltéréséről lásd a 20. szakaszt.

### 5.5 `liquid`

Metaball folyadék. A mező a blobokra vett `r^2 / d^2` összeg; a körvonal az izokontúr,
amelyet marching squares nyer ki legfeljebb 600 x 600 cellás rácson, és Catmull-Rom simít.

| Mező | Típus | Alapérték | Tartomány | Jelentés |
| --- | --- | --- | --- | --- |
| `blobs` | `{pos, radius, bone?}` tömbje | **Kötelező** | legalább 1; `radius > 0` | Egy blob. A `bone` csonthoz rögzíti. |
| `threshold` | number | 1 | `> 0` | Izoszint; az 1 az izolált blobot a sugarának megfelelő körré teszi. |
| `cell` | number | 3 | `>= 0.25` | Rácscella rétegegységben. |
| `fill` | `Paint` | null | | |
| `stroke` | stroke objektum | null | | |

### 5.6 `boolean`

| Mező | Típus | Alapérték | Jelentés |
| --- | --- | --- | --- |
| `op` | `union`, `intersect`, `difference`, `xor` | **Kötelező** | Művelet. |
| `paths` | útvonalak tömbje | **Kötelező**, legalább 2 | A `paths[0]` az alany; a vágás a többi uniója. |

Az eredmény az alany kitöltését és körvonalát veszi át; az egyszerűsítési tolerancia 0.25. Az eredményútvonalak
neve `<első útvonal neve><index>`.

### 5.7 `emitter`

`{"type": "emitter", "spec": {...}}`. A részecskék az idő zárt alakú függvényei, így bármely képkocka
kiértékelhető. Az idők másodpercben vannak.

| `spec` mező | Alapérték | Tartomány | Jelentés |
| --- | --- | --- | --- |
| `pos` | `[0,0]` | | Az emitter pozíciója (réteg-térben). |
| `bone` | null | | Csonthoz rögzít. |
| `rate` | 20 | `(0, 2000]` | Részecske másodpercenként. |
| `lifetime` | 1 | `> 0` | Másodperc. |
| `lifetime_jitter` | 0 | 0..1 | |
| `speed` | 60 | | Pixel másodpercenként. |
| `speed_jitter` | 0 | 0..1 | |
| `direction` | -90 | | Fok; a -90 felfelé. |
| `spread` | 30 | | A kúp szélessége fokban. |
| `gravity` | `[0,0]` | | Gyorsulás, pixel/másodperc². |
| `size_start`, `size_end` | 3, 3 | | Sugár az élettartam alatt. |
| `color_start`, `color_end` | `#fff`, `#fff0` | | Szín az élettartam alatt. |
| `seed` | 0 | | Determinisztikus seed. |
| `start` | 0 | | Kezdési idő másodpercben. |
| `duration` | null | | A kibocsátás ennyi másodperc után leáll; a null örökké. |
| `max_particles` | 2000 | | Szigorú felső korlát az élő részecskékre. |

### 5.8 `image`

| Mező | Típus | Alapérték | Tartomány | Jelentés |
| --- | --- | --- | --- | --- |
| `source` | `{file?, png_base64?}` | **Kötelező** | a kettő egyike | A `file` a projektkönyvtárhoz viszonyított útvonal (18. szakasz); a `png_base64` önálló. Egyik sem: `image has neither 'png_base64' nor 'file'`. |
| `origin` | `Vec2` | `[0,0]` | | A bal felső sarok réteg-térben. |
| `size` | `Vec2` vagy null | null (pixelméret) | | Megjelenített méret. |
| `mesh` | objektum vagy null | null | | Deformációs háló. |

`mesh`: `cols`, `rows` (rácsháló, mindkettő legalább 1), `weights` (csúcsonkénti csontsúlyok; az üres
statikusat jelent), és opcionálisan `free` = `{vertices, triangles, uvs}`. A szabad hálóhoz legalább 3
csúcs, legalább 1 háromszög, legfeljebb 200000 csúcs és 400000 háromszög, véges koordináták,
érvényes háromszögindexek kellenek, az `uvs` pedig vagy üres, vagy ugyanolyan hosszú, mint a `vertices`. Deformerekkel, de
háló nélkül egy 16 x 16-os rácsot használ.

```json
{"name": "img2", "type": "image", "source": {"png_base64": "iVBOR..."},
 "origin": [100, 0], "size": [80, 80],
 "mesh": {"free": {"vertices": [[100,0],[180,0],[180,80],[100,80]],
                   "triangles": [[0,1,2],[0,2,3]],
                   "uvs": [[0,0],[8,0],[8,8],[0,8]]}, "weights": []}}
```

## 6. Útvonalak, horgonypontok, paint, stroke

### 6.1 Útvonal (`paths[]`)

| JSON-név | Típus | Alapérték | Jelentés |
| --- | --- | --- | --- |
| `name` | string | `""` | Az üres nevet a hivatkozáshoz `path{index}` váltja fel. Nem tartalmazhat `::` jelet. A rétegen belül egyedi. |
| `closed` | bool | false | |
| `anchors` | array | **Kötelező** | Bézier-horgonypontok. |
| `fill` | `Paint` | null | |
| `stroke` | stroke | null | |
| `morphs` | `{name, anchors}` tömbje | üresen kimarad | Additív blend shape-ek. A horgonypontok számának meg kell egyeznie az alapútvonaléval. A súly a `path_morph` tulajdonsággal. |

### 6.2 Horgonypont

| JSON-név | Típus | Alapérték | Jelentés |
| --- | --- | --- | --- |
| `pos` | `Vec2` | **Kötelező** | Pozíció. |
| `handle_in`, `handle_out` | `Vec2` | null | Vezérlőpontok a `pos`-szal azonos térben (abszolút, nem relatív). A hiányzó fogantyú egyenlő a `pos`-szal (sarok). |
| `weights` | `{bone, weight}` tömbje | üresen kimarad | Skinning. Minden csontnak léteznie kell, és minden súly véges és `>= 0`. A súlyokat kiértékeléskor normalizálja; az üres statikus. A fogantyúk is skinelve vannak. |

### 6.3 Stroke

| JSON-név | Típus | Alapérték | Jelentés |
| --- | --- | --- | --- |
| `paint` | `Paint` | **Kötelező** | |
| `width` | number | 1 | |
| `cap` | `butt`, `round`, `square` | `round` | |
| `join` | `miter`, `round`, `bevel` | `round` | Ismeretlen érték ezt adja: ``unknown variant `foo`, expected one of `miter`, `round`, `bevel` ``. |
| `dash` | array | kimarad | Csak akkor használt, ha a hossza páros, legalább 2, és minden eleme `> 0`. |
| `dash_offset` | number | 0 | |
| `width_profile` | `[t, factor]` tömbje | kimarad | Szakaszonként lineáris szélességszorzó a hossz mentén (`t` 0..1), a végeken túl állandó, miter-határ 2. A változó szélességű stroke figyelmen kívül hagyja a `dash` és `join` értékeket. |

Példa sugaras színátmenetes kitöltésre körvonallal (a labdapéldából):

```json
"fill": {"center": [-8, -10], "radius": 44,
         "stops": [{"offset": 0, "color": "#ffd9a0"}, {"offset": 1, "color": "#d6336c"}]},
"stroke": {"paint": "#3d2c2e", "width": 2}
```

A linter (`osatura lint`) figyelmeztet a színátmenet-problémákra (L011), a switch-kulcsokra (L012) és a szerkesztői adatok problémáira (L013); lásd
`osatura-core/src/lint.rs`.

## 7. Deformerek, effektek, normáltérkép

### 7.1 `deformers[]`

Csak `vector` és `image` rétegeken engedélyezett. Mindegyiknek van `"type"` címkéje és `name` neve (a rétegen belül egyedi,
nincs benne `::`). A lista sorrendjében futnak nyugalmi térbeli geometrián, a skinning előtt és a
`wobble` előtt. Az eltolások statikusak; animáld őket az altárgy-csatornákkal.

| `type` | Mezők | Megszorítások | Animálás ezzel (cél) |
| --- | --- | --- | --- |
| `lattice` | `cols`, `rows`, `rect` `[x,y,w,h]`, `offsets` | `cols, rows >= 1`; `w, h > 0`; az `offsets` üres vagy `(cols+1)*(rows+1)` sorfolytonosan | `lattice_dx/dy` (`layer::name::c,r`) |
| `spine` | `rest` (görbepontok), `offsets`, `radius` (opcionális lágy él) | legalább 2 pont | `spine_dx/dy` (`layer::name::i`) |
| `pin` | `pins`, `offsets`, `radius` | 1–64 pin, `radius > 0`, a pinek nem esnek egybe | `pin_dx/dy` (`layer::name::i`) |
| `cage` | `rest` (sokszög), `offsets` | 3–256 csúcs, egyszerű sokszög, nem nulla terület | `cage_dx/dy` (`layer::name::i`) |
| `cloth` | mint a `lattice` | mint a `lattice` | `lattice_dx/dy` (`layer::name::c,r`) |

A lattice az eltolásokat Bézier-Bernstein-súlyokkal keveri (sima); a cloth minden rácscsúcsot
pontosan eltol (bilineárisan). A pin Wendland-féle kompakt tartójú radiális bázisfüggvényt használ. A cage
Floater-féle mean-value koordinátákat használ: a sokszögön belüli tartalom követi, a kívüli érintetlen marad.

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

Rétegenként legfeljebb 16. Minden effektnek van `"type"` címkéje, `enabled` mezője (alapérték true) és az alábbi mezői. Az
`fxN` csatornacél a tömböt indexeli, a letiltott effektekkel együtt.

| `type` | Mezők | Tartomány |
| --- | --- | --- |
| `blur` | `radius` | 0..64 |
| `drop_shadow` | `dx`, `dy`, `blur`, `color`, `opacity` (1) | az eltolások legfeljebb 1024; blur 0..64; opacity 0..1 |
| `tint` | `black`, `white`, `amount` (1) | amount 0..1 |
| `fill` | `color`, `opacity` (1) | opacity 0..1 |

Animáld az `fx_blur`, `fx_dx`, `fx_dy`, `fx_soft`, `fx_amount` tulajdonságokkal a `layer::fxN` célon.

### 7.3 `normal_map`

`{"data": base64-PNG | "path": file, "fit": "stretch" | "tile" | "exact" (alapérték stretch),
"strength": 0..8 (alapérték 1), "green_down": bool (alapérték false)}`. A `data` vagy a `path` szükséges. A
`relight` utóeffektet táplálja; a sima renderek nem használják. `Layer.normal_map`-ként az adott réteg pixel-határolódobozához illeszkedik. Az `exact` `fit` pontosan a képkocka méretű térképet követel, különben hiba.

## 8. Csatornák, sávok, easing

### 8.1 Csatorna

```json
{"target": "ball", "prop": "layer_y", "source": "keys",
 "track": {"keys": [{"frame": 0, "value": 40}, {"frame": 24, "value": 170}]},
 "mode": "replace", "weight": 1}
```

| JSON-név | Típus | Alapérték | Jelentés |
| --- | --- | --- | --- |
| `target` | string | `""` | Név, vagy `layer::sub` alak. A kamera-tulajdonságoknál figyelmen kívül hagyott. |
| `prop` | string | **Kötelező** | Az alábbi snake_case tulajdonságok egyike. |
| `source` | `keys`, `expr`, `noise`, `driven` | **Kötelező** | Lapított címke, mellette az adott forrás mezőivel. |
| `mode` | `replace`, `add`, `multiply` | `replace` | Hogyan kombinálódik az érték az aktuálissal. |
| `weight` | number | 1 | Befolyás, végesnek kell lennie; az eredmény felé keveredik. |

Kombinálás: a replace `cur + (v - cur) * w`; az add `cur + v * w`; a multiply
`cur * (1 + (v - 1) * w)`. A nem véges értékeket kihagyja. A csatornák a lista sorrendjében futnak, így egy későbbi
csatorna látja a korábbiak eredményét (a `get(...)` és a `driven` révén).

Források:

| `source` | Mezők |
| --- | --- |
| `keys` | `track`: `{keys: [{frame, value, ease?}], extrapolate?}` |
| `expr` | `expr`: string (9. szakasz) |
| `noise` | `amp`, `freq` (ciklus/másodperc), `seed` (0), `octaves` (1). A kimenet `[-amp, amp]` között; hash-alapú, determinisztikus. |
| `driven` | `input` (cél), `input_prop`, `curve`: egy sáv, amelynek „képkockái” bemeneti értékek (például könyökszög és dudorodás mértéke). |

### 8.2 Tulajdonságok

39 tulajdonság van. A tulajdonság dönti el, mit jelent a `target`.

| Csoport | Tulajdonságok | `target` |
| --- | --- | --- |
| Csont | `bone_angle`, `bone_scale`, `bone_offset_x`, `bone_offset_y` | csontnév |
| Réteg | `layer_opacity`, `layer_x`, `layer_y`, `layer_rotation`, `layer_scale_x`, `layer_scale_y`, `layer_skew`, `layer_switch`, `layer_wobble` | rétegútvonal (`parent/child`) |
| Kamera | `camera_x`, `camera_y`, `camera_zoom`, `camera_rotation`, `camera_focus`, `camera_blur` | figyelmen kívül hagyott |
| Rig | `ik_mix`, `constraint_weight` | ik- vagy kényszernév |
| Útvonal | `path_morph`, `point_dx`, `point_dy`, `grad_dx`, `grad_dy` | `layer::path::morphName`, `layer::path::anchorIndex`, `layer::path::fill.from` / `fill.to` / `stroke.from` / `stroke.to` |
| Deformer | `lattice_dx/dy`, `spine_dx/dy`, `pin_dx/dy`, `cage_dx/dy` | `layer::deformer::c,r` (lattice, cloth) vagy `layer::deformer::i` |
| Effekt | `fx_blur`, `fx_dx`, `fx_dy`, `fx_soft`, `fx_amount` | `layer::fxN` |

A színátmenet-fogantyúk jelentése: lineáris színátmeneteknél a `.from` és a `.to` a kezdő- és a végpontot mozgatja
`(dx, dy)` értékkel. Sugaras színátmeneteknél a `.from` a középpontot mozgatja, a `.to` pedig a sugarat változtatja `dx`-szel.
Csak azoknak az útvonalaknak vannak ilyen céljaik, amelyeknek a kitöltése vagy körvonala színátmenet. A `layer_switch` `switch`
réteget követel. Ismeretlen tulajdonságnál a hiba felsorol minden érvényes változatot.

Íráskori szorítások: opacity 0..1, a kamera zoomja legalább 0.01, a kamera blurja legalább 0, az `ik_mix` és
a `constraint_weight` 0..1.

### 8.3 Sáv és kulcs

`Track` = `{"keys": [...], "extrapolate": "hold"}`. A kulcs alakja `{"frame": number, "value": number,
"ease": {...}}`. A kulcsoknak végesnek és a képkockában szigorúan növekvőnek kell lenniük:
`channel N ('target'): keys must be finite and strictly increasing in frame`. A kulcson lévő ease
annak a szakasznak az alakját adja, amely ennél a kulcsnál indul.

Extrapoláció (az első és az utolsó kulcson kívül): `hold` (alapérték), `linear`, `loop`, `ping_pong`,
`loop_offset`.

### 8.4 Easing

Az `ease` egy `type` címkéjű objektum. Az alapérték `{"type": "linear"}`.

| `type` | Paraméterek | Görbe |
| --- | --- | --- |
| `linear` | | `u` |
| `step` | | Megtartja a kulcs értékét a következő kulcsig. |
| `ease_in` | | `u^2` |
| `ease_out` | | `1 - (1-u)^2` |
| `ease_in_out` | | Smoothstep. |
| `cubic_bezier` | `x1, y1, x2, y2` | CSS-stílusú görbe a (0,0) és (1,1) ponton át. |
| `smooth` | | Nem egyenletes Catmull-Rom Hermite a szomszédos kulcsokon át. |
| `spring` | `bounces`, `decay` | `1 - e^(-decay*u) * cos(2*pi*bounces*u) * window`, window = `1 - smooth01((u-0.85)/0.15)`; pontosan a célon áll meg. |
| `back` | `overshoot` | Harmadfokú görbe `3 + overshoot` kezdeti meredekséggel (legalább 0-ra szorítva). Az 1-es overshoot csúcsa 1.037 (u = 2/3); a 2-es overshoot csúcsa 1.1317 (u = 0.5556). |
| `bounce` | | Restitúció 0.6, három visszapattanás, a teljes időtartam 3.352 időegység. Az első becsapódás u = 0.2983-nál; a visszapattanási csúcsok 0.64, 0.8704, 0.953344; 1-nél ér véget. |

A `back` és a `bounce` eredeti levezetések (lásd `CLEANROOM.md`).

## 9. Kifejezések

A `"source": "expr"` csatorna képkockánként egy képletet értékel ki. A forrás legfeljebb 4096 bájt lehet, a beágyazási mélység legfeljebb 64; a hibákat
a validáció jelzi pozícióval, például
`channel 2 ('upper'): expression error: unexpected end of expression`.

Változók: `frame`, `time` (= frame / fps), `fps`, `frames`, `pi`, `t` (= frame / frames).

Operátorok (alacsonytól magas precedenciáig): `?:`, `||`, `&&`, összehasonlítások, `+ -`, `* / %`, egyoperandusú `- !`,
`^` (jobbasszociatív). A nullával osztás 0-t ad; a `%` euklideszi (`rem_euclid`).

Függvények: `sin cos tan asin acos atan atan2 abs sqrt pow min max clamp lerp smoothstep floor ceil
round fract sign mod exp ln noise` (1 vagy 2 argumentum) `fbm` (2 vagy 3) `wiggle` (2 vagy 3) `saw tri
square deg rad`, valamint `get("target", "prop")`. A nem véges függvényeredmény 0 lesz.

A `get("target", "prop")` bármely tulajdonság aktuális értékét olvassa, a korábbi
csatornák eredményeit is beleértve. Mindkét stringnek feloldhatónak kell lennie validáláskor (`channel 2: get("q", "bone_angle"): unknown
target`). Az értéket olvasó csatornának az azt író csatorna után kell állnia.

Példa a labdaprojektből:

```json
{"target": "shadow", "prop": "layer_opacity", "source": "expr",
 "expr": "clamp((get(\"ball\", \"layer_y\") - 40) / 130, 0.15, 1)"}
```

A kötési kifejezések (11. szakasz) megszorított alakot használnak: a változó a `value`, az időváltozók és a `get()` tiltottak.

## 10. Kamera, felvételek, hang, metaadatok

### 10.1 `camera`

| JSON-név | Típus | Alapérték | Tartomány | Jelentés |
| --- | --- | --- | --- | --- |
| `position` | `Vec2` vagy null | null (a színpad közepe) | | A képernyő közepén lévő világpont. |
| `zoom` | szám | 1 | legalább 0,01-re korlátozva | |
| `rotation` | szám | 0 | fok | |
| `focus` | szám | 1 | | Az a réteg-`parallax` mélység, amely éles (a `dof` és a `dof-export` használja). |
| `blur` | szám | 12 | véges, `>= 0` | Elmosási sugár pixelben, a parallax-távolság egységére vetítve. |

Parallax: a `parallax = p` értékű réteg ezt látja: `zoom = 1 + (zoom - 1) * p`,
`pos = centre + (pos - centre) * p`, `rotation = -rotation * p`. A nem véges `focus` vagy `blur`
ezt az üzenetet adja: `camera focus must be finite and camera blur a finite number >= 0`.

### 10.2 Metaadat-tömbök (betöltéskor nem validálja őket)

Ezek szerkesztői és eszközadatokat hordoznak. A motor adatként elfogadja őket; az ezeket
használó műveletek futáskor ellenőrzik őket.

| Tömb | Elemmezők |
| --- | --- |
| `markers` | `name`, `frame`, `end` (opcionális; megadva tartományt jelöl) |
| `comps` | `name`, `layers`: a rétegútvonalból `{visible, opacity, selected?}` értékre képező leképezés |
| `events` | `name`, `frame`, `data` (tetszőleges JSON) |
| `bone_vis` | `bone`, `from`, `to` (az a képkockatartomány, amelyben a csont látszik) |
| `channel_meta` | `target`, `prop`, `color`, `shy`, `solo`, `lock` (szerkesztői jelzők; a `lock` letiltja a set-key műveletet) |
| `shots` | `name`, `x`, `y`, `zoom`, `rotation` (mentett kameranézetek) |

```json
"markers": [{"name": "intro", "frame": 0, "end": 12}, {"name": "hit", "frame": 20}],
"events": [{"name": "step", "frame": 12, "data": {"sound": "step.wav"}}],
"shots": [{"name": "wide", "x": 200, "y": 150, "zoom": 1, "rotation": 0}]
```

### 10.3 `sounds[]`

| JSON-név | Típus | Tartomány | Jelentés |
| --- | --- | --- | --- |
| `name` | karakterlánc | egyedi | |
| `file` | karakterlánc | egy asset-útvonal (18. szakasz) | WAV vagy más támogatott hangformátum. |
| `start` | szám | másodperc | Elhelyezés az idővonalon. |
| `gain` | szám | 0..4 | |
| `trim_in`, `trim_out` | szám | `trim <= 86400` | A fájl elejéről és végéről levágott másodpercek. |
| `fade_in`, `fade_out` | szám | belefér a megvágott klipbe | Másodperc. |

A hiányzó fájlt a `check` jelzi (az `Engine::new` nem): `sound 's': cannot read sound file ./nofile.wav: ...`.

## 11. Klipek, skinek, kapcsolatok, interakció, lod-ok

### 11.1 `clips[]` és `clip_baked`

A klip egy újrahasznosítható `take`-et helyez az idővonalra. A klipek bake-elése hagyományos
csatornákat fűz hozzá (a kézzel készítettek után), és pillanatképet tárol a `clip_baked` mezőben, így az
újrabake-elés idempotens, a kézi munka pedig sosem törlődik.

| Klipmező | Jelentés |
| --- | --- |
| `name`, `start`, `speed`, `loops` | Elhelyezés. |
| `take` | `{name, length, tracks: [{target, prop, keys}]}` |
| `mode` | `replace`, `additive`, `multiply` |
| `weight`, `weight_keys` | Hatás; opcionális animált sáv. |
| `include`, `exclude` | A sávokat kiválasztó mintalisták. |
| `track_weights` | `[{pattern, prop, weight}]` |
| `fade_in`, `fade_out` | Képkocka. |
| `rename` | `[[from, to]]` célátnevezések. |

### 11.2 `skins[]`

`{name, group, layers, constraints, iks, bones}`. Az azonos `group` csoportba tartozó skinek egymást
kizáró változatok. A skinenkénti csontskálázás bake-eléskor egyetlen szorzó `bone_scale` lépéscsatornává olvad össze. A skinszabályokat
csak a skin-műveletek futásakor ellenőrzi a rendszer.

### 11.3 `mesh_locks[]` és `glues[]`

A glue-pár `VertexRef` = `{layer, path (default 0), index}` értéket használ; a mesh lock teljes útvonalakat nevez meg `{layer, path}` alakban, és opcionálisan követő csúcspont-indexek listáját adja meg (az üres lista minden csúcspontot jelent).

```json
"mesh_locks": [{"name": "lock", "host": {"layer": "hostm", "path": 0},
                "follower": {"layer": "follm", "path": 0}, "vertices": [], "strength": 1}],
"glues": [{"name": "g", "mode": "rigid", "strength": 1,
           "pairs": [{"a": {"layer": "hostm", "path": 0, "index": 0},
                      "b": {"layer": "s1", "path": 0, "index": 0}}]}]
```

A glue `mode` értéke `snap`, `rigid` vagy `offset`. Mindkettőt a `links::resolve` validálja, és
színpadtérben, topologikus sorrendben alkalmazza (a kiértékelés 12. lépése).

### 11.4 `machines[]` és `bindings[]` (csak futásidőben)

Az állapotgépeket és az adatkötéseket a Player futás közben vezérli. Exportáláskor és bake-eléskor
konstans `replace` csatornákká alakulnak, amelyeket a projekt végére fűz a rendszer. Minden gépnek legalább egy rétege kell legyen.

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

A bemenet `kind` értéke `bool`, `number` vagy `trigger`. Az állapot `source.kind` értéke `clip`, `segment`, `region`
vagy `blend`; a `mode` értéke `loop`, `once` vagy `ping_pong`. A `Transition` és a `Condition` elutasítja az ismeretlen
kulcsokat.

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

A kötés `weight` értékének 0..1 között kell lennie (`binding 'b': weight must be within 0..1`). A gépek és a kötések
céljait próbafordítással ellenőrzi a rendszer.

### 11.5 `containers[]` és `scrolls[]`

Flexbox-stílusú 1D elrendezés. A container mezői: `id`, `members`, `axis` (`row`, `column`), `gap`,
`cross_gap`, `padding` (4 szám), `justify`, `align`, `wrap`, `rect`. A `rect` egy `kind` mezővel rendelkezik:
`fixed` (`x,y,w,h`), `pin`, `aspect` vagy `hug`. A tagnál pontosan egy szerepel a `layer`, `container` vagy
`spacer` közül, továbbá `grow`, `shrink`, `width`, `height`, `align`, `resize`. Korlátok: 100000 tag,
64 szint mély egymásba ágyazás. A scroll mezői: `id`, `viewport`, `content`, `axis`, `offset`, `elastic`, `snap_x`,
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

A container eltolásai hozzáadódnak a réteg pozíciójához, és beleszorzódnak a skálába (a kiértékelés 7. lépése).
Az `apply_clip_masks` maszkhivatkozásokat ad a kapott projekthez, ezért módosítja a motor másolatát.

### 11.6 `lods[]`

Az `osatura lod` által írt részletességi szint (level-of-detail) menet feljegyzése: `level`, `tolerance_px`, `scale`,
`declared_px`, `bound_stage`, `path_px`, `mesh_px`, `keys_px`, `cull_px`, `stats`
(`path_anchors`, `mesh_vertices`, `keys`, `layers`, `bytes`) és `notes`. Dokumentációja annak, mit
végzett el egy menet; a renderer soha nem olvassa.


## 12. Validációs szabályok, korlátok és hibaüzenetek

Az `Engine::new` egy menetben validál, és az összes problémát visszaadja. A táblázat azt sorolja fel, ami kikényszerített;
az idézőjelek közti üzeneteket az `osatura check` szándékosan elrontott fájlokon állította elő.

### 12.1 Globális korlátok

| Szabály | Korlát | Üzenet |
| --- | --- | --- |
| fps | `(0, 1000]` | `fps must be within (0, 1000]` |
| frames | `1..=200000` | `frames must be within 1..=200000` |
| width, height | `> 0` | `width and height must be > 0` |
| width, height | `<= 32768` | `width and height must be at most 32768` |
| bármely lebegőpontos szám | `abs <= 1e7` | `value 100000000.0 at /bones/0/length is outside the supported range of +-1e7 (coordinates, scales, offsets and key values must stay below it)` |
| bones | legfeljebb 4096 | `at most 4096 bones` |
| format | `<= 1` | `project format 2 is newer than this build supports (1); update OSATURA` |
| JSON-méret | CLI `--max-input-bytes`, alapértelmezés 256 MiB; API `MAX_JSON_BYTES` 256 MiB | |

Az 1e7-es korlátot kétszer alkalmazza a rendszer. Betöltéskor a korlátot meghaladó lebegőpontos literál elbuktatja a
feldolgozást, az üzenetet pedig `invalid JSON in <file>: value ...` alakba csomagolja a rendszer. A tizedespont nélküli literál (egész szám)
átmegy a feldolgozáson, és az `Engine::new` újraellenőrzése fogja meg, ahol hagyományos listaelemként jelenik meg.
A korlát azért létezik, hogy egy későbbi szorzás ne csordíthassa túl az `f32` típust végtelenné.

### 12.2 Rig

| Szabály | Üzenet |
| --- | --- |
| a csont szülőjének korábban kell léteznie | `bone 'lower' has parent 'nope' which is missing or listed after it` |
| a csont hossza véges, `>= 0` | `bone 'lower' has invalid length` |
| `angle_limits` `min <= max`; `scale_limits` `0 < min <= max` | |
| az ik-nak 2 csont kell | `ik 'reach': needs at least two bones` |
| az ik-lánc szülői kapcsolattal összekötött | `ik 'reach': each bone must be a direct child of the previous one` |
| az ik célja nincs a láncban | |
| soft, stretch | `ik 'reach': soft must be within 0..0.9 and stretch within 1..4` |
| a kényszer nem célozhatja önmagát | |
| spring | `spring on 'upper': frequency must be in (0, 30] Hz and damping >= 0`; csontonként egy |

### 12.3 Rétegek, útvonalak, geometria

| Szabály | Üzenet |
| --- | --- |
| a réteg neve nem üres, nincs benne `/`, nincs benne `::` | `layer name 'a/b' must be non-empty and contain neither '/' nor '::'` |
| duplikált útvonal | `duplicate layer path 'arm'` (és sorok az ismétlődő animálható célokhoz) |
| az útvonal- és deformernevek nem tartalmaznak `::` jelet, és a rétegen belül egyediek | |
| a morph horgonyainak száma megegyezik az alap útvonaléval | |
| a skin-súlyok ismert csontokat neveznek meg; a súly véges, `>= 0` | |
| szabad mesh | legalább 3 csúcspont, legalább 1 háromszög, legfeljebb 200000 / 400000, véges, érvényes indexek, a `uvs` üres vagy azonos elemszámú |
| rácsos mesh | `cols`, `rows >= 1` |
| maszk | nincs önmaszkolás; `mask cycle involving layer 'arm'` |
| képforrás | `layer 'im': image has neither 'png_base64' nor 'file'` |
| liquid | legalább 1 blob; `threshold > 0`; `cell >= 0.25`; a blob `radius > 0` |
| boolean | legalább 2 útvonal |
| emitter | `rate` a `(0, 2000]` tartományban; `lifetime > 0`; a jitterek 0..1 |
| deformerek | csak vektoros és képrétegeken; az egyes típusokat lásd a 7.1 szakaszban |
| effektek | legfeljebb 16; blur és softness 0..64; az eltolások legfeljebb 1024; a mennyiségek 0..1 |
| normáltérkép | `strength` 0..8; `data` vagy `path` szükséges |
| kamera | `camera focus must be finite and camera blur a finite number >= 0` |

### 12.4 Csatornák

| Szabály | Üzenet |
| --- | --- |
| a cél feloldható | `channel 0: unknown target 'zzz' for BoneOffsetX` |
| `layer_switch` switch rétegen | `channel 2: layer_switch needs a switch layer, 'arm' is not one` |
| a súly véges | |
| a kulcsok végesek és szigorúan növekvők | `channel 0 ('target'): keys must be finite and strictly increasing in frame` |
| a kifejezés értelmezhető | `channel 2 ('upper'): expression error: unexpected end of expression` |
| a `get()` célpontjai és tulajdonságai feloldhatók | `channel 2: get("q", "bone_angle"): unknown target` |
| a vezérelt bemenet feloldható | |

Az ismeretlen `prop` vagy `ease` nevek serde-hibákat okoznak, amelyek felsorolják az összes érvényes változatot, például
``unknown variant `foo`, expected one of ...``. A hibás egész típusok ezt eredményezik:
``invalid type: floating point `48.0`, expected u32``. A hiányzó kötelező mező ezt eredményezi:
``missing field `frames` ``.

### 12.5 Egyéb validált szakaszok

- Sounds: `gain` 0..4, a `trim` legfeljebb 86400 s, a fade-ek beleférnek a megvágott klipbe, a nevek egyediek.
  A fájl létezését a `check` ellenőrzi, nem az `Engine::new`.
- Machines: legalább egy réteg (`machine 'm': needs at least one layer`); az átmenetek és a
  feltételek elutasítják az ismeretlen kulcsokat. Bindings: `binding 'b': weight must be within 0..1`. A céljaikat
  egyszeri lefordítással ellenőrzi a rendszer.
- Containers és scrolls: a tagok egyediek, legfeljebb 100000, beágyazási mélység 64, rect-szabályok.
- Mesh lock-ok és glue-k: a hivatkozásokat a `links::resolve` oldja fel.
- Kép- és hangfájlok: 18. szakasz.

### 12.6 Szerkesztői adatok: figyelmeztetések, hiba csak nem véges számokra

Az `Engine::new` és a `check` megvizsgálja a `skins`, `comps`, `markers`, `events`, `shots`, `channel_meta`, `bone_vis`, `clips` és
`lods` mezőket (`editordata` modul). Hogy minden korábban mentett projekt továbbra is betöltődjön, **csak az olyan értékek
hibák, amelyek mentett fájlban nem létezhetnek**: NaN vagy végtelen a markerekben, eseményekben, shotokban, a `bone_vis` mezőben, a comp átlátszatlanságában, a lods mezőben és a réteg átlátszatlanságában. Minden más
**figyelmeztetés**, amelyet a `check` kiír (és a `lint` L013 néven jelent): a kamera zoom értéke `<= 0` (a motor 0,01-re korlátozza), a réteg
`opacity` értéke a 0..1 tartományon kívül van (korlátozva), az `ik_mix` és a `constraint_weight` a 0..1 tartományon kívül van (korlátozva), a kulcsolt `layer_opacity` és
`camera_zoom` értékek a tartományukon kívül vannak, ismeretlen réteg-/csont-/csatornahivatkozások, duplikált nevek, az animáción kívüli időpontok,
érvénytelen skinek (`skins::validate`), klipek (`clipstack::validate`, valamint a nem feloldható take-sávcélok,
a `rename` figyelembevételével), `channel_meta` és `lods`. Az `out/` fixture-ök figyelmeztetés nélkül töltődnek be.

Az `osatura check` kiírja az ilyen jellegű lint-figyelmeztetéseket; a teljes linter (`osatura lint`) több szabályt tartalmaz (L001-től).

## 13. 1. példa: pattogó labda

Az `osatura check` (`ok: 'bouncing ball' 320×240 @ 24 fps, 48 frames, 0 bones, 0 ik,
3 layers, 5 channels`) és az `osatura render ball.json 7 out.png` parancs igazolta. Amit bemutat:

- Kulcsolt `layer_y` `bounce` easinggel az első szakaszon: a labda y=40-ről esik a
  padlóra y=170-nél, és háromszor visszapattan. Az első ütközés nagyjából a 7,2. képkockánál történik (24 / 3,352).
  A 24-től 47-ig tartó szakasz `ease_out` easinggel emelkedik vissza.
- A labdát követő árnyék: a `layer_x` és a `layer_opacity` kifejezések, amelyek a
  labdát a `get()` függvénnyel olvassák. A labda saját csatornái után szerepelnek, mert a csatornák sorrendben futnak.
- Összenyomódás érintkezéskor: egy kifejezés 0,8-at ad vissza a `layer_scale_y` számára, ha a labda y=160 alatt van.
- Kör négy Bezier-horgonyból, 16,57 kezelőhosszal (a 30-as sugár 0,5523-szorosa).

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

## 14. 2. példa: IK-s riggelt kar

Az `osatura check` igazolta (3 bones, 1 ik, 3 layers, 2 channels). Az `osatura explain arm.json
24` a hegyet a (170, 190) pontban jelenti, ami megegyezik a `target` csonttal, az `osatura render arm.json 24
out.png --bones` pedig megrajzolja a vázat. Amit bemutat:

- Az `upper` szögkorláttal rendelkező gyökér; a `lower` a gyermeke (a `rest_offset` `[90, 0]` megegyezik a
  szülő hosszával); a `target` szabad gyökércsont, amelyet az IK-lánc igyekszik elérni.
- A `reach` kétcsontos IK-lánc, ezért az analitikus megoldót használja a rendszer.
- A `sleeve` skinelt: minden horgony csontsúlyokat sorol fel. A `hand` és a `goal` a `bone`
  mezővel van csatolva.
- A `target` csont mozgatása a `bone_offset_x` és `bone_offset_y` csatornákkal a kart nyúlásra készteti.

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

## 15. 3. példa: switch-réteges száj

Az `osatura check` igazolta, és a 0., 5. és 9. képkocka renderelése, amely három különböző
szájat mutat. A tartományon kívüli kulcsok korlátozódnak: a 7-es kulcsérték a 2. gyermeket, a -3 a 0. gyermeket mutatja.

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

## 16. Kiértékelési sorrend

Az `Engine::evaluate(frame)` a képkockaszám és a projekt tiszta függvénye. A rugókat egyszer
szimulálja a rendszer a motor felépítésekor; semmi más nem hordoz állapotot a képkockák között. A lépések:

1. **Kezdeti állapot** (`initial_state`): statikus réteg-transzformációk, 0..1 közé korlátozott átlátszatlanság, a switch
   `selected` értéke, a wobble amplitúdója, kamera, `ik` mix, kényszersúlyok, valamint a
   paraméterhelyek kezdeti eltolásai (csonteltolások, deformer-eltolások, morph-súlyok stb.).
2. **Csatornák** (`apply_channels`) a lista sorrendjében, mindegyik a `mode` és a `weight` szerint kombinálódik az aktuális
   értékkel. A nem véges eredményeket átugorja. Az írások a 8.2 szakaszban felsoroltak szerint korlátozódnak.
3. **Korlát-szűkítés** (`clamp_limits`): csontonként `angle_limits` és `scale_limits`.
4. **Előremenő kinematika** (FK): az összes csont világmátrixa, a szülők elöl.
5. **IK-láncok** a lista sorrendjében, a `mix <= 0` láncokat átugorva. Minden lánc után a póz korlátozódik,
   és az FK újraszámolódik, így a következő lánc látja az eredményt.
6. **Kényszerek** a lista sorrendjében, a `weight <= 0` értékűeket átugorva: aim, copy_rotation, pin.
7. **Elrendezés-container-ek**: az eltolások hozzáadódnak a réteg x és y értékéhez, és beleszorzódnak a skálába.
8. **Bake-elt rugók**: a bake-elt szögek helyettesítik a csontszöget, majd az FK újraszámolódik.
9. **Skinmátrixok**: csontonként `skin_mat = world * inverse(rest world)`.
10. **Kamera**: gyökérrétegenként egy kameramátrix, a gyökérréteg `parallax` értékével (10.1 szakasz).
11. **Csomópontok** preorder bejárásban. A csomópont színpadmátrixa `parentStage * boneAttach(skin_mat) * local`.
    A vektorgeometria ebben a sorrendben épül fel:
    1. morphok (additív blend shape-ek);
    2. horgonyonkénti `point_dx/dy` eltolások;
    3. színátmenet-kezelők eltolásai (`grad_dx/dy`);
    4. deformerek a lista sorrendjében, majd a `wobble`;
    5. skinelés normalizált súlyokkal (az üres súlyok statikust jelentenek; a színátmenet-kezelők is
       skinelődnek).
    A skinelt geometria színpad-nyugalmi koordinátákban él, és a réteg transzformációja ezután kerül rá.
    A switch egy gyermeket mutat (kerekítve, korlátozva). A liquid, boolean és emitter geometriát
    képkockánként szintetizálja a rendszer. A kép rácsos vagy szabad meshet kap; deformerekkel, de mesh nélkül
    16×16-os rácsot kap.
12. **Mesh lock-ok és glue-k** színpadtérben, topologikus sorrendben.
13. **Nem véges értékek javítása** (`repair_nonfinite`): az a csomópont, amelynek geometriája NaN-t vagy végtelent tartalmaz,
    rejtetté válik, és a `Frame.nonfinite` beáll. Az `evaluate_checked` és a `check_all_frames_finite` ezt
    hibává alakítja.

Következmények, amelyekre építhetsz:

- A csatorna csak a listában előtte lévő csatornákat látja (a `get` és a `driven` révén).
- Az IK felülírja az FK-t a láncban lévő csontokra, a `mix` arányában (a keverés
  `old + wrap_pi(new - old) * mix`). `mix` 0 esetén a lánc tiszta FK.
- A kényszerek az IK után futnak, így az IK-csonton lévő aim kényszer nyer.
- A rugók utoljára hatnak a csontszögre, az IK és a kényszerek után.

## 17. Renderelési sorrend

- A gyökérrétegek a lista sorrendjében rajzolódnak, a gyermekek a szülőjükön belül a lista sorrendjében.
- A nem `visible` csomópontot kihagyja a rendszer (maszkforrás még lehet).
- A csomópont képernyőn kívüli rétegre renderelődik, ha `opacity < 1`, vannak effektjei, a keverési módja nem
  `normal`, vagy maszkja van. Lépések: tartalom; rétegeffektek a lista sorrendjében (kibővített vásznon); maszk
  (a maszkréteg a saját transzformációjával rajzolódik; az alfája, az átlátszatlanságát is beleértve, szorzódik;
  az `invert` megfordítja); összegzés az átlátszatlansággal és a keverési móddal.
- A mélységélességet csak a `dof` és a `dof-export` alkalmazza: sugár = `abs(parallax - focus) *
  blur`, alapértelmezés szerint legfeljebb 24 pixel.
- A csontréteg (`--bones`) rajzolódik utoljára.

## 18. Assetek és útvonalak

A projektek hivatkozhatnak fájlokra: `image.source.file`, `normal_map.path`, `sounds[].file`.

- Az útvonalaknak relatívnak kell lenniük, nem tartalmazhatnak `..` elemet, és a szimbolikus linkek követése után is a
  projekt könyvtárán belül kell maradniuk: `layer 'im': asset '/etc/passwd': absolute paths are not allowed (a project
  may only reference files inside its own directory; use --allow-external-assets to override)`, valamint
  `layer 'im': asset '../x.png': '..' is not allowed (...)`.
- Az `--allow-external-assets` egyetlen parancsra feloldja ezt a korlátozást. Csak szabályos fájlokat
  nyit meg a rendszer.
- Méretkorlátok: képek 64 MB a lemezen, hang 256 MB. Dekódolt képek: oldalanként legfeljebb 16384 és
  legfeljebb 64 megapixel.
- Kimeneti korlátok: raszter oldalanként legfeljebb 16384 és 100 megapixel; sorozatok legfeljebb 2 GiB; az exportkeret:
  képkockák száma szorozva a horgonyok számával legfeljebb 40 millió.
- Amikor a CLI projektet ír, az `Engine::new` révén újra validál, és az asset-útvonalakat a kimenet helyéhez
  képest relatívvá írja át.

Önálló fájlhoz a `png_base64` használatát részesítsd előnyben.

## 19. Verziókezelési szabályzat

- A `format` egész szám, jelenleg `1` (`FORMAT_VERSION`).
- Az a fájl, amelynek `format` értéke nagyobb, mint amit a build támogat, a 12.1 szakaszban szereplő üzenettel visszautasításra kerül;
  a motor nem találgat.
- Opcionális, alapértékkel rendelkező mező hozzáadása nem verzióváltás. A régebbi buildek figyelmen kívül hagyják (és a
  `check` ismeretlen mezőként figyelmeztet rá).
- Egy mező eltávolítása vagy újraértelmezése verziónövelést igényel, a növelés pedig migrációs lépést a
  `Project::from_json` függvényben, amely a validálás előtt frissíti a régebbi dokumentumokat.
- A Rust könyvtár API-jának saját verziója van, az `API_VERSION` 1.4.0, amelyet az `api_surface.txt` snapshot-teszt őriz; a C ABI
  `ABI_VERSION` értéke 1. Lásd: `docs/API.md`.

## 20. Ismert eltérések a kód és a megjegyzések között

Ezeket a leírás készítése közben találtuk.

1. **Javítva.** A switch-kulcs szabálya (`lint::switch_key_warnings`, L012) most azt mondja, amit a motor tesz: a 0 alatti kulcs a 0. gyermeket mutatja,
   a gyermekszámmal egyenlő vagy annál nagyobb kulcs az utolsó gyermeket mutatja (az üres switch semmit sem mutat). Teszt: `lint` egységtesztek, a motorral
   összevetve igazolva.
2. A `project.rs` megjegyzése szerint a `format` nélküli fájlok 1-es verziójúak, de a serde alapértéke a
   `FORMAT_VERSION`. Ma csak azért egyeznek, mert a konstans értéke 1. Ha a konstanst megemelik, a
   `format` nélküli fájlt a legújabb verziónak olvasnák.
3. **Javítva.** Az `unknownfields` már nem jelent kifejezetten leírt üres struktúrát (`clips[].weight_keys: {"keys": []}`);
   a `default_like` rekurzív, és egy teszt rögzíti az opcionális struktúrák körét, amelyeket az író üresen kihagy.
4. **Javítva (figyelmeztetésként).** A szerkesztői mezőket validálja a rendszer; lásd a 12.6 szakaszt.
5. **Javítva a 7. hullámban (L-71).** A skinelt réteg geometriája színpad-nyugalmi térben van, a szülő transzformációja pedig
   utána kerül rá. A `Layer.bone` mostantól `world = bone_skin * parent_stage * local` szerint csatlakozik: a skinmátrix (a csont világmátrixa × a nyugalmi mátrix inverze)
   színpadtérbeli mozgás, ezért a teljes szülőlánc után alkalmazódik. Nyugalmi helyzetben a skinmátrix az egységmátrix, így semmi
   sem mozdul; amikor a csont **pózban** van, a 2-es skálájú szülőcsoport már nem duplázza meg a csatolt réteg elmozdulását. A saját
   csont nélküli réteg a legközelebbi csonthoz csatolt őse csontját követi; a réteg saját csontja felváltja az ősét
   (a két mozgás nem halmozódik). Tesztek: `bone_attach_under_a_transformed_group_composes_bone_after_the_group`,
   `bone_attach_is_inherited_and_a_child_bone_replaces_the_parents`. Azok a projektek, amelyek a régi, megduplázott elmozdulásra támaszkodtak (póz
   csont skálázott vagy elforgatott csoport alatt), másként renderelődnek; az ilyen kombináció nélküli projektek változatlanok.
