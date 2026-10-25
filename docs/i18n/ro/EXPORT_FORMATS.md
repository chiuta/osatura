<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# Formatele de export și import OSATURA

Ce păstrează, ce aproximează și ce pierde fiecare format de fișier pe care OSATURA îl scrie sau îl citește, ce comandă îl produce și cum se
verifică rezultatul. Acest text este licențiat CC-BY-4.0. Versiunea documentată: **osatura 0.1.0**.

Fiecare linie de comandă de mai jos a fost rulată pe proiectul demo încorporat (`osatura demo demo.json`) sau pe o copie de 8 cadre a acestuia
(`small.json`, același proiect cu `"frames": 8`). Dimensiunile citate provin din acel proiect de 8 cadre, 640x480, și au doar rol
orientativ: ele cresc odată cu cadrele, ancorele și straturile, nu sunt valori fixe. Detaliile comenzilor (fiecare opțiune,
fiecare eroare) se află în [CLI_REFERENCE.md](CLI_REFERENCE.md); formatul de fișier al proiectului în sine este descris în
[PROJECT_FORMAT.md](PROJECT_FORMAT.md); starea codec-urilor și a brevetelor este în [PATENT_NOTES.md](PATENT_NOTES.md) (nu reprezintă sfat juridic).

## Cuprins

1. [Ce format pentru ce sarcină](#1-ce-format-pentru-ce-sarcină)
2. [Formatele de ieșire pe scurt](#2-formatele-de-ieșire-pe-scurt)
3. [Imagini statice: SVG, PNG, PNG pe 16 biți, OpenEXR](#3-imagini-statice)
4. [Animație raster: sprite sheet PNG, GIF, APNG, WebM, MP4](#4-animație-raster)
5. [Player HTML](#5-player-html)
6. [Lottie (.json) și dotLottie (.lottie)](#6-lottie-json-și-dotlottie-lottie)
7. [glTF binar (.glb)](#7-gltf-binar-glb)
8. [Folderul Open Cel Animation (OCA)](#8-folderul-open-cel-animation-oca)
9. [Import: SVG, OpenRaster, Lottie, imagini, GIF/APNG](#9-import-svg-openraster-lottie-imagini-gifapng)
10. [Import: motion capture BVH, MIDI și alte intrări live](#10-import-motion-capture-bvh-midi-și-alte-intrări-live)
11. [Formate audio](#11-formate-audio)
12. [Matricea de suport al funcționalităților](#12-matricea-de-suport-al-funcționalităților)
13. [Fișiere mai mici: export-fit, lod, size-report](#13-fișiere-mai-mici)
14. [Cerințe ffmpeg](#14-cerințe-ffmpeg)
15. [Indicații despre brevete și licențe](#15-indicații-despre-brevete-și-licențe)
16. [Validarea a ceea ce ai scris](#16-validarea-a-ceea-ce-ai-scris)
17. [Diferențe față de alte documente](#17-diferențe-față-de-alte-documente)

---

## 1. Ce format pentru ce sarcină

| Vrei... | Folosește | De ce |
|---|---|---|
| Aspectul exact, distribuit ca un singur fișier, rulează în orice browser, cu derulare (scrub) | `.html` | Cadre SVG inline desenate de randerul propriu al OSATURA; nimic nu este aproximat. |
| O animație vectorială mică pentru o pagină web sau o aplicație cu un player Lottie | `.json` sau `.lottie` | Vectorial, independent de rezoluție, mic; dar vezi lista de pierderi (imagini, trasee cu lățime variabilă, audio...). |
| O previzualizare în buclă pentru chat sau un README | `.gif` | Funcționează peste tot; 256 de culori pe cadru, transparență pe 1 bit, fără sunet. |
| Un înlocuitor pentru GIF, cu culoare completă și alpha | `.apng` | RGBA pe 8 biți, fără pierderi; mai mare decât GIF. |
| Un video cu sunet | `.mp4` (H.264 + AAC) sau `.webm` (VP9/AV1 + Opus) | Necesită `ffmpeg`, instalat de tine. `.webm` păstrează transparența. |
| Sprite sheet pentru un motor de joc | `.png` | Un PNG plus un atlas JSON cu dreptunghiurile tile-urilor. |
| Un schelet și mișcare pentru o unealtă 3D | `.glb` | Oase, animație, forme plate ca mesh-uri de triunghiuri. |
| Cadre pentru unelte de compoziție/pictură | `export-oca` (Open Cel Animation) sau `frames --png` | Cel-uri ca PNG, într-un singur folder. |
| Imagini statice în lumină liniară sau cu adâncime mare de biți | `render-hdr` (`.exr` sau `.png` pe 16 biți) | Rander în virgulă mobilă, OpenEXR pe 16 biți sau half/float. |
| Un singur cadru static | `render` (`.svg` sau `.png`) | Vectorial sau raster pe 8 biți. |

## 2. Formatele de ieșire pe scurt

| Format | Comandă | Extensie | Necesită ffmpeg | Alpha | Audio | Dimensiune pentru demo-ul de 8 cadre* |
|---|---|---|---|---|---|---|
| SVG (un cadru) | `render` | `.svg` | nu | da | nu | câțiva KB |
| Secvență SVG/PNG | `frames` | folder | nu | da | nu | un fișier pe cadru |
| PNG (un cadru) | `render` | `.png` | nu | da (8 biți) | nu | |
| PNG pe 16 biți | `render-hdr` | `.png` | nu | da | nu | |
| OpenEXR (half/float) | `render-hdr` | `.exr` | nu | da | nu | 466 KB half, 923 KB float (640x480) |
| Sprite sheet PNG + JSON | `export` | `.png` (+ `.atlas.json`) | nu | da | nu | |
| GIF | `export` | `.gif` | nu | 1 bit | nu | 83 KB |
| APNG | `export` | `.apng` | nu | da | nu | 131 KB |
| WebM | `export` | `.webm` | da | da | da (Opus) | 10.5 KB |
| MP4 | `export` | `.mp4` | da | nu | da (AAC) | 8.9 KB |
| Player HTML | `export` | `.html`, `.htm` | doar pentru audio | da | mp3 încorporat | 66 KB |
| Lottie | `export` | `.json` | nu | da | nu | 47 KB |
| dotLottie | `export` | `.lottie` | nu | da | nu | 11.5 KB |
| glTF binar | `export` | `.glb` | nu | da | nu | 115 KB |
| Open Cel Animation | `export-oca` | folder | nu | da | nu | un PNG pe cel |

\* GIF, APNG, WebM și MP4 la `--scale 0.5` (320x240); formatele vectoriale la dimensiune completă. „Necesită ffmpeg” înseamnă un
`ffmpeg` funcțional în `PATH` (secțiunea 14). Numele de ieșire sunt alese după extensie, fără a ține cont de majuscule; o extensie necunoscută este refuzată și
nu se scrie nimic (`unknown export format '.xyz' (use .gif .html .json .mp4 .apng .webm .png .glb)`, iar `.lottie` este de asemenea
acceptat).

Toți writer-ii refuză să suprascrie proiectul de intrare cu ieșirea, scriu printr-un fișier temporar pe care îl redenumesc și refuză
legăturile simbolice dacă nu se dă `--follow-symlinks` (CLI_REFERENCE secțiunea 1.4). Garda de dimensiune pentru exportatorii vectoriali
(Lottie, HTML, GLB) refuză un export de peste aproximativ 40 de milioane de valori (cadre x ancore, plus oase și straturi pe cadru)
cu `export would write about N million values ...`.

## 3. Imagini statice

### SVG (`render ... out.svg`, `frames`)

```
$ osatura render demo.json 12 f12.svg --bones
wrote f12.svg
$ osatura frames demo.json fr            # fr/frame_0000.svg ... frame_0047.svg
```
Ieșirea vectorială a randerului propriu al OSATURA: căile, gradientele, trasee cu profile de lățime, liniile punctate, modurile de blend, măștile și efectele
de strat sunt scrise ca SVG. `--bones` desenează suprapunerea scheletului. Pentru `.svg`, `--scale` lasă viewBox-ul documentului neschimbat. **Pierderi/limite cunoscute:** randerul nu raportează niciuna; modul în care un anumit vizualizator SVG tratează modurile de blend și filtrele depinde
de acel vizualizator. `--proxy N` scrie un proxy de scrub simplificat, care renunță la traseele fine, gradiente și moduri de blend (nu este pentru
livrabile). **Validare:** deschide-l într-un browser sau rulează `python3 -c "import xml.dom.minidom,sys; xml.dom.minidom.parse(sys.argv[1])" f12.svg`.

### PNG (`render ... out.png`, `frames --png`)

RGBA pe 8 biți, sRGB, randat de rasterizatorul propriu al motorului. `--scale N` în (0, 16] înmulțește dimensiunea în pixeli. Un proiect cu
`"background": null` se randează transparent acolo unde nu se desenează nimic. Implicit nu este gestionat prin management al culorii: valorile se amestecă în spațiul codificat sRGB.
Începând cu wave 7, un proiect poate opta pentru compunere în lumină liniară (`osatura color-space p.json out.json linear`, câmpul `color_space`):
ieșirile raster, GIF, APNG, video și `render-hdr` sunt atunci compuse în lumină liniară f32 și convertite o singură dată în sRGB; SVG și
Lottie nu se schimbă (LIMITATIONS L-22).

### PNG pe 16 biți și OpenEXR (`render-hdr`)

```
$ osatura render-hdr small.json 3 f.exr                           # HALF, 640x480, 465,611 bytes
$ osatura render-hdr small.json 3 f2.exr linear --precision float # FLOAT, 923,211 bytes
$ osatura render-hdr small.json 3 f4.png                          # 16-bit/colour RGBA, sRGB-encoded
```
* **OpenEXR:** o singură parte, scanline, **necomprimat**, canale `A B G R`, lumină liniară, alpha premultiplicat. `HALF` implicit,
  `FLOAT` cu `--precision float` (sau `--float32`). Cuvântul `linear` nu are efect pentru EXR (întotdeauna liniar).
* **PNG pe 16 biți:** codificat sRGB, cu excepția cazului în care se dă cuvântul `linear`.
* **Fidelitate:** randerul în virgulă mobilă are mai multă precizie decât cel pe 8 biți, dar grafica folosește culori sRGB obișnuite, deci fișierul
  conține valori din gama de afișare cu precizie mai mare, nu lumină HDR raportată la scenă. Fundalul proiectului nu este desenat.
* **Pierderi/limite cunoscute:** un cadru pe fișier; fără compresie, straturi sau AOV-uri; fără profil ICC; `--8bit` revine la
  rasterizatorul pe 8 biți ca sursă.
* **Validare:** `ffprobe f.exr` raportează `exr 640x480 gbrapf32le`; orice vizualizator EXR îl deschide.

## 4. Animație raster

Toate acestea randează mai întâi fiecare cadru prin motorul raster. Un proiect cu `"background": null` se randează cu pixeli transparenți
oriunde nu se desenează nimic; demo-ul are o culoare de fundal (`#f6f1e7`) și este deci opac. `--scale N` în (0, 16]
se aplică la `.gif .apng .webm .mp4 .png`; la alte formate este refuzat (`unknown option --scale for 'export .json'`).

### Sprite sheet PNG (`export ... out.png`)

```
$ osatura export small.json out3.png --columns 4 --scale 0.25
wrote out3.atlas.json
wrote out3.png                    # 4 columns x 2 rows of 160x120 tiles
```
Un PNG cu toate cadrele ca tile-uri (`--columns N`, implicit o grilă aproape pătrată: ceil(sqrt(frames)) coloane) și un atlas
`out3.atlas.json` (numele de ieșire cu extensia înlocuită prin `.atlas.json`) care listează `{index, x, y, w, h}` pentru fiecare cadru. Alpha este păstrat.
Atlasul se numea înainte `out3.json` și putea suprascrie în tăcere un fișier Lottie cu același nume de bază; noul nume evită acest lucru, iar calea atlasului trece prin aceleași gărzi de intrare/legături simbolice.

### GIF (`export ... out.gif`)

```
$ osatura export small.json out.gif --scale 0.5
wrote out.gif                      # 8 frames, 83 KB
```
* O imagine pe cadru sursă, **buclă infinită**, o paletă locală de până la 256 de culori pe cadru (cuantizată, fără dithering).
* **Timing:** întârzierile GIF sunt în centisecunde întregi, iar vizualizatoarele tratează orice valoare sub 2 cs ca 10 cs, deci 50 fps este maximul real.
  Până la 50 fps întârzierile sunt cu difuzia erorii (29.97 fps alternează 3 și 4 cs), astfel încât durata totală rămâne în limita a 1 cs față de
  `frames / fps`. Peste 50 fps secvența este **decimată la 50 fps**, durata este păstrată și se afișează un avertisment
  (`GIF cannot hold 60 fps ... use APNG, WebM or MP4 for the full rate`).
* **Transparență:** 1 bit (indicatorul de culoare transparentă este setat); pixelii semitransparenți nu sunt păstrați.
* **Pierderi cunoscute:** fără audio (`note: GIF cannot carry audio; ...` când proiectul are sunete); fără alpha parțial; imaginile mai late
  sau mai înalte de 65535 de pixeli sunt refuzate (`image too large for GIF`).
* **`--report r.json`:** scrie lista lucrurilor pe care formatul nu le-a putut păstra (aici: decimarea ratei de cadre, dacă există).
* **Sfaturi de dimensiune:** un `--scale` mai mic, mai puține cadre sau `export-fit --format gif` (secțiunea 13). GIF este cel mai puțin eficient dintre formatele
  raster pentru grafică fotografică sau bogată în gradiente.
* **Validare:** `ffprobe -v error -count_frames -select_streams v -show_entries stream=nb_read_frames -of csv=p=0 out.gif` afișează
  numărul de cadre (8).

### APNG (`export ... out.apng`)

PNG animat, RGBA pe 8 biți, fără pierderi, un cadru pe cadrul proiectului, la fps-ul proiectului (0.0417 s pe cadru la 24 fps). Alpha complet este păstrat. Fără audio. Mai mare
decât GIF pentru grafică plată (131 KB față de 83 KB în exemplu), deoarece nu există paletă. Majoritatea browserelor redau APNG; unele unelte nu o fac,
caz în care folosește WebM sau GIF. **Validare:** `ffprobe` cu `-count_frames` (8 cadre pentru exemplu).

### WebM (`export ... out.webm`) și MP4 (`export ... out.mp4`)

```
$ osatura export small.json out.mp4  --scale 0.5     # h264, yuv420p, 320x240, 8 frames
$ osatura export small.json out.webm --scale 0.5     # vp9, yuv420p + alpha_mode=1
$ osatura export small.json out.webm --codec av1     # needs an ffmpeg with libaom-av1
```
* **MP4:** H.264 (`libx264`), `yuv420p`, **fără alpha**. Audio AAC când proiectul are piste de sunet.
* **WebM:** VP9 (`libvpx-vp9`, implicit) sau AV1 (`libaom-av1`, `--codec av1`); canalul alpha este păstrat (`yuva420p`, eticheta de container
  `alpha_mode=1`); audio Opus (`libopus`) când proiectul are sunete. Orice altă valoare `--codec`: `unknown --codec 'foo' (vp9 or av1)`.
* **Sunet:** pistele de sunet ale proiectului sunt mixate (start, gain, trim, fade-uri; tăiate la lungimea proiectului) și multiplexate.
* **Necesită ffmpeg:** cadrele sunt randate ca PNG într-un director temporar privat, date lui `ffmpeg` și șterse după aceea. Dacă
  `ffmpeg` lipsește: `cannot run ffmpeg (...); install it or export a GIF/HTML instead` (cod de ieșire 1, nu se scrie nimic). Dacă îi lipsește
  encoder-ul: `ffmpeg failed`.
* **Limite cunoscute:** doar 8 biți; dimensiunea video urmează dimensiunea proiectului și `--scale` (verifică rezultatul cu `ffprobe`).
  `dof-export` scrie de asemenea `.mp4`/`.webm`, fără audio.
* **Validare:** `ffprobe -v error -show_entries stream=codec_name,width,height,pix_fmt,nb_frames,r_frame_rate -of csv=p=0 out.mp4`
  a afișat `h264,320,240,yuv420p,24/1,8`.
* **Brevete:** vezi secțiunea 15; H.264 și AAC sunt acoperite de pool-uri de licențiere, VP9/AV1 nu au fost evaluate.

## 5. Player HTML

```
$ osatura export small.json out.html
wrote out.html                     # 66 KB for 8 frames
```
Un fișier HTML autonom: **fiecare cadru ca SVG inline**, un buton play/pause, un scrubber (range input) și o etichetă de cadru,
condus de un singur `<script>` inline. Nu face **nicio cerere de rețea**: singurul URL din fișier este namespace-ul SVG. Pentru a-l deschide nu este nevoie
decât de un browser. Imaginea este exact ceea ce produce randerul SVG, deci este cel mai fidel dintre formatele „redabile”
(măștile, modurile de blend, efectele, oasele, IK, deformatoarele, particulele sunt toate coapte (baked) în cadre).

* **Audio:** dacă proiectul are sunete, audio-ul mixat este încorporat ca mp3 într-un data URI (necesită `ffmpeg`; avertisment peste 3 MiB, iar
  audio-ul este eliminat cu un avertisment peste 16 MiB).
* **Dimensiune:** crește liniar cu cadrele (aproximativ 7.7 KB pe cadru aici) și cu detaliul vectorial; fișierul nu este comprimat. Sfaturi
  uzuale: servește-l comprimat gzip (exemplul de 66 KB are aproximativ 9 KB comprimat gzip), redu numărul de cadre sau detaliul căilor cu `lod` și folosește
  `export-fit --format html`.
* **Limite:** `--report` este refuzat (`unknown option --report for 'export .html'`);
  se aplică garda de dimensiune a exportului vectorial; playerul oferă doar play/pause și un scrubber.
* **Validare:** deschide-l; `grep -c '<svg' out.html` numără cadrele; `grep -o 'http[^"<]*' out.html` trebuie să listeze doar namespace-ul SVG.

## 6. Lottie (.json) și dotLottie (.lottie)

```
$ osatura export small.json out.json --report rep.json
wrote out.json
warning: [variable-width-stroke] ink: variable-width stroke profiles are not supported by Lottie; the constant base width is used
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
$ osatura export small.json out.lottie       # a zip: manifest.json + a/animation.json
```

### Cum este construit

Fișierul este Bodymovin (`"v": "5.7.0"`) cu `fr` = fps-ul proiectului, `ip` = 0, `op` = numărul de cadre și `w`/`h` = dimensiunea scenei. **Fiecare
strat vectorial devine un shape layer a cărui geometrie de căi este coaptă (baked) pe cadru în spațiul lumii (keyframe-uri hold)**, deci oasele, IK,
skinning-ul, deformatoarele, spring-urile, particulele, camera (inclusiv parallax) și transformările părinților sunt toate *în* geometrie. Rezultatul
se redă ca originalul, dar nu este un rig editabil: nu există oase, canale sau expresii. Un strat solid de fundal este
scris când proiectul are o culoare de fundal. În consecință, fișierul crește cu `frames x anchors`; este mult mai mare decât un Lottie
construit manual, cu animație de transformare (47 KB pentru demo-ul de 8 cadre, 11.5 KB într-un `.lottie`).

### Exportat

Umpleri și contururi solide și cu gradient (opriri de gradient cu alpha; capetele gradientului urmează transformarea animată a stratului), modele punctate,
capete, îmbinări și limita miter, lățimile conturului scalate cu scala stratului, moduri de blend, opacitatea stratului, track matte-uri alpha și alpha inversat
pentru măștile de strat (o copie a sursei măștii acționează ca matte), efecte de strat (blur, drop shadow, tint, fill) pe
straturi cu o singură frunză și **opacitatea grupului ca pre-compoziție**, astfel încât copiii care se suprapun să nu se vadă unii prin alții. Numerele sunt
scrise cu cea mai scurtă formă zecimală.

### Avertizat (niciodată pierdut în tăcere)

Fiecare element de mai jos este afișat pe stderr ca `warning: [code] layer: message` și scris în `--report FILE` ca
`{"format":"lottie","count":N,"warnings":[{"code","layer","message"}]}`:

| Cod | Semnificație |
|---|---|
| `image-layer` | Straturile de imagine nu sunt exportate (ieșirea Lottie conține doar straturi vectoriale). |
| `variable-width-stroke` | Profilele de lățime nu sunt acceptate; se folosește lățimea de bază constantă. |
| `stroke-anisotropic` | Scală neuniformă sau skew pe strat; contururile Lottie sunt uniforme, deci lățimea este scalată cu media geometrică. |
| `gradient-anisotropic` | Un gradient pe un astfel de strat este aproximat (gradientele Lottie nu sunt deformate prin shear). |
| `blend-add` | Modul de blend `add` nu are echivalent Lottie; se folosește `screen`. |
| `group-blend` | Un mod de blend pe un grup nu este exportat. |
| `group-mask` | O mască pe un grup nu este exportată (măștile pe straturi singulare sunt). |
| `group-effect` | Efectele de strat pe un grup (de mai multe straturi) nu sunt exportate. |
| `mask-source-not-vector` | Sursa măștii nu este un strat vectorial, deci masca nu este exportată. |
| `depth-of-field` | Canalele `camera_focus`/`camera_blur` nu sunt aplicate; folosește `dof-export`. |
| `audio` | Lottie nu poate transporta audio; N pistă(e) de sunet nu sunt incluse. |

### Pierdut, fără echivalent Lottie

Oase, IK, canale, mașini de stări, data binding-uri, containere, regiuni de scroll, markers/events, sunete, lanțuri de post-efecte (`fx`),
normal map-uri și structura editabilă a rig-ului. Culorile efectelor pe strat nu pot fi animate (se folosește prima valoare), iar
dimensiunile sunt în unități de scenă (LIMITATIONS L-23).

### dotLottie (`.lottie`)

Un zip cu `manifest.json` (`{"animations":[{"id":"animation"}],"generator":"OSATURA","version":"2"}`) și
`a/animation.json` (același Lottie ca mai sus, comprimat deflate: 46,843 de octeți au devenit un fișier de 11,515 octeți). `--report` funcționează în același
mod. **Importul** unui `.lottie` nu este acceptat (`import-lottie` are nevoie de JSON; dezarhivează mai întâi `a/animation.json`).

### Validare și avertizări

* `python3 -m json.tool out.json > /dev/null` pentru corectitudinea formei; `python3 -c "import json;d=json.load(open('out.json'));print(d['v'],d['fr'],d['op'],len(d['layers']))"`.
* `unzip -l out.lottie` trebuie să arate exact cele două intrări.
* Dus-întors: `osatura import-lottie out.json back.json` urmat de `osatura check back.json` verifică faptul că OSATURA își poate citi propria
  ieșire (dus-întorsul are pierderi: rig-ul dispare).
* **Nu a fost disponibil niciun player Lottie independent** la momentul scrierii (LIMITATIONS L-28): ieșirea a fost verificată față de structura câmpurilor
  Lottie și față de importatorul propriu al OSATURA. Testează în playerul tău țintă (lottie-web, dotLottie player, Skottie, ...) și citește
  avertismentele; dacă exactitatea contează, folosește `.html` sau video.
* **Sfaturi de dimensiune:** `export-fit --format lottie` caută prin patru parametri (`decimals`, `path_eps`, `key_tol`, `frame_step`) în raport cu un buget de octeți;
  `lod` reduce ancorele înainte de export; `size-report` arată dimensiunea Lottie coapte (baked) pe fiecare strat (secțiunea 13).
* **Brevete:** Lottie și dotLottie sunt formate deschise; nimic din PATENT_NOTES.md nu le privește.

## 7. glTF binar (.glb)

```
$ osatura export small.json out.glb
wrote out.glb
warning: [glb-limit] blend mode Screen of 'sun' is not exported (glTF materials only alpha-blend); it is drawn as normal
warning: [glb-limit] animated opacity of 'target' is not exported (frozen at frame 0)
```
glTF 2.0 binar (magic `glTF`, versiunea 2, `"generator": "osatura"`), doar funcționalități de bază. Proiectul trebuie să aibă oase
(altfel `the project has no bones to export`).

* **Exact:** ierarhia oaselor și animația ei (eșantionată la fiecare cadru, interpolare liniară, shear eliminat).
* **Umplerile** devin mesh-uri plate de triunghiuri (ear clipping). Formele sunt rigide (un strat cu un os urmează nodul acelui os) sau **cu skin**
  (JOINTS_0/WEIGHTS_0, 4 influențe pe vertex) când fiecare ancoră poartă greutăți de os.
* **Contururile** sunt triangulate în mesh-uri proprii (lățime, profil de lățime, capete butt/round/square, îmbinări miter/round/bevel, linii punctate);
  contururile translucide pot arăta mai întunecate acolo unde triunghiurile se suprapun.
* **Gradientele** (liniare și radiale) devin un mic PNG generat, folosit ca textură de culoare de bază; spread de tip pad (clamp).
* **Măștile** sunt coapte (baked) prin tăierea triunghiurilor față de umplerea stratului-mască, doar pentru o sursă de mască vectorială cu umpleri opace și fără
  contururi, doar pentru forma măștii din cadrul 0. **Straturile de imagine** devin quad-uri texturate sau mesh-uri de deformare cu PNG încorporat.
* **Morph-urile de cale** devin ținte morph cu greutăți animate când verificarea pe cadru reproduce geometria motorului; în caz contrar
  calea rămâne o formă statică din cadrul 0, iar motivul este raportat.
* **Neexportate (raportate ca avertismente `[glb-limit]`):** moduri de blend altele decât normal (desenate ca normal), opacitatea animată a stratului (înghețată la
  cadrul 0), animația de transformare a straturilor fără os, efecte de strat, deformatoare dincolo de forma lor din cadrul 0, căi animate fără un
  morph valid, forme de mască animate, sunet.
* **Coordonate:** y este negat (glTF are y în sus); formele sunt stratificate cu un mic offset z (0.01 pe formă) pentru ordinea de desenare.
* **Validare:** antetul este `glTF`, versiunea 2, iar câmpul de lungime este egal cu dimensiunea fișierului (one-liner `python3` în secțiunea 16). Folosește un validator
  sau vizualizator glTF la alegere pentru verificarea completă; niciunul nu este inclus și niciunul nu a fost rulat pentru acest document. Exemplul de mai sus are 22 de noduri,
  15 mesh-uri, 1 skin, 1 animație și 7 imagini încorporate.
* **Sfaturi de dimensiune:** mai puține ancore vectoriale (`lod`), fără gradiente (fiecare este o textură), mai puține cadre (animația este eșantionată pe cadru).
* **Brevete:** format deschis; nicio indicație în PATENT_NOTES.md.

## 8. Folderul Open Cel Animation (OCA)

```
$ osatura export-oca small.json oca1                 # one layer "render" with all composited frames
$ osatura export-oca small.json oca2 --layers        # one OCA layer per top-level layer
wrote oca2/small.oca
```
Scrie `<out_dir>/<project name>.oca` (JSON, `ocaVersion` 1.1) plus cel-uri PNG `layerN/fNNNN.png`. Fără `--layers` există un singur strat
`render` cu cadrele compuse; cu `--layers` fiecare strat de nivel superior este randat singur pe transparență (11 straturi pentru demo).
`--scale N` înmulțește dimensiunea cel-urilor. **Pierderi cunoscute:** cel-urile sunt rastere (nu supraviețuiesc vectori, oase sau canale); opacitatea și blend-ul
stratului nu sunt scrise (fiecare strat este un `paintlayer`, opacitate 1); sunetele nu sunt exportate; fișierele dintr-un folder existent sunt suprascrise.
**Import:** nu există nicio comandă `import-oca` în CLI. **Validare:** fișierul `.oca` este JSON (`python3 -m json.tool`); fiecare cel listat
trebuie să existe ca PNG.

## 9. Import: SVG, OpenRaster, Lottie, imagini, GIF/APNG

Importurile creează un proiect **nou** (nu se îmbină într-unul existent), dacă nu se specifică altfel; folosește `script` sau editează JSON-ul pentru a combina.

### SVG (`import-svg`)

```
$ osatura import-svg art.svg svg.json
warning: unsupported paint 'url(#g)'; using grey
warning: <text> is not supported and was skipped
wrote svg.json (3 paths)
```
Subset: `path` (toate comenzile), `rect` (colțurile rotunjite sunt ignorate), `circle`, `ellipse`, `line`, `polyline`, `polygon`, `g`/`a` cu `transform`,
`fill` solid, `stroke`, `stroke-width`. Rezultat: 24 fps, 24 de cadre, dimensiune din `viewBox` sau `width`/`height` (altfel 512x512), **un singur strat
vectorial `svg`**. Sărite cu o linie `warning:` (afișată pe **stdout**): `<text>`, `<filter>`, `<clipPath>`, `<mask>`, `<pattern>`,
`<linearGradient>`/`<radialGradient>` (o vopsea cu gradient revine la gri), `<image>`, `<use>`, `<foreignObject>`. Limite: 8 MB de SVG,
200,000 de ancore. Pentru fotografii și rastere folosește `trace` / `autotrace` (CLI_REFERENCE secțiunea 9).

### OpenRaster (`import-ora`)

```
$ osatura import-ora art.ora ora.json --fps 12 --frames 24
```
Un **strat de imagine pentru fiecare strat raster** (PNG încorporat), ordine de jos în sus, straturile ascunse sunt păstrate, dar invizibile, grupurile sunt aplatizate, opacitatea,
vizibilitatea și poziția pe strat, precum și operațiile de compunere uzuale `svg:` (multiply, screen, overlay, darken, lighten, color-dodge,
color-burn, hard-light, soft-light, difference, exclusion, plus mapat la add). Opacitatea unei stive imbricate se înmulțește în straturile ei, iar modul ei de
blend este moștenit, dar straturile care se suprapun în interiorul unei stive translucide **nu** sunt aplatizate mai întâi. Numele duplicate primesc `_1`, `_2`.
Limite: 2000 de intrări de arhivă, 128 MiB pe intrare, straturi de 64 Mpixeli. Un fișier care nu este zip: `not a zip/ORA file: ...` (cod de ieșire 1). Nu există export
ORA. Importul PSD nu este implementat (LIMITATIONS L-32).

### Lottie (`import-lottie`)

```
$ osatura import-lottie in.json out.json [fx.json]
warning: layer 'text' of type 5 skipped
imported 1 layers; wrote out.json
```
Un **subset** deliberat; fiecare funcționalitate sărită este o linie `warning:`.
* **Importat:** shape layers cu căi `sh`, `rc`, `el`; umpleri solide și cu gradient (capetele/centrul/raza animate ale gradientului devin
  canale `grad_dx`/`grad_dy`); contururi; grupuri; transformările stratului (poziție, scală, rotație, opacitate statice sau cu chei; căile de poziție
  curbe sunt coapte (baked) pe cadru); moduri de blend; un strat solid de fundal; straturi de imagine cu PNG/JPEG data URI încorporate; straturi null ca părinți
  (transformările lor sunt coapte în copii); măști de strat (add/subtract/intersect cu opacitate, inversare, forme cu chei); luma matte-uri
  (aproximate); precompoziții; intervalele in/out ale stratului (ca pistă de opacitate cu pas); easing-ul cheilor (hold devine step, mânerele bezier devin
  cubic-bezier, lipsa devine liniar).
* **Căi animate:** cheile de formă cu aceeași topologie (același număr de ancore și deschis/închis) devin o cale de bază plus o **țintă morph**
  pentru fiecare cheie distinctă suplimentară (`k1`, `k2`, ...) cu canale de greutate `path_morph` care poartă easing-ul. O topologie diferită folosește prima formă
  și afișează un avertisment.
* **Efecte (`ef`):** Gaussian blur, drop shadow, tint și fill devin **efecte pe strat**, cu numerele animate ca canale `fx_*`;
  tint/fill/tritone/levels statice pe shape layers sunt coapte în culori. Culorile efectelor nu pot fi animate (se folosește prima valoare); celelalte
  efecte sunt avertismente.
* **Sărite cu avertisment:** text și alte tipuri de straturi, asset-uri audio, puncte de ancorare animate (se folosește prima valoare), căi de mișcare curbe pe
  alte proprietăți decât poziția, măști pe o sursă track-matte, lanțuri de părinți prea lungi pentru bake, mai mult de 2000 de straturi, animație mai lungă
  de 20,000 de cadre.
* `fx.json` este păstrat pentru compatibilitate: se scrie un array gol.
* **Neacceptat:** intrare `.lottie` zip. **Capcană:** nu există gardă de suprascriere față de intrare: `import-lottie in.json in.json` înlocuiește
  Lottie-ul cu proiectul.

### Imagini, GIF și APNG ca straturi cadru-cu-cadru

```
$ osatura import-seq demo.json is.json frames seq hold=2 loop=on start=4 x=10 y=20      # folder of PNGs
$ osatura import-anim demo.json ia.json gifl dof.gif loop=off start=0                    # animated GIF/APNG
```
`import-seq` preia toate PNG-urile dintr-un folder în ordinea numelor de fișier (toate de aceeași dimensiune); `import-anim` citește un GIF sau APNG animat folosind
întârzierile proprii ale fișierului (un PNG simplu dă un cadru). Ambele adaugă un strat switch cu o imagine încorporată pe cadru plus un canal cu pas; lucrează pe un proiect
**existent**. Un GIF lung face proiectul mare (fiecare cadru este un PNG încorporat). `trace` și `autotrace` transformă bitmap-urile în căi
vectoriale (`trace` citește canalul alpha; `autotrace` cuantizează culorile și potrivește curbe Bezier).

## 10. Import: motion capture BVH, MIDI și alte intrări live

### BVH (`import-bvh`, `mocap-info`, `mocap-match`, `mocap-retarget`)

```
$ osatura mocap-info walk.bvh
$ osatura import-bvh demo.json walk.bvh ib.json --joint-map LeftUpLeg=thigh,LeftLeg=shin --axis Zrotation --tol 0.5
wrote ib.json (+2 channels from 4 BVH frames)
```
`import-bvh` mapează canalele de rotație ale articulațiilor alese pe oase, ca canale `bone_angle` cu chei, eșantionate la fps-ul și numărul de cadre ale proiectului
(`--axis` selectează `Zrotation`, `Xrotation` sau `Yrotation`; `--tol` simplifică cheile). `mocap-match` propune o potrivire articulație-os cu un tabel de
încredere, iar `mocap-retarget` coace (bake) canale adăugate cu mișcarea rădăcinii, o vedere (`front|side|top|yaw=DEG`) și scală. Subsetul text BVH
doar: fără formate mocap binare, fără FBX (intenționat). **Limite (LIMITATIONS L-45):** nu a fost testat niciun fișier real de motion capture; proiecția
2-D elimină adâncimea; răsucirea în jurul axei unui os, degetele și fața sunt ignorate; rig-ul își păstrează lungimile oaselor, deci alunecarea picioarelor este doar
redusă.

### MIDI și alte intrări live (`live-map`)

```
$ osatura live-map small.json song.mid mapping.json out.json
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 0 events (0 after the last frame skipped); wrote out.json
```
Un **Standard MIDI File** (`.mid`; mesaje MIDI 1.0, running status, meta-evenimente de tempo, diviziune în ticks sau SMPTE) sau un JSON de evenimente
(`{"events":[{"source","id","value","time"}]}`, valoare normalizată 0..1, timp în secunde) este transformat în canale cu chei, events și take-uri printr-un
fișier de mapare. Surse: `midi_cc midi_note midi_pitch_bend mouse_x mouse_y mouse_button gamepad_axis gamepad_button key face`. Cel mult
3,000,000 de evenimente; timpi în limita a 7 zile. Nimic din CLI nu comunică cu un dispozitiv MIDI; studio folosește Web MIDI al browserului. MIDI este **doar
import** (nu există export MIDI). Fișierele malformate returnează o eroare, niciodată o cădere.

## 11. Formate audio

| Utilizare | Formate | Note |
|---|---|---|
| Analiză (`lipsync`, `lipsync-text`, `speech-motion`, `audio-react`, `beats`, `waveform`) | doar **WAV** | PCM întreg pe 8/16/24/32 de biți sau float pe 32 de biți, orice număr de canale mixat la mono, până la 256 MiB. Convertește mai întâi MP3/OGG (`ffmpeg -i in.mp3 out.wav`). |
| Piste de sunet ale proiectului (`sound add`) | WAV, OGG, MP3, orice acceptă `ffprobe` | Stocate ca **referință** (cale relativă, cadru de start, gain 0..4, fereastră de trim, fade-uri); nimic nu este încorporat; fișierul trebuie să rămână în folderul proiectului, cu excepția cazului în care se folosește `--allow-external-assets`. |
| Ieșire mixată (`sound mix`) | `.wav` (PCM 44.1 kHz pe 16 biți), `.ogg` (Vorbis q4), `.mp3` (96 kb/s) | Mixată prin ffmpeg pentru **lungimea proiectului**; `.flac` și altele: `unknown audio format '.flac' (use .wav, .ogg or .mp3)`. |
| În exporturi | `.mp4` AAC, `.webm` Opus, `.html` mp3 încorporat | `.gif`/`.apng` rămân fără sunet, cu o notă; Lottie avertizează `[audio]`; GLB și OCA elimină sunetul. |
| Indicii pentru gură (`mouth`) | text Papagayo/Moho-switch, TSV Rhubarb | Nu este audio: fișiere de indicii. |

Encoder-ele folosite: `libmp3lame` (mp3), `libvorbis` (ogg), `aac`, `libopus`. Fără pan/EQ/ducking pe pistă sau buclare (ROADMAP secțiunea 9).

## 12. Matricea de suport al funcționalităților

Rândurile sunt lucruri pe care le poate conține un proiect, coloanele sunt formatele de ieșire. **da** = păstrat ca în proiect, **aprox.** = păstrat cu o
diferență vizibilă, **baked** = prezent doar ca rezultate pe cadru, **nu** = pierdut. Celulele pentru formatele pe care nu le-am putut verifica într-un vizualizator independent
descriu ce scrie exportatorul.

| Funcționalitate | SVG / PNG | Sprite sheet / APNG | GIF | WebM / MP4 | HTML | Lottie / .lottie | GLB | OCA |
|---|---|---|---|---|---|---|---|---|
| Forme vectoriale, umpleri | da | baked (raster) | baked (raster) | baked (raster) | da (SVG) | da | aprox. (triunghiuri) | baked (raster) |
| Gradiente | da | baked | baked | baked | da | da (aprox. dacă stratul are skew) | da (textură) | baked |
| Contururi cu profil de lățime | da | baked | baked | baked | da | aprox. (lățime constantă) | da | baked |
| Linii punctate, capete, îmbinări | da | baked | baked | baked | da | da | da (triangulate) | baked |
| Straturi de imagine (bitmap) | da | baked | baked | baked | da | nu (`image-layer`) | da | baked |
| Oase și animație | baked | baked | baked | baked | baked | baked | **da (exact)** | baked |
| Skinning / deformatoare | baked | baked | baked | baked | baked | baked | cu skin; deformatoare în cadrul 0 | baked |
| Cameră și parallax | baked | baked | baked | baked | baked | baked (mișcările camerei sunt în geometrie) | nu (formele provin din cadrul 0) | baked |
| Moduri de blend | da | baked | baked | baked | da | aprox. (`add` devine `screen`; niciunul pe grupuri) | nu (normal) | baked |
| Măști de strat | da | baked | baked | baked | da | doar straturi singulare | forma din cadrul 0, restricționat | baked |
| Efecte de strat (blur, shadow...) | da | baked | baked | baked | da | straturi cu o singură frunză | nu | baked |
| Opacitate animată | baked | baked | baked | baked | baked | da | nu (înghețată la cadrul 0) | baked |
| Morph-uri de cale | baked | baked | baked | baked | baked | baked | ținte morph (când sunt valide) | baked |
| Depth of field | prin `dof-export` | prin `dof-export` | prin `dof-export` | prin `dof-export` | nu | nu (avertisment) | nu | nu |
| Lanț de post-efecte (`fx`) | doar PNG | nu | nu | nu | nu | nu | nu | nu |
| Transparență | da | da (alpha) | 1 bit | WebM da, MP4 nu | da | da | da | da |
| Audio | nu | nu | nu | **da** | da (mp3) | nu (avertisment) | nu | nu |
| Buclare | n/a | sheet: n/a, APNG: da | da | depinde de player | player | player (dotLottie 2.0 lasă bucla în seama playerului sau a unei mașini de stări) | n/a | n/a |
| Rig/canale editabile ulterior | nu | nu | nu | nu | nu | nu | doar oase | nu |
| Mașini de stări, binding-uri, containere, scroll | nu | nu | nu | nu | nu | nu | nu | nu |
| Independent de rezoluție | da (SVG) | nu | nu | nu | da | da | da | nu |
| Necesită ffmpeg | nu | nu | nu | **da** | doar pentru audio | nu | nu | nu |

„Baked” înseamnă că efectul funcționalității se află în pixeli sau în geometria pe cadru, dar funcționalitatea în sine nu mai poate fi editată ulterior.

## 13. Fișiere mai mici

1. **Vezi unde sunt octeții:** `osatura size-report project.json` listează fișierul proiectului, Lottie-ul baked și pagina HTML pe straturi,
   imagini și canale (`--json` pentru scripturi).
2. **Încadrează automat într-un buget:** `osatura export-fit project.json out.json --max-size 50KB --format lottie` caută prin parametrii exportatorului și
   păstrează cel mai bun SSIM care se încadrează. Formate: `lottie`, `html`, `gif`, `apng`, `webm`, `project` (un JSON de proiect subțiat). `--measure gzip` bugetează
   dimensiunea gzip, `--min-ssim X` stabilește un prag de calitate, `--max-seconds N` este o limită reală de timp (se păstrează cel mai bun rezultat de până atunci). Cod de ieșire 1 cu cel mai
   mic rezultat când bugetul nu poate fi atins.
3. **Redu mai întâi grafica:** `osatura lod project.json --tolerance-px 1 --levels 3 --out-dir lv --verify` scrie proiecte cu niveluri de detaliu, cu o
   eroare declarată (și, cu `--verify`, măsurată) în pixeli de ieșire; `lod-pick --screen-px WxH` alege nivelul cel mai grosier pentru un ecran dat.
4. **Pe format:**
   * Lottie/HTML/GLB cresc cu `frames x anchors`: mai puține cadre, mai puține ancore, un fps mai mic, intervale de chei tăiate (unelte `keys`).
   * GIF/APNG cresc cu pixeli x cadre: `--scale 0.5` înseamnă o reducere de 4 ori; GIF se potrivește graficii plate, APNG se potrivește alpha-ului, WebM/MP4 se potrivesc oricărui conținut cu mișcare.
   * `.lottie` este același Lottie comprimat deflate (de aproximativ 4 ori mai mic în exemplu); servește `.html`/`.json` comprimate gzip.
   * Imaginile dintr-un proiect sunt încorporate ca PNG base64: micșorează-le înainte de import.
5. Limite: un Lottie al unui rig cu 14 straturi nu poate fi făcut oricât de mic (un buget de 1 KB pe exemplu raportează „every knob is at its floor”);
   căutarea necesită timp pe proiecte mari.

## 14. Cerințe ffmpeg

OSATURA nu include și nu leagă niciodată ffmpeg; rulează `ffmpeg` (și `ffprobe`) găsit în `PATH` ca **subproces extern** al propriei tale instalări.

| Necesar pentru | Encoder/decoder | Verificat pe |
|---|---|---|
| `export .mp4` | `libx264` (`yuv420p`), `aac` | ffmpeg 6.1.1 |
| `export .webm` (implicit) | `libvpx-vp9`, `libopus` | ffmpeg 6.1.1 |
| `export .webm --codec av1` | `libaom-av1`, `libopus` | ffmpeg 6.1.1 |
| `dof-export .mp4/.webm` | la fel ca mai sus, `--codec` | ffmpeg 6.1.1 |
| `sound mix .ogg` | `libvorbis` | ffmpeg 6.1.1 |
| `sound mix .mp3`, audio `.html` | `libmp3lame` | ffmpeg 6.1.1 |
| `face-track` dintr-un video | doar decoder-e | |
| verificarea conținutului la `sound add` | `ffprobe` (fără el, pentru fișierele non-WAV se verifică doar existența) | |

Verifică-ți instalarea cu `ffmpeg -hide_banner -encoders | grep -E 'libx264|aac|libvpx-vp9|libaom-av1|libopus|libvorbis|libmp3lame'`. Tot restul
(GIF, APNG, HTML fără audio, Lottie, dotLottie, GLB, OCA, PNG, SVG, EXR) funcționează fără ffmpeg.
Dacă ffmpeg lipsește, comanda eșuează cu `cannot run ffmpeg (...)` și nu scrie nimic; un encoder lipsă dă `ffmpeg failed`.

## 15. Indicații despre brevete și licențe

* [PATENT_NOTES.md](PATENT_NOTES.md) consemnează ce **nu** a fost analizat. Pentru export, rândul relevant este *Video and audio codecs through ffmpeg
  (`export` .mp4 / .webm, `soundtrack`)*: H.264 și AAC sunt acoperite de pool-uri de licențiere; VP9 și AV1 sunt promovate ca fără redevențe, dar pentru ambele au fost anunțate
  pretenții ale unor pool-uri și ale unor titulari individuali; **stare: neevaluat, nu considera VP9/AV1 drept clarificate.** Deoarece ffmpeg este instalarea proprie a utilizatorului, fiecare
  alegere de codec este responsabilitatea utilizatorului, iar OSATURA nu trebuie să includă ffmpeg.
* Writer-ii deterministi pentru raster, GIF, APNG, PNG, SVG, OpenEXR, OCA, ORA, glTF și Lottie sunt implementări proprii OSATURA, pornind de la descrieri publice
  ale formatelor (clean-room, vezi CLEANROOM.md). Starea lor privind brevetele nu este evaluată dincolo de aceasta: PATENT_NOTES.md listează doar domeniile de mai sus
  (puppeteering condus de față, mașini de stări și data binding, FABRIK/MVC/greutăți, umplere stencil-then-cover, codec-uri).
* GIF: brevetele LZW au expirat de mult în toate jurisdicțiile; aceasta este cunoaștere generală, nu o clarificare consemnată în depozit.
* Licențele fișierelor produse: fișierele pe care le exporți sunt ale tale; motorul este MPL-2.0 iar CLI AGPL-3.0-or-later, ceea ce se aplică programelor, nu
  fișierelor pe care acestea le scriu. Studio oferă codul sursă corespunzător la `/source` (AGPL secțiunea 13).
* Aceasta nu este consultanță juridică.

## 16. Validarea a ceea ce ai scris

| Format | Verificare | Rezultat așteptat |
|---|---|---|
| orice proiect | `osatura check out.json` | `ok: ...` (și după `import-*`) |
| GIF | `ffprobe -v error -count_frames -select_streams v -show_entries stream=nb_read_frames -of csv=p=0 out.gif` | numărul de cadre (8) |
| APNG | aceeași comandă pe `out.apng` | numărul de cadre (8) |
| MP4 | `ffprobe -v error -show_entries stream=codec_name,width,height,pix_fmt,nb_frames,r_frame_rate -of csv=p=0 out.mp4` | `h264,320,240,yuv420p,24/1,8` |
| WebM | la fel pe `out.webm` cu `:stream_tags=alpha_mode` | `vp9,...,yuv420p,...,1` (alpha_mode 1) |
| EXR | `ffprobe f.exr` | `exr 640x480 gbrapf32le` |
| Lottie | `python3 -c "import json;d=json.load(open('out.json'));print(d['v'],d['fr'],d['op'],d['w'],d['h'],len(d['layers']))"` | `5.7.0 24.0 8 640.0 480.0 12` |
| Avertismente Lottie | `osatura export p.json out.json --report rep.json` | `rep.json` listează fiecare aproximare |
| dotLottie | `unzip -l out.lottie` | doar `manifest.json` și `a/animation.json` |
| Dus-întors Lottie | `osatura import-lottie out.json back.json && osatura check back.json` | `ok:` |
| GLB | `python3 -c "import struct;b=open('out.glb','rb').read();print(b[:4],struct.unpack('<II',b[4:12]),len(b))"` | `b'glTF' (2, N)` cu N egal cu lungimea |
| HTML | `grep -c '<svg' out.html`; `grep -o 'http[^"<]*' out.html` | un `<svg` pe cadru; doar URL-ul namespace-ului SVG |
| SVG | deschide într-un browser sau parsează ca XML | bine format |
| Sprite sheet | `python3 -m json.tool out.atlas.json` | `{index,x,y,w,h}` pe cadru |
| OCA | `python3 -m json.tool <name>.oca` | fiecare cel referit există |
| Regresie vizuală | `osatura ci diff a.png b.png --metric ssim` | cod de ieșire 0 când sunt egale; vezi [CI.md](CI.md) |

Pentru verificări vizuale ale unui export cu pierderi față de original, `osatura export-fit` raportează deja SSIM și CIEDE2000 față de randarea originală, iar
`osatura compare` scrie o imagine înainte/după/diferență a două proiecte.

## 17. Diferențe față de alte documente

* `docs/LIMITATIONS.md` L-29 spune că importul Lottie nu preia asset-uri de imagine, efecte, precomp-uri sau matte-uri; importatorul actual importă imagini
  cu data URI încorporate, precompoziții, efecte pe strat (blur, drop shadow, tint, fill) și aproximează luma matte-urile (secțiunea 9). Lista
  funcționalităților sărite din secțiunea 9 este cea despre care comanda avertizează în realitate.
* `docs/LIMITATIONS.md` L-32 listează OCA printre formatele de import; CLI are un **export** OCA (`export-oca`) și nicio comandă `import-oca`.
* Textul de utilizare al lui `export` numește doar `.gif .html .json .mp4` plus „formatele deschise”; lista completă de extensii este cea din secțiunea 2.
* Alte neconcordanțe între textul de utilizare și comportament sunt adunate la sfârșitul [CLI_REFERENCE.md](CLI_REFERENCE.md).
