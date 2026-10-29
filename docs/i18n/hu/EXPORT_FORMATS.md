<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# OSATURA export- és importformátumok

Mit őriz meg, mit közelít és mit hagy el az OSATURA által írt vagy olvasott összes fájlformátum, melyik parancs állítja elő, és hogyan
ellenőrizhető az eredmény. Ez a szöveg CC-BY-4.0 licencű. Dokumentált verzió: **osatura 0.1.0**.

Az alábbi parancssorok mindegyikét a beépített demóprojekten (`osatura demo demo.json`) vagy annak 8 képkockás másolatán
(`small.json`, ugyanaz a projekt `"frames": 8` beállítással) futtattuk. Az idézett méretek ebből a 8 képkockás, 640x480-as projektből származnak, és csak
tájékoztatásul szolgálnak: a képkockák, horgonyok és rétegek számával skálázódnak, nem valamilyen rögzített értékkel. A parancsok részletei (minden opció,
minden hiba) a [CLI_REFERENCE.md](CLI_REFERENCE.md) fájlban vannak; magának a projektnek a fájlformátuma a
[PROJECT_FORMAT.md](PROJECT_FORMAT.md) fájlban; a kodekek és a szabadalmak helyzete a [PATENT_NOTES.md](PATENT_NOTES.md) fájlban (nem jogi tanács).

## Tartalom

1. [Melyik formátum melyik feladatra](#1-melyik-formátum-melyik-feladatra)
2. [A kimeneti formátumok áttekintése](#2-a-kimeneti-formátumok-áttekintése)
3. [Állóképek: SVG, PNG, 16 bites PNG, OpenEXR](#3-állóképek)
4. [Raszteres animáció: PNG sprite sheet, GIF, APNG, WebM, MP4](#4-raszteres-animáció)
5. [HTML-lejátszó](#5-html-lejátszó)
6. [Lottie (.json) és dotLottie (.lottie)](#6-lottie-json-és-dotlottie-lottie)
7. [glTF bináris (.glb)](#7-gltf-bináris-glb)
8. [Open Cel Animation (OCA) mappa](#8-open-cel-animation-oca-mappa)
9. [Import: SVG, OpenRaster, Lottie, képek, GIF/APNG](#9-import-svg-openraster-lottie-képek-gifapng)
10. [Import: BVH mozgásrögzítés, MIDI és más élő bemenet](#10-import-bvh-mozgásrögzítés-midi-és-más-élő-bemenet)
11. [Hangformátumok](#11-hangformátumok)
12. [Funkciótámogatási mátrix](#12-funkciótámogatási-mátrix)
13. [Kisebb fájlok készítése: export-fit, lod, size-report](#13-kisebb-fájlok-készítése)
14. [Az ffmpeg követelményei](#14-az-ffmpeg-követelményei)
15. [Szabadalmi és licenc-mutatók](#15-szabadalmi-és-licenc-mutatók)
16. [Az írt fájlok ellenőrzése](#16-az-írt-fájlok-ellenőrzése)
17. [Eltérések más dokumentumoktól](#17-eltérések-más-dokumentumoktól)

---

## 1. Melyik formátum melyik feladatra

| Ezt szeretnéd... | Használd ezt | Miért |
|---|---|---|
| A pontos megjelenést, egyetlen fájlként megosztva, bármely böngészőben lejátszható, léptethető | `.html` | Az OSATURA saját renderere által rajzolt beágyazott SVG-képkockák; semmi sem közelített. |
| Kis vektoros animációt weboldalhoz vagy Lottie-lejátszós alkalmazáshoz | `.json` vagy `.lottie` | Vektoros, felbontásfüggetlen, kicsi; de lásd az elhagyott elemek listáját (képek, változó szélességű vonalak, hang...). |
| Ismétlődő előnézetet csevegéshez vagy README-hez | `.gif` | Mindenhol működik; képkockánként 256 szín, 1 bites átlátszóság, hang nélkül. |
| GIF-helyettesítőt teljes színnel és alfával | `.apng` | Veszteségmentes 8 bites RGBA; nagyobb, mint a GIF. |
| Videót hanggal | `.mp4` (H.264 + AAC) vagy `.webm` (VP9/AV1 + Opus) | Az `ffmpeg` telepítését neked kell elvégezned. A `.webm` megtartja az átlátszóságot. |
| Sprite sheet játékmotorhoz | `.png` | Egy PNG plusz egy JSON-atlasz a csempék téglalapjaival. |
| Csontvázat és mozgást 3D-eszközhöz | `.glb` | Csontok, animáció, lapos alakzatok háromszöghálóként. |
| Képkockákat kompozitáló/festő eszközökhöz | `export-oca` (Open Cel Animation) vagy `frames --png` | A cel-ek PNG-ként, egy mappában. |
| Lineáris fényű vagy nagy bitmélységű állóképeket | `render-hdr` (`.exr` vagy 16 bites `.png`) | Lebegőpontos renderer, 16 bites vagy half/float OpenEXR. |
| Egyetlen állóképet | `render` (`.svg` vagy `.png`) | Vektoros vagy 8 bites raszteres. |

## 2. A kimeneti formátumok áttekintése

| Formátum | Parancs | Kiterjesztés | Kell hozzá ffmpeg | Alfa | Hang | Méret a 8 képkockás demóra* |
|---|---|---|---|---|---|---|
| SVG (egy képkocka) | `render` | `.svg` | nem | igen | nem | néhány KB |
| SVG/PNG sorozat | `frames` | mappa | nem | igen | nem | képkockánként egy fájl |
| PNG (egy képkocka) | `render` | `.png` | nem | igen (8 bit) | nem | |
| 16 bites PNG | `render-hdr` | `.png` | nem | igen | nem | |
| OpenEXR (half/float) | `render-hdr` | `.exr` | nem | igen | nem | 466 KB half, 923 KB float (640x480) |
| PNG sprite sheet + JSON | `export` | `.png` (+ `.atlas.json`) | nem | igen | nem | |
| GIF | `export` | `.gif` | nem | 1 bit | nem | 83 KB |
| APNG | `export` | `.apng` | nem | igen | nem | 131 KB |
| WebM | `export` | `.webm` | igen | igen | igen (Opus) | 10,5 KB |
| MP4 | `export` | `.mp4` | igen | nem | igen (AAC) | 8,9 KB |
| HTML-lejátszó | `export` | `.html`, `.htm` | csak hanghoz | igen | beágyazott mp3 | 66 KB |
| Lottie | `export` | `.json` | nem | igen | nem | 47 KB |
| dotLottie | `export` | `.lottie` | nem | igen | nem | 11,5 KB |
| glTF bináris | `export` | `.glb` | nem | igen | nem | 115 KB |
| Open Cel Animation | `export-oca` | mappa | nem | igen | nem | cel-enként egy PNG |

\* A GIF, APNG, WebM és MP4 értékei `--scale 0.5` mellett (320x240); a vektoros formátumoké teljes méretben. A „Kell hozzá ffmpeg” azt jelenti, hogy működő
`ffmpeg` van a `PATH`-on (14. szakasz). A kimenet nevét a kiterjesztés alapján választja a rendszer, kis- és nagybetűkre való tekintet nélkül; az ismeretlen kiterjesztést elutasítja, és
semmit sem ír (`unknown export format '.xyz' (use .gif .html .json .mp4 .apng .webm .png .glb)`, a `.lottie` is
elfogadott).

Minden író megtagadja, hogy a bemeneti projektet felülírja a kimenettel, ideiglenes fájlon keresztül ír és átnevezi azt, és megtagadja a
szimbolikus linkeket, hacsak nem adod meg a `--follow-symlinks` kapcsolót (CLI_REFERENCE 1.4 szakasz). A vektoros exportálók méretvédelme
(Lottie, HTML, GLB) elutasítja azt az exportot, amely nagyjából 40 milliónál több értéket tartalmazna (képkockák × horgonyok, plusz képkockánként a csontok és rétegek)
`export would write about N million values ...` üzenettel.

## 3. Állóképek

### SVG (`render ... out.svg`, `frames`)

```
$ osatura render demo.json 12 f12.svg --bones
wrote f12.svg
$ osatura frames demo.json fr            # fr/frame_0000.svg ... frame_0047.svg
```
Az OSATURA saját rendererének vektoros kimenete: az útvonalak, színátmenetek, szélességprofilos vonalak, szaggatott vonalak, keverési módok, maszkok és réteg-
effektek SVG-ként íródnak. A `--bones` a csontváz-réteget rajzolja meg. `.svg` esetén a `--scale` nem változtatja meg a dokumentum viewBox értékét. **Ismert elhagyások/korlátok:** a renderer egyet sem jelent; az, hogy egy adott SVG-megjelenítő hogyan kezeli a keverési módokat és a szűrőket, az
a megjelenítőtől függ. A `--proxy N` egyszerűsített léptető proxyt ír, amely elhagyja a finom vonalakat, a színátmeneteket és a keverési módokat (nem
leszállítandó anyagnak való). **Ellenőrzés:** nyisd meg böngészőben, vagy `python3 -c "import xml.dom.minidom,sys; xml.dom.minidom.parse(sys.argv[1])" f12.svg`.

### PNG (`render ... out.png`, `frames --png`)

8 bites RGBA, sRGB, a motor saját rasztereződője rendereli. A `--scale N` a (0, 16] tartományban megszorozza a pixelméretet. Az
`"background": null` projekt átlátszóan renderelődik ott, ahol semmi sincs rajzolva. Alapértelmezés szerint nem színkezelt: az értékek sRGB-kódolt térben keverednek.
A 7. hullám óta a projekt lineáris fényű kompozitálást is választhat (`osatura color-space p.json out.json linear`, `color_space` mező):
a raszteres, GIF, APNG, videó és `render-hdr` kimenetek ekkor f32 lineáris fényben kompozitálódnak, és csak egyszer alakulnak sRGB-be; az SVG és
a Lottie nem változik (LIMITATIONS L-22).

### 16 bites PNG és OpenEXR (`render-hdr`)

```
$ osatura render-hdr small.json 3 f.exr                           # HALF, 640x480, 465,611 bytes
$ osatura render-hdr small.json 3 f2.exr linear --precision float # FLOAT, 923,211 bytes
$ osatura render-hdr small.json 3 f4.png                          # 16-bit/colour RGBA, sRGB-encoded
```
* **OpenEXR:** egyrészes, scanline, **tömörítetlen**, `A B G R` csatornák, lineáris fény, premultiplikált alfa. Alapértelmezés szerint `HALF`,
  `FLOAT` a `--precision float` (vagy `--float32`) megadásakor. A `linear` szónak nincs hatása EXR esetén (mindig lineáris).
* **16 bites PNG:** sRGB-kódolt, hacsak nem adod meg a `linear` szót.
* **Hűség:** a lebegőpontos renderer nagyobb pontosságot hordoz, mint a 8 bites, de a grafika hagyományos sRGB-szín, így a fájl
  megjelenítési tartományú értékeket tartalmaz nagyobb pontossággal, nem jelenetalapú HDR fényt. A projekt háttere nincs megrajzolva.
* **Ismert elhagyások/korlátok:** fájlonként egy képkocka; nincs tömörítés, réteg vagy AOV; nincs ICC-profil; a `--8bit` a 8 bites
  rasztereződőre esik vissza forrásként.
* **Ellenőrzés:** az `ffprobe f.exr` ezt jelenti: `exr 640x480 gbrapf32le`; bármely EXR-megjelenítő megnyitja.

## 4. Raszteres animáció

Ezek mindegyike előbb a raszteres motoron renderel minden képkockát. Az `"background": null` projekt átlátszó
pixelekkel renderelődik mindenhol, ahol semmi sincs rajzolva; a demónak háttérszíne van (`#f6f1e7`), ezért átlátszatlan. A `--scale N` a (0, 16] tartományban
a `.gif .apng .webm .mp4 .png` formátumokra vonatkozik; más formátumokon elutasított (`unknown option --scale for 'export .json'`).

### PNG sprite sheet (`export ... out.png`)

```
$ osatura export small.json out3.png --columns 4 --scale 0.25
wrote out3.atlas.json
wrote out3.png                    # 4 columns x 2 rows of 160x120 tiles
```
Egy PNG, amely az összes képkockát csempeként tartalmazza (`--columns N`, alapértelmezés a közel négyzetes rács: ceil(sqrt(frames)) oszlop), valamint egy
`out3.atlas.json` atlasz (a kimeneti név a kiterjesztés `.atlas.json`-ra cserélésével), amely képkockánként `{index, x, y, w, h}` értéket sorol fel. Az alfa megmarad.
Az atlasz korábban `out3.json` volt, és csendben felülírhatott egy azonos nevű Lottie-fájlt; az új név ezt elkerüli, és az atlasz útvonala ugyanazokon a bemeneti/szimbolikus link védelmeken megy át.

### GIF (`export ... out.gif`)

```
$ osatura export small.json out.gif --scale 0.5
wrote out.gif                      # 8 frames, 83 KB
```
* Forrásképkockánként egy kép, **végtelen ismétléssel**, képkockánként legfeljebb 256 színű helyi paletta (kvantált, dithering nélkül).
* **Időzítés:** a GIF-késleltetések egész századmásodpercek, és a megjelenítők a 2 cs alattit 10 cs-nak veszik, így 50 fps a valódi maximum.
  50 fps-ig a késleltetések hibadiffúziósak (29,97 fps esetén 3 és 4 cs váltakozik), így a teljes időtartam 1 cs-on belül marad a
  `frames / fps` értéktől. 50 fps felett a sorozatot **50 fps-re ritkítja a rendszer**, az időtartam megmarad, és figyelmeztetés íródik ki
  (`GIF cannot hold 60 fps ... use APNG, WebM or MP4 for the full rate`).
* **Átlátszóság:** 1 bites (az átlátszó szín jelzője be van állítva); a félig átlátszó pixelek nem őrződnek meg.
* **Ismert elhagyások:** nincs hang (`note: GIF cannot carry audio; ...`, ha a projektnek vannak hangjai); nincs részleges alfa; a 65535 pixelnél szélesebb
  vagy magasabb képeket elutasítja (`image too large for GIF`).
* **`--report r.json`:** kiírja azoknak a dolgoknak a listáját, amelyeket a formátum nem tudott hordozni (itt: a képkockasebesség-ritkítás, ha volt).
* **Mérettippek:** kisebb `--scale`, kevesebb képkocka, vagy `export-fit --format gif` (13. szakasz). A GIF a legkevésbé hatékony a raszteres
  formátumok közül fényképszerű vagy színátmenetekben gazdag grafikához.
* **Ellenőrzés:** az `ffprobe -v error -count_frames -select_streams v -show_entries stream=nb_read_frames -of csv=p=0 out.gif` kiírja a
  képkockák számát (8).

### APNG (`export ... out.apng`)

Animált PNG, 8 bites RGBA, veszteségmentes, projektképkockánként egy képkocka a projekt fps értékén (24 fps-nél 0,0417 s képkockánként). A teljes alfa megmarad. Hang nincs. Lapos grafikához nagyobb,
mint a GIF (131 KB a 83 KB-tal szemben a példában), mert nincs paletta. A legtöbb böngésző lejátssza az APNG-t; néhány eszköz nem,
ilyenkor használj WebM-et vagy GIF-et. **Ellenőrzés:** `ffprobe` a `-count_frames` kapcsolóval (8 képkocka a példára).

### WebM (`export ... out.webm`) és MP4 (`export ... out.mp4`)

```
$ osatura export small.json out.mp4  --scale 0.5     # h264, yuv420p, 320x240, 8 frames
$ osatura export small.json out.webm --scale 0.5     # vp9, yuv420p + alpha_mode=1
$ osatura export small.json out.webm --codec av1     # needs an ffmpeg with libaom-av1
```
* **MP4:** H.264 (`libx264`), `yuv420p`, **alfa nélkül**. AAC hang, ha a projektnek vannak hangsávjai.
* **WebM:** VP9 (`libvpx-vp9`, alapértelmezett) vagy AV1 (`libaom-av1`, `--codec av1`); az alfacsatorna megmarad (`yuva420p`, a konténer címkéje:
  `alpha_mode=1`); Opus hang (`libopus`), ha a projektnek vannak hangjai. Bármely más `--codec` érték: `unknown --codec 'foo' (vp9 or av1)`.
* **Hang:** a projekt hangsávjai összekeverésre kerülnek (kezdet, erősítés, vágás, fade-ek; a projekt hosszánál elvágva), és muxolódnak.
* **Kell hozzá ffmpeg:** a képkockák PNG-ként renderelődnek egy privát ideiglenes könyvtárba, átadódnak az `ffmpeg`-nek, majd törlődnek. Ha
  az `ffmpeg` hiányzik: `cannot run ffmpeg (...); install it or export a GIF/HTML instead` (1-es kilépési kód, semmi sem íródik). Ha hiányzik
  az enkóder: `ffmpeg failed`.
* **Ismert korlátok:** csak 8 bit; a videó mérete a projekt méretét és a `--scale` értéket követi (az eredményt ellenőrizd az `ffprobe`-bal).
  A `dof-export` is ír `.mp4`/`.webm` fájlt, hang nélkül.
* **Ellenőrzés:** az `ffprobe -v error -show_entries stream=codec_name,width,height,pix_fmt,nb_frames,r_frame_rate -of csv=p=0 out.mp4`
  ezt írta ki: `h264,320,240,yuv420p,24/1,8`.
* **Szabadalmak:** lásd a 15. szakaszt; a H.264-re és az AAC-re licencelési közösségek vonatkoznak, a VP9/AV1 nincs értékelve.

## 5. HTML-lejátszó

```
$ osatura export small.json out.html
wrote out.html                     # 66 KB for 8 frames
```
Egyetlen önálló HTML-fájl: **minden képkocka beágyazott SVG-ként**, lejátszás/szünet gomb, időcsúszka (range input) és képkockafelirat,
egyetlen beágyazott `<script>` vezérli. **Nem küld hálózati kérést**: a fájlban az egyetlen URL az SVG-névtér. A megnyitásához
semmi más nem kell, csak egy böngésző. A kép pontosan az, amit az SVG-renderer előállít, ezért ez a leghűbb a „lejátszható”
formátumok közül (a maszkok, keverési módok, effektek, csontok, IK, deformerek, részecskék mind a képkockákba vannak sütve).

* **Hang:** ha a projektnek vannak hangjai, a kevert hang mp3 data URI-ként van beágyazva (`ffmpeg` kell hozzá; 3 MiB felett figyelmeztetés, a
  hang 16 MiB felett figyelmeztetéssel elmarad).
* **Méret:** lineárisan nő a képkockák számával (itt képkockánként kb. 7,7 KB) és a vektoros részletességgel; a fájl nincs tömörítve. Tipikus
  tippek: szolgáld ki gzip-pel (a 66 KB-os példa gzip-pel kb. 9 KB), csökkentsd a képkockák számát vagy az útvonalak részletességét a `lod` paranccsal, és használd az
  `export-fit --format html` parancsot.
* **Korlátok:** a `--report` elutasított (`unknown option --report for 'export .html'`);
  a vektorexport-méretvédelem érvényes; a lejátszó csak lejátszást/szünetet és időcsúszkát kínál.
* **Ellenőrzés:** nyisd meg; a `grep -c '<svg' out.html` megszámolja a képkockákat; a `grep -o 'http[^"<]*' out.html` csak az SVG-névteret listázhatja.

## 6. Lottie (.json) és dotLottie (.lottie)

```
$ osatura export small.json out.json --report rep.json
wrote out.json
warning: [variable-width-stroke] ink: variable-width stroke profiles are not supported by Lottie; the constant base width is used
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
$ osatura export small.json out.lottie       # a zip: manifest.json + a/animation.json
```

### Felépítése

A fájl Bodymovin (`"v": "5.7.0"`), ahol `fr` = a projekt fps értéke, `ip` = 0, `op` = a képkockák száma és `w`/`h` = a színpad mérete. **Minden
vektoros réteg egy alakzatréteggé válik, amelynek útvonal-geometriáját képkockánként világtérbe sütik (hold kulcsképkockák)**, így a csontok, IK,
skinelés, deformerek, rugók, részecskék, a kamera (a parallaxot is beleértve) és a szülő-transzformációk mind *benne vannak* a geometriában. Az eredmény
úgy játszik, mint az eredeti, de nem szerkeszthető rig: nincsenek csontok, csatornák, kifejezések. Egy tömör háttérréteg
íródik, ha a projektnek van háttérszíne. Következésképpen a fájl a `frames x anchors` szorzattal nő; sokkal nagyobb, mint egy
kézzel épített, transzformációs animációjú Lottie (47 KB a 8 képkockás demóra, 11,5 KB egy `.lottie` fájlban).

### Exportálva

Tömör és színátmenetes kitöltések és vonalak (alfás színátmeneti megállók; a színátmenet végpontjai követik az animált réteg-transzformációt), szaggatott
minták, végzáródások, csatlakozások és a miter-határ, a réteg skálájával skálázott vonalszélességek, keverési módok, réteg-átlátszatlanság, alfa és fordított alfa
track matte-ek a rétegmaszkokhoz (a maszkforrás másolata szolgál matte-ként), rétegeffektek (blur, drop shadow, tint, fill) egyetlen
levélből álló rétegeken, és a **csoport átlátszatlansága előkompozícióként**, hogy az egymást fedő gyermekek ne látszódjanak át egymáson. A számok
a legrövidebb tizedes alakjukban íródnak.

### Figyelmeztetéssel jelzett (sosem hagyja el csendben)

Az alábbi elemek mindegyike a stderr-re íródik `warning: [code] layer: message` alakban, és a `--report FILE` fájlba mint
`{"format":"lottie","count":N,"warnings":[{"code","layer","message"}]}`:

| Kód | Jelentés |
|---|---|
| `image-layer` | A képrétegek nincsenek exportálva (a Lottie-kimenet csak vektoros rétegeket tartalmaz). |
| `variable-width-stroke` | A szélességprofilok nem támogatottak; az állandó alapszélesség van használva. |
| `stroke-anisotropic` | Nem egyenletes skála vagy ferdítés a rétegen; a Lottie-vonalak egyenletesek, ezért a szélességet a geometriai középpel skálázza a rendszer. |
| `gradient-anisotropic` | Az ilyen rétegen lévő színátmenet közelített (a Lottie-színátmenetek nem nyírhatók). |
| `blend-add` | Az `add` keverési módnak nincs Lottie-megfelelője; a `screen` van használva. |
| `group-blend` | A csoporton lévő keverési mód nincs exportálva. |
| `group-mask` | A csoporton lévő maszk nincs exportálva (az egyes rétegeken lévők igen). |
| `group-effect` | A (több rétegből álló) csoporton lévő rétegeffektek nincsenek exportálva. |
| `mask-source-not-vector` | A maszkforrás nem vektoros réteg, ezért a maszk nincs exportálva. |
| `depth-of-field` | A `camera_focus`/`camera_blur` csatornák nincsenek alkalmazva; használd a `dof-export` parancsot. |
| `audio` | A Lottie nem tud hangot hordozni; N hangsáv nem szerepel benne. |

### Lottie-megfelelő nélkül elhagyva

Csontok, IK, csatornák, állapotgépek, adatkötések, container-ek, görgetési régiók, markerek/események, hangok, utóeffekt-láncok (`fx`),
normáltérképek és a rig szerkeszthető szerkezete. A rétegenkénti effektszínek nem animálhatók (az első érték van használva), és a
méretek színpadegységben vannak (LIMITATIONS L-23).

### dotLottie (`.lottie`)

Egy [dotLottie 2.0](https://dotlottie.io/spec/2.0/) zip `manifest.json` fájllal (`{"animations":[{"id":"animation"}],"generator":"OSATURA","version":"2"}`) és
`a/animation.json` fájllal (ugyanaz a Lottie, mint fent, deflate-tel tömörítve: 46 843 bájtból 11 515 bájtos fájl lett). A `--report` ugyanúgy
működik. A `.lottie` **importálása** nem támogatott (az `import-lottie` JSON-t igényel; előbb csomagold ki az `a/animation.json` fájlt). A v0.1.0-alpha.4-ig a konténer dotLottie 1.0 volt (`animations/animation.json`, `autoplay`/`loop` a jegyzékben); a 2.0 elrendezést a jelenlegi dotLottie lejátszók lejátsszák (@lottiefiles/dotlottie-web-bel ellenőrizve).

### Ellenőrzés és fenntartások

* `python3 -m json.tool out.json > /dev/null` a jólformáltság ellenőrzéséhez; `python3 -c "import json;d=json.load(open('out.json'));print(d['v'],d['fr'],d['op'],len(d['layers']))"`.
* Az `unzip -l out.lottie` pontosan a két bejegyzést kell mutassa.
* Körbejárás: az `osatura import-lottie out.json back.json`, majd az `osatura check back.json` igazolja, hogy az OSATURA el tudja olvasni a saját
  kimenetét (a körbejárás veszteséges: a rig elvész).
* **Független Lottie-lejátszó nem állt rendelkezésre** az írás idején (LIMITATIONS L-28): a kimenetet a Lottie mezőszerkezetével és
  az OSATURA saját importerével vetettük össze. Tesztelj a célodnak megfelelő lejátszóban (lottie-web, dotLottie player, Skottie, ...), és olvasd el a
  figyelmeztetéseket; ha a pontosság fontos, használj `.html` fájlt vagy videót.
* **Mérettippek:** az `export-fit --format lottie` négy paramétert (`decimals`, `path_eps`, `key_tol`, `frame_step`) keres egy bájtkeretnek megfelelően;
  a `lod` az export előtt csökkenti a horgonyok számát; a `size-report` rétegenként mutatja a bake-elt Lottie méretét (13. szakasz).
* **Szabadalmak:** a Lottie és a dotLottie nyílt formátumok; a PATENT_NOTES.md semmit sem tartalmaz róluk.

## 7. glTF bináris (.glb)

```
$ osatura export small.json out.glb
wrote out.glb
warning: [glb-limit] blend mode Screen of 'sun' is not exported (glTF materials only alpha-blend); it is drawn as normal
warning: [glb-limit] animated opacity of 'target' is not exported (frozen at frame 0)
```
glTF 2.0 bináris (`glTF` magic, 2-es verzió, `"generator": "osatura"`), csak az alapfunkciók. A projektnek rendelkeznie kell csontokkal
(`the project has no bones to export` egyébként).

* **Pontos:** a csonthierarchia és animációja (minden képkockán mintavételezve, lineáris interpoláció, a nyírás elhagyva).
* **A kitöltések** lapos háromszöghálóvá válnak (ear clipping). Az alakzatok merevek (a csonttal rendelkező réteg követi a csontcsomópontot) vagy **skinelt**ek
  (JOINTS_0/WEIGHTS_0, csúcspontonként 4 hatás), ha minden horgony csontsúlyokat hordoz.
* **A vonalak** saját hálókká háromszögelődnek (szélesség, szélességprofil, butt/round/square végek, miter/round/bevel csatlakozások, szaggatás);
  az áttetsző vonalak sötétebbnek tűnhetnek ott, ahol a háromszögek átfedik egymást.
* **A színátmenetek** (lineáris és sugaras) egy kis generált PNG-vé válnak, amely alapszín-textúraként szolgál; pad (clamp) szétterülés.
* **A maszkok** a háromszögek maszkréteg kitöltésével való vágásával vannak bake-elve, csak átlátszatlan kitöltésű, vonal nélküli vektoros maszkforrásra,
  és csak a 0. képkocka maszkalakzatára. A **képrétegek** textúrázott négyszögekké vagy deformációs hálókká válnak beágyazott PNG-vel.
* **Az útvonal-morphok** animált súlyú morph targetekké válnak, ha a képkockánkénti ellenőrzés reprodukálja a motor geometriáját; egyébként
  az útvonal statikus, 0. képkockás alakzat marad, és az okot jelzi a rendszer.
* **Nincs exportálva (`[glb-limit]` figyelmeztetésként jelezve):** a normáltól eltérő keverési módok (normálként rajzolva), az animált rétegátlátszatlanság (a 0. képkockán
  befagyasztva), a nem csont rétegtranszformáció-animáció, a rétegeffektek, a deformerek a 0. képkockás alakzatukon túl, az érvényes
  morph nélküli animált útvonalak, az animált maszkalakzatok, a hang.
* **Koordináták:** az y negált (a glTF y-fel felfelé áll); az alakzatok kis z-eltolással (alakzatonként 0,01) rétegződnek a rajzolási sorrend érdekében.
* **Ellenőrzés:** a fejléc `glTF`, 2-es verzió, és a hosszmező megegyezik a fájlmérettel (`python3` egysoros a 16. szakaszban). A teljes ellenőrzéshez használj
  tetszőleges glTF-validátort vagy -megjelenítőt; egyik sincs mellékelve, és ehhez a dokumentumhoz egyiket sem futtattuk. A fenti példában 22 csomópont,
  15 háló, 1 skin, 1 animáció és 7 beágyazott kép van.
* **Mérettippek:** kevesebb vektoros horgony (`lod`), színátmenetek nélkül (mindegyik egy textúra), kevesebb képkocka (az animáció képkockánként mintavételezett).
* **Szabadalmak:** nyílt formátum; a PATENT_NOTES.md-ben nincs róla utalás.

## 8. Open Cel Animation (OCA) mappa

```
$ osatura export-oca small.json oca1                 # one layer "render" with all composited frames
$ osatura export-oca small.json oca2 --layers        # one OCA layer per top-level layer
wrote oca2/small.oca
```
Kiírja az `<out_dir>/<project name>.oca` fájlt (JSON, `ocaVersion` 1.1) és a `layerN/fNNNN.png` PNG cel-eket. A `--layers` nélkül egyetlen `render`
réteg van a kompozitált képkockákkal; a `--layers` mellett minden legfelső szintű réteg külön, átlátszóságon renderelődik (11 réteg a demóhoz).
A `--scale N` megszorozza a cel méretét. **Ismert elhagyások:** a cel-ek rasztereskek (vektorok, csontok és csatornák nem maradnak meg); a réteg átlátszatlansága és a keverés
nem íródik ki (minden réteg `paintlayer`, átlátszatlanság 1); a hangok nincsenek exportálva; egy meglévő mappában lévő fájlok felülíródnak.
**Import:** a CLI-ben nincs `import-oca` parancs. **Ellenőrzés:** az `.oca` fájl JSON (`python3 -m json.tool`); minden felsorolt cel-nek
léteznie kell PNG-ként.

## 9. Import: SVG, OpenRaster, Lottie, képek, GIF/APNG

Az importok **új** projektet hoznak létre (nem olvadnak bele egy meglévőbe), hacsak másként nincs jelezve; az egyesítéshez használd a `script` parancsot vagy szerkeszd a JSON-t.

### SVG (`import-svg`)

```
$ osatura import-svg art.svg svg.json
warning: unsupported paint 'url(#g)'; using grey
warning: <text> is not supported and was skipped
wrote svg.json (3 paths)
```
Részhalmaz: `path` (minden parancs), `rect` (a lekerekített sarkok figyelmen kívül hagyva), `circle`, `ellipse`, `line`, `polyline`, `polygon`, `g`/`a` `transform`-mal,
tömör `fill`, `stroke`, `stroke-width`. Eredmény: 24 fps, 24 képkocka, a méret a `viewBox`-ból vagy a `width`/`height` értékből (egyébként 512x512), **egyetlen vektoros
réteg, `svg`**. `warning:` sorral átugorva (a **stdout**-ra írva): `<text>`, `<filter>`, `<clipPath>`, `<mask>`, `<pattern>`,
`<linearGradient>`/`<radialGradient>` (a színátmenetes festék szürkére esik vissza), `<image>`, `<use>`, `<foreignObject>`. Korlátok: 8 MB SVG,
200 000 horgony. Fényképekhez és rasztereshez használd a `trace` / `autotrace` parancsot (CLI_REFERENCE 9. szakasz).

### OpenRaster (`import-ora`)

```
$ osatura import-ora art.ora ora.json --fps 12 --frames 24
```
**Rasztereréteg-enként egy képréteg** (beágyazott PNG), alulról felfelé sorrendben, a rejtett rétegek megmaradnak, de láthatatlanok, a csoportok laposítva, rétegenkénti
átlátszatlanság, láthatóság és pozíció, valamint a gyakori `svg:` kompozitműveletek (multiply, screen, overlay, darken, lighten, color-dodge,
color-burn, hard-light, soft-light, difference, exclusion, a plus az add-ra leképezve). A beágyazott köteg átlátszatlansága beleszorzódik a rétegeibe, és a keverési módja
öröklődik, de az áttetsző köteg belsejében lévő átfedő rétegek **nem** laposodnak előbb össze. Az ismétlődő nevek `_1`, `_2` utótagot kapnak.
Korlátok: 2000 archívumbejegyzés, bejegyzésenként 128 MiB, 64 Mpixeles rétegek. A nem zip fájl: `not a zip/ORA file: ...` (1-es kilépési kód). ORA-export
nincs. A PSD-import nincs megvalósítva (LIMITATIONS L-32).

### Lottie (`import-lottie`)

```
$ osatura import-lottie in.json out.json [fx.json]
warning: layer 'text' of type 5 skipped
imported 1 layers; wrote out.json
```
Szándékos **részhalmaz**; minden kihagyott funkció egy `warning:` sor.
* **Importált:** `sh` útvonalú alakzatrétegek, `rc`, `el`; tömör és színátmenetes kitöltések (az animált színátmenet-végpontok/középpont/sugár
  `grad_dx`/`grad_dy` csatornákká válnak); vonalak; csoportok; réteg-transzformációk (statikus vagy kulcsolt pozíció, skála, forgatás, átlátszatlanság; az ívelt pozíció-
  útvonalak képkockánként bake-elve); keverési módok; tömör háttérréteg; képrétegek beágyazott PNG/JPEG data URI-kkal; null rétegek szülőként
  (transzformációik a gyermekekbe bake-elve); rétegmaszkok (add/subtract/intersect átlátszatlansággal, invertálással, kulcsolt alakzatokkal); luma matte-ek
  (közelítve); előkompozíciók; a réteg be/ki tartományai (átlátszatlansági lépéssávként); kulcsképkocka-easing (a hold lépéssé, a bezier-kezelők
  cubic-bezier-ré válnak, a nincs lineárissá).
* **Animált útvonalak:** az azonos topológiájú alakzatkulcsok (azonos horgonyszám és nyitott/zárt állapot) egy alap útvonallá, valamint minden további különböző kulcshoz egy **morph target**
  (`k1`, `k2`, ...) lesznek, `path_morph` súlycsatornákkal, amelyek az easinget hordozzák. A különböző topológia az első alakzatot használja,
  és figyelmeztetést ír ki.
* **Effektek (`ef`):** a Gaussian blur, drop shadow, tint és fill **rétegenkénti effektekké** válnak, az animált számok `fx_*` csatornákként;
  az alakzatrétegeken lévő statikus tint/fill/tritone/levels színekbe van bake-elve. Az effektszínek nem animálhatók (az első érték van használva); a többi
  effekt figyelmeztetés.
* **Figyelmeztetéssel kihagyva:** szöveg és más rétegtípusok, hang-assetek, animált horgonypontok (az első érték van használva), az ívelt mozgásútvonalak a
  pozíciótól eltérő tulajdonságokon, track matte forráson lévő maszkok, a bake-eléshez túl hosszú szülőláncok, 2000-nél több réteg, 20 000 képkockánál hosszabb
  animáció.
* Az `fx.json` kompatibilitási okokból megmarad: üres tömb íródik.
* **Nem támogatott:** `.lottie` zip bemenet. **Csapda:** nincs felülírás elleni védelem a bemenettel szemben: az `import-lottie in.json in.json` a
  Lottie-t a projekttel cseréli le.

### Képek, GIF és APNG képkockánkénti rétegként

```
$ osatura import-seq demo.json is.json frames seq hold=2 loop=on start=4 x=10 y=20      # folder of PNGs
$ osatura import-anim demo.json ia.json gifl dof.gif loop=off start=0                    # animated GIF/APNG
```
Az `import-seq` egy mappa összes PNG-jét fájlnév szerinti sorrendben veszi (mind azonos méretű); az `import-anim` animált GIF-et vagy APNG-t olvas a fájl
saját késleltetéseivel (a sima PNG egy képkockát ad). Mindkettő switch réteget ad hozzá képkockánként egy beágyazott képpel, plusz egy lépéscsatornával; **meglévő**
projekten dolgoznak. A hosszú GIF nagyra duzzasztja a projektet (minden képkocka beágyazott PNG). A `trace` és az `autotrace` bitképeket alakít vektoros
útvonalakká (a `trace` az alfacsatornát olvassa; az `autotrace` színeket kvantál és Bezier-görbéket illeszt).

## 10. Import: BVH mozgásrögzítés, MIDI és más élő bemenet

### BVH (`import-bvh`, `mocap-info`, `mocap-match`, `mocap-retarget`)

```
$ osatura mocap-info walk.bvh
$ osatura import-bvh demo.json walk.bvh ib.json --joint-map LeftUpLeg=thigh,LeftLeg=shin --axis Zrotation --tol 0.5
wrote ib.json (+2 channels from 4 BVH frames)
```
Az `import-bvh` a kiválasztott ízületi forgatási csatornákat csontokra képezi kulcsolt `bone_angle` csatornákként, a projekt fps és képkockaszám szerint mintavételezve
(az `--axis` a `Zrotation`, `Xrotation` vagy `Yrotation` értéket választja; a `--tol` egyszerűsíti a kulcsokat). A `mocap-match` ízület–csont párosítást javasol
bizalmi táblázattal, a `mocap-retarget` pedig hozzáfűzött csatornákat bake-el gyökérmozgással, nézettel (`front|side|top|yaw=DEG`) és skálával. A BVH
csak szöveges részhalmaz: nincsenek bináris mocap-formátumok, nincs FBX (tervezetten). **Korlátok (LIMITATIONS L-45):** valódi mozgásrögzítő fájlt nem teszteltünk; a 2D
vetítés eldobja a mélységet; a csonttengely körüli csavarodást, az ujjakat és az arcot figyelmen kívül hagyja; a rig megtartja a csonthosszait, így a lábcsúszás csak
csökken.

### MIDI és más élő bemenet (`live-map`)

```
$ osatura live-map small.json song.mid mapping.json out.json
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 0 events (0 after the last frame skipped); wrote out.json
```
Egy **Standard MIDI File** (`.mid`; MIDI 1.0 üzenetek, running status, tempó meta-események, tick vagy SMPTE osztás) vagy egy eseményeket tartalmazó JSON
(`{"events":[{"source","id","value","time"}]}`, az érték 0..1-re normalizált, az idő másodpercben) kulcsolt csatornákká, eseményekké és take-ekké alakul egy
leképezőfájl segítségével. Források: `midi_cc midi_note midi_pitch_bend mouse_x mouse_y mouse_button gamepad_axis gamepad_button key face`. Legfeljebb
3 000 000 esemény; az idők 7 napon belül. A CLI semmilyen MIDI-eszközzel nem beszélget; a studio a böngésző Web MIDI-jét használja. A MIDI **csak
importálható** (MIDI-export nincs). A hibás formátumú fájlok hibát adnak, soha nem összeomlást.

## 11. Hangformátumok

| Felhasználás | Formátumok | Megjegyzések |
|---|---|---|
| Elemzés (`lipsync`, `lipsync-text`, `speech-motion`, `audio-react`, `beats`, `waveform`) | csak **WAV** | 8/16/24/32 bites egész vagy 32 bites lebegőpontos PCM, tetszőleges csatornaszám monóra keverve, legfeljebb 256 MiB. Az MP3/OGG fájlokat előbb alakítsd át (`ffmpeg -i in.mp3 out.wav`). |
| Projekt hangsávjai (`sound add`) | WAV, OGG, MP3, bármi, amit az `ffprobe` elfogad | **Hivatkozásként** tárolva (relatív útvonal, kezdőképkocka, erősítés 0..4, vágási ablak, fade-ek); semmi sincs beágyazva; a fájlnak a projektmappán belül kell maradnia, hacsak nem adod meg az `--allow-external-assets` kapcsolót. |
| Kevert kimenet (`sound mix`) | `.wav` (44,1 kHz PCM 16 bites), `.ogg` (Vorbis q4), `.mp3` (96 kb/s) | Az ffmpeg-en át keverve a **projekt hosszára**; `.flac` és mások: `unknown audio format '.flac' (use .wav, .ogg or .mp3)`. |
| Exportokon belül | `.mp4` AAC, `.webm` Opus, `.html` beágyazott mp3 | A `.gif`/`.apng` megjegyzéssel némák maradnak; a Lottie `[audio]` figyelmeztetést ad; a GLB és az OCA elhagyja a hangot. |
| Szájállás-jelzések (`mouth`) | Papagayo/Moho-switch szöveg, Rhubarb TSV | Nem hang: jelzésfájlok. |

Használt enkóderek: `libmp3lame` (mp3), `libvorbis` (ogg), `aac`, `libopus`. Nincs sávonkénti pan/EQ/ducking vagy ismétlés (ROADMAP 9. szakasz).

## 12. Funkciótámogatási mátrix

A sorok azok a dolgok, amelyeket egy projekt tartalmazhat, az oszlopok a kimeneti formátumok. **yes** = a projektben lévő módon továbbvíve, **approx** = látható
eltéréssel továbbvíve, **baked** = csak képkockánkénti eredményként van jelen, **no** = elhagyva. Azoknál a formátumoknál, amelyeket nem tudtunk független megjelenítőben ellenőrizni, a cellák
azt írják le, amit az exportáló kiír.

| Funkció | SVG / PNG | Sprite sheet / APNG | GIF | WebM / MP4 | HTML | Lottie / .lottie | GLB | OCA |
|---|---|---|---|---|---|---|---|---|
| Vektoros alakzatok, kitöltések | yes | baked (raster) | baked (raster) | baked (raster) | yes (SVG) | yes | approx (triangles) | baked (raster) |
| Színátmenetek | yes | baked | baked | baked | yes | yes (approx if layer skewed) | yes (texture) | baked |
| Szélességprofilos vonalak | yes | baked | baked | baked | yes | approx (constant width) | yes | baked |
| Szaggatások, végek, csatlakozások | yes | baked | baked | baked | yes | yes | yes (triangulated) | baked |
| Kép (bitkép) rétegek | yes | baked | baked | baked | yes | no (`image-layer`) | yes | baked |
| Csontok és animáció | baked | baked | baked | baked | baked | baked | **yes (exact)** | baked |
| Skinelés / deformerek | baked | baked | baked | baked | baked | baked | skinned; deformers at frame 0 | baked |
| Kamera és parallax | baked | baked | baked | baked | baked | baked (camera moves are in the geometry) | no (shapes come from frame 0) | baked |
| Keverési módok | yes | baked | baked | baked | yes | approx (`add` becomes `screen`; none on groups) | no (normal) | baked |
| Rétegmaszkok | yes | baked | baked | baked | yes | single layers only | frame-0 shape, restricted | baked |
| Rétegeffektek (blur, shadow...) | yes | baked | baked | baked | yes | single-leaf layers | no | baked |
| Animált átlátszatlanság | baked | baked | baked | baked | baked | yes | no (frozen at frame 0) | baked |
| Útvonal-morphok | baked | baked | baked | baked | baked | baked | morph targets (when valid) | baked |
| Mélységélesség | via `dof-export` | via `dof-export` | via `dof-export` | via `dof-export` | no | no (warned) | no | no |
| Utóeffekt-lánc (`fx`) | PNG only | no | no | no | no | no | no | no |
| Átlátszóság | yes | yes (alpha) | 1-bit | WebM yes, MP4 no | yes | yes | yes | yes |
| Hang | no | no | no | **yes** | yes (mp3) | no (warned) | no | no |
| Ismétlés | n/a | sheet: n/a, APNG: yes | yes | player-dependent | player | lejátszó (a dotLottie 2.0 a lejátszóra vagy egy állapotgépre bízza az ismétlést) | n/a | n/a |
| Utólag szerkeszthető rig/csatornák | no | no | no | no | no | no | bones only | no |
| Állapotgépek, kötések, container-ek, görgetés | no | no | no | no | no | no | no | no |
| Felbontásfüggetlen | yes (SVG) | no | no | no | yes | yes | yes | no |
| Kell hozzá ffmpeg | no | no | no | **yes** | audio only | no | no | no |

A „Baked” azt jelenti, hogy a funkció hatása a pixelekben vagy a képkockánkénti geometriában van meg, de maga a funkció utólag nem szerkeszthető.

## 13. Kisebb fájlok készítése

1. **Nézd meg, hol vannak a bájtok:** az `osatura size-report project.json` rétegek, képek és csatornák szerint listázza a projektfájlt, a bake-elt Lottie-t és a HTML-oldalt
   (`--json` szkriptekhez).
2. **Illeszd automatikusan egy kerethez:** az `osatura export-fit project.json out.json --max-size 50KB --format lottie` végigkeresi az exportáló paramétereit, és
   megtartja a legjobb SSIM-et, amely belefér. Formátumok: `lottie`, `html`, `gif`, `apng`, `webm`, `project` (lecsupaszított projekt-JSON). A `--measure gzip` a
   gzip-méretre ad keretet, a `--min-ssim X` minőségi alsó határt állít be, a `--max-seconds N` valódi falióra-korlát (az addigi legjobb eredmény megmarad). Ha a keret nem
   tartható, 1-es kilépési kóddal a legkisebb eredményt adja.
3. **Előbb csökkentsd a grafikát:** az `osatura lod project.json --tolerance-px 1 --levels 3 --out-dir lv --verify` részletességi szintű projekteket ír deklarált
   (és a `--verify` mellett mért) hibával kimeneti pixelekben; a `lod-pick --screen-px WxH` kiválasztja az adott képernyőhöz a legdurvább szintet.
4. **Formátumonként:**
   * A Lottie/HTML/GLB a `frames x anchors` szorzattal nő: kevesebb képkocka, kevesebb horgony, alacsonyabb fps, megvágott kulcstartományok (`keys` eszközök).
   * A GIF/APNG a pixelek × képkockák szorzatával nő: a `--scale 0.5` 4-szeres csökkentés; a GIF lapos grafikához való, az APNG alfához, a WebM/MP4 mindenhez, ami mozog.
   * A `.lottie` ugyanaz a Lottie deflate-tel tömörítve (a példában kb. 4-szer kisebb); a `.html`/`.json` fájlokat gzip-pel szolgáld ki.
   * A projektben lévő képek base64 PNG-ként vannak beágyazva: az importálás előtt kicsinyítsd őket.
5. Korlátok: egy 14 rétegű rig Lottie-ja nem tehető tetszőlegesen kicsivé (a példán az 1 KB-os keret ezt jelenti: „every knob is at its floor”); a
   keresés nagy projekteken időt igényel.

## 14. Az ffmpeg követelményei

Az OSATURA soha nem csomagolja vagy linkeli az ffmpeg-et; a `PATH`-on található `ffmpeg`-et (és `ffprobe`-ot) futtatja a saját telepítésed **külső alfolyamataként**.

| Mihez kell | Enkóder/dekóder | Ellenőrizve ezen |
|---|---|---|
| `export .mp4` | `libx264` (`yuv420p`), `aac` | ffmpeg 6.1.1 |
| `export .webm` (alapértelmezett) | `libvpx-vp9`, `libopus` | ffmpeg 6.1.1 |
| `export .webm --codec av1` | `libaom-av1`, `libopus` | ffmpeg 6.1.1 |
| `dof-export .mp4/.webm` | ugyanaz, mint fent, `--codec` | ffmpeg 6.1.1 |
| `sound mix .ogg` | `libvorbis` | ffmpeg 6.1.1 |
| `sound mix .mp3`, `.html` hang | `libmp3lame` | ffmpeg 6.1.1 |
| `face-track` videóból | csak dekóderek | |
| `sound add` tartalomellenőrzés | `ffprobe` (nélküle nem-WAV fájloknál csak a létezést ellenőrzi) | |

Ellenőrizd a sajátodat így: `ffmpeg -hide_banner -encoders | grep -E 'libx264|aac|libvpx-vp9|libaom-av1|libopus|libvorbis|libmp3lame'`. Minden más
(GIF, APNG, HTML hang nélkül, Lottie, dotLottie, GLB, OCA, PNG, SVG, EXR) ffmpeg nélkül működik.
Ha az ffmpeg hiányzik, a parancs `cannot run ffmpeg (...)` üzenettel hibázik, és semmit sem ír; a hiányzó enkóder `ffmpeg failed` üzenetet ad.

## 15. Szabadalmi és licenc-mutatók

* A [PATENT_NOTES.md](PATENT_NOTES.md) azt rögzíti, ami **nem** lett átvizsgálva. Az exportnál a releváns sor a *Video and audio codecs through ffmpeg
  (`export` .mp4 / .webm, `soundtrack`)*: a H.264-re és az AAC-re licencelési közösségek vonatkoznak; a VP9 és az AV1 jogdíjmentesként van forgalmazva, de közösségi és egyéni
  követeléseket jelentettek be mindkettőre; **státusz: nem értékelt, a VP9/AV1-et ne kezeld tisztázottként.** Mivel az ffmpeg a felhasználó saját telepítése, minden
  kodekválasztás a felhasználó felelőssége, és az OSATURA nem csomagolhatja az ffmpeg-et.
* A determinisztikus raszter-, GIF-, APNG-, PNG-, SVG-, OpenEXR-, OCA-, ORA-, glTF- és Lottie-írók az OSATURA saját megvalósításai nyilvános
  formátumleírásokból (clean-room, lásd CLEANROOM.md). Szabadalmi helyzetüket ezen túl nem értékeltük: a PATENT_NOTES.md csak a fenti területeket sorolja fel
  (arcvezérelt bábozás, állapotgépek és adatkötés, FABRIK/MVC/súlyok, stencil-then-cover kitöltés, kodekek).
* GIF: az LZW-szabadalmak minden joghatóságban rég lejártak; ez általános ismeret, nem a tárolóban rögzített tisztázás.
* Az előállított fájlok licencei: az általad exportált fájlok a tiéd; a motor MPL-2.0, a CLI AGPL-3.0-or-later licencű, ami a programokra vonatkozik, nem az általuk írt
  fájlokra. A studio a megfelelő forráskódot a `/source` címen kínálja (AGPL 13. szakasz).
* Ez nem jogi tanács.

## 16. Az írt fájlok ellenőrzése

| Formátum | Ellenőrzés | Várható eredmény |
|---|---|---|
| bármely projekt | `osatura check out.json` | `ok: ...` (az `import-*` után is) |
| GIF | `ffprobe -v error -count_frames -select_streams v -show_entries stream=nb_read_frames -of csv=p=0 out.gif` | képkockaszám (8) |
| APNG | ugyanez a parancs az `out.apng` fájlon | képkockaszám (8) |
| MP4 | `ffprobe -v error -show_entries stream=codec_name,width,height,pix_fmt,nb_frames,r_frame_rate -of csv=p=0 out.mp4` | `h264,320,240,yuv420p,24/1,8` |
| WebM | ugyanez az `out.webm` fájlon `:stream_tags=alpha_mode` kiegészítéssel | `vp9,...,yuv420p,...,1` (alpha_mode 1) |
| EXR | `ffprobe f.exr` | `exr 640x480 gbrapf32le` |
| Lottie | `python3 -c "import json;d=json.load(open('out.json'));print(d['v'],d['fr'],d['op'],d['w'],d['h'],len(d['layers']))"` | `5.7.0 24.0 8 640.0 480.0 12` |
| Lottie-figyelmeztetések | `osatura export p.json out.json --report rep.json` | a `rep.json` minden közelítést felsorol |
| dotLottie | `unzip -l out.lottie` | csak a `manifest.json` és az `a/animation.json` |
| Lottie-körbejárás | `osatura import-lottie out.json back.json && osatura check back.json` | `ok:` |
| GLB | `python3 -c "import struct;b=open('out.glb','rb').read();print(b[:4],struct.unpack('<II',b[4:12]),len(b))"` | `b'glTF' (2, N)`, ahol N egyenlő a hosszal |
| HTML | `grep -c '<svg' out.html`; `grep -o 'http[^"<]*' out.html` | képkockánként egy `<svg`; csak az SVG-névtér URL-je |
| SVG | nyisd meg böngészőben vagy értelmezd XML-ként | jólformált |
| Sprite sheet | `python3 -m json.tool out.atlas.json` | képkockánként `{index,x,y,w,h}` |
| OCA | `python3 -m json.tool <name>.oca` | minden hivatkozott cel létezik |
| Vizuális regresszió | `osatura ci diff a.png b.png --metric ssim` | 0-s kilépési kód, ha egyenlők; lásd [CI.md](CI.md) |

A veszteséges export eredetihez viszonyított vizuális ellenőrzéséhez az `osatura export-fit` már jelenti az SSIM és a CIEDE2000 értéket az eredeti rendereléshez képest, az
`osatura compare` pedig előtte/utána/különbség képet ír két projektről.

## 17. Eltérések más dokumentumoktól

* A `docs/LIMITATIONS.md` L-29 szerint a Lottie-import nem vesz át kép-asseteket, effekteket, előkompozíciókat vagy matte-eket; a jelenlegi importer átveszi a beágyazott
  data URI-s képeket, az előkompozíciókat, a rétegenkénti effekteket (blur, drop shadow, tint, fill), és közelíti a luma matte-eket (9. szakasz). A kihagyott
  funkciók 9. szakaszbeli listája az, amiről a parancs valóban figyelmeztet.
* A `docs/LIMITATIONS.md` L-32 az OCA-t az importformátumok között sorolja fel; a CLI-nek van OCA-**exportja** (`export-oca`), de nincs `import-oca` parancsa.
* Az `export` használati szövege csak a `.gif .html .json .mp4` formátumokat nevezi meg, plusz „a nyílt formátumokat”; a kiterjesztések teljes listája a 2. szakaszban található.
* A használati szöveg és a viselkedés további eltéréseit a [CLI_REFERENCE.md](CLI_REFERENCE.md) vége gyűjti össze.
