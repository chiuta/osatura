<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# Referință pentru linia de comandă OSATURA

Referință pentru binarul `osatura` (crate-ul `osatura-cli`, AGPL-3.0-or-later; leagă motorul MPL-2.0
`osatura-core`). Versiune documentată: **osatura 0.1.0**. Acest text este licențiat CC-BY-4.0.

Fiecare sinopsis de mai jos este copiat din textul de utilizare al programului (`osatura` fără argumente sau
`osatura <command> --help`). Fiecare exemplu a fost executat pe proiectul demo încorporat, iar rezultatul afișat
este trunchiat din rezultatul real. Acolo unde textul de utilizare și comportamentul real nu coincid, acest document descrie
comportamentul real și listează discrepanța în [Neconcordanțe cunoscute între documentație și comportament](#neconcordanțe-cunoscute-între-documentație-și-comportament) la final.

Documente conexe: [EXPORT_FORMATS.md](EXPORT_FORMATS.md) (ce păstrează și ce pierde fiecare format de export/import),
[CI.md](CI.md) (manifeste CI pentru animații), [API.md](API.md) (bibliotecă/ABI C), [PATENT_NOTES.md](PATENT_NOTES.md)
(starea codecurilor și a brevetelor, nu sfat juridic).

## Cuprins

1. [Convenții comune tuturor comenzilor](#1-convenții-comune-tuturor-comenzilor)
2. [Primii pași: proiectul demo](#2-primii-pași-proiectul-demo)
3. [Crearea și inspecția proiectelor](#3-crearea-și-inspecția-proiectelor)
4. [Randare și export](#4-randare-și-export)
5. [Rigging, skin-uri, IK și ajutoare de layout](#5-rigging-skin-uri-ik-și-ajutoare-de-layout)
6. [Animație, chei, timeline și generatoare](#6-animație-chei-timeline-și-generatoare)
7. [Fizică, cloth și deformatoare](#7-fizică-cloth-și-deformatoare)
8. [Audio, lip-sync și sunete](#8-audio-lip-sync-și-sunete)
9. [Import (SVG, OpenRaster, Lottie, BVH, GIF/APNG, secvențe PNG, bitmap-uri)](#9-import)
10. [Intrare live, MIDI, urmărirea feței și motion capture](#10-intrare-live-midi-urmărirea-feței-și-motion-capture)
11. [Mașini de stări, legări de date, containere și regiuni de derulare](#11-mașini-de-stări-legări-de-date-containere-și-regiuni-de-derulare)
12. [Efecte, adâncime de câmp, onion skin, trasee și cameră](#12-efecte-profunzime-de-câmp-onion-skin-urme-și-cameră)
13. [Nivel de detaliu, rapoarte de dimensiune și export-fit](#13-nivel-de-detaliu-rapoarte-de-dimensiune-și-export-fit)
14. [Asigurarea calității: check, lint, ci, golden, fuzz, diff](#14-asigurarea-calității)
15. [Controlul versiunilor (`vcs`)](#15-controlul-versiunilor-vcs)
16. [Studio, colaborare, server agent MCP și registrul UI](#16-studio-colaborare-serverul-agent-mcp-și-registrul-ui)
17. [Baza de date de poziții, sugestii și alte ajutoare](#17-bază-de-date-de-poze-sugestii-și-alte-ajutoare)
18. [Neconcordanțe cunoscute între documentație și comportament](#neconcordanțe-cunoscute-între-documentație-și-comportament)
19. [Index alfabetic al comenzilor](#index-alfabetic-al-comenzilor)

---

## 1. Convenții comune tuturor comenzilor

### 1.1 Forma liniei de comandă

```
osatura <command> [sub-command] <positional arguments...> [options]
```

* Opțiunile pot apărea **oriunde** în linie, și după sub-comandă, și după argumentele poziționale
  (`osatura render demo.json 0 o.png --scale 2` și `osatura --scale 2 render ...` sunt echivalente).
* **Opțiunile cu valoare** iau cuvântul următor (`--scale 2`) sau forma `--key=value` (`--scale=2`). Forma cu `=` funcționează pentru
  orice opțiune cu valoare. O valoare care conține ea însăși `=` este în regulă în forma cu spațiu (`--remap a=b`).
* **Flagurile** (opțiuni booleene precum `--bones`) nu primesc valoare. `--bones=1` este refuzat:
  `--bones is a flag and takes no value: write --bones (without =), got '--bones=1'`.
* **Opțiuni stricte.** O opțiune (flag sau opțiune cu valoare) pe care comanda nu o documentează este o eroare, cu o sugestie
  „did you mean” (exit 1), în loc să fie ignorată:
  ```
  $ osatura render demo.json 0 o.svg --bone
  unknown option --bone for 'render' (did you mean --bones?); see: osatura render --help
  ```
  Verificarea se aplică fiecărei comenzi care are o linie de utilizare, inclusiv `check`, `stats`, `lint`, `coach`, `contracts` și
  `machine list` (versiunile mai vechi ignorau în tăcere flagurile suplimentare de acolo). `--json`, `--quiet` și `--allow-empty` **nu** sunt
  globale: sunt refuzate de orice comandă a cărei linie de utilizare nu le listează.
  **Verificarea se face per sub-comandă (per formă de sinopsis), nu per comandă.** O opțiune listată pentru o formă este refuzată
  la formele-surori, iar mesajul numește forma care o acceptă:
  ```
  $ osatura ci diff a.png b.png --quiet
  unknown option --quiet for 'ci diff' (--quiet applies to: ci run); see: osatura ci --help
  ```
  Așadar `--quiet` este doar pentru `ci run`, `--json` doar pentru `vcs log|status|diff|show`, `ui commands|shortcuts` și
  forma `trail ... --target`, `--tol` doar pentru `golden check` (și `ci diff`), `--scale` doar pentru `golden write`, iar
  fiecare editare `keyedit` acceptă doar opțiunile scrise după numele ei în sinopsis. (`export` își reunește în continuare cele două
  linii de sinopsis, pentru că formatul se alege după extensia fișierului; `--codec`, `--columns` și `--scale` care nu
  se aplică extensiei alese sunt semnalate chiar de `export`.)
* Unele comenzi mai vechi primesc **cuvinte** `key=value` în loc de `--options` (`cloth`, `physics`, `cage`, `warp-pivot`,
  `soft-scale`, `mesh-lock`, `glue`, `audio-react`, `beats`, `import-seq`, `import-anim`, `layer-pivot`, `chan-meta`,
  `waveform`). Acestea sunt poziționale și se scriu exact ca în sinopsis (`gravity=900`, nu `--gravity 900`).
* Un **cadru** este un număr (fracțiile sunt permise acolo unde are sens). Un proiect cu `frames = N` are cadrele `0..N-1`.
  Straturile se adresează prin **cale** (`group/child`), oasele prin **nume**; camera are ținta goală `""`.
* **Numele proprietăților** (folosite de `--prop`, `slider`, `drive`, fișierele de mapare, ...) sunt numele în snake_case:
  `bone_angle bone_scale bone_offset_x bone_offset_y layer_opacity layer_x layer_y layer_rotation layer_scale_x
  layer_scale_y layer_skew layer_switch layer_wobble camera_x camera_y camera_zoom camera_rotation camera_focus
  camera_blur ik_mix constraint_weight path_morph point_dx point_dy lattice_dx lattice_dy spine_dx spine_dy pin_dx
  pin_dy cage_dx cage_dy fx_blur fx_dx fx_dy fx_soft fx_amount grad_dx grad_dy`.  Unele comenzi afișează aceleași proprietăți în CamelCase (`BoneAngle`) în mesajele lor; valorile opțiunilor sunt întotdeauna în snake_case.

### 1.2 Opțiuni globale

| Opțiune | Semnificație |
|---|---|
| `--allow-external-assets` | Permite unui proiect să numească fișiere prin cale absolută sau în afara propriului director (sunete, imagini, normal map-uri...). Implicit: refuzat. |
| `--follow-symlinks` | Scrie în **ținta** unei legături simbolice date ca fișier de ieșire. Implicit: refuzat. |
| `--max-input-bytes N` | Cel mai mare fișier de proiect care se citește. Implicit `268435456` (256 MiB). Trebuie să fie un număr întreg pozitiv. |
| `<command> --help` | Afișează pe stdout liniile de utilizare ale unei comenzi (exit 0). |
| `<command> --version`, `--version` | `osatura 0.1.0`; fără comandă afișează și linia de licență și oferta de cod sursă AGPL. |
| `--json`, `--quiet`, `--allow-empty` | Opțiuni per comandă, **refuzate** (`unknown option`) de comenzile a căror sinopsis nu le listează (`--json`: `pick`, `select-box`, `timeline`, `trail`, `size-report`, `collab-sim`, `vcs`, `ui`, `face-detect`, `pose-suggest`; `--allow-empty`: `physics`, `live-map`, `face-bake`; `--quiet`: `ci run`). Scripturile care le-au transmis uniform tuturor comenzilor trebuie să înceteze să o facă. |

Exemple (verificate):

```
$ osatura --max-input-bytes 100 check demo.json
cannot read demo.json: 51450 bytes exceed the input limit of 100 bytes (raise it with --max-input-bytes N)
[exit 1]

$ osatura check /dev/zero
cannot read /dev/zero: not a regular file (devices, pipes and directories are refused)
[exit 1]

$ ln -s real_target.svg link.svg                       # dangling link
$ osatura render demo.json 0 link.svg
cannot write link.svg: link.svg: refusing to write through a symbolic link; pass --follow-symlinks to write to the link's target, or name the target file yourself
[exit 1]
```

### 1.3 Citirea fișierelor de intrare

* Fișierele de intrare trebuie să fie **fișiere obișnuite** de cel mult `--max-input-bytes`. Dispozitivele, pipe-urile, directoarele și
  substituțiile de proces (`<(...)`) sunt refuzate: scrie mai întâi datele într-un fișier.
* Fișierele de proiect sunt JSON. Un proiect cu chei necunoscute este **acceptat**, iar cheile necunoscute sunt raportate ca
  `warning: unknown field bogus_key (ignored)` de `check` (și `lint`, `studio`). Fișierele auxiliare pe care le redactezi pentru o comandă
  (contracte, mapări, mașini, containere, regiuni de derulare, manifeste ci, hărți drive ...) sunt analizate **strict**: o cheie
  necunoscută este o eroare și nu se aplică nimic (`unknown field(s): elastic.damping (typo? nothing was applied)`).
* Un proiect este validat ori de câte ori este încărcat pentru randare sau export (`<file> is not a valid project:` urmat de
  câte o problemă pe linie). Comenzile de editare recompilează și proiectul pe care urmează să-l scrie și refuză să scrie unul care nu mai este valid.
* **Căile resurselor** dintr-un proiect (fișiere de sunet, fișiere de imagine, normal map-uri) se rezolvă relativ la **directorul fișierului de
  proiect**. Căile absolute și orice cale care conține `..` sunt refuzate, dacă nu se dă `--allow-external-assets`:
  ```
  $ osatura sound add s.json o.json x /etc/passwd
  sound add: asset '/etc/passwd': absolute paths are not allowed (a project may only reference files inside its own directory; use --allow-external-assets to override) (nothing was added)
  ```
  Flagul trebuie dat din nou la fiecare comandă ulterioară care încarcă proiectul respectiv (`check ext.json` eșuează fără el și reușește cu el).

### 1.4 Scrierea fișierelor de ieșire

* **Scrieri atomice.** Majoritatea comenzilor scriu printr-un fișier temporar din **același director** (`.<name>.tmp-<pid>-<random>`), îi fac `fsync`,
  copiază permisiunile fișierului vechi și îl `rename` peste țintă, astfel încât o cădere sau un disc plin nu lasă niciodată un fișier scris pe jumătate.
  Dacă nu se poate crea un fișier temporar (director doar în citire, dar fișier inscriptibil), fișierul este suprascris pe loc, ceea ce **nu** este atomic.
  FIFO-urile și fișierele speciale de tip `/dev/stdout` sunt scrise direct.
* **Legăturile simbolice** ca ieșire sunt refuzate dacă nu se dă `--follow-symlinks` (atunci se scrie ținta legăturii; o legătură orfană rămâne eroare).
  Un director dat ca fișier de ieșire este o eroare (`is a directory`).
* **Niciodată peste intrare.** `render`, `render-hdr`, `dof`, `dof-export`, `export` (și fișierul auxiliar `.json` de sprite-sheet al unui `.png`,
  și `--report`), `layout-preview`, `render-gpu`, `export-fit`, `fuzz-grid`, `onion`, `trail` și `golden` refuză o ieșire care este același fișier
  cu proiectul de intrare: `refusing to overwrite the input project demo.json with output of the same name`.
* **Comenzile de editare** (cele cu o pereche `<project.json> <out.json>`: `reparent`, `keys`, `physics`, ...) au voie să scrie **pe loc**
  (`osatura reparent p.json p.json hand none` funcționează și înlocuiește `p.json`). `import-lottie` refuză o ieșire (sau `fx.json`) care este
  același fișier cu intrarea sa; celelalte comenzi de import citesc alt tip de fișier decât scriu.
* **Ieșirile existente sunt suprascrise în tăcere** de orice comandă. Nu există `--force`.
* **Orice ieșire adresată de utilizator** (inclusiv rapoartele și golden-urile `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, fișierele `face-*`,
  fișierele de poziții, snapshot-urile, `vcs` și jurnalul de colaborare al studioului) trece prin același scriitor atomic, care refuză legăturile simbolice. Ieșirile
  `.mp4`/`.webm` scrise de ffmpeg sunt verificate pentru legături simbolice înainte de pornirea ffmpeg. Directoare părinte: `lod`, `export-fit`, `ci` și `face-*`
  creează directoarele lipsă; `fuzz`, `fuzz-grid` și `export` nu (`No such file or directory`).

### 1.5 stdout, stderr și coduri de ieșire

* Rezultatele („wrote out.json”, tabele, JSON) merg pe **stdout**. Erorile și majoritatea avertismentelor merg pe **stderr**. `import-svg` își afișează
  avertismentele pe stdout; `export` afișează avertismentele Lottie/GIF/GLB pe stderr.
* Codul de ieșire **0** înseamnă succes. Codul de ieșire **1** înseamnă orice eroare *sau* o verificare eșuată: un fișier ilizibil/invalid, argumente greșite,
  `lint` cu erori, `diff` cu diferențe, un contract încălcat, un golden care nu se potrivește, o rată de eșec fuzz peste limită,
  un buget `export-fit` care nu a putut fi respectat, `lod --verify` peste limită, divergență `collab-sim`, o verificare `ci` eșuată.
  Codul de ieșire **2** este folosit doar de `ci` (nu s-a putut evalua nimic: proiect/manifest lipsă, golden alterat).
* Nicio argumentare, o comandă necunoscută, un număr greșit de argumente poziționale sau `osatura --help` afișează utilizarea completă pe
  **stderr** și ies cu **1**. `osatura <command> --help` afișează pe stdout liniile de utilizare ale comenzii și iese cu 0.
  **Excepție (Windows, începând cu 0.1.0-alpha.2):** când `osatura.exe` este pornit prin dublu clic din Explorer (fereastra de consolă a fost creată pentru el și niciun alt proces nu o împarte) și nu are argumente, nu afișează utilizarea: creează `Documents\OSATURA\first-rig.json` (din `demo-person`) dacă lipsește, rulează `studio` pe acel fișier la `127.0.0.1:8080`, deschide browserul și ține consola deschisă până o închizi sau apeși Enter după o eroare. Pornit dintr-un terminal, se comportă ca mai sus. Restul programului rulează pe un fir cu stivă de 256 MiB (firul principal de pe Windows are doar 1 MiB).

### 1.6 Programe externe

`ffmpeg` (găsit în `PATH`) este necesar pentru: exportul `.mp4`/`.webm` și `dof-export` către acele formate, `sound mix`,
audio în `.html` (mp3) și `face-track` pe un fișier video. Encodere necesare: `libx264` (mp4), `libvpx-vp9` / `libaom-av1` (webm),
`aac` (sunet mp4), `libopus` (sunet webm), `libvorbis`, `libmp3lame` (sound mix). Nu este inclus niciun cod sau binar ffmpeg; vezi
[PATENT_NOTES.md](PATENT_NOTES.md). Nu se execută nimic altceva.

### 1.7 Notă despre performanță

Toate duratele din acest document provin dintr-o build **debug** și sunt de 10-30 de ori mai lente decât o build release. Comenzile intensive pe cadre
(`safety`, `golden`, `ci`, `export-fit`, `proxy-bench`) au fost rulate pe o copie redusă a demo-ului, de 8 cadre (`small.json`, obținută setând
`"frames": 8`).

---

## 2. Primii pași: proiectul demo

```
$ osatura demo demo.json
wrote demo.json
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels
$ osatura render demo.json 12 f12.png --bones --scale 2
wrote f12.png
$ osatura export demo.json demo.gif
wrote demo.gif
```

Demo-ul are oasele `arm_up arm_low hand thigh shin foot_ctrl`, un lanț IK `leg` și straturile de nivel superior
`sky sun hills ground shadow arm leg target blush mouth(switch: closed/open/smile) ink`. Toate exemplele din această referință îl folosesc,
dacă nu se spune altfel.

---

## 3. Crearea și inspecția proiectelor

### `demo`

**Sinopsis:** `osatura demo <out.json>`

Scrie ca fișier de proiect demo-ul de rig încorporat (640x480, 24 fps, 48 de cadre, 6 oase, picior IK, 14 straturi).
**Ieșire:** `wrote <out.json>`. **Erori:** argument lipsă (utilizare, exit 1); cale neinscriptibilă. **Conexe:** `demo-fx`, `check`.
**Limite:** demo-ul este o resursă fixă; este baza fiecărui exemplu din acest document.

```
$ osatura demo demo.json
wrote demo.json
```

### `demo-person`

**Sinopsis:** `osatura demo-person <out.json>`

Scrie ca fișier de proiect demo-ul cu persoana care merge (640x480, 24 fps, 32 de cadre, 42 de oase, 45 de straturi): o persoană văzută din lateral, cu cap, ureche și coadă de cal, gât, trunchi, două brațe cu mâini și câte cinci degete și două picioare cu labe și degetele piciorului. Ciclul de mers are doi pași și se repetă fără cusătură: bazinul se mișcă în sus și în jos și se înclină, trunchiul se apleacă și se rotește în contrasens, capul este ținut drept, cu o mică aplecare întârziată, brațele se balansează împotriva picioarelor, cu coatele îndoindu-se mai mult la balansul înainte, iar mâinile, degetele, urechea și coada de cal urmează ceva mai târziu. Fiecare picior aterizează pe călcâi, se așază plat și pleacă de pe vârful labei; solul, gardul și umbra se derulează exact cu viteza piciorului sprijinit, deci picioarele nu alunecă. Acesta este proiectul pe care lansatoarele Windows (`osatura.exe`, `osatura-studio.exe`) îl creează ca `first-rig.json` la prima pornire; `osatura demo` rămâne baza mică a exemplelor din acest document.
**Ieșire:** `wrote <out.json>`. **Erori:** argument lipsă (utilizare, exit 1); cale neinscriptibilă. **Conexe:** `demo`, `studio`.
**Limite:** cheile sunt eșantionate din două în două cadre, cu o cheie suplimentară înainte de cadrul 0 și una după ultimul cadru, astfel încât bucla este lină; `lint` raportează aceste două chei ca aflându-se în afara liniei de timp (informație, nu eroare). Figura este o vedere laterală, deci nu există întoarcere sau adâncime.

```
$ osatura demo-person person.json && osatura check person.json
wrote person.json
ok: 'OSATURA walking person' 640×480 @ 24 fps, 32 frames, 42 bones, 0 ik, 45 layers, 48 channels
```

### `demo-fx`

**Sinopsis:** `osatura demo-fx <out.json>`

Scrie demo-ul „lichid / boolean / particule” (640x480, 72 de cadre, 3 oase, 3 straturi) care exersează straturile procedurale.

```
$ osatura demo-fx fx.json && osatura check fx.json
wrote fx.json
ok: 'OSATURA fx demo' 640×480 @ 24 fps, 72 frames, 3 bones, 0 ik, 3 layers, 3 channels
```

### `check`

**Sinopsis:** `osatura check <project.json>`

Încarcă și validează un proiect. Afișează linii `warning:` (câmpuri necunoscute, probleme de gradient, probleme de chei switch și avertismentele de date de editor ale codului lint L013: zoom de cameră <= 0, opacitate de strat în afara 0..1, referințe sau timpi greșiți în skin-uri, comp-uri, markere, evenimente, shot-uri, channel_meta, bone_vis, clip-uri și lod-uri) și o linie-rezumat `ok:`. Sunt erori doar valorile care nu pot exista într-un fișier salvat (NaN sau infinit în acele structuri), așa că proiectele mai vechi se încarcă în continuare.
Se deschid și pistele de sunet: un fișier audio ilizibil/inutilizabil este o **eroare**. **Exit:** 0 valid; 1 invalid.

```
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels

$ osatura check unk.json                      # a project with two unknown keys
warning: unknown field bogus_key (ignored)
warning: unknown field bones[0].lenght (ignored)
ok: 'small' 640×480 @ 24 fps, 8 frames, 6 bones, 1 ik, 14 layers, 14 channels

$ osatura check bad3.json                     # IK chain whose target bone does not exist
bad3.json is not a valid project:
  - ik 'leg': unknown target bone 'nope'
[exit 1]

$ osatura check bad.json                      # {"format":1,"name":"x"}
invalid JSON in bad.json: missing field `width`
[exit 1]
```

**Conexe:** `lint` (calitatea rigului/animației), `ci` (poartă de regresie). **Capcană:** `check` nu randează nimic; un proiect poate trece `check` și totuși
să eșueze mai târziu pe un cadru (pozițiile `NaN` sunt prinse de `render`/`export`, care evaluează mai întâi fiecare cadru).

### `stats`

**Sinopsis:** `osatura stats <project.json>`

Afișează numărătorile ca JSON: `name, size, fps, frames, seconds, bones, ik_chains, constraints, springs, layers, channels, keyed_channels, keys, markers`.
Ieșirea este întotdeauna JSON; nu există opțiunea `--json` (este refuzată ca orice opțiune necunoscută).

```
$ osatura stats demo.json
{ "name": "OSATURA demo", "size": [640.0, 480.0], "fps": 24.0, "frames": 48, "seconds": 2.0, "bones": 6,
  "ik_chains": 1, "constraints": 0, "springs": 0, "layers": 14, "channels": 14, "keyed_channels": 8, "keys": 37, "markers": 0 }
```
(ieșirea reală este formatată pe mai multe linii.) Reține că `layers` numără fiecare nod, inclusiv copiii grupurilor/switch-urilor, în timp ce `check` afișează același număr.

### `diff`

**Sinopsis:** `osatura diff <a.json> <b.json>`

Diferența semantică dintre două proiecte: `+` adăugat, `-` eliminat, `~` modificat. Insensibilă la ordine pentru listele adresate prin nume.
**Exit:** 0 identice (`no differences`); **1** când există diferențe (numărul `N difference(s)` merge pe stderr), deci poate condiționa un script.

```
$ osatura diff demo.json demo.json
no differences
$ osatura diff demo.json d2.json              # fps 24->30, arm_up longer, layer 'sun' deleted
~ fps: 24 → 30
~ bone 'arm_up' changed (length)
- layer 'sun' removed
3 difference(s)            (stderr)
[exit 1]
```
**Conexe:** `vcs diff` (istoric), `compare` (diferență de pixeli), `ci diff` (diferență de imagine).

### `tag`

**Sinopsis:** `osatura tag <project.json>`

Propune nume de roluri standard (`arm.upper.l`, `hand.l`, `face.mouth` ...) pentru oase și straturi pe baza numelor lor, cu un grad de încredere, și listează rolurile standard pentru care nu s-a găsit niciun os. Doar citire; folosește rolurile ca ghid pentru `mocap-match` sau `autorig`.

```
$ osatura tag demo.json
bone  arm_up                   -> arm.upper.l    (45%)
bone  hand                     -> hand.l         (45%)
layer mouth                    -> face.mouth     (90%)
no bone found for: head, neck, torso, hips, face.eye.l, ...
```

### `explain`

**Sinopsis:** `osatura explain <project.json> <bone-or-layer> <frame>`

„De ce e aici?” - listează contribuțiile ordonate (poziția de repaus, fiecare canal cu modul și ponderea lui, constrângerile, IK) care produc transformarea unui os sau strat la un cadru, precum și unghiul rezultat al capului/vârfului/lumii.

```
$ osatura explain demo.json hand 12
- bone 'hand' (length 45), parent 'arm_low'
- ancestry: hand ← arm_low ← arm_up
- rest pose: offset (100.00, 0.00), angle 0.00°
- channel #2 BoneAngle: keys → 20.000 (5 key(s), Hold outside range); mode replace, weight 1.00
- channel #3 BoneAngle: expression `6 * sin(time * 2 * pi * 2)`; mode add, weight 1.00
- result at frame 12: head (174.6, 126.4), tip (203.5, 92.0), world angle -50.0°
```
**Erori:** nume de os/strat necunoscut; cadru în afara animației.

### `pick`

**Sinopsis:** `osatura pick <project.json> --frame N --x X --y Y [--tol PX] [--root] [--bones] [--points a,b] [--locked a,b] [--json]`

Ce se află sub un punct al canvasului, de sus în jos (hit-testing așa cum îl face studioul).

| Opțiune | Semnificație |
|---|---|
| `--frame N`, `--x X`, `--y Y` | Obligatorii: cadrul și coordonatele canvasului. |
| `--tol PX` | Toleranța de selecție în pixeli de canvas (implicit 3). |
| `--root` | Raportează strămoșul de nivel superior (`mouth`) în loc de stratul cel mai interior (`mouth/closed`). |
| `--bones` | Ia în considerare și oasele. |
| `--points a,b` | Straturile ale căror puncte de control (ancore/mânere/vârfuri de mesh) pot fi selectate. |
| `--locked a,b` | Straturi de ignorat (tratate ca blocate). |
| `--json` | `{"hit":{...},"hits":[...]}`. |

```
$ osatura pick demo.json --frame 0 --x 209 --y 104 --tol 4
hit: layer ink (distance 0.00)
  under: layer mouth/closed (distance 0.00)
  under: layer arm (distance 0.00)
  under: layer sky (distance 0.00)
$ osatura pick demo.json --frame 0 --x 209 --y 104 --tol 4 --root --locked mouth
hit: layer ink (distance 0.00)
  under: layer arm (distance 0.00)
  under: layer sky (distance 0.00)
$ osatura pick demo.json --frame 0 --x 300 --y 240 --json
{"hit":{"distance":0.0,"target":{"kind":"layer","path":"sky"}},"hits":[{"distance":0.0,"target":{"kind":"layer","path":"sky"}}]}
```
**Capcană:** stratul de fundal (`sky`) este o țintă obișnuită de lovire, deci aproape întotdeauna există cel puțin o lovitură.

### `select-box`

**Sinopsis:** `osatura select-box <project.json> --frame N (--rect x0,y0,x1,y1 | --lasso x,y;x,y;x,y) [--mode bbox|contain|intersect] [--root] [--bones] [--json]`

Selecție cu marquee/lasso. `--mode` hotărăște când un strat se consideră în interior: `bbox` (caseta lui de încadrare atinge regiunea), `contain` (complet în interior), `intersect` (implicit: geometria lui desenată intersectează regiunea). `--root` raportează strămoșii de nivel superior, `--bones` include oasele.
Pune între ghilimele o valoare `--lasso` într-un shell (`;` separă punctele).

```
$ osatura select-box demo.json --frame 0 --rect 200,95,220,115 --root
layer sky
layer arm
layer mouth
layer ink
$ osatura select-box demo.json --frame 0 --rect 0,0,640,480 --mode contain
layer sun
layer shadow
...
$ osatura select-box demo.json --frame 0 --lasso "100,100;300,100;300,300" --bones --json
{"targets":[{"kind":"bone","name":"arm_low"},{"kind":"bone","name":"hand"},{"kind":"layer","path":"sky"}, ...]}
```

### `timeline`

**Sinopsis:** `osatura timeline <project.json> [--json]`

Vizualizarea dope sheet ca text: piste per canal grupate pe os/strat, cu cadrele-cheie (`keys @ ...`), canale `expr` / `driven`, blocări și culori. Numerele `#N` sunt indecșii de canal folosiți de `keyedit --channels` și de selecțiile `#N@F0..F1`.

```
$ osatura timeline demo.json
timeline: 48 frames at 24 fps, 14 channels
Bones
  arm_up
    #0 bone_angle (keys @ 0 12 24 36 48)
    arm_low
      #1 bone_angle (keys @ 0 12 24 36 48)
      hand
        #2 bone_angle (keys @ 0 12 24 36 48)
        #3 bone_angle (expr)
  foot_ctrl
    #4 bone_offset_x (keys @ 0 12 24 36 48)
...
```
`--json` dă grupurile cu id-uri, părinți, copii, piste, `frames`, `locked`.

### `timecode`

**Sinopsis:** `osatura timecode <frame|HH:MM:SS;FF> [--fps 24] [--drop] [--nodrop]`

Conversie de timecode SMPTE. Un număr simplu este un cadru și afișează un timecode; un argument care conține `:` sau `;` este analizat și afișează numărul cadrului.
29.97, 59.94 și 119.88 fps folosesc implicit timecode drop-frame (`;` înaintea cadrelor); `--nodrop` numără în schimb 30 (sau 60/120); `--drop` forțează drop-frame (eroare pentru alte rate).
`--drop` și `--nodrop` se exclud reciproc.

```
$ osatura timecode 100
00:00:04:04
$ osatura timecode 100 --fps 29.97
00:00:03;10
$ osatura timecode 1800 --fps 29.97 --drop
00:01:00;02
$ osatura timecode 1800 --fps 29.97 --nodrop
00:01:00:00
$ osatura timecode 00:00:04:04
100
$ osatura timecode 00:00:04;04 --fps 24
drop-frame timecode is only defined for 29.97, 59.94 and 119.88 fps         [exit 1]
```
Nu citește un proiect; `--fps` are implicit 24 (nu fps-ul unui proiect).

### `snapshot`

**Sinopsis:**
```
osatura snapshot save <project.json> <dir> [--keep 20]
osatura snapshot list <project.json> <dir>
osatura snapshot restore <project.json> <dir> <out.json> [--number N]
osatura snapshot recover <project.json> <dir> <out.json>
```
`--keep` aparține doar lui `save` și `--number` doar lui `restore`; orice altă combinație este refuzată (`unknown option --keep for 'snapshot list' (applies to: snapshot save)`), la fel ca un argument de fișier lipsă sau în plus.

Un istoric numerotat al unui proiect într-un director (independent de `vcs`). `save` stochează proiectul curent ca `rev-NNNNNN-<hash>.json` și păstrează doar cele mai noi `--keep` revizii (implicit 20, fișierele mai vechi se șterg); salvarea unui proiect identic cu ultima revizie afișează `unchanged since the last revision` și nu stochează nimic. `list` afișează reviziile. `restore` scrie revizia `--number N` în `<out.json>`; `recover` scrie cea mai nouă revizie.

```
$ osatura snapshot save s.json snaps --keep 3
saved revision 1 (4922f48a86f24554)
$ osatura snapshot list s.json snaps
     1  4922f48a86f24554  snaps/rev-000001-4922f48a86f24554.json
$ osatura snapshot restore s.json snaps restored.json --number 1
wrote restored.json
$ osatura snapshot recover s.json snaps recovered.json
wrote recovered.json
```
`list` pe un director inexistent nu afișează nimic și iese cu 0. **Conexe:** `vcs` pentru ramuri și merge-uri.

---

## 4. Randare și export

### `render`

**Sinopsis:** `osatura render <project.json> <frame> <out.svg|out.png> [--bones] [--scale N] [--clamp] [--proxy N]`

`--proxy N` randează un proxy de derulare (N = 1..4): un `.png` este un bitmap la rezoluția 1/2^N, un `.svg` este format din vectori simplificați; ambele renunță la contururile fine, gradiente și blend-uri.

Randează un cadru. Un `<frame>` egal sau mai mare decât lungimea proiectului este o eroare (`frame 99 is outside the animation ... valid frames are 0..N-1 ... pass --clamp`); `--clamp` randează în schimb ultimul cadru și afișează un avertisment pe stderr. (Doar `render` are această verificare; `dof`, `render-hdr` și `fx` nu.) Extensia alege formatul (`.svg` vector, `.png` raster RGBA pe 8 biți); orice altă extensie este refuzată **înainte** de a se scrie ceva.

| Opțiune | Implicit | Semnificație |
|---|---|---|
| `--bones` | oprit | Desenează suprapunerea scheletului (capetele ca puncte, oasele ca linii). |
| `--scale N` | 1 | Scara ieșirii, un număr în (0, 16]. Pentru `.png` înmulțește dimensiunea în pixeli; pentru `.svg` documentul își păstrează viewBox-ul. |
| `--proxy N` | oprit | Proxy de derulare, N = 1..4: `.png` = bitmap la rezoluția 1/2^N, `.svg` = vectori simplificați; ambele renunță la contururile fine, gradiente și moduri de blend. Gândit pentru derulare interactivă, nu pentru livrabile. |

```
$ osatura render demo.json 12 f12.png --bones --scale 2
wrote f12.png
$ osatura render demo.json 0 /tmp/x.xyz
render writes .png or .svg; '/tmp/x.xyz' has another extension (nothing was written)       [exit 1]
$ osatura render demo.json 0 o.svg --scale 0
--scale needs a number in (0, 16]                                                         [exit 1]
$ osatura render demo.json abc o.svg
bad frame number 'abc'                                                                    [exit 1]
$ osatura render demo.json 2 px.png --proxy 2        # 640x480 project -> 160x120 PNG
wrote px.png
```
**Erori:** proiect lipsă/invalid, poziție non-finită la acel cadru, ieșirea este chiar proiectul de intrare. **Conexe:** `frames`, `render-hdr`, `onion`, `compare`, `fx`.
**Limite:** rasterizatorul PNG este cel propriu al motorului (8 biți, sRGB; amestecarea nu este gestionată color decât dacă proiectul optează prin `color-space ... linear`). Efectele de strat, măștile și modurile de blend sunt desenate de ambele.

### `frames`

**Sinopsis:** `osatura frames <project.json> <out_dir> [--png] [--bones] [--scale N]`

Randează **fiecare** cadru într-un director (creat dacă lipsește) ca `frame_0000.svg` ... (sau `.png` cu `--png`). Mai întâi se verifică dacă toate cadrele au poziții finite.

```
$ osatura frames demo.json fr --png
wrote 48 frames to fr
$ ls fr | head -2
frame_0000.png
frame_0001.png
```
**Capcană:** fișierele existente cu aceleași nume sunt suprascrise; fișierele fără legătură din director rămân neatinse. Folosește ieșirea ca intrare pentru `ffmpeg` sau `import-seq`.

### `export`

**Sinopsis:**
```
osatura export <project.json> <out.gif> [--scale N] [--report r.json]
osatura export <project.json> <out.html|out.htm>
osatura export <project.json> <out.json|out.lottie> [--report r.json]
osatura export <project.json> <out.mp4> [--scale N]
osatura export <project.json> <out.apng> [--scale N]
osatura export <project.json> <out.webm> [--scale N] [--codec vp9|av1]
osatura export <project.json> <out.png> [--scale N] [--columns N]
osatura export <project.json> <out.glb> [--report r.json]
```
(o formă pentru fiecare format; extensia ieșirii, insensibilă la majuscule, alege forma. O opțiune pe care forma nu o listează este refuzată: `unknown option --scale for 'export .json' (applies to: export .gif, export .mp4, ...)`. Cu o extensie necunoscută sau lipsă se raportează în schimb eroarea de format.)

Extensia ieșirii selectează formatul:

| Extensie | Rezultat | Necesită ffmpeg | Opțiuni aplicabile |
|---|---|---|---|
| `.gif` | GIF în buclă la rata de cadre a proiectului (vezi [EXPORT_FORMATS.md](EXPORT_FORMATS.md)) | nu | `--scale`, `--report` |
| `.html`, `.htm` | un player HTML autonom (cadre SVG inline, play/pauză, scrubber, audio încorporat opțional) | doar pentru audio | - |
| `.json` | Lottie (Bodymovin) | nu | `--report` |
| `.lottie` | container dotLottie (zip: `manifest.json` + `a/animation.json`) | nu | `--report` |
| `.mp4` | H.264 yuv420p (+ AAC când proiectul are sunete) | da (`libx264`, `aac`) | `--scale` |
| `.apng` | PNG animat | nu | `--scale` |
| `.webm` | VP9 (implicit) sau AV1 cu alpha (+ sunet Opus) | da (`libvpx-vp9` / `libaom-av1`, `libopus`) | `--scale`, `--codec vp9\|av1` |
| `.png` | sprite sheet **plus** metadatele `<out>.atlas.json` (niciodată `<out>.json`) | nu | `--scale`, `--columns N` |
| `.glb` | glTF 2.0 binar (oase, animație, umpleri/contururi cu skin, gradiente, măști, imagini, morph target-uri) | nu | `--report` |

Opțiuni:

| Opțiune | Semnificație |
|---|---|
| `--scale N` | Scara rasterului (0, 16] pentru `.gif .mp4 .apng .webm .png`. La celelalte formate este refuzată (`unknown option --scale for 'export .json'`); înainte de wave 7 era doar un avertisment. |
| `--report FILE` | Scrie ca JSON lista lucrurilor pe care formatul nu le-a putut transporta (`.gif .json .lottie .glb`): `{"format","count","warnings":[{"code","layer","message"}]}`. Aceleași avertismente se afișează întotdeauna pe stderr. Fișierul nu poate fi proiectul de intrare. |
| `--codec vp9\|av1` | Doar `.webm` (implicit `vp9`); orice altă valoare: `unknown --codec 'foo' (vp9 or av1)`. |
| `--columns N` | Doar sprite sheet `.png`: plăci pe rând (implicit: o grilă aproape pătrată, ceil(sqrt(frames)): 3 coloane pentru 8 cadre). |

Exemple (toate verificate pe o copie de 8 cadre a demo-ului, `--scale 0.5`):

```
$ osatura export small.json out.gif --scale 0.5
wrote out.gif
$ osatura export small.json out.json --report rep.json
wrote out.json
warning: [variable-width-stroke] ink: variable-width stroke profiles are not supported by Lottie; the constant base width is used
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
$ cat rep.json
{ "count": 2, "format": "lottie", "warnings": [ {"code": "variable-width-stroke", "layer": "ink", ...}, {"code": "gradient-anisotropic", ...} ] }
$ osatura export small.json out3.png --columns 4 --scale 0.25
wrote out3.json
wrote out3.png                          # 4 columns x 2 rows of 160x120 tiles; out3.json lists {index,x,y,w,h} per frame
$ osatura export small.json out.glb
wrote out.glb
warning: [glb-limit] blend mode Screen of 'sun' is not exported (glTF materials only alpha-blend); it is drawn as normal
warning: [glb-limit] animated opacity of 'target' is not exported (frozen at frame 0)
$ osatura export small.json out.mp4 --scale 0.5       # 320x240 h264 yuv420p, 8 frames
wrote out.mp4
$ osatura export small.json out.webm --codec av1      # needs a libaom-av1 build of ffmpeg
```

Erori și refuzuri:

```
$ osatura export small.json z.xyz
unknown export format '.xyz' (use .gif .html .htm .json .lottie .mp4 .apng .webm .png .glb)
$ osatura export small.json small.json
refusing to overwrite the input project small.json with output of the same name
$ osatura export ora.json ora.glb                       # a project of image layers and no bones
the project has no bones to export
$ PATH=/nonexistent osatura export small.json y.mp4
cannot run ffmpeg (...); install it or export a GIF/HTML instead                          [exit 1, nothing written]
```
Alte refuzuri: `ffmpeg failed` (encoder lipsă), `export would write about N million values ...` (protecția de dimensiune Lottie/HTML/GLB: limita este 40 de milioane de valori = cadre x (ancore + oase + straturi); scurtează animația sau simplifică traseele), poziții non-finite pe orice cadru și `image too large for GIF`.

**Ieșiri / efecte secundare:** `.png` scrie și metadatele sprite sheet-ului în `<out-name>.atlas.json` (de ex. `out.png` -> `out.atlas.json`), deci nu mai poate înlocui un fișier Lottie `out.json`; fișierul auxiliar este supus acelorași protecții față de intrare/legături simbolice. `.mp4`/`.webm` randează cadre PNG într-un director temporar privat, care se șterge după aceea.
**Sunet:** `.mp4` și `.webm` mixează pistele de sunet ale proiectului (start, gain, trim, fade-uri; tăiate la lungimea proiectului); `.html` încorporează un mp3 mixat (avertisment peste 3 MiB, eliminat peste 16 MiB, cu un avertisment); `.gif` și `.apng` afișează `note: GIF cannot carry audio; ...` și rămân fără sunet; Lottie avertizează `[audio]`.
**Transparență:** un proiect cu `"background": null` se randează cu pixeli transparenți oriunde nu se desenează nimic. `.apng`, sprite sheet-ul `.png` și `.webm` (`yuva420p`, tag de container `alpha_mode=1`) păstrează canalul alpha; `.gif` păstrează transparența pe 1 bit (verificat: indicatorul culorii transparente este setat); `.mp4` nu are alpha. Un proiect cu culoare de fundal (demo-ul are `#f6f1e7`) este opac.
**Conexe:** `export-fit` (încadrare într-un buget de octeți), `size-report`, `lod`, `dof-export`, `export-oca`, `render-hdr`. Detalii complete despre formate: [EXPORT_FORMATS.md](EXPORT_FORMATS.md).

### `export-oca`

**Sinopsis:** `osatura export-oca <project.json> <out_dir> [--layers]`

Scrie un folder **Open Cel Animation**: `<out_dir>/<project name>.oca` (JSON, `ocaVersion` 1.1) plus cel-uri PNG `layerN/fNNNN.png`. Fără `--layers` există un singur strat `render` care conține cadrele compuse; cu `--layers` fiecare strat de **nivel superior** este randat singur pe transparență în propriul strat OCA (11 straturi pentru demo). `--scale N` înmulțește dimensiunea cel-urilor.

```
$ osatura export-oca small.json oca2 --layers
wrote oca2/small.oca
$ ls oca2
layer0 layer1 ... layer10 small.oca
```
**Limite:** cel-urile sunt rastere (nu supraviețuiesc vectori, oase sau canale); opacitatea/blend-ul stratului nu se scriu (fiecare strat este `paintlayer`, opacitate 1); sunetele nu se exportă; folderul se creează dacă este nevoie, iar fișierele din el se suprascriu.

### `render-hdr`

**Sinopsis:** `osatura render-hdr <project.json> <frame> <out.exr|out.png> [linear] [--precision half|float] [--float32] [--8bit] [--scale N] [--clamp]`

Randare float (32 de biți) (un `<frame>` în afara proiectului este o eroare, `--clamp` ia cel mai apropiat cadru valid, ca la `render`): `.exr` scrie un **OpenEXR** dintr-o singură parte, pe scanline, necomprimat (canale HALF implicit, FLOAT cu `--precision float` sau `--float32`), în lumină liniară și premultiplicat; `.png` scrie un **PNG RGBA pe 16 biți**, codat sRGB, dacă nu se dă cuvântul literal `linear` ca al cincilea argument. Fundalul proiectului **nu** este desenat (imaginea este transparentă oriunde nu se desenează nimic). `--8bit` folosește rasterizatorul vechi pe 8 biți ca sursă (nu poate fi combinat cu ieșire float32). `--scale N` se aplică.

```
$ osatura render-hdr small.json 3 f.exr
wrote f.exr (640x480)                       # 465,611 bytes: HALF
$ osatura render-hdr small.json 3 f2.exr linear --precision float
wrote f2.exr (640x480)                      # 923,211 bytes: FLOAT
$ osatura render-hdr small.json 3 f4.png
wrote f4.png (640x480)                      # PNG 16-bit/colour RGBA
$ osatura render-hdr small.json 3 f6.exr --precision bogus
unknown precision 'bogus' (use half or float)
$ osatura render-hdr small.json 3 f7.jpg
output must end in .exr or .png
```
`ffprobe` citește EXR-ul ca `exr 640x480 gbrapf32le`. Pentru `.exr`, cuvântul `linear` nu are efect (EXR este întotdeauna liniar). **Limite:** fără compresie, un cadru pe fișier, fără straturi/AOV-uri; grafica sursă are culoare sRGB obișnuită, deci fișierul conține valori din gama de afișare la precizie mai mare, nu lumină HDR raportată la scenă. Fără gestionare de culoare dincolo de conversia sRGB<->liniar.

### `render-gpu`

**Sinopsis:** `osatura render-gpu <project.json> <out.png> [--frame N] [--adapter software|auto] [--scale S] [--bench N]`

Randare GPU (wgpu) a unui cadru. Disponibilă doar în build-urile făcute cu `--features gpu`; build-ul implicit răspunde:

```
$ osatura render-gpu small.json g.png
render-gpu needs a build with the gpu feature: cargo build -p osatura-cli --features gpu       [exit 1]
```
Într-un build GPU, scenele neacceptate (imagini, oase) sunt raportate ca erori, niciodată desenate greșit. `--bench N` este o opțiune de benchmark. Această comandă nu a putut fi rulată în build-ul de test (fără feature-ul GPU), deci flagurile ei sunt luate doar din textul de utilizare.

### `compare`

**Sinopsis:** `osatura compare <a.json> <b.json> <frame> <out.png> [--gain 4 --scale N]`

Scrie un singur PNG: **înainte | după | diferență**, unul lângă altul (al treilea panou este o hartă de căldură, negru = identic, roșu -> galben = schimbare mai mare, înmulțită cu `--gain`, implicit 4). Ambele proiecte trebuie să se randeze la aceeași dimensiune (`frame sizes differ (...); compare projects of the same size`).

```
$ osatura compare small.json d2.json 3 cmp.png --gain 4 --scale 0.25
wrote cmp.png
```
**Conexe:** `diff` (structură), `ci diff` (două PNG-uri, metrici perceptuale), `golden`.

### `proxy-bench`

**Sinopsis:** `osatura proxy-bench <project.json> [--frames N]`

Milisecunde pe cadru, un singur fir, la calitate maximă și la fiecare nivel de proxy de derulare (vezi `render --proxy`).

```
$ osatura proxy-bench small.json --frames 2
2 frames of small (640x480), ms per frame, single thread:
  evaluate only           0.08
  svg (no raster)         0.48   vector proxy: level 0 0.32, level 2 0.24, level 4 0.24
  raster level 0       1360.87   (full quality)
  raster level 1        319.14   (4.3x faster than level 0)
  ...
  raster level 4          6.84   (199.0x faster than level 0)
```
(valori dintr-un build debug). 

---

## 5. Rigging, skin-uri, IK și ajutoare de layout

Toate comenzile din această secțiune urmează tiparul `osatura <cmd> <project.json> <out.json> ...`: citesc un proiect, îl modifică, **îl revalidează**
și scriu `<out.json>` (care poate fi egal cu intrarea). La eroare nu se scrie nimic.

### `autorig`

**Sinopsis:** `osatura autorig <project.json> <out.json> [seed]`

Adaugă automatizări standard deduse din **numele** oaselor/straturilor: lanțuri IK (mix 0, astfel încât FK continuă să funcționeze), o buclă de respirație (necesită un os de torace) și clipit (necesită straturi pentru ochi). Afișează câte o linie pentru fiecare decizie: `+` adăugat, `-` omis, cu motivul. Rulează și `eye-clip`.

```
$ osatura autorig small.json auto.json 7
+ IK arm_l -> arm_l_ctrl (mix 0, bend +1)
- arm_r: upper/lower bone not found
- leg_l: already has an IK chain
- breathing: no torso bone
- blink: no eye layers
wrote auto.json
```
`seed` (argument pozițional opțional) inițializează temporizarea aleatoare a clipitului. **Limite:** se bazează exclusiv pe nume (`tag` arată cum sunt citite numele); nu creează niciodată oase.

### `slider`

**Sinopsis:** `osatura slider <project.json> <out.json> <name> <poses>` unde `poses` este `at:target.prop=val,target.prop=val;at:...`

Creează un **os de control** `ctrl_<name>` al cărui `offset_x` comandă mai multe proprietăți între poziții stocate (un slider de poziții). Animează `bone_offset_x` al controlerului afișat.

```
$ osatura slider small.json sl.json pose "0:arm_up.bone_angle=0;24:arm_up.bone_angle=30"
controller bone: ctrl_pose (animate its offset_x)
wrote sl.json
```
`target.prop` folosește numele de proprietăți în snake_case (`arm_up.angle` eșuează cu `unknown property 'angle'`). Ultimul punct separă ținta de proprietate, deci căi de strat precum `grp/child.layer_opacity` funcționează.

### `view-swap`

**Sinopsis:** `osatura view-swap <project.json> <out.json> <switch_layer> <bone.prop> <from:child,from:child,...>`

Comandă un strat switch dintr-o proprietate de os cu o funcție în trepte: perechile `from:child` sunt sortate după `from`, iar switch-ul arată copilul ultimei perechi al cărei `from` nu depășește valoarea de comandă (`-40:0,0:1,40:2` = copilul 0 sub 0, copilul 1 de la 0, copilul 2 de la 40). Folosește-l pentru vederi de rotire (față/profil/spate) comandate de un os de yaw. Indecșii copiilor trebuie să existe (`child index 5 is out of range (the layer has 3)`); stratul trebuie să fie un switch.

```
$ osatura view-swap small.json vs.json mouth foot_ctrl.bone_offset_x "-40:0,0:1,40:2"
wrote vs.json
```

### `align` și `distribute`

**Sinopsis:**
`osatura align <project.json> <out.json> <mode> <layer,layer,...|layer> [stage]`  (mode: `left right top bottom center-x center-y`)
`osatura distribute <project.json> <out.json> <x|y> <gaps|centers> <layer,layer,...>`

`align` mută straturile astfel încât casetele de încadrare ale curbelor lor aplatizate să se alinieze (între ele sau, pentru un singur strat urmat de cuvântul `stage`, la canvas). `distribute` distanțează uniform trei sau mai multe straturi pe o axă, egalizând fie **spațiile** dintre casete, fie **centrele**.

```
$ osatura align small.json al.json center-x sun,hills stage
wrote al.json
$ osatura align small.json al2.json top ground
select at least two layers (or one with --stage)                 [exit 1]
$ osatura distribute small.json di.json x gaps sun,ground,hills
wrote di.json
```
Acestea editează pozițiile straturilor (transformare statică), deci canalele cu chei `layer_x`/`layer_y` le suprascriu în continuare. (Mesajul de eroare menționează `--stage`; argumentul real este cuvântul simplu `stage`.)

### `trim`

**Sinopsis:** `osatura trim <project.json> <out.json> <layer> <path_index> <start> <end> [offset]`

Coace un *trim path*: înlocuiește traseul cu numărul `path_index` al unui strat vectorial doar cu partea dintre `start` și `end` (fracții 0..1 din lungimea lui, ambele deplasate cu `offset`; traseele închise se înfășoară). Rezultatul este o **polilinie deschisă cu același contur și fără umplere** - distructiv, deci păstrează originalul. Pentru un efect de desenare progresivă, coace câte o copie tăiată pentru fiecare cadru.

```
$ osatura trim small.json tr.json ink 0 0 0.5 0.25
wrote tr.json
```

### `reparent`

**Sinopsis:** `osatura reparent <project.json> <out.json> <bone> <new_parent|none>`

Schimbă părintele unui os **fără a-l muta**: offsetul/unghiul de repaus se recalculează astfel încât poziția de repaus rămâne neschimbată (verificat: după aceea diferă doar `parent`, `rest_angle`, `rest_offset`). `none` îl face rădăcină. Un părinte care ar crea un ciclu este refuzat.

```
$ osatura reparent small.json rp.json hand none
wrote rp.json
$ osatura diff small.json rp.json
~ bone 'hand' changed (parent, rest_angle, rest_offset)
```

### `parent-switch`

**Sinopsis:** `osatura parent-switch <project.json> <out.json> <bone> <frame:parent|none,...>`

Coace re-părintirea animată (o mână care apucă un obiect): la fiecare cadru de comutare, poziția în lume a osului relativ la noul părinte este capturată și păstrată (urmărire rigidă) sau înghețată în spațiul lumii pentru `none`. Osul devine rădăcină, iar poziția lui în lume se scrie ca simple chei de offset/unghi pentru fiecare cadru; canalele lui vechi rămân în fișier, dar sunt suprascrise de cele coapte. **Limite:** refuzat pentru oasele implicate în IK, constrângeri sau oase cu arc; scara nu este preluată.

```
$ osatura parent-switch demo.json ps.json hand 0:none,24:arm_up
wrote ps.json
```

### `skin`

**Sinopsis (subcomenzi):**
```
osatura skin <project.json> <out.json> add <group> <name> <layer,layer,...>
osatura skin <project.json> <out.json> rig <group> <name> <constraint,...|-> <ik,...|->
osatura skin <project.json> <out.json> bones <group> <name> <bone,...|->
osatura skin <project.json> <out.json> apply <group> <name>
osatura skin <project.json> <out.json> schedule <group> <frame:name,frame:name,...>
```

Un *set de skin-uri* este identificat printr-o etichetă `<group>` (un set de „costume” sau de „expresii”; în exemplu, întâmplător, este egală cu numele unui grup de straturi). Fiecare skin numit arată un set de straturi, date prin **calea stratului** (de ex. `skins/box`); aplicarea unui skin îi arată straturile și ascunde straturile oricărui alt skin din același grup.
`add` definește un skin; `rig` listează constrângerile și lanțurile IK active doar cât timp acel skin este activ; `bones` listează oasele care există doar cât timp este activ (altfel sunt reduse la scara 0); `apply` activează static un skin; `schedule` setează cheile skin-ului activ în timp, ca chei în trepte pe **opacitatea stratului** (deci nu anima altfel opacitatea acelor straturi). `-` înseamnă „niciunul”.

```
$ osatura skin sc.json sk1.json add skins red skins/box
wrote sk1.json
$ osatura skin sk1.json sk2.json add skins green skins/box2
$ osatura skin sk2.json sk3.json apply skins green
$ osatura skin sk3.json sk4.json schedule skins 0:red,4:green
$ osatura skin sk4.json sk5.json bones skins red -
$ osatura skin sk4.json sk5.json rig skins red - leg
$ osatura skin sc.json x.json add skins red box
no layer 'box'                                                  [exit 1]     (use the full path skins/box)
```
**Limite (ROADMAP §9):** un os care există doar într-un skin nu este niciodată eliminat din rigul evaluat (indecșii rămân; este scalat la 0); un strat ponderat pe un os existent doar într-un skin, dar care nu se află în acel skin, dispare odată cu el, iar acest lucru nu este verificat.

### `bone-vis`

**Sinopsis:** `osatura bone-vis <project.json> <out.json> <bone> <from> <to>|clear`

Arată osul în **suprapunerile** studioului doar pentru un interval de cadre (controlul aglomerării). Nu modifică randarea/exportul. `clear` elimină intervalul (`range removed`).

### `chan-meta`

**Sinopsis:** `osatura chan-meta <project.json> <out.json> <target> <prop|*> [color=#rrggbb] [shy=on|off] [solo=on|off] [lock=on|off]`

Metadate dope sheet ale unui canal: culoarea de afișare, *shy* (ascuns în liste), *solo* (în studio se previzualizează doar aceste canale) și **lock** (refuză editările: canalele blocate sunt omise de `keys`, refuzate de `keyedit`, `audio-react` și `live-map`, și nu sunt niciodată atinse de `clip bake`). Evaluarea și exportul le ignoră pe toate patru. `*` selectează toate proprietățile țintei.

```
$ osatura chan-meta small.json cm.json arm_up bone_angle 'color=#ff0000' shy=on lock=on
wrote cm.json
```

### `reference`

**Sinopsis:** `osatura reference <project.json> <out.json> <layer> <on|off>`

Marchează un strat ca **referință de trasare**: se afișează în studio, nu se exportă niciodată.

### `parallax`

**Sinopsis:** `osatura parallax <project.json> <out.json> <far_layer,...,near_layer> <far_value> <near_value>`

Atribuie adâncime multiplan (valori `parallax` distribuite între `far_value` și `near_value`) straturilor de nivel superior listate, în ordinea dată (primul = cel mai îndepărtat). Odată cu mișcarea camerei, straturile se deplasează cu cantități diferite; `dof` estompează după `|parallax - focus|`.

```
$ osatura parallax small.json pa.json sky,hills,ground 0.2 1.0
wrote pa.json
```

### `eye-clip`

**Sinopsis:** `osatura eye-clip <project.json> <out.json>`

Mascează fiecare strat de iris/pupilă cu stratul cu forma ochiului de pe aceeași parte (găsit după nume, vezi `tag`) folosind o mască de strat obișnuită, astfel încât pupilele nu părăsesc niciodată ochiul când se mișcă. Rulează și prin `autorig`. Când nu se găsesc nume potrivite, fișierul se scrie oricum, neschimbat.

### `comp`

**Sinopsis:**
```
osatura comp capture|apply <project.json> <out.json> <name>
osatura comp list <project.json>
```

Stări de straturi cu nume („comp-uri”: vizibilitate, opacitate și copilul selectat al straturilor switch, per cale de strat; **nu** transformările). `capture` stochează starea curentă ca `name`, `apply` o restaurează, `list` afișează numele (`list` nu primește fișier de ieșire și nu scrie nimic; un al patrulea cuvânt moștenit, `comp list p.json p.json`, este încă acceptat și ignorat).

```
$ osatura comp capture small.json c1.json pose1
wrote c1.json
$ osatura comp list c1.json c1.json
pose1 (14 layers)
$ osatura comp apply c1.json c2.json pose1
wrote c2.json
```

### `retarget`

**Sinopsis:** `osatura retarget <source.json> <dest.json> <out.json> --pairs srcChain=dstChain[,..]`

Retargetează după *intenție*: animația **efectorului** IK (osul țintă) al fiecărui lanț sursă este scalată cu raportul razelor de acțiune ale celor două membre (suma lungimilor oaselor) și reorientată către osul țintă al lanțului destinație. Canalele efectorului fără chei nu pot fi scalate; sunt trecute mai departe redenumite și raportate ca linii `note:`. `--pairs srcChain=dstChain[,...]` este obligatoriu. În exemplu sursa și destinația sunt același proiect, ceea ce este util doar ca test rapid.

```
$ osatura retarget demo.json demo.json rt.json --pairs leg=leg
wrote rt.json (+14 channels)
```
**Conexe:** `mocap-retarget` pentru date BVH; `walk`, `mirror`.

---

## 6. Animație, chei, timeline și generatoare

### `keys`

**Sinopsis:**
```
osatura keys shift <project.json> <out.json> --by N [--from F --to F --target NAME --prop PROP]
osatura keys scale <project.json> <out.json> --factor X [--pivot F --from F --to F --target NAME --prop PROP]
osatura keys ease <project.json> <out.json> --ease linear|step|ease_in|ease_out|ease_in_out|smooth [--from F --to F --target NAME --prop PROP]
osatura keys reverse <project.json> <out.json> [--from F --to F --target NAME --prop PROP]
```
Fiecare operație acceptă doar propriile opțiuni (`keys shift --factor 2` este refuzat); `--by`, `--factor` și `--ease` sunt obligatorii pentru operația lor (un `--by` sau `--factor` omis era înainte o operație nulă tăcută).

Operații în bloc asupra **tuturor canalelor cu chei** (filtrate prin `--target` / `--prop`; un interval `--from/--to` limitează cheile atinse). Canalele blocate sunt omise cu `skipped locked channel ...`.

| Operație | Necesită | Efect |
|---|---|---|
| `shift` | `--by N` | Mută cheile cu N cadre. |
| `scale` | `--pivot F --factor X` | Scalează timpii cheilor în jurul cadrului pivot. |
| `ease` | `--ease E` | Setează easing-ul cheilor (easing-uri cu nume; folosește `keyedit ease` pentru `bezier:`). |
| `reverse` | - | Inversează ordinea cheilor din interval. |

```
$ osatura keys shift small.json k1.json --target arm_up --prop bone_angle --by 2
wrote k1.json (5 key(s) changed)
$ osatura keys scale small.json k2.json --pivot 0 --factor 0.5 --from 0 --to 48
wrote k2.json (37 key(s) changed)
$ osatura keys reverse small.json k4.json
wrote k4.json (37 key(s) changed)
```
Cheile mutate în afara `0..frames-1` rămân în fișier (lint raportează `L007`); această comandă nu extinde lungimea proiectului. **Conexe:** `keyedit` (precis, bazat pe selecție), `loop-solve`.

### `keyedit`

**Sinopsis:** `osatura keyedit <project.json> <out.json|-> <sub-command> [options]`

Editări de dope sheet ca **un singur lot sigur la anulare**. Cu `-` ca ieșire nu se scrie nimic, iar operația planificată se afișează ca JSON (rulare de probă). Sub-comenzi și opțiuni:

```
osatura keyedit <project.json> <out.json|-> move --sel SEL [--dt N] [--dv N] [--collision replace|merge|refuse] [--no-snap] [--free]
osatura keyedit <project.json> <out.json|-> scale --sel SEL --pivot F --factor X | dup --sel SEL --offset N | delete --sel SEL | ease --sel SEL --ease E | value --sel SEL --value V | smooth --sel SEL [--kind monotone|catmull_rom]
osatura keyedit <project.json> <out.json|-> copy --sel SEL --clip clip.json | paste --clip clip.json --at F [--remap from_target:prop=to_target:prop]
osatura keyedit <project.json> <out.json|-> add --target T --prop P --at F [--value V] [--ease E] | ripple --at F --by N --mode insert|delete [--scope 0,1] [--adjust-frames]
osatura keyedit <project.json> <out.json|-> offset --channels 0,1 --dt N | extrapolate --channels 0,1 --extrapolate loop | loop --channels 0,1 [--end F] | handle --sel SEL --side in|out --t F --v V [--mode free|aligned|auto] | lock --target T [--prop P] [--value on|off]
```

* **SEL** sunt elemente `TARGET:PROP[@F0..F1]` unite prin `;`, sau `#CHANNEL[@F0..F1]` (indexul canalului din `timeline`), sau `all`. Pune-l între ghilimele (`;`, `@`, `#`).
* **E** (easing) este `linear`, `step`, `ease_in`, `ease_out`, `ease_in_out`, `smooth`, `bounce`, `bezier:x1,y1,x2,y2` sau un obiect JSON.
* `--collision` hotărăște ce se întâmplă când o cheie mutată ajunge peste o cheie existentă (`replace` implicit, `merge`, `refuse`); `--no-snap` păstrează cadrele fracționare; `--free` ridică restricția de cadre întregi.
* Linia de ieșire: `<op>: N keys changed, wrote <out>`. `copy` cu o ieșire reală afișează `copy: 0 keys changed, project unchanged, clip saved`.
* Operațiile care ar pune o cheie în afara `0..=frames` sunt refuzate cu un mesaj precis (`duplicate: frame 52 is outside the project range 0..=48`), cu excepția cazului în care `ripple --adjust-frames` extinde proiectul.

```
$ osatura keyedit demo.json - move --sel "arm_up:bone_angle@12..36" --dt 2        # dry run: prints the batch JSON
{ "op": { "op": "batch", "ops": [ { "op": "set", "path": "/channels/0/track", ... } ] } }
$ osatura keyedit demo.json k1.json move --sel "arm_up:bone_angle@12..36" --dt 2 --collision merge
move: 3 keys changed, wrote k1.json
$ osatura keyedit demo.json k5.json ease --sel "arm_up:bone_angle" --ease bezier:0.4,0,0.2,1
ease: 5 keys changed, wrote k5.json
$ osatura keyedit demo.json k7.json smooth --sel "arm_up:bone_angle" --kind monotone
smooth: 4 keys changed, wrote k7.json
$ osatura keyedit demo.json k8.json copy --sel "arm_up:bone_angle@0..12" --clip c2.json
copy: 0 keys changed, project unchanged, clip saved
$ osatura keyedit demo.json k8.json paste --clip c2.json --at 30 --remap arm_up:bone_angle=arm_low:bone_angle
paste: 2 keys changed, wrote k8.json
$ osatura keyedit demo.json k9.json add --target arm_up --prop bone_angle --at 6 --value 5 --ease linear
add: 1 keys changed, wrote k9.json
$ osatura keyedit demo.json k10.json ripple --at 12 --by 6 --mode insert --adjust-frames
ripple: 28 keys changed, wrote k10.json            # project now 54 frames
$ osatura keyedit demo.json k1.json move --sel "arm_up:bone_angle@12..36" --dt 12 --collision refuse
a key already exists at frame 48                    [exit 1]
$ osatura keyedit demo.json k1.json move --sel "zzz:bone_angle" --dt 1
nothing matches 'zzz:bone_angle'                    [exit 1]
$ osatura keyedit demo.json k15.json lock --target arm_up --prop bone_angle --value on
lock: 1 keys changed, wrote k15.json
```
**Capcane:** `delete` cu `#0@12..12` elimină exact o cheie; `ripple --mode delete --scope 0,1` limitează deplasarea la canalele 0 și 1; `copy` și `paste` schimbă între ele un fișier clip, nu clipboard-ul sistemului. Canalele blocate refuză `move/scale/...` (folosește mai întâi `lock --value off`).

### `marker`

**Sinopsis:** `osatura marker <project.json> <out.json> <name> <frame> [--end F]`

Adaugă un marker de timeline (un punct cu nume sau, cu `--end`, un interval). Un marker cu același nume este înlocuit. Markerele sunt sortate după cadru.

```
$ osatura marker small.json m1.json hit 4 --end 6
wrote m1.json (marker 'hit' at 4 = 00:00:00:04)
```

### `event`

**Sinopsis:** `osatura event <project.json> <out.json> <name> <frame> [--data JSON]`

Adaugă un eveniment cu nume (un declanșator de runtime pentru playere) la un cadru, cu payload JSON opțional. JSON invalid: `--data is not valid JSON: ...`.

```
$ osatura event small.json ev1.json boom 3 --data '{"power":2}'
wrote ev1.json (1 events)
```

### `idle`

**Sinopsis:**
```
osatura idle blink <project.json> <out.json> --target T --prop P [--interval S --jitter X --closed F --seed N]
osatura idle breath <project.json> <out.json> --target T --prop P [--amp A --period F]
osatura idle sway <project.json> <out.json> --target T --prop P [--amp A --freq HZ --seed N]
```
Fiecare generator acceptă doar propriile opțiuni (`idle breath --seed 2` este refuzat).

Generează un canal idle procedural și îl **adaugă la sfârșit** (canalele existente rămân).

| Generator | Opțiuni (valori implicite) | Țintă tipică |
|---|---|---|
| `blink` | `--interval` secunde medii între clipiri, `--jitter`, `--closed` cadre cu ochii închiși, `--seed` | un `layer_switch` (ochi deschis/închis) sau `layer_scale_y` |
| `breath` | `--amp` (0.03), `--period` cadre (3 s la fps-ul proiectului) | `bone_scale` al unui torace |
| `sway` | `--amp` (2), `--freq` Hz (0.3), `--seed` (1) | `bone_angle` |

```
$ osatura idle blink small.json i1.json --target mouth --prop layer_switch --interval 1 --seed 3
wrote i1.json
$ osatura idle sway small.json i3.json --target arm_up --prop bone_angle --amp 3 --freq 0.5
wrote i3.json
```
`--target` și `--prop` sunt obligatorii (`--target is required`).

### `follow` (două forme)

**Follow-through (opțiuni):** `osatura follow <project.json> <out.json> --target SRC --prop P --chain a,b,c [--delay F --decay X]`
Adaugă copii întârziate și atenuate ale unui canal existent (`--target`/`--prop` numesc sursa) la oasele lanțului: overlapping action / follow-through (`--delay` cadre pe verigă, implicit 3; `--decay` pe verigă, implicit 0.7). O copie întârziată este tăiată la ultimul cadru al proiectului (nu se scrie nicio cheie după cadrul `frames`, deci `lint` L007 nu se declanșează); cadrele dintre ultima cheie păstrată și tăietură sunt eșantionate astfel încât curba întârziată să fie exactă pe fiecare cadru întreg.

```
$ osatura follow small.json f1.json --target arm_up --prop bone_angle --chain arm_low,hand --delay 2 --decay 0.6
wrote f1.json (+2 channels)
```

**Urmărirea unui traseu (poziționale):** `osatura follow <project.json> <out.json> <layer|bone:NAME> <layer_with_path> <start_frame> <end_frame> [no-orient]`
Coace un strat (sau osul rădăcină `bone:NAME`) care se deplasează de-a lungul primului traseu al altui strat între două cadre, rotindu-se după tangenta traseului, cu excepția cazului în care se dă `no-orient`.

```
$ osatura follow small.json fp.json target ink 0 7
wrote fp.json
$ osatura follow small.json fp2.json bone:hand ink 0 7 no-orient
path following needs a root bone (no parent)                      [exit 1]
```

### `take-record`, `take-apply`, `take-list`

**Sinopsis:**
```
osatura take-record <project.json> <library.json> <name> <from> <to> [target_prefix,...]
osatura take-apply  <project.json> <out.json> <library.json> <name> <at> [from=to,...] [speed] [--blend FRAMES]
osatura take-list   <library.json>
```
Un **take** este o felie reutilizabilă de animație stocată într-un fișier de bibliotecă. `take-record` captează cadrele `from..to` ale canalelor a căror țintă începe cu unul dintre prefixe (toate canalele, când se omite); biblioteca se creează dacă lipsește, iar un take cu același nume este înlocuit. `take-apply` lipește take-ul la cadrul `at`, putând **retargeta** (perechile `from=to` redenumesc țintele) și schimba `speed`; argumentele sunt verificate strict: o intrare fără `=`, un nume sursă pentru care take-ul nu are pistă sau o viteză care nu este un număr pozitiv este o eroare, iar o bibliotecă ilizibilă numește fișierul (`cannot read the take library ...`); `--blend N` face cross-fade cu animația existentă pe N cadre la ambele capete.

```
$ osatura take-record demo.json lib.json wave 0 24 arm_
recorded 2 tracks into lib.json
$ osatura take-list lib.json
wave (24 frames, 2 tracks)
$ osatura take-apply demo.json ta.json lib.json wave 10 arm_up=arm_low 1.5 --blend 4
applied 2 tracks; wrote ta.json
```

### `clip`

**Sinopsis:**
```
osatura clip add <project.json> <out.json> <library.json> <take> <clip> [--at F --speed S --loops N --mode replace|additive|multiply --weight W
                   --include a_*,b --exclude c --track-weight arm_*=1,leg_*=0.2 --rename a=b --fade-in F --fade-out F]
osatura clip list <project.json>
osatura clip remove <project.json> <out.json> <clip>
osatura clip set-weight <project.json> <out.json> <clip> [W] [--keys frame:w,frame:w] [--track PATTERN[:prop]]
osatura clip bake <project.json> <out.json>
```
Un **clip stack** pe straturi (animație neliniară): clipurile referă take-uri dintr-o bibliotecă, rulează la `--at` cu `--speed`/`--loops` și se amestecă `replace`, `additive` sau `multiply` cu o pondere (constantă, cu chei în timp sau pe tipar de pistă). `clip add/set-weight/remove` doar editează stiva; **`clip bake` regenerează canalele deținute de stivă** (canalele făcute manual și cele blocate nu sunt niciodată atinse).

```
$ osatura clip add demo.json cl1.json lib.json wave w1 --at 6 --speed 1 --loops 2 --mode additive --weight 0.8 --fade-in 2 --include 'arm_*' --rename arm_up=hand
added clip 'w1' (1 clips); run clip bake to generate channels
$ osatura clip list cl1.json
1. w1 take 'wave' frames 6..30 x1 speed 1 Replace weight 1
0 baked channels
$ osatura clip set-weight cl1.json cl2.json w1 0.5
updated clip 'w1'; run clip bake to update channels
$ osatura clip set-weight cl1.json cl3.json w1 --keys 0:0,12:1
$ osatura clip bake cl2.json cl5.json
baked 1 clips into 4 channels (0 replaced); wrote cl5.json
$ osatura clip remove cl2.json cl6.json w1
removed clip 'w1'; run clip bake to update channels
```
(Exemplul `clip list` arată al doilea `clip add` cu același nume, care l-a înlocuit pe primul.) **Limite (ROADMAP §9):** canalele coapte sunt eșantionate pe cadre întregi; nu există `clip refresh` pentru a reîncărca un take editat din bibliotecă.

### `make-loop`

**Sinopsis:** `osatura make-loop <project.json> <out.json> [targets|all] [props] [--inclusive]`

Face ca canalele `replace` cu chei puse manual să se termine unde au început: fiecare curbă este eșantionată pe cadre, linia dreaptă de la prima la ultima ei valoare se scade (eliminarea derivei), iar prima valoare se adaugă la loc. Aplicată offsetului X al unui os care călătorește, transformă un mers într-un mers pe loc. Perioada buclei este N = `frames` al proiectului, deci redarea repetată a `0..N-1` nu arată un cadru duplicat la îmbinare. `--inclusive` este convenția veche în care ultimul cadru este egal cu primul. Fără argumente se procesează fiecare canal cu chei. Afișează câte o linie `removed drift` pentru fiecare canal modificat.

```
$ osatura make-loop demo.json ml2.json foot_ctrl bone_offset_x
foot_ctrl BoneOffsetX: removed drift 0.79
wrote ml2.json
$ osatura make-loop demo.json ml.json all bone_angle --inclusive
```
**Limite:** detrending liniar clasic, nu o re-sincronizare a mișcării; canalele conduse de expresii nu sunt atinse. **Deplasarea este eliminată intenționat** (un proiect care redă `0..N-1` în mod repetat nu poate călători la nesfârșit, deci bucla este pe loc): un canal a cărui schimbare întreagă a fost deplasare devine constant, iar comanda afișează pentru el o linie `note: ... is now constant`; `lint` raportează apoi acel canal ca L005 (informație), ceea ce este de așteptat. Numește țintele și proprietățile pentru a lăsa în pace deplasarea altor canale sau folosește `loop-solve` pentru mișcări deja ciclice.

### `loop-solve`

**Sinopsis:** `osatura loop-solve <project.json> <out.json> [--window 24] [--inclusive]`

Rezolvă **îmbinarea**: amestecă sfârșitul canalelor cu chei în începutul lor pe o fereastră, astfel încât animația să facă buclă fără un salt brusc. Afișează eroarea de îmbinare înainte și după.

```
$ osatura loop-solve demo.json ls.json --window 24
wrote ls.json: 8 channel(s) adjusted, seam 0.000 -> 0.000
```
`--window` este în cadre (implicit 24). Compară cu contractul `loop` al lui `lint` (`contracts`).

### `walk`

**Sinopsis:** `osatura walk <project.json> <out.json> --thigh-l B --thigh-r B [--shin-l B --shin-r B --arm-l B --arm-r B --hips B --period N --stride DEG --knee DEG --bob PX --swing DEG]`

Generează un ciclu de mers în buclă ca și canale cu chei pe oasele pe care le numești (este necesară cel puțin o opțiune de os: `name at least one bone (--thigh-l, --thigh-r, ...)`). Valori implicite: `--period 24` cadre, `--stride 28`, `--knee 40`, `--bob 4`, `--swing 22`. Canalele folosesc extrapolare în buclă și sunt **adăugate la sfârșit**.

```
$ osatura walk small.json wk.json --thigh-l arm_up --thigh-r arm_low --period 8 --stride 20
wrote wk.json (+2 channels)
```

### `mirror`

**Sinopsis:** `osatura mirror <project.json> <out.json>`

Oglindește toate canalele cu chei față de axa verticală: țintele își schimbă partea (prefixele `L_`/`R_`, sufixele `_L`/`_R` și `_l`/`_r`, cuvintele `left`/`right`), iar rotațiile și translațiile orizontale își schimbă semnul. Canalele oglindite **înlocuiesc** canalele proiectului (nu se adaugă lângă originale). Linia de rezultat este `wrote X (N of M channel target(s) swapped sides)`; când nicio țintă nu are nume stânga/dreapta sau proiectul nu are canale, un avertisment pe stderr spune acest lucru (se inversează doar semnele).

### `foot-lock`, `stretch`, `magnet`

```
osatura foot-lock <project.json> <out.json> <ctrl_bone> <ground_y> [tolerance] [min_frames]
osatura stretch   <project.json> <out.json> <follower_bone> <leader_bone> [min=1] [max=3]
osatura magnet    <project.json> <out.json> <ctrl_bone> <bone:NAME|x,y> <radius> <strength 0..1>
```

* **`foot-lock`** eșantionează poziția în lume a controlerului la fiecare cadru; secvențele de cel puțin `min_frames` cadre în care acesta se află pe sol (axa y a scenei este orientată în jos: piciorul este jos când `y >= ground_y - tolerance`) sunt *contacte*, iar un canal de corecție **aditiv** fixează controlerul acolo unde a început fiecare contact. Animația originală rămâne neatinsă; șterge canalele adăugate pentru a anula.
  ```
  $ osatura foot-lock demo.json fl.json foot_ctrl 385 2 2
  contact frames 23..47 pinned at (479.2, 384.1)
  wrote fl.json
  ```
* **`stretch`** face ca `follower_bone` să țintească capul lui `leader_bone` și să se scaleze (uniform, între `min` și `max`, implicit 1 și 3) astfel încât vârful lui să-l atingă. Este copt ca un canal de unghi aditiv plus un canal de scară multiplicativ; copiii followerului se scalează odată cu el. Liderul nu poate depinde de follower (`the leader depends on the follower (it is one of its descendants)`).
  ```
  $ osatura stretch p.json st.json flw lead 1 2
  corrected 8 frames
  wrote st.json
  ```
* **`magnet`** coace o atracție de proximitate a unui controler spre capul unui os (urmărit la fiecare cadru) sau spre un punct fix: atenuare lină, `strength` complet (0..1) la țintă și zero la `radius`. Copt ca canale de offset aditive.
  ```
  $ osatura magnet p.json mg.json lead 200,100 200 0.8
  pulled on 8 frames
  wrote mg.json
  ```
**Limite (ROADMAP §9):** toate trei sunt rezultate coapte din eșantionare pe cadre: rulează-le din nou după editarea animației sursă. `stretch` folosește o scară uniformă. `foot-lock` și `magnet` așteaptă un os de control rădăcină.

### `shot`, `camera-seq`

```
osatura shot       <project.json> <out.json> <name> <x> <y> <zoom> [rotation]
osatura camera-seq <project.json> <out.json> <frame:shot[:cut],...>
```
`shot` salvează un **shot de cameră** cu nume (centru x, y, zoom, rotație opțională în grade). `camera-seq` pune chei camerei prin shot-urile salvate: la fiecare `frame` camera se mută la shot (cu easing) sau, cu sufixul `:cut`, sare la el. Scrie canalele `camera_x/y/zoom/rotation`.

```
$ osatura shot small.json sh.json wide 320 240 1 15
wrote sh.json (1 shots)
$ osatura shot sh.json sh2.json close 400 200 2
wrote sh2.json (2 shots)
$ osatura camera-seq sh2.json cs.json 0:wide,4:close:cut
wrote cs.json
```

### `spring-fit`

**Sinopsis:** `osatura spring-fit --start V --samples a,b,c,... [--fps 24]`

Potrivește **frecvența (Hz) și raportul de amortizare** unui arc cu o curbă de răspuns desenată (eșantioane pe cadru ale unui pas de la `--start` la ultima valoare). Afișează perechea potrivită și JSON-ul de lipit în `springs` al unui proiect. Nu citește un proiect.

```
$ osatura spring-fit --start 0 --samples 0.5,0.9,1.1,1.0,0.98,1.0 --fps 24
frequency 4.68 Hz, damping 0.61 (rms error 0.028 of the step size)
add to the project: {"bone": "<bone>", "frequency": 4.68, "damping": 0.61}
```

### `script`

**Sinopsis:** `osatura script <project.json> <out.json> <script.rhai>`

Rulează un **script Rhai în sandbox** care editează proiectul prin cereri validate (o cerere respinsă lasă totul neschimbat:
`request 5 (box.LayerOpacity at 0) was rejected, nothing changed: ...`). Funcții încorporate: `fps() frames() bones() layers() set_key(target, prop, frame, value[, ease]) add_bone(name, parent (empty = root), length, x, y, angle) add_rect add_circle add_polygon|add_blob add_line add_gradient_rect add_ik add_aim add_copy_rotation add_pin add_group(name[, [layers]]) add_switch move_layer set_parent delete_layer delete_bone set_layer set_bone add_channel_expr add_noise add_marker add_event bone_info layer_info world print`.
Un script care nu aplică nicio cerere afișează `warning: script: 0 requests were applied`. Build-urile fără feature-ul `scripting` răspund `this build has no scripting support`.

```
$ cat t.rhai
add_rect("box", 10.0, 10.0, 50.0, 50.0, "#ff0000");
add_rect("box2", 80.0, 10.0, 50.0, 50.0, "#00ff00");
add_group("skins", ["box", "box2"]);
set_key("skins/box", "layer_opacity", 0, 1.0);
print(fps());
$ osatura script small.json sc.json t.rhai
24.0
4 request(s) applied; wrote sc.json
```
Straturile dintr-un grup se adresează prin cale (`skins/box`). `print` merge pe stdout.

---

## 7. Fizică, cloth și deformatoare

Toate acestea **coc** o simulare sau un rezultat geometric în canale obișnuite cu chei: proiectul rămâne determinist, iar rezultatul se redă oriunde (inclusiv în Lottie/GLB, acolo unde se poate). Rulează din nou comanda după ce schimbi animația sursă.
Ele primesc `<project.json> <out.json>` și cuvinte `key=value` (fără `--`).

### `physics`

**Sinopsis:**
```
osatura physics <project.json> <out.json> <bone,bone,...> [gravity=900] [wind=x,y] [gust=0.5] [damping=0.985]
    [circle=x,y,r[@bone];...] [capsule=x1,y1,x2,y2,r[@bone];...] [follow=bone] [--allow-empty]
    [drive=bone:gain;...] [vortex=bone|x,y:swirl[:radial[:radius]];...] [preset=hair|tail|rope|antenna|cape]
    [merge=keep|replace] [lateral=0..1]
```
Coace o **simulare de lanț** Verlet (păr, coadă, frânghie, antenă, pelerină) pe oasele listate (rădăcina lanțului prima) ca chei `bone_angle`, câte una pe cadru.

| Cuvânt | Semnificație |
|---|---|
| `gravity=` | Accelerație în jos (implicit 900 unități de scenă/s^2). |
| `wind=x,y`, `gust=`, | Vector de vânt constant; amplitudinea rafalei unui vânt modulat de zgomot. |
| `damping=` | Amortizarea vitezei pe pas (implicit 0.985). |
| `circle=x,y,r[@bone];...`, `capsule=x1,y1,x2,y2,r[@bone];...` | Coliziuni, opțional atașate unui os (îl urmează). Capsulele sunt acceptate (verificat cu `capsule=100,300,300,300,20`; o notă mai veche „capsule colliders NOT done” din ROADMAP §9 este depășită). |
| `follow=bone` | Rădăcina lanțului este purtată odată cu mișcarea acestui os (inerție). |
| `drive=bone:gain;...` | Corpuri care împing lanțul (viteza lor înmulțită cu gain). |
| `vortex=bone\|x,y:swirl[:radial[:radius]];...` | Câmpuri de vârtej în jurul unui os sau punct. |
| `preset=` | Valori de pornire pentru `hair tail rope antenna cape`; cuvintele scrise **după** `preset=` îl suprascriu. |
| `merge=keep\|replace` | `keep` (implicit) lasă neatinse oasele care au deja animație; `replace` le suprascrie. |
| `lateral=0..1` | Cu mai multe lanțuri (`bone,bone/bone,bone`) leagă între ele șuvițele vecine. |
| `--allow-empty` | Acceptă o rulare care nu a copt nimic. |

```
$ osatura physics p.json ph.json tail1,tail2,tail3 gravity=500 preset=tail
baked 3 bones; wrote ph.json
$ osatura physics p.json ph4.json tail1,tail2/tail3 lateral=0.5            # two strands
baked 3 bones; wrote ph4.json
$ osatura physics p.json ph3.json tail1,tail2,tail3 follow=arm_up circle=300,300,40 wind=10,0 gust=0.5 lateral=0.3 drive=lead:0.5
baked 3 bones; wrote ph3.json
$ osatura physics demo.json ph.json arm_low,hand gravity=500 preset=tail   # these bones are keyed already
kept existing animation on 'arm_low' (use merge=replace to overwrite)
kept existing animation on 'hand' (use merge=replace to overwrite)
physics: nothing baked - 2 bone(s) already have animation and merge=keep (the default) leaves it alone; nothing was written. Use merge=replace to overwrite the existing animation, or --allow-empty to accept an unchanged file     [exit 1]
$ osatura physics demo.json ph.json hand preset=bogus
unknown preset 'bogus' (try hair, tail, rope, antenna, cape)                [exit 1]
```
(`p.json` este demo-ul plus trei oase fără chei `tail1..tail3`, create cu un script Rhai - vezi `script`.) **Capcană:** o coacere care nu schimbă nimic este o eroare, cu excepția cazului în care se dă `--allow-empty`.
**Limite (ROADMAP §9):** coliziunile ignoră scara oaselor (doar transformare rigidă); cercurile statice se ciocnesc doar în puncte (cercurile atașate și de-a lungul legăturilor); fără coliziune capsulă-capsulă sau autocoliziune.

### `cloth`

**Sinopsis:**
```
osatura cloth <project.json> <out.json> <layer> [pin=top,left@bone,row:N,col:N,v:N,corners,none] [gravity=900]
    [wind=x,y] [gust=0.5] [gust-hz=0.5] [gust-len=px] [stiffness=1] [shear=0.05] [bend=0.2] [max-stretch=1.1]
    [damping=0.99] [cols=N rows=N] [rect=x,y,w,h] [name=cloth] [substeps=4] [iterations=10] [preroll=frames]
    [circle=x,y,r[@bone];...] [capsule=x1,y1,x2,y2,r[@bone];...] [from=F] [to=F]
```
Coace o **suprafață de cloth** deterministă pe mesh-ul unui strat de *imagine*, sau pe un strat *vectorial* când se dă `rect=x,y,w,h` (stratul vectorial primește un mesh de tip grilă peste acel dreptunghi), ca canale de deformare per vârf. `pin=` alege vârfurile fixate (marginile `top`, `left`..., `corners`, `row:N`, `col:N`, `v:N`, `@bone` atașează pinii la un os, `none`). `from`/`to` limitează intervalul de cadre copt, iar `preroll` rulează simularea înainte de cadrul `from`, astfel încât cloth-ul să pornească din repaus.

```
$ osatura cloth small.json cloth.json sun rect=100,60,120,120 cols=6 rows=6 pin=top gravity=900 wind=40,0
baked cloth on 'sun' (84 channels); wrote cloth.json
$ osatura cloth small.json cloth.json sun
a vector layer needs rect=x,y,w,h                    [exit 1]
$ osatura cloth small.json cloth.json nosuch
no layer at 'nosuch'                                 [exit 1]
```
Cele `84 channels` sunt chei `lattice_dx` / `lattice_dy` (țintă `sun::cloth::<col>,<row>`) ale celor 42 de puncte nefixate din grila 7x7 (rândul de sus este fixat). Sunt canale reale în fișier, deci o coacere de cloth mărește proiectul (98 de canale în loc de 14 aici) - folosește `size-report` pentru a vedea costul. **Limite (ROADMAP §9):** cloth-ul este plan (fără cute în afara planului, deci forfecarea este implicit conformă); refuză mesh-urile libere (doar mesh-uri grilă); fără autocoliziune; copt pe cadre întregi.

### `cage`

**Sinopsis:** `osatura cage <project.json> <out.json> <layer> [rect=x,y,w,h|auto] [hull] [verts=N] [pad=P] [name=cage]`

Adaugă în jurul unui strat un **deformator cage cu coordonate de valoare medie**: conținutul din interiorul cage-ului urmează vârfurile cage-ului, conținutul din exterior rămâne neschimbat. `auto` (implicit când lipsește `rect`) folosește limitele cu margine, `verts=N` este numărul de vârfuri pe latură, `hull` folosește anvelopa convexă, `pad=` adaugă margine casetei, `name=` numește deformatorul (implicit `cage`).

```
$ osatura cage small.json cg.json sun auto verts=3
added cage 'cage' (12 vertices) to 'sun'; wrote cg.json
```
**Limite (ROADMAP §9):** fără potrivire automată a cage-ului dincolo de o casetă cu margine sau anvelopa convexă (fără trasare concavă); nu există atenuare lină în afara cage-ului (apare o treaptă vizibilă acolo unde grafica traversează marginea cage-ului); mesh-urile libere nu au offseturi cu chei pe fiecare vârf (folosește un cage).

### `cage-pose`

**Sinopsis:** `osatura cage-pose <project.json> <out.json> <layer> <frame> [name=cage] [move=i:dx,dy;...] [all=dx,dy] [ease=linear]`

Pune chei offseturilor vârfurilor cage-ului la un cadru (canale `cage_dx` / `cage_dy`). `move=` deplasează vârfuri individuale, `all=` deplasează fiecare vârf. Canalele blocate sunt refuzate.

```
$ osatura cage-pose cg.json cp.json sun 4 all=5,0
keyed cage 'cage' on 'sun' at frame 4; wrote cp.json
$ osatura cage-pose cg.json cp.json sun 4 move=0:5,5
keyed cage 'cage' on 'sun' at frame 4; wrote cp.json
```

### `warp-pivot` și `soft-scale`

**Sinopsis:**
```
osatura warp-pivot <project.json> <out.json> <layer> <frame> [deformer=NAME] [verts=1,4-7] [region=x,y,w,h]
    [pivot=x,y|bone:NAME|centroid|center] [rotate=DEG] [scale=S|sx,sy] [move=dx,dy] [radius=R] [falloff=smooth] [ease=linear]
osatura soft-scale <project.json> <out.json> <layer> <frame> [deformer=NAME] [verts=..|region=..] scale=S|sx,sy
    [radius=R] [falloff=smooth] [pivot=..]
```
`warp-pivot` rotește/scalează/mută o **selecție** de puncte de control (deformatoare cage, lattice, cloth, pin, spine; un mesh de imagine de tip grilă primește o grilă de warp per vârf) în jurul unui pivot și pune chei rezultatului la cadru. Selecția este `verts=` (indecși și intervale) și/sau `region=`; o selecție goală este o eroare (`the selection is empty (give verts= and/or region=)`). `radius=` adaugă o **atenuare lină** (`linear smooth gauss sharp constant`). `soft-scale` este scalare proporțională (soft-selection); fără `verts`/`region` întregul mesh este redimensionat în jurul centrului casetei sale de încadrare prin `scale=F`, `width=W`, `height=H` sau `size=WxH`, cu `aspect=keep|free`.

```
$ osatura warp-pivot cg.json wp.json sun 4 verts=0-3 rotate=15 pivot=centroid radius=30 falloff=gauss
warp-pivot on 'sun' keyed at frame 4; wrote wp.json
$ osatura soft-scale cg.json ss.json sun 4 scale=1.2 pivot=center
soft-scale on 'sun' keyed at frame 4; wrote ss.json
```

### `layer-pivot`

**Sinopsis:** `osatura layer-pivot <project.json> <out.json> <layer> pivot=x,y|bone:NAME`

Mută pivotul (ancora) unui strat **fără a schimba imaginea**, astfel încât canalele de rotație și scară se rotesc în jurul noului punct.

```
$ osatura layer-pivot small.json lp.json sun pivot=100,100
moved the pivot of 'sun'; wrote lp.json
```

### `mesh-lock`

**Sinopsis:** `osatura mesh-lock add <project.json> <out.json> <name> host=LAYER[:path] follower=LAYER[:path] [verts=1,4-7] [strength=1]`
și `osatura mesh-lock list|remove <project.json> [<out.json> <name>]`

Face ca vârfurile follower-ului să urmeze **suprafața mesh-ului gazdă** la fiecare cadru (transfer de deplasare baricentric), de exemplu o cămașă care urmează un corp. Ciclurile sunt refuzate; gazda și follower-ul trebuie să aibă același `parallax` (`mesh lock 'lock1': host and follower must have the same parallax`).

```
$ osatura mesh-lock add two.json ml1.json lock1 host=ra follower=rb verts=0-1 strength=0.5
added mesh lock 'lock1'; wrote ml1.json
$ osatura mesh-lock list ml1.json
lock1: rb:0 follows ra:0 vertices [0, 1] strength 0.5
1 mesh lock(s)
$ osatura mesh-lock remove ml1.json ml2.json lock1
removed mesh lock 'lock1'; wrote ml2.json
```

### `glue`

**Sinopsis:** `osatura glue add <project.json> <out.json> <name> a=LAYER[:path] b=LAYER[:path] pairs=i:j;i:j|auto=RADIUS [mode=rigid|snap|offset] [strength=1]`
și `osatura glue list|remove <project.json> [<out.json> <name>]`

Lipește vârfurile stratului `b` de vârfurile stratului `a` (b urmează a). `pairs=i:j;...` listează explicit perechile de vârfuri, `auto=RADIUS` împerechează fiecare vârf `b` aflat la o distanță de cel mult rază de un vârf `a`. Moduri: `rigid` păstrează offsetul de repaus în cadrul lui `a`, `snap` le face să coincidă, `offset` păstrează un offset constant.

```
$ osatura glue add two.json gl1.json g1 a=ra b=rb auto=5 mode=snap
added glue 'g1' (4 pair(s)); wrote gl1.json
$ osatura glue list gl1.json
g1: Snap strength 1 ra:0#0->rb:0#0 ra:0#1->rb:0#1 ra:0#2->rb:0#2 ra:0#3->rb:0#3
1 glue(s)
$ osatura glue add small.json x.json g1 a=sun b=hills auto=20 mode=snap
no vertex of 'hills' lies within 20 of a vertex of 'sun'          [exit 1]
```

### `mesh-trace`

**Sinopsis:** `osatura mesh-trace <project.json> <out.json> <layer> [density=24] [tolerance=1.5] [smooth=N] [hotspot=x,y,r;...] [edge=D/4] [grading=0.5] [lambda=0.5] [taubin=1] [smooth-boundary=0] [threshold=128] [bones=R] [hot-size=E] [skin=1]`

Trasează **silueta alpha a unui strat de imagine** într-un mesh liber de triunghiuri cu densitate variabilă (mai dens de-a lungul conturului, în jurul hot-spoturilor și al oaselor), îl netezește și îl leagă de oase prin skin. Afișează numărul de vârfuri/triunghiuri și calitatea.

```
$ osatura mesh-trace o.json mt.json top density=12 tolerance=2 bones=0
wrote mt.json: 9589 vertices (1494 on the outline), 17682 triangles, area 1227677 px2
  quality min 0.484 mean 0.890, min angle 18.3 deg, mean edge 3.0 px at the outline / 13.7 px inside
```
(`o.json` este un import OpenRaster de 1280x960; o `density` mai mică produce triunghiuri mai puține și mai mari. Rularea a durat 5 s într-un build debug.) **Limite:** necesită un strat de imagine (o siluetă alpha); un mesh atât de dens mărește mult fișierul de proiect - verifică `size-report`. Mesh-urile libere nu pot fi simulate ca cloth.

### `skin-diffuse`

**Sinopsis:** `osatura skin-diffuse <project.json> <layer> <out.json> [--influences 4]`

Calculează **greutăți de skin prin difuzie de căldură** pentru un strat cu mesh, pornind de la oasele proiectului, păstrând cel mult `--influences` (implicit 4) oase pe vârf. Atenție la ordinea argumentelor: **stratul înaintea ieșirii**. Stratul trebuie să fie un strat de imagine cu mesh.

```
$ osatura skin-diffuse demo.json arm sd.json
layer 'arm' is not an image layer                          [exit 1]
$ osatura skin-diffuse o.json top sd.json
layer 'top' has no mesh                                    [exit 1]
$ osatura skin-diffuse mt.json top sd.json               # mt.json from mesh-trace
wrote sd.json
```

---

## 8. Audio, lip-sync și sunete

Intrările audio ale comenzilor de analiză (`lipsync`, `lipsync-text`, `speech-motion`, `audio-react`, `beats`, `waveform`) trebuie să fie **WAV** (PCM întreg pe 8/16/24/32 de biți sau float pe 32 de biți, orice număr de canale, mixat în mono, până la 256 MiB). MP3/OGG trebuie convertite mai întâi (`ffmpeg -i in.mp3 in.wav`). **Pistele** de sunet stocate într-un proiect (`sound`) pot avea orice format pe care îl poate citi ffmpeg.

### `lipsync`

**Sinopsis:** `osatura lipsync <project.json> <audio.wav> <switch_layer> <out.json> [--map 0,1,2,3,4]`

Comandă un **strat switch** (forme ale gurii) pornind de la intensitatea sonoră și luminozitatea spectrală: o euristică pe amplitudine / treceri prin zero care alege câte una din cinci forme pe cadru (`rest, small, open, round, wide`) și elimină pâlpâirea. **Nu este recunoaștere de foneme.** `--map` dă indexul copilului switch-ului pentru fiecare dintre cele cinci forme; valoarea implicită depinde de numărul de copii ai switch-ului: 1 copil -> toate; 2 -> închis + deschis; 3 -> închis + deschis + rotund/larg; 4 -> închis + mic + deschis + rotund/larg; 5+ -> în ordinea de mai sus. Înlocuiește un canal `layer_switch` cu chei existent al acelui strat. Lungimea proiectului **nu** se schimbă (un WAV de 2 s pe un proiect de 8 cadre pune chei pe 8 cadre).

```
$ osatura lipsync small.json speech.wav mouth lip.json
wrote lip.json (2.00s of audio → 8 frames)
$ osatura lipsync small.json speech.wav mouth lip.json --map 0,1,2,1,0
$ osatura lipsync small.json speech.wav mouth lip.json --map 0,1
--map needs exactly 5 numbers (rest,small,open,round,wide)          [exit 1]
$ osatura lipsync small.json speech.wav ink lip.json
layer 'ink' is not a switch layer                                   [exit 1]
$ osatura lipsync small.json notwav.mp3 mouth x.json
invalid WAV: Failed to read enough bytes.                           [exit 1]
```

### `lipsync-text`

**Sinopsis:** `osatura lipsync-text <project.json> <audio.wav> <transcript.txt> <switch_layer> <out.json> [--map 0,1,2,3,4] [--lang ro|en] [--words-out timing.json|timing.srt]`

Aliniază o **transcriere** cu sunetul (aliniere energie + text: cuvintele și silabele sunt așezate peste regiunile de vorbire găsite în audio) și deduce formele gurii din silabe. `--lang` (implicit `ro`) selectează regulile pentru vocale/silabe; există doar `ro` și `en`. `--words-out` scrie duratele cuvintelor ca JSON (cuvinte cu silabe) sau, pentru un nume `.srt`, ca subtitrări.

```
$ osatura lipsync-text small.json speech.wav transcript.txt mouth lt.json --lang en --words-out words.json
wrote lt.json (3 words, 2 exposure changes; energy + text alignment, not phoneme recognition)
$ head -c 140 words.json
{ "words": [ { "text": "hello", "start": 0.0, "end": 0.22, "syllables": [ {"text":"hel","start":0.0,"end":0.16}, {"text":"lo",...
$ osatura lipsync-text ... --words-out w.srt ; head -4 w.srt
1
00:00:00,000 --> 00:00:01,220
hello there world
```
**Limite (ROADMAP §9):** fără sincronizare pe literă (la nivel de fonem), fără modele de ritm specifice vorbitorului, fără limbi în afara ro/en. Un ton sintetic este aliniat ca vorbire - folosește înregistrări reale.

### `speech-motion`

**Sinopsis:** `osatura speech-motion <project.json> <audio.wav> <transcript.txt> <out.json> [--head bone] [--brow bone_or_layer] [--nod deg] [--fall deg] [--tilt deg] [--brow-px px] [--seed n] [--lang ro|en] [--words-out timing.json|timing.srt]`

Adaugă încuviințări, căderi, înclinări ale capului și ridicări de sprâncene **aditive**, sincronizate cu cuvintele aliniate. `--head` numește osul capului, `--brow` un os sau strat pentru sprâncene; opțiunile de unghi sunt în grade, `--brow-px` în pixeli, `--seed` fixează variația.

```
$ osatura speech-motion small.json speech.wav transcript.txt sm.json --head arm_up --lang en
wrote sm.json (1 additive channel(s) from 3 words)
```

### `mouth`

**Sinopsis:** `osatura mouth <project.json> <cues.txt> <switch_layer> <out.json> [--map 0,1,2,3,4] [--rhubarb]`

Importă **repere** de gură gata făcute în loc să analizeze sunetul: fișiere Papagayo/Moho-switch (`frame PHONEME` pe linie, prima linie opțională `MohoSwitch1`) sau TSV Rhubarb (`seconds<TAB>shape`, recunoscut după punctul zecimal; secundele se convertesc la fps-ul proiectului). Liniile goale și comentariile `#` sunt ignorate. Fonemele se asociază celor cinci clase de forme (`rest/MBP/X/A` rest; `etc/FV/L/B/G/H` small; `AI/D` open; `O/U/WQ/F` round; `E/C` wide); formele necunoscute se consideră `small`. `--rhubarb` tratează `E` din Rhubarb ca *round* (`E` din Papagayo este wide). `--map` implicit este `0,1,2,3,4`.

```
$ printf '0 rest\n2 open\n4 round\n6 wide\n' > cues.txt
$ osatura mouth small.json cues.txt mouth mo.json --rhubarb
wrote mo.json (4 cues)
```

### `audio-react`

**Sinopsis:** `osatura audio-react <project.json> <out.json> <audio.wav> <target> <prop> [gain=1] [offset=0] [floor=0.05] [attack=1] [release=4] [add=on]`

Intensitatea sonoră comandă o proprietate: o cheie pe cadru, urmând nivelul netezit (RMS relativ la cel mai puternic cadru). Ieșire = (nivelul filtrat prin `floor`) x `gain` + `offset`; `attack`/`release` sunt timpii de creștere/scădere în cadre; `add=on` face canalul aditiv în loc să înlocuiască. Un canal blocat este refuzat. Opțiunile sunt cuvinte `key=value`; una necunoscută dă `unknown option 'foo'`.

```
$ osatura audio-react small.json ar.json speech.wav ink layer_rotation gain=10
wrote ar.json
$ osatura audio-react small.json ar.json speech.wav arm_up bone_angle foo=1
unknown option 'foo'                                                [exit 1]
```

### `beats`

**Sinopsis:** `osatura beats <project.json> <out.json> <audio.wav> [sensitivity=1.5] [gap=4] [as=markers|events] [name=beat]`

Detectează salturile de intensitate peste media recentă (`sensitivity` 1.2-3, mai mare = mai puține bătăi) aflate la cel puțin `gap` cadre distanță și le înregistrează ca **markere** de timeline (implicit) sau **evenimente** numite `name`. Afișează cadrele găsite.

```
$ osatura beats small.json bt.json speech.wav as=events name=hit sensitivity=1.2
1 beats at frames [1]; wrote bt.json
$ osatura beats small.json bt.json speech.wav
0 beats at frames []; wrote bt.json
```

### `waveform`

**Sinopsis:** `osatura waveform <audio.wav> <out.json> [buckets=800]`

Vârfuri min/max pentru desenarea unei forme de undă: `{"sample_rate","duration","peaks":[[min,max],...]}`. Nu citește un proiect.

```
$ osatura waveform speech.wav wf.json buckets=50
wrote wf.json (50 buckets)
```

### `sound`

**Sinopsis:**
```
osatura sound list   <project.json>
osatura sound add    <project.json> <out.json> <name> <file.wav|ogg|mp3|...> [start_frame] [gain] [trim/fade options]
osatura sound edit   <project.json> <out.json> <name> [--start F] [--gain G] [trim/fade options]
osatura sound remove <project.json> <out.json> <name>
osatura sound mix    <project.json> <out.wav|out.ogg|out.mp3>
    trim/fade options (seconds, or frames with an f suffix such as 36f): --trim-in T --trim-out T --fade-in T --fade-out T
```
Pistele de sunet sunt **referințe** (cale de fișier relativă la proiect, cadru de start, gain 0..4, fereastră de trim în interiorul fișierului, fade-uri). Nimic nu este încorporat în proiect; fișierul audio trebuie să-l însoțească și trebuie să se afle în directorul proiectului, dacă nu se dă `--allow-external-assets`. `edit` schimbă doar câmpurile date (`0` șterge un trim). `--trim-out 0` înseamnă „până la sfârșit”. `sound mix` randează doar sunetul mixat pentru **lungimea proiectului** (prin ffmpeg: wav = PCM 44,1 kHz pe 16 biți, ogg = Vorbis q4, mp3 = 96 kb/s).
Sunetul mixat este multiplexat și în `.mp4` (AAC), `.webm` (Opus) și încorporat în `.html`; `.gif`/`.apng` nu îl pot transporta.

```
$ osatura sound add small.json s1.json music speech.wav 2 0.8 --fade-in 0.5 --trim-out 36f
wrote s1.json
$ osatura sound list s1.json
music: speech.wav at frame 2 gain 0.8 trim 0s..1.5s fade-in 0.5s
$ osatura sound edit s1.json s2.json music --gain 0.5 --trim-in 0.1 --fade-out 4f
$ osatura sound list s2.json
music: speech.wav at frame 2 gain 0.5 trim 0.1s..1.5s fade-in 0.5s fade-out 0.16666667s
$ osatura sound mix s2.json mix.mp3
wrote mix.mp3 (0.33 s)
$ osatura sound mix s2.json mix.flac
unknown audio format '.flac' (use .wav, .ogg or .mp3)            [exit 1]
$ osatura sound add small.json x.json x nosuch.wav              # a missing file is refused, nothing written
```
`check` eșuează când un fișier de sunet referit lipsește sau nu poate fi citit. Fără ffprobe, pentru fișierele non-WAV se verifică doar existența (nu și faptul că fișierul este audio). **Limite (ROADMAP §9):** fără pan/EQ/ducking per pistă, doar curba liniară de fade, fără repetarea unui clip în buclă, fără re-sincronizare.

---

## 9. Import

### `import-svg`

**Sinopsis:** `osatura import-svg <file.svg> <out.json> [--split]`

Linia de utilizare: `osatura import-svg <file.svg> <out.json> [--split]  SVG subset -> one vector layer, or one layer per shape named by its id (warnings for the rest)`

Creează un proiect nou (24 fps, 24 de cadre; dimensiunea provine din `viewBox` sau din `width`/`height`, altfel 512x512) cu **un singur strat vectorial `svg`** care conține formele unui subset SVG: `path` (toate comenzile), `rect` (colțurile rotunjite sunt ignorate), `circle`, `ellipse`, `line`, `polyline`, `polygon`, `g`/`a` cu `transform`, precum și `fill`/`stroke`/`stroke-width` pline. Tot restul este omis, cu o linie `warning:` **pe stdout** (când elementul este în afara `<defs>`; un `<filter>`, `<clipPath>`, `<mask>` sau `<pattern>` din interiorul `<defs>` și atributele `filter`, `clip-path` și `mask` sunt ignorate fără avertisment): `<text>`, `<filter>`, `<clipPath>`, `<mask>`, `<pattern>`, `<linearGradient>`/`<radialGradient>`, `<image>`, `<use>`, `<foreignObject>`. O umplere cu gradient (`fill="url(#g)"`) revine la gri. Limite: 8 MB de SVG, 200.000 de ancore, un plafon pentru numărul de elemente.

```
$ osatura import-svg s.svg svg.json
warning: unsupported paint 'url(#g)'; using grey
warning: <text> is not supported and was skipped
warning: <filter> is not supported and was skipped
warning: <image> is not supported and was skipped
wrote svg.json (3 paths)
$ osatura check svg.json
ok: 's' 200×100 @ 24 fps, 24 frames, 0 bones, 0 ik, 1 layers, 0 channels
```
Cu `--split`, fiecare formă devine **propriul strat**, numit după `id`-ul formei, în ordinea din fișier. O formă fără id este numită după eticheta și poziția ei (`rect1`, `circle6`), un id repetat primește `_1`, `_2`..., iar caracterele pe care un nume de strat nu le poate conține (`/`, `::`, caractere de control) devin `_`; numele sunt tăiate la 100 de caractere. Aceasta este forma de folosit când desenul urmează să fie echipat cu schelet parte cu parte; vezi [TUTORIAL_SVG.md](TUTORIAL_SVG.md).

```
$ osatura import-svg pip.svg pip.json --split
wrote pip.json (16 paths, 16 layers)
```
Import apoi îmbinare: nu există nicio comandă care să îmbine stratul într-un proiect existent; folosește `script` sau editează JSON-ul.

### `import-ora`

**Sinopsis:** `osatura import-ora <file.ora> <out.json> [--fps N] [--frames N]`

OpenRaster -> proiect: **un strat de imagine pentru fiecare strat raster** (PNG încorporat), ordinea de jos în sus este păstrată, straturile ascunse sunt păstrate, dar invizibile, grupurile sunt aplatizate, iar `opacity`, `visibility`, poziția (`x`,`y`) fiecărui strat și operațiile de compunere uzuale `svg:` (multiply, screen, overlay, darken, lighten, color-dodge/burn, hard/soft-light, difference, exclusion, plus -> add) sunt preluate. Proiectul are `--frames` cadre (implicit 1) la `--fps` (implicit 24). Opacitatea unei stive imbricate se înmulțește în straturile ei, iar modul ei de amestecare este moștenit, dar straturile care se suprapun în interiorul unei stive translucide **nu** sunt aplatizate mai întâi (se văd unele prin altele). Numele de straturi duplicate primesc sufixele `_1`, `_2`.

```
$ osatura import-ora t.ora ora.json
wrote ora.json (2 layers)
$ osatura import-ora t.ora ora2.json --fps 12 --frames 24
$ osatura import-ora layer.png bad.json
not a zip/ORA file: invalid Zip archive: Could not find EOCD       [exit 1]
```
Limite: 2000 de intrări în arhivă, 128 MiB per intrare, straturi de 64 Mpixeli.

### `import-lottie`

**Sinopsis:** `osatura import-lottie <in.json> <out.json> [fx.json]`

Importă ca proiect un fișier Lottie (Bodymovin) - un **subset** deliberat. Afișează `imported N layers; wrote <out>` și câte o linie `warning:` pentru fiecare funcționalitate omisă.

* **Importate:** straturi de forme cu trasee `sh`, `rc`, `el`, umpleri pline și cu gradient (punctele finale/centrul/raza animate ale gradientului devin canale `grad_dx/grad_dy`), contururi, grupuri, transformările stratului (poziție/scalare/rotație/opacitate statice sau cu chei, traiectorii curbe ale poziției coapte pe cadre), moduri de amestecare, un strat de fundal solid, straturi de imagine cu URI-uri de date PNG/JPEG încorporate, straturi null ca părinți (transformările părinților sunt coapte în copii), măști de strat (add/subtract/intersect cu opacitate, inversare, forme cu chei), luma matte (aproximate), precompoziții, intervalele in/out ale straturilor (ca pistă de opacitate în trepte), easing-ul cheilor (hold -> step; mânere bezier -> cubic-bezier; fără mânere -> liniar).
* **Trasee animate:** un traseu ale cărui chei de formă au aceeași topologie (același număr de ancore, deschis/închis) devine un traseu de bază plus câte o **țintă de morfare** pentru fiecare cheie distinctă următoare (`k1`, `k2`, ...), cu canale de pondere `path_morph` care poartă easing-ul. Topologie diferită: se folosește prima formă și se afișează un avertisment.
* **Efecte (`ef`):** Gaussian blur, drop shadow, tint și fill devin **efecte pe strat** (`effects` pe strat, orice tip de strat), cu numerele animate sub formă de canale `fx_*`; tint/fill/tritone/levels statice pe straturi de forme sunt coapte în culori. Culorile efectelor nu pot fi animate (se folosește prima valoare); celelalte efecte, precum și efectele de culoare aflate după un blur/shadow pe straturi care nu sunt de forme, produc avertismente. Se păstrează cel mult primele câteva efecte pe strat.
* **Omise cu avertisment:** text și alte tipuri de straturi, resurse audio, puncte de ancorare animate (prima valoare), traiectorii curbe de mișcare pe alte proprietăți decât poziția (segmente drepte), măști pe o sursă track-matte, lanțuri de părinți prea lungi pentru a fi coapte, mai mult de 2000 de straturi, animații mai lungi de 20.000 de cadre.
* `fx.json` este păstrat pentru compatibilitate: se scrie un array gol (blur și drop shadow sunt acum efecte pe strat în interiorul proiectului).

```
$ osatura import-lottie out2.json li.json                   # a Lottie file made by `osatura export`
imported 11 layers; wrote li.json
$ osatura import-lottie in.json out.json                    # hand-made file: animated triangle with animated blur, plus a null and a text layer
warning: layer 'text' of type 5 skipped
imported 1 layers; wrote out.json
$ osatura check out.json
ok: 't' 200×200 @ 24 fps, 24 frames, 0 bones, 0 ik, 1 layers, 2 channels       # channels: blob::path::k1 path_morph, blob::fx0 fx_blur
$ osatura import-lottie out2.json li2.json fx.json
wrote 0 post effect(s) to fx.json
imported 11 layers; wrote li2.json
$ echo '{}' > bad.json; osatura import-lottie bad.json o.json
missing 'w'                                                        [exit 1]
```
**Capcană:** `import-lottie in.json in.json` este refuzat (`refusing to overwrite the input ...`), la fel și pentru `fx.json`. **Limite:** intrarea `.lottie` (zip) nu este citită (`stream did not contain valid UTF-8`) - dezarhivează mai întâi `a/animation.json`; munca pătratică din morfările de forme este limitată doar de plafoanele de dimensiune.

### `import-bvh`

**Sinopsis:** `osatura import-bvh <project.json> <file.bvh> <out.json> --joint-map Joint=bone,.. [--axis Zrotation --tol 0.5]`

Mapează canalele de rotație ale articulațiilor BVH alese pe oasele rig-ului, sub formă de canale `bone_angle` cu chei, eșantionate la fps-ul și numărul de cadre ale proiectului. `--joint-map` este obligatoriu; `--axis` (implicit `Zrotation`) alege ce canal de rotație BVH este citit (`Zrotation`, `Xrotation`, `Yrotation`); `--tol` (implicit 0,5 grade) simplifică cheile. Pentru potrivire automată, mișcarea rădăcinii, vederi și scalare folosește `mocap-retarget` (secțiunea 10).

```
$ osatura import-bvh demo.json m.bvh ib.json --joint-map LeftUpLeg=thigh,LeftLeg=shin --axis Zrotation --tol 0.5
wrote ib.json (+2 channels from 4 BVH frames)
```

### `import-seq`

**Sinopsis:** `osatura import-seq <project.json> <out.json> <layer_name> <folder> [hold=1] [loop=on|off] [start=0] [x=0] [y=0]`

Importă o **secvență PNG** (toate fișierele PNG din folder, în ordinea numelor) ca strat cadru cu cadru: un strat switch numit `<layer_name>`, cu câte un strat de imagine pentru fiecare cadru și un canal step care expune fiecare imagine timp de `hold` cadre începând de la cadrul `start`, în poziția `(x, y)`; `loop=on` o repetă. Toate imaginile trebuie să aibă aceeași dimensiune (`img.png: size 640x480 differs from the first image (160x120)`).

```
$ osatura import-seq demo.json is.json frames seq hold=2 loop=on start=4 x=10 y=20
imported 3 images; wrote is.json
```

### `import-anim`

**Sinopsis:** `osatura import-anim <project.json> <out.json> <layer_name> <file.gif|file.apng|file.png> [loop=on|off] [start=0] [x=0] [y=0]`

Importă un **GIF animat sau APNG** ca strat cadru cu cadru, folosind întârzierile proprii ale cadrelor din fișier (convertite în cadre de proiect). Un PNG simplu dă un singur cadru.

```
$ osatura import-anim demo.json ia.json gifl dof.gif loop=off start=0
imported 4 frames as layer 'gifl'; wrote ia.json
$ osatura import-anim demo.json ia2.json apngl dof.apng
imported 3 frames as layer 'apngl'; wrote ia2.json
```
Imaginile cadrelor devin PNG-uri încorporate: un GIF lung face proiectul mare.

### `trace`

**Sinopsis:** `osatura trace <image.png> <out.json> [--threshold 128 --tolerance 1.0 --min-area 12 --fill #000000]`

Conversie simplă bitmap -> vector pentru imagini cu **transparență**: urmărește **canalul alfa** al PNG-ului (marching squares la `--threshold`, 0..255, cu interpolare liniară a marginilor), simplifică conturile (Ramer-Douglas-Peucker) până la `--tolerance` pixeli, elimină regiunile mai mici de `--min-area`, detectează golurile prin includere și scrie un proiect nou (numit `traced`, canvas = dimensiunea imaginii) cu conturile ca trasee umplute cu culoarea `--fill`. O imagine complet opacă produce, prin urmare, un singur contur în jurul întregii imagini (ca în exemplu). Afișează `wrote <out> (N contours)`. Pentru imaginile color folosește `autotrace`.

```
$ osatura trace img.png tr.json --threshold 128 --tolerance 1.0 --min-area 12 --fill '#112233'
wrote tr.json (1 contours)
```

### `autotrace`

**Sinopsis:** `osatura autotrace <image.png|jpg> <out.json|out.svg> [--mode fill|line] [--colors N=8] [--min-area A=8] [--tol T=1] [--smooth N=3] [--corner DEG=55] [--denoise N=0] [--quant kmeans|mediancut] [--seed S] [--fit bezier|polygon|vw] [--zorder area|luminance] [--threshold T] [--invert] [--prune N=4] [--into project.json] [--name L] [--fit-stage] [--png preview.png]`

Vectorizare locală a bitmap-urilor - **fără model, fără rețea**: cuantizare a culorilor (`--colors`, k-means sau median cut), extragerea conturelor, potrivire Schneider cu curbe Bezier cubice (`--fit`; `polygon` și Visvalingam-Whyatt `vw` sunt alternative), goluri prin regula par-impar; `--mode line` subțiază contururile la linii centrale (Zhang-Suen, `--prune` elimină cioturile scurte). Rezultatul este un proiect nou, un `.svg` nou sau - cu `--into project.json` - o copie a unui proiect existent cu un grup de straturi nou `--name` (implicit `trace`); `--fit-stage` (împreună cu `--into`) scalează uniform grupul nou pentru a-l încadra și centra pe scena proiectului, iar `--png` scrie o randare de previzualizare. Afișează numărătorile și **eroarea măsurată față de sursă** (eroare medie absolută, RMSE, PSNR, SSIM).

```
$ osatura autotrace img.png at.json --colors 4 --png prev.png
layer group 'trace' added
wrote at.json: 44 regions, 0 strokes, 44 paths, 324 anchors
fidelity vs source: MAE 13.38/255, RMSE 17.59, PSNR 23.2 dB, SSIM 0.960
$ osatura autotrace img.png at.svg --mode line --threshold 100
wrote at.svg: 0 regions, 11 strokes, 11 paths, 47 anchors
fidelity vs source: MAE 52.47/255, RMSE 58.73, PSNR 12.8 dB, SSIM 0.903
$ osatura autotrace img.png at2.json --into d.json --name traced
layer group 'traced' added
wrote at2.json: 51 regions, 0 strokes, 51 paths, 347 anchors
```
**Limite (ROADMAP §9):** un vectorizator clasic - fotografiile devin benzi de culoare; nu există vectorizare învățată sau semantică. `--png` este aici o opțiune cu valoare (numele fișierului de previzualizare), spre deosebire de celelalte flaguri `--png`.

### `autotrace-bench`

**Sinopsis:** `osatura autotrace-bench`

Vectorizează forme sintetice, gradiente și contururi încorporate și afișează un tabel de fidelitate (MAE, RMSE, PSNR, SSIM, numărul de regiuni/ancore, milisecunde). Nu primește argumente; doar citire.

```
$ osatura autotrace-bench
image          mode  colors  tol     MAE    RMSE   PSNR   SSIM regions strokes  anchors      ms
shapes         fill       6    1    2.19   12.10   26.5  0.975      11       0       33    13.6
...
```

---

## 10. Intrare live, MIDI, urmărirea feței și motion capture

### `live-map`

**Sinopsis:** `osatura live-map <project.json> <events.json|song.mid> <mapping.json> <out.json> [--takes lib.json] [--tol 0.01] [--allow-empty]`

Transformă o **performanță live înregistrată** (MIDI, mouse, gamepad, taste, semnale faciale) în canale cu chei, evenimente și take-uri obișnuite (nedistructiv, totul sau nimic, blocările sunt respectate). Intrarea este fie un fișier Standard MIDI `.mid`, fie un JSON de evenimente `{"events":[{"source","id","value","time"}]}` (`value` normalizat 0..1, `time` în secunde; `source` = `midi_cc midi_note midi_pitch_bend mouse_x mouse_y mouse_button gamepad_axis gamepad_button key face`; `id` = pentru MIDI `"<channel 1-16>:<number>"`, pentru butonul mouse-ului `"0"`, pentru gamepad `"<pad>:<index>"`, pentru taste codul DOM, de exemplu `KeyA`).

Fișierul de mapare are `params` (source/id -> `target`+`prop`, cu `in_min in_max out_min out_max deadzone invert curve smooth additive`) și `triggers` (trecerea unui prag -> acțiunea `jump_to_frame`, `play_take`, `set_key`, ...); cheile necunoscute sunt erori. `--takes` este o bibliotecă de take-uri pentru acțiunile `play_take` (trebuie să existe), `--tol` simplifică cheile.

```
$ cat map.json
{"params":[{"source":"mouse_x","id":"0","target":"arm_up","prop":"bone_angle","out_min":-30,"out_max":30}],
 "triggers":[{"source":"key","id":"KeyA","action":{"type":"jump_to_frame","frame":3}}]}
$ osatura live-map small.json ev.json map.json out.json
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 1 events (0 after the last frame skipped); wrote out.json
$ osatura live-map small.json song.mid mapm.json outm.json              # MIDI CC 1 on channel 1 -> arm_up
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 0 events (0 after the last frame skipped); wrote outm.json
$ osatura live-map small.json ev.json map.json out.json --takes lib.json    # lib.json missing
cannot read lib.json: No such file or directory (os error 2)             [exit 1]
$ osatura live-map small.json empty.json map.json out.json
note: map 0 (arm_up.BoneAngle): no matching events, nothing keyed
0 events -> +0 channels, ...; wrote out.json
```
**Capcană:** dacă evenimentele au fost citite, dar **nu s-a produs nimic** (greșeală de scriere într-un id, sursă greșită), comanda eșuează și nu scrie nimic, în afară de cazul în care se folosește `--allow-empty`; cu zero evenimente la intrare, scrie pur și simplu proiectul nemodificat. Declanșatoarele de salt sunt înregistrate ca un eveniment numit `jump` cu `data.to` (axa timpului nu este rescrisă).
**Limite (ROADMAP §9):** această comandă nu oferă intrare live de la webcam/nivel audio în studio (`audio-react` funcționează offline); partea interactivă este oferită de studio.

### `drive`

**Sinopsis:** `osatura drive <project.json> <signals.csv> <maps.json> <out.json> [--tol 0.5]`

Performanță -> chei pornind de la un **CSV**: rândul de antet denumește semnalele, fiecare rând următor este un cadru (indicele rândului = cadrul). `maps.json` este un array de `{input, target, prop, in_min, in_max, out_min, out_max, deadzone, invert, curve, smooth}`; `--tol` (implicit 0,5) simplifică cheile.

```
$ printf 'jaw,speed\n0,0\n0.5,0.2\n1,0.4\n0.5,0.6\n' > sig.csv
$ echo '[{"input":"jaw","target":"arm_low","prop":"bone_angle","out_min":0,"out_max":40,"smooth":0.2}]' > maps.json
$ osatura drive demo.json sig.csv maps.json dr.json --tol 0.5
wrote dr.json (+1 channels)
```

### `face-synth`, `face-train`, `face-eval`, `face-detect`, `face-track`, `face-calibrate`, `face-bake`

Un pipeline condus de față, **în întregime local și antrenat exclusiv pe fețe sintetice** (capete stilizate/de desene animate, 23 de repere, layout `osatura23`). Un model mic antrenat sintetic este încorporat în binar; `--model` îl suprascrie. **Fotografiile, webcam-urile și videoclipurile reale nu sunt validate** (ROADMAP §9; vezi și [PATENT_NOTES.md](PATENT_NOTES.md): puppeteering-ul condus de față nu a fost verificat din punct de vedere al brevetelor).

| Comandă | Sinopsis | Ce face |
|---|---|---|
| `face-synth` | `osatura face-synth <out-dir> [--frames 90] [--size 160] [--seed 7] [--fps 30]` | Animație sintetică de cap: cadre PNG `000000.png...`, `truth.ndjson` (reperele reale) și `params.json` (semnalele reale); `--fps` este rata de cadre la care sunt temporizate semnalele (scrisă în antetul NDJSON). |
| `face-train` | `osatura face-train --samples N --seed S --out model.bin [--mine-rounds 1] [--scene N] [--reg-win N] [--reg-cell N] [--det-win N] [--det-cell N] [--ridge-lambda X] [--stages N] [--refine-win N] [--stage-patch N] [--stage-cell N] [--stage-lambda X]` | Antrenează propriul detector exclusiv pe fețe sintetice (fără descărcări). `--scene` este dimensiunea imaginii de antrenare (64..512); `--reg-win`, `--reg-cell`, `--det-win`, `--det-cell`, `--ridge-lambda`, `--stages`, `--refine-win`, `--stage-patch`, `--stage-cell` și `--stage-lambda` sunt parametrii de reglaj pentru dimensiunea modelului și pentru regresie (valorile implicite sunt cele cu care a fost antrenat modelul încorporat). |
| `face-eval` | `osatura face-eval [--model m.bin] [--eval-n 200] [--seed 9] [--scene 128] [--report r.json] [--json]` | Acuratețe sintetică pe date rezervate, fals pozitive și baleiaje de robustețe (JSON; `--report` îl scrie, `--json` îl afișează chiar și cu `--report`; `--scene` este dimensiunea imaginii de test). |
| `face-detect` | `osatura face-detect <image> [--model m.bin] [--json] [--overlay o.png] [--threshold T] [--max-faces N] [--min-side N] [--max-side N] [--max-dim N]` | Casete de față și 23 de repere; `--overlay` le desenează pe un PNG; `--min-side` / `--max-side` limitează dimensiunea feței în pixeli, iar `--max-dim` micșorează mai întâi o imagine mare. |
| `face-track` | `osatura face-track <video-or-frame-dir> <out.ndjson> [--model m.bin] [--fps 30] [--max-dim 256]` | NDJSON cu repere pentru `face-bake`. Un director de imagini este citit direct; un videoclip trece prin ffmpeg. |
| `face-calibrate` | `osatura face-calibrate <landmarks.ndjson> <out.json> [--calibrate-frames N] [--map mapfile.json] [--fps F] [--layout L]` | Calculează calibrarea feței neutre din primele N cadre. |
| `face-bake` | `osatura face-bake <project.json> <landmarks.ndjson> <mapfile.json> <out.json> [--calibrate-frames N\|0] [--calib calib.json] [--fps F] [--layout L] [--tol T] [--allow-empty]` | Coace semnalele faciale (`mouth_open`, `eye_l_open`, `eye_r_open`, `brow_raise`, `yaw`, `pitch`, `roll`, ...) în canale, cu un fișier de mapare în stilul `live-map` al cărui `source` este `face`. |

```
$ osatura face-synth syn --frames 6 --size 96 --seed 3
wrote 6 frames (96x96), truth.ndjson and params.json to syn
$ osatura face-train --samples 80 --seed 1 --out m.bin --scene 64        # 47 s in a debug build
wrote m.bin (649900 bytes) in 47.0s: 80 samples, seed 1, mined negatives 9
trained on synthetic faces only: expect stylised/cartoon faces to work, photographs and webcams are NOT validated
$ osatura face-train --samples 60 --seed 1 --out m.bin --scene 2
scene size must be 64..512                                                [exit 1]
$ osatura face-detect syn/000000.png --model m.bin
syn/000000.png: face 0: box centre (48.1, 47.4) side 69.0 score 4.49
$ osatura face-track syn trk.ndjson --model m.bin --fps 15 --max-dim 96
wrote trk.ndjson: 6 frames, face found in 6
$ osatura face-calibrate syn/truth.ndjson calib.json --calibrate-frames 3 --fps 15
6 frames, neutral from the first 3; wrote calib.json
$ echo '{"params":[{"source":"face","id":"mouth_open","target":"arm_low","prop":"bone_angle","out_min":0,"out_max":60}],"triggers":[]}' > map.json
$ osatura face-bake small.json trk.ndjson map.json baked.json --calibrate-frames 3 --fps 15 --tol 0.5
6 landmark frames -> +1 channels (K keys in them), 0 trigger keys set, 0 events; wrote baked.json
$ osatura face-eval --eval-n 6 --model m.bin --report ev.json            # writes the JSON report
```
Aceste comenzi își scriu fișierele de model/raport cu scriitorul atomic care refuză legăturile simbolice (folderele părinte sunt create). `face-bake` și `live-map` raportează `+N channels (K keys in them), M trigger keys set` și avertizează pe stderr când înregistrarea este mai lungă decât proiectul (cadrele în plus nu sunt coapte; lungimea proiectului rămâne neschimbată).

### `mocap-info`, `mocap-match`, `mocap-retarget`

```
osatura mocap-info     <file.bvh>
osatura mocap-match    <project.json> <file.bvh> [--map map.json]
osatura mocap-retarget <project.json> <file.bvh> <out.json> [--view front|side|top|yaw=DEG] [--map map.json] [--scale auto|N] [--fps F]
    [--smooth N] [--start F] [--end F] [--rest-frame F] [--min-conf C] [--weight W] [--mirror] [--no-root] [--in-place] [--keep-length] [--footlock BONE --ground Y]
```
* **`mocap-info`** afișează ierarhia, canalele, rata de cadre și întinderea unui fișier BVH.
  ```
  $ osatura mocap-info m.bvh
  4 joints (2 end sites), 15 channels per frame
  4 frames at 0.0417 s (24.000 fps), duration 0.125 s
  Hips  offset (0.000, 0.000, 0.000)  [Xpos Ypos Zpos Zrot Xrot Yrot]
    LeftUpLeg  offset (5.000, 0.000, 0.000)  [Zrot Xrot Yrot]
  ...
  ```
* **`mocap-match`** afișează **tabelul de potrivire automată articulație -> os** (slot, articulație BVH, os din rig, încredere, dovada `via`, raport de lungime, notă), apoi articulațiile/oasele nepotrivite și avertismentele. Se folosesc atât numele, cât și topologia. `--map` este un obiect JSON `{"bvh joint": "rig bone" | null}` (`null` sau `""` exclude o articulație; este acceptat și învelișul `{"map": {...}}`). Se încheie întotdeauna cu codul 0 - când nu s-a potrivit nimic, avertizează doar `no bones matched: supply a --map file`.
  ```
  $ osatura mocap-match demo.json m.bvh --map map2.json         # {"Hips":null,"LeftUpLeg":"thigh","LeftLeg":"shin"}
  slot         bvh joint                  rig bone                conf  via         len x  note
  shin.l       LeftLeg                    shin                    1.00  manual      12.00  manual
  thigh.l      LeftUpLeg                  thigh                   1.00  manual      12.00  manual
  unmatched BVH joints: Hips, RightUpLeg
  ...
  ```
* **`mocap-retarget`** coace mișcarea sub formă de canale **adăugate**: proiectează rotațiile 3D în `--view` 2D (`front` implicit, `side`, `top` sau `yaw=DEG`), scalează mișcarea rădăcinii (`--scale auto` sau un număr), reeșantionează la `--fps` (implicit cel al proiectului), netezește (`--smooth` cadre), preia cadrele BVH `--start..--end`, folosește `--rest-frame` ca poză de repaus, elimină potrivirile sub `--min-conf`, amestecă cu `--weight`, oglindește cu `--mirror`, elimină mișcarea rădăcinii cu `--no-root`, convertește un clip cu deplasare în unul pe loc cu `--in-place` și fixează un picior cu `--footlock BONE --ground Y`. Afișează tabelul, notele și `N bones, F frames at R fps, root scale S, +C channels; wrote <out>`. Un clip mai lung decât proiectul **extinde lungimea proiectului**; o notă după rezumat indică de la ce lungime la ce lungime. `--keep-length` păstrează în schimb lungimea proiectului și trunchiază cheile la ultimul cadru (acolo se plasează o cheie interpolată; o notă numără cheile necoapte). Dacă nimic nu depășește pragul de încredere: `no bone matches to retarget (all below the confidence threshold?)` (cod de ieșire 1). Notă: pentru această comandă `--scale` este `auto|N`, nu scara obișnuită de randare.
  ```
  $ osatura mocap-retarget demo.json m.bvh mr.json --map map2.json --view side --smooth 1 --in-place --no-root
  ... (table)
  2 bones, 4 frames at 24 fps, root scale 12.0000, +2 channels; wrote mr.json
  ```
**Limite (ROADMAP §9):** nu a fost testat niciun fișier real de motion capture (niciunul nu este inclus: exemplele folosesc un BVH scris de mână, cu 4 cadre); proiecția 2D elimină adâncimea (un os orientat spre cameră își păstrează ultimul unghi).

---

## 11. Mașini de stări, legări de date, containere și regiuni de derulare

Aceste comenzi gestionează părțile de **interacțiune în timpul execuției** ale unui proiect: mașinile de stări (`machines`), legările de date
(`bindings`), containerele de layout (`containers`) și regiunile de derulare (`scrolls`). Toate își păstrează datele în interiorul
fișierului de proiect; formele `add` citesc un mic fișier JSON auxiliar, îl validează și scriu un proiect nou.
Modelul de runtime în sine este documentat în [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

### `machine` (add, list, remove, validate)

**Sinopsis:**
```
osatura machine add      <project.json> <out.json> <machine.json>
osatura machine list     <project.json>
osatura machine remove   <project.json> <out.json> <name>
osatura machine validate <project.json>
```

O **mașină** are `inputs` denumite (`bool`, `number` sau `trigger`) și unul sau mai multe `layers`; fiecare strat are o stare
`entry`, `states` și `transitions`. O stare redă o **sursă**: `{"kind":"segment","start":F,"end":F}` (cadre ale
canalelor cu chei ale proiectului), `{"kind":"clip","clip":NAME}`, `{"kind":"region","marker":NAME}` sau un blend tree 1-D
`{"kind":"blend","input":NAME,"children":[{"threshold":T,"source":{...}}]}`. Câmpurile unei stări: `name`, `source`,
`mode` (`loop` implicit, `once`, `ping_pong`), `speed` (1), `blend_in` (secunde). Câmpurile unei tranziții: `from` (o stare sau
`"*"`), `to`, `when` (o listă de `{"input":NAME,"op":eq|ne|lt|le|gt|ge,"value":V}`, toate trebuie să fie îndeplinite; `{"input":"go"}`
singur înseamnă „este egal cu 1"), `exit_time` (0..1), `duration` (secunde de crossfade) și `ease`. Cheile necunoscute dintr-o tranziție
sunt o **eroare** (o greșeală de scriere precum `conditions` ar transforma altfel o tranziție condiționată într-una necondiționată).

```
$ cat mach.json
{"name":"m","inputs":[{"name":"go","kind":"bool"}],"layers":[{"name":"base","entry":"idle","states":[
 {"name":"idle","source":{"kind":"segment","start":0,"end":12}},
 {"name":"act","source":{"kind":"segment","start":12,"end":36}}],
 "transitions":[{"from":"idle","to":"act","when":[{"input":"go"}],"duration":0.1}]}]}
$ osatura machine add demo.json d1.json mach.json
added machine 'm' (1 machines); wrote d1.json
$ osatura machine list d1.json
m  inputs [go:bool]
  layer base (entry idle, mask []): 2 states, 1 transitions
$ osatura machine validate d1.json
1 machines, 0 bindings: ok
```

`machine remove` refuză să elimine o mașină pe care o mai controlează o legare
(`binding 'goBind' still drives machine 'm'`, cod de ieșire 1): elimină mai întâi legarea.
**Erori:** o mașină nevalidă (stare necunoscută, nume duplicate, referință greșită la o intrare) este respinsă cu un mesaj și
nu se scrie nimic. **Conexe:** `machine-sim`, `bind`, `state-bake` (formatul mai vechi de mașină bazat pe cadre).
**Limite:** mașinile sunt evaluate de runtime și de `machine-sim`; exportatoarele nu coc nimic din toate acestea (Lottie, GIF
și celelalte fișiere conțin doar cadre, vezi [EXPORT_FORMATS.md](EXPORT_FORMATS.md)).

### `machine-sim`

**Sinopsis:** `osatura machine-sim <project.json> <script.json> <out_dir|out.json> [--fps N --svg-frames N]`

Redă evenimente de intrare temporizate pe o mașină și înregistrează starea la fiecare tick. Scriptul este
`{"machine": NAME?, "duration": SECONDS?, "events": [{"t": SECONDS, "set": {"input": value}, "trigger": ["name"],
"value": {"external.path": number}}]}`. `machine` este implicit unica mașină a proiectului, `duration` este implicit ultimul
eveniment plus o secundă; `value` alimentează legările de date (vezi `bind`). `--fps` implicit este fps-ul proiectului.
`--svg-frames N` randează suplimentar N cadre SVG distribuite pe durata simulării.

```
$ cat sc.json
{"duration":1.5,"events":[{"t":0.5,"value":{"hud.go":1}}]}
$ osatura machine-sim d3.json sc.json sim.json --fps 24 --svg-frames 0
   0.000s  base     Entered idle
   0.500s  base     Exited idle
   0.500s  base     Entered act
37 ticks at 24 fps, 3 events, 0 svg frames; wrote sim.json
$ osatura machine-sim d3.json sc.json simdir --svg-frames 2     # a name that does not end in .json is a folder
...
37 ticks at 24 fps, 3 events, 2 svg frames; wrote simdir/sim.json
$ ls simdir
frame_00000.svg  frame_00036.svg  sim.json
```
`sim.json` conține `machine`, `fps`, `ticks[]` (`tick`, `time`, `states` ca perechi `[layer,state]`, `fading`) și jurnalul
de evenimente. **Conexe:** `bind`, `machine`.

### `state-bake`

**Sinopsis:** `osatura state-bake <project.json> <machine.json> <events.json> <out.json> [--tol 0.01]`

Coace o mașină de stări **bazată pe cadre** (un format diferit, mai vechi decât `machine`) în canale obișnuite cu chei.
`machine.json` este `{"initial":NAME,"states":[{"name","length","looped","channels":[...]}],"transitions":[{"from":NAME
or "*","to","when":COND,"fade":FRAMES,"ease"}]}`, unde `COND` este una dintre `{"type":"event","name"}`,
`{"type":"above"|"below"|"equals","input","value"}`, `{"type":"finished"}`, `{"type":"after","frames"}`.
`events.json` este o listă de `{"frame":N,"name":NAME,"value":V}`. Rezultatul este adăugat la canalele proiectului
(`--tol` este toleranța de reducere a cheilor), proiectul este compilat și scris.

```
$ echo '[{"frame":10,"name":"go"}]' > ev.json
$ osatura state-bake demo.json sm.json ev.json baked.json --tol 0.01
wrote baked.json (+1 channels)
```
**Capcană:** canalul copt este adăugat după cele existente; canalele rulează în ordinea listei, așa că elimină sau pune pe mut
canalul original pentru aceeași țintă dacă vrei ca varianta coaptă să prevaleze. Un fișier de mașină în formatul `machine`
(cu `layers`) este refuzat: `invalid JSON in mach.json: missing field 'initial'`.

### `bind` (add, list, remove) și `bind-eval`

**Sinopsis:**
```
osatura bind add    <project.json> <out.json> <name> <value.path> <target:prop|@machine:input>
                     [--expr E --remap a,b,c,d --no-clamp --smooth S --mode replace|add|multiply --weight W]
osatura bind list   <project.json>
osatura bind remove <project.json> <out.json> <name>
osatura bind-eval   <project.json> <values.json> [--data out.json]
```

O **legare de date** mapează o cale de valoare externă (de exemplu `hud.size`) pe o proprietate a proiectului (`sun:layer_opacity`,
numele proprietăților sunt cele în snake_case din secțiunea 1) sau pe o intrare a unei mașini (`@m:go`). `--remap a,b,c,d` mapează
intervalul de intrare `a..b` pe `c..d` (limitat, în afară de cazul în care se folosește `--no-clamp`); `--expr` este o expresie peste valoare;
`--smooth S` aplică un filtru trece-jos pe S secunde; `--mode` și `--weight` controlează modul în care rezultatul se combină cu canalele.

```
$ osatura bind add d1.json d2.json goBind hud.go @m:go
added binding 'goBind' (1 bindings); wrote d2.json
$ osatura bind add d2.json d3.json sz hud.size sun:layer_opacity --remap 0,100,0.2,1 --smooth 0.1
added binding 'sz' (2 bindings); wrote d3.json
$ osatura bind list d3.json
goBind  hud.go -> @m:go
sz  hud.size -> sun:LayerOpacity remap 0..100 -> 0.2..1 clamp smooth 0.1s
$ echo '{"hud.go":1,"hud.size":50}' > vals.json
$ osatura bind-eval d3.json vals.json --data bv.json
goBind = 1
sz = 0.6000000238418579
$ osatura bind add d2.json x.json sz hud.size sun:scale
unknown property 'scale'                                                  [exit 1]
$ osatura bind remove d3.json d7.json sz
removed binding 'sz'
```
`bind-eval` afișează rezultatul fiecărei legări pentru valorile date (`--data` le scrie și ca JSON). `list` afișează numele
proprietăților în CamelCase; linia de comandă primește întotdeauna snake_case. **Conexe:** `machine`, `machine-sim` (harta sa `value`).

### `container` (add, list, remove, solve)

**Sinopsis:**
```
osatura container add    <project.json> <out.json> <container.json>
osatura container list   <project.json>
osatura container remove <project.json> <out.json> <id>
osatura container solve  <project.json> [--size WxH --data out.json]
```

Un **container** este o casetă de layout de tip flexbox: `id`, `members`, `axis` (`row`|`column`), `gap`, `cross_gap`, `padding`
(`[left,top,right,bottom]`), `justify` (`start|center|end|space_between|space_around`), `align`
(`start|center|end|stretch`), `wrap` (`none|wrap`) și `rect`, o regulă pentru propriul dreptunghi pe canvas:
`{"kind":"fixed",...}`, `{"kind":"pin","left","right","top","bottom","width","height"}` (distanțe față de marginile canvasului; oricare două
pentru fiecare axă), `{"kind":"aspect",...}` sau `{"kind":"hug"}`. Un **membru** este exact unul dintre `{"layer":PATH}`,
`{"container":ID}` sau `{"spacer":true}`, cu `grow`, `shrink`, `width`, `height`, `min_w/max_w/min_h/max_h` și
o suprascriere `align`; `"resize":true` permite solverului să scaleze stratul la slotul său.

```
$ cat cont.json
{"id":"row1","axis":"row","gap":10,"padding":[10,10,10,10],
 "rect":{"kind":"pin","left":10,"right":10,"top":10,"height":100},
 "members":[{"layer":"sun"},{"layer":"blush","grow":1,"resize":true}]}
$ osatura container add d3.json d4.json cont.json
added container 'row1' (1 containers); wrote d4.json
$ osatura container list d4.json
row1: Row, 2 members, gap 10, justify Start, align Start, wrap None, rect {"kind":"pin","left":10.0,...}
$ osatura container solve d4.json --size 800x480 --data solved.json
canvas 800x480
container row1: x 10.00 y 10.00 w 780.00 h 100.00
layer sun: dx -430.000 dy 0.000 scale 1.0000 x 1.0000
layer blush: dx 280.432 dy -158.242 scale 33.8889 x 1.0000
```
Fără `--size` se folosește dimensiunea proprie a proiectului. `solve` afișează dreptunghiul fiecărui container și, pentru fiecare strat, decalajul și
scara pe care layout-ul le aplică peste animație. **Erori:** un strat necunoscut, un container imbricat lipsă sau un
ciclu sunt respinse de `add` (nu se scrie nimic). **Conexe:** `scroll`, `layout-preview`.
**Limite:** layout-ul este aplicat de runtime/previzualizare; fișierele exportate conțin layout-ul rezolvat pentru dimensiunea proprie a proiectului.

### `scroll` (add, list, remove, eval)

**Sinopsis:**
```
osatura scroll add    <project.json> <out.json> <scroll.json>
osatura scroll list   <project.json>
osatura scroll remove <project.json> <out.json> <id>
osatura scroll eval   <project.json> <id> [--offset X,Y --size WxH --velocity VX,VY --time S]
```

O **regiune de derulare** are `id`, `content` (id-ul unui container care nu este membru al altui container), `viewport`
(o regulă de dreptunghi ca mai sus), `axis` (`x`, `y` sau ambele), `offset`, `elastic` (`{"limit","stiffness","damping_ratio"}`; efect
de bandă elastică în loc de limitare fermă), `snap_x`/`snap_y` (decalaje de repaus), `fling` (secunde din viteza de eliberare proiectate pentru a găsi
punctul de aterizare, implicit 0,15), `clip_layer` (un strat dreptunghi folosit ca mască pentru conținut) și `input` (o cale de valoare externă care
comandă decalajul).

```
$ cat scroll.json
{"id":"sc1","content":"row1","axis":"x","viewport":{"kind":"pin","left":0,"right":0,"top":0,"height":120},
 "elastic":{"limit":60,"stiffness":200,"damping_ratio":0.8},"snap_x":[0,100,200]}
$ osatura scroll add d4.json d5.json scroll.json
added scroll 'sc1' (1 scrolls); wrote d5.json
$ osatura scroll list d5.json
sc1: content row1, axis X, offset [0.0, 0.0], elastic true, snap x [0.0, 100.0, 200.0] y [], clip -
$ osatura scroll eval d5.json sc1 --offset 400,0
viewport [0.0, 0.0, 640.0, 120.0] content [-59.92364, 0.0, 640.0, 120.0] max [0.0, 0.0]
requested offset [400.0, 0.0] -> applied [59.92364, 0.0]
$ osatura scroll eval d5.json sc1 --offset 80,0 --velocity -500,0 --time 0.5
...
requested offset [80.0, 0.0] -> applied [44.184174, 0.0]
t 0.100s offset [22.794, 0.000]
t 0.300s offset [0.051, 0.000]
t 0.500s offset [-0.197, 0.000]
```
Fără `--velocity`, `eval` afișează decalajul limitat (sau cu efect de bandă elastică) corespunzător celui cerut. Cu `--velocity`,
simulează eliberarea (inerție, snap, arc) timp de `--time` secunde și afișează la fiecare 0,1 s. `--size WxH` evaluează pentru o altă dimensiune de canvas.
În exemplu, conținutul are lățimea viewport-ului, deci tot ce depășește 0 este derulare în exces și arată banda elastică.

### `layout-preview`

**Sinopsis:** `osatura layout-preview <project.json> <out.svg> --size WxH [--frame N --scroll id=X,Y]`

Randează un cadru cu layout-ul **rezolvat din nou pentru o altă dimensiune de canvas** și cu contururile containerelor desenate, astfel încât să
poți verifica cum se reorganizează un design pe un telefon sau pe un ecran lat. `--scroll id=X,Y` stabilește un decalaj de derulare (repetă pentru mai multe regiuni).

```
$ osatura layout-preview d5.json lp.svg --size 800x480 --scroll sc1=50,0
wrote lp.svg (800x480, 1 containers, 2 layers placed)
```

## 12. Efecte, profunzime de câmp, onion skin, urme și cameră

### `fx`

**Sinopsis:** `osatura fx <project.json> <frame> <chain.json> <out.png> [--scale N] [--layer-normals] [--clamp]`

Un `<frame>` din afara proiectului este o eroare; `--clamp` randează cel mai apropiat cadru valid.

Randează un cadru și rulează peste el un **lanț de post-efecte**. `chain.json` este o listă de efecte, fiecare cu un `type`:
`blur {radius}`, `drop_shadow {dx,dy,blur,color,opacity}`, `glow {radius,strength,color?}`, `outline {width,color}`,
`posterize {levels}`, `pixelate {size}`, `levels {in_black,in_white,gamma,out_black?,out_white?}`, `fade {amount}` și
`relight {ambient,bevel,height,lights:[{x,y,z,color?,intensity?,radius?,cast_shadow?,spot?,falloff?}],ground_shadow?,
normal_map?,ambient_color?}`. Fără `normal_map`, relight umbrește un bevel obținut din canalul alfa estompat.

```
$ cat chain.json
[{"type":"drop_shadow","dx":6,"dy":6,"blur":3,"color":"#000000","opacity":0.5},{"type":"glow","radius":6,"strength":0.5}]
$ osatura fx demo.json 12 chain.json fx.png --scale 0.5
wrote fx.png (2 effect(s))
$ cat relight.json
[{"type":"relight","ambient":0.4,"bevel":6,"height":8,"lights":[{"x":300,"y":100,"z":120,"intensity":1.0}],"ground_shadow":0.3}]
$ osatura fx demo.json 12 relight.json lit.png --scale 0.5
wrote lit.png (1 effect(s))
```
`--layer-normals` umbrește fiecare strat cu propria hartă de normale (vezi `layer-normal`), în loc de o singură hartă pentru întreaga imagine.
O lumină care omite `z` este respinsă (`missing field 'z'`). **Limite:** lanțul este un pipeline pentru imagini statice, doar PNG; niciunul dintre
aceste efecte nu ajunge în Lottie, GIF sau playerul HTML (exportatorul Lottie are propriul subset de efecte, vezi EXPORT_FORMATS.md).

### `color-space`

**Sinopsis:** `osatura color-space <project.json> <out.json> srgb|linear`

Stabilește `color_space` al proiectului, spațiul în care compozitorul raster (PNG, GIF, APNG, video, sprite sheet, `compare`, `ci`) și compozitorul în virgulă mobilă (`render-hdr`) amestecă. `srgb` (implicit și cel cu care a fost înregistrat fiecare golden) amestecă valorile stocate, codificate sRGB; `linear` convertește fiecare culoare (umpleri, contururi, opriri de gradient, fundal, pixeli de imagine) în lumină liniară, compune acolo în f32 și convertește în sRGB o singură dată, la final. Un strat negru de 50 % peste alb dă atunci 188/255 în loc de 128/255, iar gradientele interpolează în lumină liniară. Cu `srgb` cheia nu este scrisă, deci fișierele și golden-urile existente nu se schimbă.

```
$ osatura color-space demo.json lin.json linear
wrote lin.json (color_space: linear)
$ osatura render lin.json 0 lin.png
wrote lin.png
$ osatura color-space lin.json x.json p3
unknown colour space 'p3' (srgb or linear)
```
**Limite:** randarea pe 8 biți trece la compozitorul în virgulă mobilă pentru proiectele liniare (rasterizator propriu de acoperire, deci marginile cu anti-aliasing diferă ușor de rezultatul sRGB); culorile efectelor de strat (shadow, glow, tint) sunt folosite așa cum sunt scrise, fără conversie; `dof` și lanțul de post-efecte (`fx`) rămân în sRGB; SVG, Lottie și randatorul GPU ignoră setarea; reeșantionarea imaginilor are loc în sRGB, înainte de conversie. Modurile de amestecare se aplică valorilor liniare.

### `layer-normal`

**Sinopsis:** `osatura layer-normal <project.json> <out.json> <layer> <normal.png|none> [--strength S] [--fit stretch|tile|exact] [--green-down]`

Atașează (sau, cu `none`, elimină) o hartă de normale în spațiu tangent pe un strat de imagine, pentru `fx ... --layer-normals`.
`--fit` stabilește cum se potrivește o hartă de altă dimensiune (`stretch` implicit, `tile`, `exact`); `--green-down` inversează canalul
verde (hărți în stil DirectX); `--strength` scalează înclinarea (1 = așa cum este stocată, 0 = plat).

```
$ osatura layer-normal demo.json dn.json sun n.png --strength 1 --fit stretch
wrote dn.json
$ osatura fx dn.json 12 chain.json fxn.png --scale 0.5 --layer-normals
wrote fxn.png (2 effect(s))
$ osatura layer-normal dn.json dn2.json sun none
wrote dn2.json
```

### `onion` (două forme)

**Sinopsis:**
```
osatura onion <project.json> <frame> <out.png> [--before 2 --after 1 --step 1 --scale N]
osatura onion <project.json> <out.svg> --frame N [--before B --after A --step S --keyed --outline --all-parts
               --layers a,b --tint-before C --tint-after C --opacity X --falloff X --ghosts-only]
```
Prima formă scrie un PNG cu fantome estompate ale cadrelor dinainte și de după. A doua scrie o imagine statică SVG: cadrul
cu fantome onion-skin colorate suprapuse. `--keyed` folosește cele mai apropiate **cadre cheie** în loc de pași fixi;
`--outline` desenează fantomele ca contururi; `--all-parts` creează fantome pentru fiecare piesă (implicit: piesele în mișcare), `--layers` le limitează,
`--ghosts-only` omite cadrul propriu-zis.

```
$ osatura onion demo.json 12 on.png --before 2 --after 1 --step 2 --scale 0.5
wrote on.png
$ osatura onion demo.json on.svg --frame 12 --before 2 --after 2 --step 3 --outline --opacity 0.4
wrote on.svg (4 ghost(s) around frame 12)
$ osatura onion demo.json on2.svg --frame 12 --keyed --ghosts-only
wrote on2.svg (3 ghost(s) around frame 12)
```
Forma este aleasă după al doilea argument: un număr înseamnă forma PNG, un nume terminat în `.svg` înseamnă forma SVG.

### `trail` (două forme)

**Sinopsis:**
```
osatura trail <project.json> <bone> <out.svg> [--from F --to F]
osatura trail <project.json> [out.svg] --target layer:PATH|bone:NAME[:head|tip]|point:LAYER:KIND:A[:B] [--from F --to F --smooth N --json]
```
Desenează **urma de mișcare** a vârfului unui os (prima formă) sau a oricărei ținte (a doua formă: un strat, capul/vârful unui os sau un
punct de deformator), cu un marcaj pe cadru și markere la chei. `--smooth N` netezește curba; `--json` afișează curba eșantionată
ca JSON (iar atunci `out.svg` este opțional).

```
$ osatura trail demo.json hand tr.svg --from 0 --to 24
wrote tr.svg (25 points)
$ osatura trail demo.json tr3.svg --target layer:sun --from 0 --to 24
wrote tr3.svg (25 ticks, 0 key marker(s))
$ osatura trail demo.json tr2.svg --target bone:hand:tip --from 0 --to 24 --smooth 2 --json
{ "curve": [ [212.8, 83.84], [213.77, 83.99], ...
```

### `dof` și `dof-export`

**Sinopsis:**
```
osatura dof        <project.json> <frame> <out.png> [--focus 1.0 --strength 12 --max-radius 24 --scale N --clamp]
osatura dof-export <project.json> <out.gif|out.apng|out.mp4|out.webm|out-folder> [--from F --to F --focus N --strength N --max-radius N --scale N --codec vp9|av1]
```
Un `<frame>` din afara proiectului este o eroare pentru `dof` (și pentru `render-hdr` și `fx`); `--clamp` randează în schimb cel mai apropiat cadru valid, ca la `render`.

Profunzime de câmp: fiecare **strat de nivel superior** este estompat cu `|parallax - focus|` ori `strength` (pixeli pe unitate de
distanță de paralaxă, plafonat la `--max-radius`). Fără `--focus` / `--strength`, valorile urmează canalele cu chei
`camera_focus` / `camera_blur` (focus implicit 1.0). Straturile își iau adâncimea din `parallax` (vezi secțiunea 5).
`dof-export` randează intervalul `--from`..`--to` (numere întregi de cadre; implicit: de la 0 până la sfârșit) într-un GIF animat, un
PNG animat, un videoclip `.mp4`/`.webm` (necesită `ffmpeg`; `--codec` este implicit `vp9`; nu include audio) sau, pentru un nume fără
extensie, într-un folder de PNG-uri numerotate (`f0000.png`...). Orice altă extensie este refuzată (`dof-export writes .gif, .apng,
.mp4, .webm or a folder (no extension)`, nu se scrie nimic).

```
$ osatura dof demo.json 12 dof.png --focus 1.0 --strength 12 --max-radius 24 --scale 0.5
wrote dof.png
$ osatura dof-export demo.json dof.gif --from 0 --to 6 --scale 0.25
wrote dof.gif
$ osatura dof-export demo.json dofdir --from 0 --to 2 --scale 0.25      # dofdir/f0000.png ... f0002.png
wrote dofdir
```
**Conexe:** `parallax`, `render --proxy`, `shot`/`camera-seq` pentru mișcări de cameră. **Limite:** estomparea este o
aproximare pe strat, nu un model de lentilă (fără forme bokeh, fără tratarea ocluziilor); nu este exportată în Lottie.

## 13. Nivel de detaliu, rapoarte de dimensiune și export-fit

### `size-report`

**Sinopsis:** `osatura size-report <project.json> [--tol T] [--json]`

Arată **unde se află octeții**: fișierul de proiect, Lottie-ul copt și pagina HTML, pe straturi, imagini și canale
(chei și câte pot fi eliminate la toleranța `--tol`, implicit 0,05).

```
$ osatura size-report small.json
project file: 17959 bytes (3511 gzip)
  by layer (JSON bytes; embedded image base64 in brackets):
         2983  arm
         2053  leg
  ...
  by channel (keys, removable at tol 0.05):
          420  foot_ctrl.BoneOffsetX  keys 5 removable 1
  ...
baked Lottie: 46843 bytes (11180 gzip), by layer:
         9181  arm
```
`--json` afișează aceleași date ca JSON (`html.defs_bytes_total`, `frame_bytes_max`, ...). Folosește-l înainte de `export-fit` sau `lod`
pentru a vedea ce strat trebuie simplificat.

### `lod`

**Sinopsis:** `osatura lod <project.json> --tolerance-px 0.5 [--scale S | --size WxH] [--levels 4] [--ratio 2] [--out-dir D] [--report r.json] [--verify] [--frames N]`

Construiește **niveluri de detaliu** L1..Ln, fiecare cu o *limită de eroare declarată în pixeli de ieșire*: simplificarea traseelor, reducerea
cheilor, colapsul mesh-urilor și eliminarea (culling). Nivelul k are toleranța `--tolerance-px x ratio^(k-1)`. `--scale` sau `--size` indică
cât de mare este afișată ieșirea (implicit 1 px pe unitate). `--out-dir` (implicit `.`) primește `<name>_L1.json` ... iar
`--report` un raport JSON. `--verify` randează `--frames N` cadre și **măsoară** abaterea reală (cod de ieșire 1 dacă depășește
limita).

```
$ osatura lod small.json --tolerance-px 1 --levels 3 --out-dir lv --report rep.json --verify --frames 4
level  tol_px  anchors  mesh_v  keys  layers    bytes  declared_px  path  mesh  keys  cull  measured_px  ratio
L0      0.000       74       0    37      14    17959       0.0000 ...                              -      -
L1      1.000       70       0    37      14    17651       0.3861 0.386 0.000 0.000 0.000       0.3342   1.2x
L2      2.000       69       0    37      14    17573       0.9277 ...                           0.8185   1.1x
L3      4.000       65       0    37      14    17251       1.5372 ...                           1.3367   1.1x
scale 1 px per unit; level k tolerance = 1 px x 2^(k-1); files lv/s_L1..L3.json
  L1: keys: 3 of 8 keyed channels have a documented lever, 0 reduced
  L1: removed 4 path anchor(s)
$ ls lv
s_L1.json  s_L2.json  s_L3.json
```
**Capcană:** economiile pe un demo mic, construit de mână, sunt minuscule (aproximativ 4 % la 4 px); LOD-urile merită pe artă vectorizată automat sau cu multe mesh-uri.
Raportul și fișierele de nivel sunt scrise cu o scriere simplă (non-atomică).

### `lod-pick`

**Sinopsis:** `osatura lod-pick <project.json> --screen-px WxH [--max-px 0.5] [--tolerance-px 0.25] [--levels 4] [--out chosen.json]`

Alege **cel mai grosier nivel** a cărui limită declarată rămâne în `--max-px` pixeli atunci când grafica este afișată pe un
ecran de `--screen-px` (scara este dimensiunea ecranului împărțită la dimensiunea proiectului). `--out` scrie acel nivel.

```
$ osatura lod-pick small.json --screen-px 320x240 --out chosen.json
level  tol_px  anchors ...
screen 320x240 -> scale 0.5000 px/unit; budget 0.5 px -> level L3 (declared 0.4638 px, 69 anchors, 0 mesh vertices, 37 keys)
wrote chosen.json
$ osatura lod-pick small.json --screen-px 64x48 --max-px 2 --out chosen2.json
screen 64x48 -> scale 0.1000 px/unit; budget 2 px -> level L4 (declared 0.5310 px, 62 anchors, 0 mesh vertices, 37 keys)
```

### `export-fit`

**Sinopsis:** `osatura export-fit <project.json> <out> --max-size 500KB [--format lottie|html|gif|apng|webm|project] [--min-ssim X] [--max-seconds 60] [--measure raw|gzip] [--frames N] [--search ladder|descent|grid] [--report out.json] [--pareto out.csv] [--audit] [--no-baseline]`

Găsește **cel mai bun export de calitate care încape într-un buget de octeți**: caută printre parametrii de calitate ai exportatorului (Lottie: `decimals`,
`path_eps`, `key_tol`, `frame_step`; celelalte formate au propriii parametri) și notează fiecare candidat cu SSIM față de
randarea originală. `--max-size` acceptă `500KB` (1000 de octeți), `1.5MB`, `64000`, `200KiB` (1024 de octeți). Formatul provine din
`--format` sau din extensia fișierului de ieșire (`.json` este Lottie, `.html`, `.gif`, `.apng`, `.webm`); `project` scrie un JSON de
proiect subțiat. `--measure gzip` stabilește bugetul pe dimensiunea gzip (pentru servere care comprimă). `--min-ssim` este un prag minim de calitate.
`--max-seconds` este un **buget real de timp de ceas** (implicit 60; `0` = nelimitat): la expirare se păstrează cel mai bun rezultat de până atunci.
`--frames N` este numărul de cadre eșantionate pentru SSIM (implicit 8). `--search` alege strategia (`descent` implicit).
`--report` / `--pareto` scriu înregistrarea căutării ca JSON / CSV; `--audit` afișează fiecare candidat; `--no-baseline` omite
compararea cu setările implicite.

```
$ osatura export-fit small.json fit.json --max-size 30KB --format lottie --report fr.json
lottie -> fit.json: 25756 bytes raw, 5736 gzip (budget 30000 raw); SSIM mean 0.9699, worst 0.9308 at frame 5, max CIEDE2000 68.67; 5 candidates built
time budget exhausted (60 s, checked inside every candidate): the search stopped early and the best result so far is kept (...)
$ osatura export-fit small.json fit.html --max-size 60KB --format html --max-seconds 20 --frames 4
html -> fit.html: 59166 bytes raw, 8550 gzip (budget 60000 raw); SSIM mean 1.0000, worst 1.0000 at frame 5, max CIEDE2000 8.54; 1 candidates built
existing exporter at defaults: 66391 bytes raw, 8933 gzip
$ osatura export-fit small.json fit2.json --max-size 1KB --format lottie --max-seconds 10 --frames 3
lottie -> fit2.json: 14751 bytes raw, 2418 gzip (budget 1000 raw); ...
knobs at their floor: decimals, path_eps, key_tol, frame_step
budget not met: the smallest export found is 14751 bytes (raw), above the budget of 1000 bytes: ...          [exit 1]
$ osatura export-fit small.json fit.xyz --max-size 1KB
cannot tell the format from the output name; pass --format lottie|html|gif|apng|webm|project   [exit 1]
```
**Coduri de ieșire:** 0 când rezultatul încape în buget (chiar dacă bugetul de timp a oprit căutarea mai devreme); 1 când bugetul
nu poate fi atins (cel mai mic export este totuși scris) sau la argumente greșite. **Capcane:** lucrează pe ieșirea coaptă,
deci nu poate elimina mai mult decât permit parametrii (un Lottie de 1 KB este imposibil pentru un rig cu 14 straturi); fiecare candidat este un
export complet, așa că proiectele mari au nevoie de un `--max-seconds` generos (un build de debug este mult mai lent decât unul de release). Fișierele de
raport sunt scrise cu o scriere simplă. **Conexe:** `size-report`, `lod`, `export` (secțiunea 4), EXPORT_FORMATS.md (sfaturi de dimensiune).

## 14. Asigurarea calității

Conexe, documentate mai devreme: `check` (validare, secțiunea 3), `diff` (diff semantic, secțiunea 3), `compare` (diff de imagine între două
randări, secțiunea 4).

### `lint`

**Sinopsis:** `osatura lint <project.json>`

Verificări statice ale rig-ului și ale animației. Afișează câte o linie pentru fiecare constatare: `Severity CODE [target:property] message`. Cod de ieșire 1 dacă vreo
constatare are severitatea Error sau mai mare; Info și Warning ies cu 0.
```
$ osatura lint small.json
Info L007 [:CameraX] key at frame 24 lies outside the 0..8 timeline
Info L007 [arm_low:BoneAngle] key at frame 12 lies outside the 0..8 timeline
```
`L007` este o cheie aflată în afara timeline-ului (aici pentru că `small.json` este demo-ul tăiat la 8 cadre).

### `contracts`

**Sinopsis:** `osatura contracts <project.json> <contracts.json>`

**Contracte de mișcare**: teste unitare pentru animație. `contracts.json` este o listă de contracte cu un `type`:
`loop_seamless {tol}`, `tip_range {bone,axis,min,max}`, `max_speed {bone,px_per_frame}` (în enum-ul `Contract` al motorului
există și alte tipuri). Încălcările se afișează pe stdout, numărul lor pe stderr, iar codul de ieșire este 1.
```
$ cat contracts.json
[{"type":"loop_seamless","tol":0.5},{"type":"tip_range","bone":"hand","axis":"y","min":0,"max":400},{"type":"max_speed","bone":"hand","px_per_frame":3}]
$ osatura contracts small.json contracts.json
contract 0 violated at frame 8: bone 'arm_up' ends 35.4 px away from where it started (tolerance 0.5)
...
contract 2 violated at frame 6: tip of 'hand' moves 3.3 px/frame (limit 3)
7 violation(s)                                                            [stderr, exit 1]
```

### `coach`

**Sinopsis:** `osatura coach <project.json>`

Sugestii de timing și de principii ale animației (overlap, anticipation, easing). Doar sfaturi, cod de ieșire întotdeauna 0.
```
$ osatura coach small.json
C003 [, arm_low, arm_up, foot_ctrl, hand, mouth, target] frame 0: 7 parts start moving on frame 0; offset them by a few frames (overlap) for a more natural feel
```

### `safety`

**Sinopsis:** `osatura safety <project.json> [--scale N]`

Raport despre flash-uri, flash-uri roșii și mișcarea camerei (o aproximare a WCAG 2.3.1; un ajutor de inginerie, nu o
certificare pentru difuzare). Randează fiecare cadru la `--scale` (folosește o valoare mică pentru proiecte lungi).
```
$ osatura safety small.json --scale 0.25
flashes      ok    worst 0 transitions/s (frame 0)
red flashes  ok    worst 0 transitions/s (frame 0)
camera       pan 0.06 widths/s, rotation 0°/s, zoom ×1.00/s
```

### `reduced-motion`

**Sinopsis:** `osatura reduced-motion <project.json> <out.json> [--factor 0.25]`

Scrie o copie mai calmă: mișcarea camerei și wobble-ul sunt scalate cu `--factor` (implicit 0,25).
```
$ osatura reduced-motion small.json calm.json --factor 0.25
wrote calm.json
```

### `golden`

**Sinopsis:**
```
osatura golden write <project.json> <golden.json> [--scale N]
osatura golden check <project.json> <golden.json> [--tol T]
```
`write --scale N` primește N în (0, 1]: o valoare mai mare este **refuzată** (`golden write: --scale 2 is above 1 ...`, nu se scrie nimic; înainte era limitată silențios la 1). `check` randează din nou la scara stocată în fișierul golden și nu are opțiunea `--scale`: `golden check ... --scale N` este refuzat ca opțiune necunoscută.


Un test de regresie ușor: `write` înregistrează un hash și o miniatură de 16x16 pentru fiecare cadru în `golden.json`; `check`
randează din nou și compară. **Exact în mod implicit** (`--tol 0`): orice diferență a hash-ului de pixeli duce la eșec. `--tol T` (T > 0) relaxează
verificarea: un hash modificat trece totuși dacă distanța dintre miniaturi este cel mult T. Un `--tol` negativ sau nefinit este o eroare.
Pentru golden-uri bazate pe imagini, cu o metrică perceptuală, folosește `ci`.
```
$ osatura golden write small.json g.json --scale 0.25
wrote g.json (8 frames)
$ osatura golden check small.json g.json
all 8 frames match exactly
$ osatura golden check small.json g.json --tol 0.01
all 8 frames match (within thumbnail tolerance 0.01)
```
Cadrele diferite sunt listate (schimbarea hash-ului, distanța dintre miniaturi, numărul celor 256 de celule de miniatură care diferă, cea mai mare
diferență de canal din 255), iar codul de ieșire este 1:
```
$ osatura golden check changed.json g.json
frame 1 differs: pixel hash changed, thumbnail distance 0.003, 12 of 256 thumbnail cells differ, max channel difference 31/255
2 frame(s) differ (exact comparison; --tol T accepts small differences)       [stderr, exit 1]
```
Un cadru prezent doar de o parte este raportat ca `one side only`. **Limite:** fișierul golden păstrează doar hash-ul și
miniatura, deci numerele raportate sunt la nivel de miniatură, nu la nivel de pixel. O modificare invizibilă în randare (de exemplu
o culoare de fundal acoperită complet de un strat opac, cum este cerul din demo) lasă în mod legitim hash-ul neschimbat.

### `ci` (init, run, bless, diff)

**Sinopsis:**
```
osatura ci init <project.json> [--out osatura-ci.json] [--no-goldens]
osatura ci run  [manifest] [--update] [--bless] [--prune] [--only a,b] [--junit f.xml] [--html f.html] [--json f.json] [--artifacts dir] [--quiet]
osatura ci bless [manifest]
osatura ci diff a.png b.png [--metric ssim|de2000|de76|mse] [--tol T] [--ignore x,y,w,h;...] [--aa N] [--out heat.png] [--side s.png] [--overlay o.png] [--flicker f.gif]
```
**CI pentru animație**: cadre golden comparate cu o metrică perceptuală (SSIM pe luma, CIEDE2000), plus verificări structurale și
bugete. Formatul manifestului și fluxul de lucru sunt descrise în [CI.md](CI.md). `exit 0` = trecut, `1` = o verificare a eșuat,
`2` = eroare (manifest ilizibil, ...).
```
$ osatura ci init small.json --out ci.json
wrote ci.json with 8 checks:
  validate     validate
  render       frames
  render       frames-float
  determinism  determinism
  fuzz         pose-fuzz
  limits       limits
  export-size  gif-size
$ osatura ci run ci.json --junit j.xml --html r.html --json ci_r.json
[PASS   ] validate     validate           project compiles; 8 lint finding(s), none at 'error' or above
[PASS   ] render       frames             6 frame(s) match
[PASS   ] determinism  determinism        6 render(s) repeated byte-identically
[INFO   ] fuzz         pose-fuzz          98 of 100 pose samples fail (98.00%); not a gate: non-gating (gate = false)
[PASS   ] export-size  gif-size           gif export is 82881 bytes (budget 124322)
[INFO   ] perf         render-speed       median 429.67 ms/frame, budget 1633 ms (+50% tolerance = 2449.50 ms)
8 checks: 6 passed, 0 failed, 0 errors, 2 info, 0 updated, 0 blessed (14.7 s)
$ osatura ci diff a.png b.png --metric ssim --out heat.png
size        480x360
SSIM        0.90536   (luma)   channels min 0.90300   (R, G, B)
de2000      max 68.674   mean 1.9434
pixels      11124 differ (6.4375%), 31 forgiven as anti-aliasing, 0 ignored
regions     118
images differ: luma SSIM 0.90536 < 0.99; ...                                [stderr, exit 1]
$ osatura ci run nonexist.json
cannot read manifest nonexist.json: No such file or directory               [exit 2]
```
`init` scrie manifestul și, în afară de cazul în care se folosește `--no-goldens`, PNG-urile golden în folderul `goldens/` de lângă el. `run
--update` rescrie golden-urile lipsă sau modificate, `--bless` acceptă randările noi ca golden-uri, `--prune` elimină
fișierele golden pe care nicio verificare nu le mai referă, `--only a,b` rulează verificările denumite. `bless` este `run --bless`. `diff` compară două PNG-uri: `--tol` este pragul
metricii, `--ignore` maschează dreptunghiuri (`x,y,w,h;...`), `--aa N` iartă diferențele de anti-aliasing, iar `--out`,
`--side`, `--overlay`, `--flicker` scriu o hartă termică, o imagine alăturată, o suprapunere și un GIF flicker.
**Capcană:** rapoartele (`--junit`, `--html`, `--json`) și fișierele golden sunt scrise cu scriitorul atomic care refuză legăturile simbolice (creând
folderele părinte); o cale de raport care este o legătură simbolică este refuzată după ce verificările au rulat (verificările sunt lente, așa că dă o cale reală). Un build de debug este lent: cele 8 verificări de mai sus au durat 15 s pe un proiect de 8 cadre.

### `fuzz`

**Sinopsis:** `osatura fuzz <project.json> [--samples N] [--seed S] [--strategy random|lhs|halton|sweep|corners|pca] [--frame F] [--report out.json] [--heatmap out.svg] [--minimize] [--gate] [--max-fail-rate R] [--props angle,scale,offx,offy] [--only BONES] [--margin DEG] [--db posedb.json] [--ignore KINDS] [--no-deformers]`

**Pose fuzzing**: eșantionează poze în limitele oaselor și raportează unde se defectează rig-ul (pliuri, întinderi, vârfuri,
autointersecție, IK de neatins, încălcări de limite, NaN, în afara canvasului). `--strategy` alege eșantionatorul (`pca` eșantionează de-a lungul axelor PCA ale pozelor cheie ale proiectului sau ale bazei de date de poze
indicate cu `--db`), `--minimize` reduce o poză care eșuează la un reproducător minim, `--heatmap` scrie o hartă de solicitare,
`--ignore` omite tipurile de eșec, `--only` restrânge la anumite oase. Fără gate, `fuzz` este un **raport**: afișează rezumatul și iese cu 0 (`(report only: exit 0; add --gate, ...)`). `--gate` eșuează (cod de ieșire 1) când orice eșantion eșuează; `--max-fail-rate R` (o **fracție** 0..1) implică verificarea la acea rată. (Verificarea `fuzz` proprie a lui `ci` are propria setare `gate` și nu este afectată.)
```
$ osatura fuzz small.json --samples 30 --seed 1 --strategy lhs --report fz.json --heatmap hm.svg --max-fail-rate 1.0
wrote fz.json
wrote hm.svg
fuzz 'small': 30 samples (lhs, seed 1, frame 0), 25 parameters, 30 failures (100.0%), max severity 54.62, baseline 0.00, 0.40 ms/sample
  ik_unreachable: 27
  off_canvas: 1
  stretch: 2
$ osatura fuzz small.json --samples 10 --max-fail-rate 0.5
fuzz: failure rate 100.00% exceeds --max-fail-rate 50.00%                   [stderr, exit 1]
```
Ținta IK a demo-ului poate fi atinsă în mod deliberat doar într-un interval mic, deci aproape fiecare poză aleatorie este „de neatins":
tratează rata ca pe un număr de regresie, nu ca pe un criteriu trecut/eșuat pentru acest rig. Raportul și heatmap-ul sunt scrise atomic, iar o cale care este o legătură simbolică este refuzată (în afară de cazul în care se folosește `--follow-symlinks`); folderul părinte trebuie să existe.

### `fuzz-grid`

**Sinopsis:** `osatura fuzz-grid <project.json> --x PARAM --y PARAM [--res N] out.svg|out.png|out.json`

O hartă de severitate a doi parametri (`bone:NAME:PROP`, de exemplu `bone:arm_up:angle`) pe o grilă `N x N` (implicit 32).
Tipul ieșirii urmează extensia.
```
$ osatura fuzz-grid small.json --x bone:arm_up:angle --y bone:arm_low:angle --res 8 grid.svg
wrote grid.svg (8x8 cells, 59.4% failing; x = bone:arm_up:angle, y = bone:arm_low:angle)
```
Aceeași rezervă privind scrierea simplă ca la `fuzz`.

## 15. Controlul versiunilor (`vcs`)

**Sinopsis:**
```
osatura vcs init <project.json> [--author A]
osatura vcs commit [-m MSG | --message MSG] [--author A] [--repo DIR]
osatura vcs log [REV] [--limit N] [--json] [--repo DIR]
osatura vcs status [--json] [--repo DIR]
osatura vcs diff [A [B]] [--json] [--repo DIR]
osatura vcs show [REV] [--json] [--repo DIR]
osatura vcs branch [NAME [AT] | -d NAME] [--repo DIR]
osatura vcs tag [NAME [AT]] [--repo DIR]
osatura vcs checkout REV [--force] [--repo DIR]
osatura vcs merge REV [--no-ff] [-m MSG | --message MSG] [--author A] [--abort] [--repo DIR]
osatura vcs cherry-pick|revert REV [--author A] [--repo DIR]
osatura vcs resolve PATH [--ours] [--theirs] [--value V] [--repo DIR]
osatura vcs fsck|gc [--repo DIR]
```
Opțiunile depind de subcomandă: `--json` doar la `log`, `status`, `diff` și `show`; `--force` doar la `checkout`; `--ours`, `--theirs`, `--value` (exact una) doar la `resolve`.

Un mic istoric adresat prin conținut pentru **un singur fișier de proiect**, cu **îmbinare semantică**: modificările sunt comparate ca
căi de document (JSON pointers), nu ca linii de text, deci două persoane care editează chei diferite nu intră niciodată în conflict. `init` creează
un folder `.osatura-vcs/` lângă proiect; ramura implicită este `main`. Reviziile (`REV`) sunt nume de ramuri, nume de etichete
sau prefixe de id-uri de commit.

```
$ osatura vcs init p.json
initialised ./.osatura-vcs for p.json (no commits yet; run: osatura vcs commit -m "...")
$ osatura vcs commit -m first --author alexio
[5f80a63179] first
$ (edit p.json)
$ osatura vcs status
on branch main
HEAD 5f80a63179
uncommitted changes:
  project: background changed "#f6f1e7" -> "#ffffff"
$ osatura vcs commit -m "white bg"
[29c09d9368] white bg
$ osatura vcs branch alt          # branch alt at 29c09d9368
$ osatura vcs checkout alt        # checked out alt (29c09d9368)
$ osatura vcs log --limit 5
4e21f15940  unknown  30 fps
29c09d9368  unknown  white bg
5f80a63179  alexio  first
$ osatura vcs merge alt           # fast-forward or a merge commit; --no-ff forces a merge commit
merge committed as f08db3f77f
$ osatura vcs tag v1 ; osatura vcs tag
tag v1 at 4e21f15940
v1  4e21f15940
$ osatura vcs fsck
ok: 50 objects, history and refs are consistent
$ osatura vcs gc
gc: kept 50 objects, removed 0 unreachable
```

**Conflicte.** Când ambele părți au modificat aceeași valoare, `merge` se oprește, păstrează partea *noastră* în fișierul de proiect, listează
conflictele și iese cu 1:
```
$ osatura vcs merge main
merge stopped with 1 conflict(s); the project file holds our side for each:
  CONFLICT /background
      project: background: ours "#222222" vs theirs "#333333" (base "#000000")
resolve each with: osatura vcs resolve <path> --ours|--theirs|--value <json>, then: osatura vcs commit   (or: osatura vcs merge --abort)
$ osatura vcs resolve background --theirs
no conflict at 'background' (see 'vcs status')                              [exit 1]   (the path needs its leading slash)
$ osatura vcs resolve /background --theirs
resolved /background; 0 conflict(s) left - run: osatura vcs commit
$ osatura vcs commit -m merged
[bd405f446a] merged
$ osatura vcs merge --abort       # alternatively: merge aborted; project restored to HEAD
```
Cât timp un merge, cherry-pick sau revert este nerezolvat, `checkout`, `revert` și `commit` sunt refuzate cu
`a merge / cherry-pick / revert is in progress: resolve the conflicts and commit, or run 'vcs merge --abort'`.
Alte detalii: `diff` fără argument compară fișierul de lucru cu HEAD, `diff A B` compară două revizii (`--json` afișează o listă
de `{kind,path,description}`); `show` afișează un commit și lista lui de modificări; `branch` fără argument listează ramurile (`*` o marchează
pe cea curentă); `branch -d NAME` refuză să șteargă ramura aflată în checkout; `checkout --force` renunță la
modificările necomise; `revert REV` și `cherry-pick REV` creează un commit nou (sau se opresc cu conflicte, ca `merge`); `--repo DIR`
selectează un alt folder de depozit decât cel curent; `--author` este implicit `unknown`; `log --json` afișează obiectele
de commit. **Limite:** un fișier per depozit, doar local (fără remote-uri, fără push/pull), fără urmărirea redenumirilor dincolo de căile JSON;
resursele binare referite de proiect nu sunt versionate.

## 16. Studio, colaborare, serverul agent MCP și registrul UI

### `studio`

**Sinopsis:** `osatura studio <project.json> [--port N] [--ui-config F] [--bind ADDR] [--allow-host NAMES] [--public-origin URL] [--source-url URL] [--collab-token T (16+ characters)] [--collab-secret-file F [--collab-revoke NAMES]] [--collab-rate N] [--collab-log LOG [--collab-nosync]]`

Pornește **editorul din browser** (anulare/refacere, timeline, panouri live și facial, VCS și operații în stil agent) ca server web
local și rămâne blocat până la Ctrl-C. Valori implicite: `--port 8080`, `--bind 127.0.0.1`. Proiectul este validat la pornire; salvările merg
în același fișier (`POST /api/save`).
```
$ osatura studio demo.json --port 18931 --ui-config ui.json
AGPL-3.0 source offer: http://127.0.0.1:18931/source
OSATURA studio on http://127.0.0.1:18931  (Ctrl-C to stop; saves go to demo.json)
UI settings (layout, shortcuts) in ui.json
```
Opțiuni: `--ui-config F` păstrează layout-ul și scurtăturile (implicit `~/.config/osatura/ui.json`, pe Windows `%APPDATA%\OSATURA\ui.json`); `--source-url URL` stabilește
linkul către sursa publicată afișat la `/source` și de `--version` (AGPL-3.0 secțiunea 13; binarele oficiale de lansare folosesc implicit commit-ul din care au fost construite); `--allow-host NAMES` și `--public-origin URL` acceptă
cereri sub un nume public, în spatele unui reverse proxy TLS; `--collab-token T` (cel puțin 16 caractere, de exemplu
`openssl rand -hex 16`) activează sesiunea multiutilizator și este **obligatoriu pentru orice `--bind` non-loopback**; `--collab-nosync` este valid doar împreună cu `--collab-log` (altfel, eroare); `--collab-log LOG`
păstrează sesiunea partajată și o reia după o repornire (`--collab-nosync` omite sincronizarea la pornire).

**Opțiuni de securizare** (valul 7; detalii și modelul de amenințări în [SECURITY.md](SECURITY.md)): `--collab-secret-file F` înlocuiește
singurul bearer partajat cu **token-uri pentru fiecare participant**: F conține un secret aleatoriu (prima linie, cel puțin 16 caractere, mod 600 pe
Unix, altfel este refuzat) și fiecare persoană primește un token de la `osatura collab-token NAME`; un token este legat de un singur nume
(la alăturare se folosește acel nume, nimeni nu se poate alătura sub numele altcuiva), expiră și este verificat prin recalcularea HMAC-SHA-256 al său, astfel încât
serverul nu ține niciun tabel. `--collab-revoke NAMES` (separate prin virgulă, necesită `--collab-secret-file`) refuză token-urile acelor
nume înainte de expirare. `--collab-token` și `--collab-secret-file` pot fi combinate (token-ul partajat acționează atunci ca
cheie de administrator); oricare dintre ele satisface regula conform căreia un bind non-loopback necesită o credențială. `--collab-rate N` limitează fiecare adresă de
client la N cereri pe secundă (rafale de 2N; răspuns `429` cu `Retry-After`); valoarea implicită este 100 când se cere o credențială
și nicio limită în caz contrar; 0 o dezactivează. Un bind non-loopback afișează avertismente (HTTP simplu; toate interfețele pentru
`0.0.0.0` / `::`; un token partajat nu are nume și nu expiră).

**Reguli de siguranță** (verificate): serverul răspunde doar cererilor al căror `Host` este unul dintre propriile nume
(`curl -H 'Host: evil.example' .../api/state` dă 421); `Origin` și `Sec-Fetch-Site` de la o altă pagină dau 403; corpul unui POST
trebuie să fie `application/json` (`curl -X POST -d '{}'` dă 415). Un token scurt este refuzat
(`--collab-token must be at least 16 characters`), iar `--bind 0.0.0.0` fără token este refuzat
(`refusing to listen on 0.0.0.0:PORT: the studio has no accounts and no TLS, ...`). Serverul nu are conturi și nu are TLS: pune
un proxy TLS în față pentru orice altceva decât propria ta mașină; când `--public-origin` este un URL `https://`, cookie-ul de autentificare primește atributul `Secure` pentru cererile care sosesc sub acel nume (sau cu `Origin` https / `X-Forwarded-Proto: https`). Pagina servește un mic favicon SVG la `/favicon.ico`. API-ul HTTP (de exemplu `GET /api/state`, `GET /api/project`,
`POST /api/op`, `POST /api/save`, `GET /frame/N.svg`) există pentru pagina însăși și nu este un API public stabil.

### `collab-token`

**Sinopsis:** `osatura collab-token <name> [--secret-file F] [--ttl 28800]`

Afișează un token de colaborare pentru fiecare participant, pentru un studio pornit cu `--collab-secret-file F`: `ap1.<name>.<expires>.<mac>`,
unde `mac` este HMAC-SHA-256 în hexazecimal al numelui și al termenului limită, sub secret. `<name>` are între 1 și 32 de caractere: litere,
cifre, `_` și `-`. `--ttl` este durata de viață în secunde (implicit 28800 = 8 ore, cel mult 2592000 = 30 de zile). Secretul este citit din
fișierul `F` (prima linie) sau, fără `--secret-file`, din variabila de mediu `OSATURA_COLLAB_SECRET`; nu este niciodată
acceptat ca argument, unde alți utilizatori l-ar putea vedea. Token-ul merge pe stdout, termenul limită pe stderr.

```
$ openssl rand -hex 32 > collab.secret && chmod 600 collab.secret
$ osatura studio demo.json --collab-secret-file collab.secret
$ osatura collab-token ana --secret-file collab.secret --ttl 3600
ap1.ana.1790000000.3f0c...
token for 'ana', valid until Unix time 1790000000 (in 3600 s); give it to that person only; ...
$ osatura collab-token ana --secret-file collab.secret --ttl 0
--ttl needs a whole number of seconds (1 to 2592000)
```
Persoana lipește token-ul pe pagina de autentificare a studioului (el este apoi păstrat într-un cookie HttpOnly care expiră odată cu token-ul) sau îl trimite ca
`X-Collab-Token`. Un token expirat primește răspuns `401`, cu termenul limită în mesaj; numele participantului este preluat din
token. **Conexe:** `studio`.

### `collab-sim`

**Sinopsis:** `osatura collab-sim <project.json> [--clients N] [--ops M] [--seed S] [--undo-pct P] [--replays K] [--require-valid] [--raw] [--json]`

Un banc de testare pentru motorul de colaborare: N replici editează proiectul concurent, cu ordine de livrare aleatorie și
duplicate, apoi verifică dacă toate dețin același document (cod de ieșire 1 dacă nu). Editările implicite sunt editări valide ale proiectului
(numere, valori de chei, vizibilitate, straturi proprii); `--raw` folosește editări JSON arbitrare (de obicei nu un proiect valid);
`--require-valid` face ca un rezultat care nu se compilează să ducă și el la cod de ieșire 1.
```
$ osatura collab-sim small.json --clients 3 --ops 20 --seed 2 --json
{"buffered":12,"clients":3,"clocks_identical":true,"converged":true,"converged_ok":true,"deliveries":34,"duplicates_dropped":0,"edits":20,"envelopes":17,...}
```
`converged_ok` înseamnă doar convergența; `ok` cere în plus ca documentul îmbinat să se compileze.

### `mcp`

**Sinopsis:** `osatura mcp <project.json>`

Un **server pentru agenți**: JSON-RPC 2.0 (Model Context Protocol) peste stdin/stdout, câte un mesaj JSON pe linie. Încarcă
proiectul și expune uneltele `get_project`, `stats`, `lint`, `coach`, `safety`, `apply_ops`, `set_key`, `undo`, `redo`,
`render_frame` și `save`. Nu se scrie nimic până nu se apelează `save`. Stdout transportă doar mesaje de protocol.
```
$ printf '%s\n' \
 '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}' \
 '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
 '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
 '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"lint","arguments":{}}}' | osatura mcp demo.json
{"id":1,"jsonrpc":"2.0","result":{"capabilities":{"tools":{}},"protocolVersion":"2024-11-05","serverInfo":{"name":"osatura","version":"0.1.0"}}}
{"id":2,"jsonrpc":"2.0","result":{"tools":[{"description":"Return the project JSON, or the value at a JSON pointer ...","name":"get_project",...}]}}
{"id":3,"jsonrpc":"2.0","result":{"content":[{"text":"[]","type":"text"}],"isError":false}}
```
Serverul se oprește (cod 0) când stdin se închide.

### `ui`

**Sinopsis:**
```
osatura ui commands|shortcuts [--ui-config F] [--json]
osatura ui layout presets|show <name>|validate <file.json>
osatura ui search <query...> [--ui-config F]
```
**Registrul de comenzi** al studioului: `commands` listează fiecare comandă cu categoria, titlul și combinațiile de taste;
`shortcuts` listează scurtăturile efective (cele implicite plus cele din `--ui-config`) și eventualele conflicte; `layout presets` listează
presetările de layout de dock, `layout show NAME` afișează una ca JSON, iar `layout validate FILE` verifică un fișier de layout;
`search` ordonează comenzile așa cum o face paleta de comenzi.
```
$ osatura ui commands
Edit
  edit.undo                    Undo                               global:Ctrl+Z
  edit.redo                    Redo                               global:Ctrl+Shift+Z  global:Ctrl+Y
$ osatura ui layout presets
animate  V0.62(H0.74([*stage] | [*layers selection bones live face json]) | [*timeline])  [valid]
rig      H0.66(V0.84([*stage] | [*timeline (folded)]) | V0.50([*layers bones] | [*selection json]))  hidden: live,face  [valid]
compact  V0.62([*stage] | [*timeline layers selection bones live face json])  [valid]
live     H0.58(V0.70([*stage] | [*timeline]) | [*live face bones selection])  hidden: layers,json  [valid]
$ osatura ui layout validate demo.json
demo.json: unknown field `format`, expected one of `version`, `root`, `floating`, `hidden` at line 2 column 10   [exit 1]
```

## 17. Bază de date de poze, sugestii și alte ajutoare

Aceste comenzi construiesc o **bază de date de poze** (rotațiile oaselor la cadrele cheie ale unui proiect și ale fiecărui take dintr-o bibliotecă)
și folosesc structura învățată a acesteia (axe PCA, opțional o rețea mică) pentru a sugera poze. Este un ajutor statistic antrenat pe
pozele *tale*, nu un model general. Fișierele lor de model și de raport sunt scrise cu o scriere simplă.

### `pose-db` (build, info, train-mlp)

**Sinopsis:**
```
osatura pose-db build <out.json> [--project P.json] [--library takes.json] [--step N]
osatura pose-db info  <db.json>
osatura pose-db train-mlp <db.json> <out.json|out.bin> [--hidden 64,64] [--epochs N] [--lr R] [--seed S]
```
```
$ osatura pose-db build db.json --project demo.json
wrote db.json: 5 poses of 3 bones in 1 sequence(s)
$ osatura pose-db info db.json
5 poses, 3 bones: arm_up, arm_low, hand
  sequence 0: project (project, 5 poses)
PCA: 2 axes keep 99.2% of the variance (total 950.0 deg^2)
  axis 1: std 27.34 deg, cumulative 78.7%
  axis 2: std 13.95 deg, cumulative 99.2%
typical nearest-pose spacing 14.43 deg, typical step between consecutive poses 35.82 deg
$ osatura pose-db train-mlp db.json mlp.json --hidden 8 --epochs 20 --seed 1
wrote mlp.json: 6 -> [8] -> 3 (epochs 20, loss 1.0189 -> 0.4921)
trained on the library itself (no held-out data here); run pose-bench for held-out accuracy
```
`--project` adaugă pozele de la cadrele cheie ale proiectului, `--library` take-urile unei biblioteci de take-uri (`take-record`, secțiunea 6),
`--step N` eșantionează suplimentar la fiecare N cadre. Rețeaua mică (SGD simplu) este opțională; folosește-o cu `--model`, mai jos.

### `pose-suggest` (between, complete, next)

**Sinopsis:**
```
osatura pose-suggest between  <project.json> <db.json|-> --a F1 --b F2 [--t 0.5] [--snap 0.5] [--top 5] [--apply RANK --out out.json] [--json]
osatura pose-suggest complete <project.json> <db.json|-> --frame F [--fix bone=angle,...] [--reach bone=x,y,...] [--free b1,b2] [--prior 1] [--model m.json] [--apply RANK --out out.json]
osatura pose-suggest next     <project.json> <db.json|-> --frame F [--k 8]
```
`-` în loc de o bază de date învață din proiectul însuși. `between` propune o poză intermediară între două cadre, care rămâne
pe varietatea învățată (candidați `linear`, `pca`, `manifold`); `complete` găsește cea mai plauzibilă poză completă, date fiind unghiurile
fixe (`--fix`), țintele vârfurilor (`--reach`) și oasele libere; `next` arată ce a urmat în bibliotecă după poze asemănătoare cu cea de la
cadru. `--apply RANK` (împreună cu `--out`) scrie candidatul ales sub formă de chei.
```
$ osatura pose-suggest between demo.json db.json --a 0 --b 24 --t 0.5 --top 3
between: 5 library poses, 2 PCA axes (99% of the variance)
 1. linear      score 1.00  plausibility 1.00  key at frame 12.0
      linear blend of the angles
      arm_up=0.0 arm_low=-7.5 hand=-10.0
 3. manifold    score 1.00  plausibility 1.00  key at frame 12.0
      PCA blend pulled toward the library
$ osatura pose-suggest complete demo.json db.json --frame 12 --fix arm_up=-40
 1. pca-ls      score 0.96  plausibility 1.00  constraint error 0.18  key at frame 12.0
      regularised least squares in PCA space (start 1)
      arm_up=-39.8 arm_low=24.9 hand=11.2
 2. library-nn  score 0.25  plausibility 1.00  constraint error 15.00  key at frame 12.0
(scores rank candidates against this library; they are heuristics, not probabilities)
$ osatura pose-suggest next demo.json db.json --frame 12 --k 3
 1. successor   score 0.60  plausibility 1.00  key at frame 24.0
      what followed pose 1 of 'project'
```

### `pose-bench`

**Sinopsis:** `osatura pose-bench`

Afișează acuratețea pe date rezervate a metodelor de sugerare, pe o **bibliotecă sintetică de tip mers** (24 de cicluri de antrenare, 6
rezervate; completarea vede 3 din 10 oase; antrenează și rețeaua mică). Cu cât mai mic, cu atât mai bine. Nu citește niciun proiect. Este
**lentă într-un build de debug** (minute); numere dintr-un build de release:
```
completion  mean pose                                   20.96 deg RMS
completion  nearest library pose                         1.77 deg RMS
completion  PCA least squares                            0.51 deg RMS
completion  MLP                                          1.64 deg RMS
between     linear blend                                 1.83 deg RMS
next        Markov mean                                  0.72 deg RMS
```

### `pose-match`

**Sinopsis:** `osatura pose-match <project.json> <params.json> <goals.json> <out.json> [--generations N --seed N]`

Caută (un optimizator evolutiv, implicit 80 de generații, seed 1, interval valid 1..100000) valori ale unui set de
parametri care aduc vârfurile oaselor în pozițiile-țintă din spațiul scenei. `params.json` este o listă de
`{"target","prop","min","max","start"}`; `goals.json` este o listă de `{"bone","at":[x,y],"weight"}`. Rezultatul este adăugat sub formă de
canale; pierderea este suma ponderată a pătratelor erorilor vârfurilor, în px².
```
$ echo '[{"target":"arm_up","prop":"bone_angle","min":-60,"max":40,"start":0},{"target":"arm_low","prop":"bone_angle","min":-60,"max":60,"start":0}]' > params.json
$ echo '[{"bone":"hand","at":[260,200],"weight":1.0}]' > goals.json
$ osatura pose-match demo.json params.json goals.json out.json --generations 30 --seed 3
wrote out.json (loss 3756.156 px²)
```
**Capcană:** pierderea din exemplu este mare pentru că intervalele nu pot atinge ținta: verifică numărul, nu doar
codul de ieșire (se scrie un rezultat chiar și atunci când potrivirea este slabă).
## Neconcordanțe cunoscute între documentație și comportament

Acolo unde textul de utilizare al programului (afișat de `osatura` fără argumente) sau un document vecin spune
un lucru, iar programul face altceva, secțiunile de mai sus descriu ce face programul cu adevărat. Diferențele găsite
la verificarea fiecărui exemplu, în ordinea probabilității de a crea probleme:

1. **Remediat - flaguri necunoscute.** Orice comandă cu o linie de utilizare respinge acum `--words` nedocumentate (cu un indiciu „ai vrut să spui"),
   inclusiv `check`, `stats`, `lint`, `coach`, `contracts` și `machine list`. `--json`, `--quiet` și `--allow-empty` sunt
   specifice fiecărei comenzi și sunt refuzate în alte părți (o modificare incompatibilă pentru scripturile care le transmiteau peste tot). Începând cu wave 5, verificarea se face **pe subcomandă** (`ci diff ... --quiet`, `trail <bone> ... --json`, `ui layout presets --json`, `golden write --tol` sunt refuzate) și acoperă și opțiunile cu valoare. Teste: `osatura-cli/tests/doc_mismatch.rs` (matricea de acceptare) și `options_are_validated_per_sub_command` în `main.rs`.
2. **Remediat - ieșiri cu `fs::write` simplu.** `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, `face-*`, fișierele de poze, snapshot-urile,
   `vcs` și jurnalul de colaborare folosesc scriitorul atomic care refuză legăturile simbolice; ieșirile ffmpeg sunt verificate în prealabil. Un test condus de tabel
   acoperă aproximativ 24 de comenzi. Neacoperite cap-coadă: `face-train`/`face-eval` (test unitar doar pe scriitorul lor).
3. **Remediat - `golden check`** este exact în mod implicit; `--tol` îl relaxează; afișează numărul de diferențe și cea mai mare diferență.
   Afirmația „o schimbare a culorii de fundal trece" a fost un artefact al demo-ului (stratul cerului acoperă fundalul), nu o eroare.
4. **Remediat - codul de ieșire al `fuzz`.** Implicit este doar raport (cod 0); `--gate` sau `--max-fail-rate` activează verificarea. Vezi `fuzz`.
5. **Remediat - `dof-export`** scrie `.mp4` și `.webm` (prin ffmpeg, cu `--codec`, fără audio); textul de utilizare le listează acum. Un nume
   cu orice altă extensie este refuzat și nu se scrie nimic.
6. **Remediat - `export` `.png`** scrie `<name>.atlas.json`, niciodată `<name>.json`; eroarea de format necunoscut listează fiecare extensie.
7. **Valori implicite pe care textul de utilizare nu le menționează:** `studio` ascultă pe `127.0.0.1:8080`; `fuzz-grid --res` este 32; `export-fit`
   eșantionează 8 cadre, caută cu `descent` și se oprește după 60 s (`--max-seconds 0` = nelimitat); `reduced-motion --factor`
   este 0,25; `size-report --tol` este 0,05; `pose-match` rulează 80 de generații cu seed 1; `lod --out-dir` este folderul
   curent.
8. **`machine remove` poate refuza** (`binding 'goBind' still drives machine 'm'`); textul de utilizare nu menționează
   verificarea dependenței. `machine` și `state-bake` citesc două formate de mașină diferite (`layers`/`inputs` față de
   `initial`/`states`); linia de utilizare a `state-bake` nu are descriere și nu indică formatul.
9. **Remediat - `stats`** nu mai acceptă `--json` (ieșirea este întotdeauna JSON; linia de utilizare o spune).
10. **`vcs`:** ramura implicită este `main` (`checkout master` eșuează cu `unknown revision 'master'`); `vcs resolve`
    primește calea conflictului *cu* bara oblică inițială, exact cum o afișează `merge` (`/background`), iar `background` singur dă
    `no conflict at 'background'`; textul de utilizare scrie „see: osatura vcs", dar `osatura vcs` fără argumente afișează o
    utilizare pe o singură linie și iese cu 1.
11. **Linii de utilizare fără descriere.** `state-bake`, `pose-match`, `pose-bench`, `follow`, `keys`, `event`, `marker`,
    `idle`, `mouth`, `beats`, `export`, `frames`, `onion` și altele au o sinopsis, dar nicio explicație, sau o descriere care
    se află pe o linie de continuare; comportamentul lor din acest document provine din rularea lor.
12. **Denumirea proprietăților.** Mesajele și ieșirile `list` afișează proprietățile în CamelCase (`LayerOpacity`, `BoneAngle`),
    dar fiecare opțiune, fișier de mapare și fișier auxiliar primește numele în snake_case (`layer_opacity`, `bone_angle`); forma CamelCase
    este respinsă (`bind add ... sun:LayerOpacity` eșuează cu `unknown property 'LayerOpacity'`).
13. **`cloth`:** textul de utilizare vorbește despre „per-vertex deformation"; rezultatul copt este stocat ca
    canale `lattice_dx` / `lattice_dy` pe punctele rețelei pânzei (pentru grila implicită 7x7, cele 42 de puncte nefixate), vezi secțiunea 7.
14. **`render-gpu`** există doar în build-urile făcute cu `--features gpu`; nu a putut fi rulat în build-ul de test, deci opțiunile sale
    sunt preluate din textul de utilizare și nu sunt verificate aici.
15. **Lent în build-urile de debug.** `pose-bench`, `face-train`, `ci run` și `export-fit` durează minute într-un build neoptimizat.
    Duratele din acest document provin din binarul de debug și sunt mult mai scurte într-un build de release.
16. **Remediat (wave 5) - `golden write --scale`** peste 1 era limitat silențios la 1; acum este refuzat (scara este în (0, 1]).
    `golden check` ignora un `--scale` și folosea scara stocată; acum refuză opțiunea.
17. **Remediat (wave 5) - sinopsisuri.** Fiecare opțiune din textul de utilizare al unei comenzi se află în sinopsisul ei de aici și invers (testul
    `cli_reference_synopses_match_usage`), iar fiecare linie de utilizare fără alternative este repetată aici cuvânt cu cuvânt (testul
    `cli_reference_repeats_usage_lines_verbatim`). `render-gpu --features gpu` și `layer-normal ... relight --layer-normals`
    din descrierile de utilizare denumesc o funcționalitate de build și opțiunea altei comenzi, nu opțiuni ale acelor comenzi.
18. **Alte documente sunt mai vechi decât programul în două locuri** (vezi [EXPORT_FORMATS.md](EXPORT_FORMATS.md) secțiunea 17):
    `docs/LIMITATIONS.md` L-29 spune că importul Lottie nu preia imagini, efecte, precompoziții sau mattes (importatorul le tratează acum,
    cu avertismentele listate în secțiunea 9), iar L-32 listează OCA printre formatele de import, deși CLI-ul are doar
    `export-oca`.
## Index alfabetic al comenzilor

Fiecare comandă, cu un rezumat de o linie și secțiunea care o documentează. Subcomenzile (`machine add`, `vcs commit`, ...) se află în secțiunea comenzii lor părinte.

| Comandă | Rezumat | Secțiune |
|---|---|---|
| `--version` | afișează versiunea, licența, oferta de sursă și linkurile de suport | [1](#1-convenții-comune-tuturor-comenzilor) |
| `align` | aliniază straturile între ele sau față de scenă | [5](#5-rigging-skin-uri-ik-și-ajutoare-de-layout) |
| `audio-react` | volumul sonor comandă o proprietate | [8](#8-audio-lip-sync-și-sunete) |
| `autorig` | IK, respirație și clipit configurate pornind de la numele oaselor și ale straturilor | [5](#5-rigging-skin-uri-ik-și-ajutoare-de-layout) |
| `autotrace` | vectorizare locală a bitmap-urilor (benzi de culoare, potrivire Bezier) | [9](#9-import) |
| `autotrace-bench` | tabel de fidelitate al vectorizatorului pe forme sintetice | [9](#9-import) |
| `beats` | detectează salturile de volum și înregistrează markere sau evenimente | [8](#8-audio-lip-sync-și-sunete) |
| `bind` | legări de date: add, list, remove | [11](#11-mașini-de-stări-legări-de-date-containere-și-regiuni-de-derulare) |
| `bind-eval` | evaluează legările pentru valori date | [11](#11-mașini-de-stări-legări-de-date-containere-și-regiuni-de-derulare) |
| `bone-vis` | afișează un os doar în overlay-uri, pentru un interval de cadre | [5](#5-rigging-skin-uri-ik-și-ajutoare-de-layout) |
| `cage` | adaugă un deformator cușcă cu coordonate de valoare medie | [7](#7-fizică-cloth-și-deformatoare) |
| `cage-pose` | pune chei pe decalajele vârfurilor cuștii la un cadru | [7](#7-fizică-cloth-și-deformatoare) |
| `camera-seq` | pune chei pe cameră prin shot-urile salvate | [6](#6-animație-chei-timeline-și-generatoare) |
| `chan-meta` | culoarea canalului și flagurile shy, solo și lock | [5](#5-rigging-skin-uri-ik-și-ajutoare-de-layout) |
| `check` | validează un proiect (cod de ieșire 1 dacă este nevalid) | [3](#3-crearea-și-inspecția-proiectelor) |
| `ci` | CI pentru animație: golden-uri, diff perceptual, verificări structurale | [14](#14-asigurarea-calității) |
| `clip` | așază un take în stiva de clipuri | [6](#6-animație-chei-timeline-și-generatoare) |
| `cloth` | coace o simulare deterministă de pânză | [7](#7-fizică-cloth-și-deformatoare) |
| `coach` | sugestii de timing și de principii | [14](#14-asigurarea-calității) |
| `collab-sim` | test de convergență al colaborării | [16](#16-studio-colaborare-serverul-agent-mcp-și-registrul-ui) |
| `collab-token` | emite un token de colaborare pentru fiecare participant (HMAC-SHA-256, expiră) | [16](#16-studio-colaborare-serverul-agent-mcp-și-registrul-ui) |
| `color-space` | alege compunerea sRGB sau în lumină liniară pentru proiect | [12](#12-efecte-profunzime-de-câmp-onion-skin-urme-și-cameră) |
| `comp` | stări denumite ale straturilor (vizibilitate, opacitate, alegerea switch-ului) | [5](#5-rigging-skin-uri-ik-și-ajutoare-de-layout) |
| `compare` | imagine înainte / după / diferență a două proiecte | [4](#4-randare-și-export) |
| `container` | containere de layout: add, list, remove, solve | [11](#11-mașini-de-stări-legări-de-date-containere-și-regiuni-de-derulare) |
| `contracts` | contracte de mișcare (teste unitare pentru animație) | [14](#14-asigurarea-calității) |
| `demo` | scrie proiectul demo de rig încorporat | [3](#3-crearea-și-inspecția-proiectelor) |
| `demo-fx` | scrie demo-ul cu lichid / boolean / particule | [3](#3-crearea-și-inspecția-proiectelor) |
| `diff` | diferențe semantice între două proiecte | [3](#3-crearea-și-inspecția-proiectelor) |
| `distribute` | distribuie uniform trei sau mai multe straturi | [5](#5-rigging-skin-uri-ik-și-ajutoare-de-layout) |
| `dof` | randare cu profunzime de câmp a unui cadru | [12](#12-efecte-profunzime-de-câmp-onion-skin-urme-și-cameră) |
| `dof-export` | profunzime de câmp animată (GIF/APNG/video/folder) | [12](#12-efecte-profunzime-de-câmp-onion-skin-urme-și-cameră) |
| `drive` | semnale CSV -> chei prin fișier de mapare | [10](#10-intrare-live-midi-urmărirea-feței-și-motion-capture) |
| `event` | adaugă un eveniment de runtime denumit (cu payload JSON opțional) | [6](#6-animație-chei-timeline-și-generatoare) |
| `explain` | de ce o proprietate are valoarea respectivă: contribuții ordonate | [3](#3-crearea-și-inspecția-proiectelor) |
| `export` | exportă GIF, APNG, WebM/MP4, foaie PNG, HTML, Lottie, .lottie, glTF/GLB, ... | [4](#4-randare-și-export) |
| `export-fit` | cel mai bun export de calitate care încape într-un buget de octeți | [13](#13-nivel-de-detaliu-rapoarte-de-dimensiune-și-export-fit) |
| `export-oca` | scrie un folder Open Cel Animation | [4](#4-randare-și-export) |
| `eye-clip` | maschează straturile de iris/pupilă după forma ochiului | [5](#5-rigging-skin-uri-ik-și-ajutoare-de-layout) |
| `face-bake` | repere -> chei | [10](#10-intrare-live-midi-urmărirea-feței-și-motion-capture) |
| `face-calibrate` | calibrarea feței neutre | [10](#10-intrare-live-midi-urmărirea-feței-și-motion-capture) |
| `face-detect` | casete de față și 23 de repere într-o imagine | [10](#10-intrare-live-midi-urmărirea-feței-și-motion-capture) |
| `face-eval` | raport de acuratețe pe date rezervate al detectorului | [10](#10-intrare-live-midi-urmărirea-feței-și-motion-capture) |
| `face-synth` | animație sintetică de cap | [10](#10-intrare-live-midi-urmărirea-feței-și-motion-capture) |
| `face-track` | NDJSON cu repere din video sau cadre | [10](#10-intrare-live-midi-urmărirea-feței-și-motion-capture) |
| `face-train` | antrenează detectorul de fețe pe fețe sintetice | [10](#10-intrare-live-midi-urmărirea-feței-și-motion-capture) |
| `follow` | generatoare de acțiune suprapusă / follow-through | [6](#6-animație-chei-timeline-și-generatoare) |
| `foot-lock` | fixează un controler în timpul contactului cu solul | [6](#6-animație-chei-timeline-și-generatoare) |
| `frames` | randează fiecare cadru într-un folder (SVG, sau PNG cu --png) | [4](#4-randare-și-export) |
| `fuzz` | găsește pozele în care rig-ul se defectează | [14](#14-asigurarea-calității) |
| `fuzz-grid` | hartă de severitate a doi parametri | [14](#14-asigurarea-calității) |
| `fx` | randează un cadru cu un lanț de post-efecte | [12](#12-efecte-profunzime-de-câmp-onion-skin-urme-și-cameră) |
| `glue` | lipește vârfuri de mesh unele de altele | [7](#7-fizică-cloth-și-deformatoare) |
| `golden` | scrie/verifică cadre de regresie | [14](#14-asigurarea-calității) |
| `idle` | generează un canal idle procedural | [6](#6-animație-chei-timeline-și-generatoare) |
| `import-anim` | GIF/APNG animat -> strat cadru cu cadru | [9](#9-import) |
| `import-bvh` | motion capture BVH -> canale | [9](#9-import) |
| `import-lottie` | subset Lottie -> proiect | [9](#9-import) |
| `import-ora` | OpenRaster -> proiect | [9](#9-import) |
| `import-seq` | secvență PNG -> strat cadru cu cadru | [9](#9-import) |
| `import-svg` | subset SVG -> strat vectorial | [9](#9-import) |
| `keyedit` | editări de dope sheet ca un singur lot sigur la anulare | [6](#6-animație-chei-timeline-și-generatoare) |
| `keys` | operații de masă asupra cheilor din toate canalele cu chei | [6](#6-animație-chei-timeline-și-generatoare) |
| `layer-normal` | atașează o hartă de normale unui strat | [12](#12-efecte-profunzime-de-câmp-onion-skin-urme-și-cameră) |
| `layer-pivot` | mută pivotul unui strat fără a schimba imaginea | [7](#7-fizică-cloth-și-deformatoare) |
| `layout-preview` | randare cu layout-ul rezolvat din nou pentru o altă dimensiune de canvas | [11](#11-mașini-de-stări-legări-de-date-containere-și-regiuni-de-derulare) |
| `lint` | verificări statice de rig/animație (cod de ieșire 1 la erori) | [14](#14-asigurarea-calității) |
| `lipsync` | comandă un strat switch de gură după volumul audio (nu pe bază de foneme) | [8](#8-audio-lip-sync-și-sunete) |
| `lipsync-text` | aliniază o transcriere la audio și deduce formele gurii | [8](#8-audio-lip-sync-și-sunete) |
| `live-map` | intrare live înregistrată (MIDI, mouse, gamepad, taste) -> chei, take-uri, evenimente | [10](#10-intrare-live-midi-urmărirea-feței-și-motion-capture) |
| `lod` | niveluri de detaliu cu limite de eroare declarate în pixeli | [13](#13-nivel-de-detaliu-rapoarte-de-dimensiune-și-export-fit) |
| `lod-pick` | cel mai grosier nivel care încape într-un buget de pixeli pe un ecran | [13](#13-nivel-de-detaliu-rapoarte-de-dimensiune-și-export-fit) |
| `loop-solve` | face canalele cu chei să facă buclă fără cusătură | [6](#6-animație-chei-timeline-și-generatoare) |
| `machine` | mașini de stări de runtime: add, list, remove, validate | [11](#11-mașini-de-stări-legări-de-date-containere-și-regiuni-de-derulare) |
| `machine-sim` | redă intrări temporizate: jurnal de stări plus cadre SVG | [11](#11-mașini-de-stări-legări-de-date-containere-și-regiuni-de-derulare) |
| `magnet` | coace o atracție de proximitate | [6](#6-animație-chei-timeline-și-generatoare) |
| `make-loop` | face un ciclu să facă buclă fără cusătură (opțional pe loc) | [6](#6-animație-chei-timeline-și-generatoare) |
| `marker` | adaugă un marker sau un interval pe timeline | [6](#6-animație-chei-timeline-și-generatoare) |
| `mcp` | server pentru agenți (JSON-RPC peste stdio) | [16](#16-studio-colaborare-serverul-agent-mcp-și-registrul-ui) |
| `mesh-lock` | urmăritorii urmează suprafața unui mesh gazdă | [7](#7-fizică-cloth-și-deformatoare) |
| `mesh-trace` | vectorizează silueta alfa a unei imagini într-un mesh triunghiular cu skin | [7](#7-fizică-cloth-și-deformatoare) |
| `mirror` | oglindește stânga<->dreapta toate canalele cu chei | [6](#6-animație-chei-timeline-și-generatoare) |
| `mocap-info` | ierarhia și întinderea unui fișier BVH | [10](#10-intrare-live-midi-urmărirea-feței-și-motion-capture) |
| `mocap-match` | potrivirea articulațiilor BVH cu oasele rig-ului | [10](#10-intrare-live-midi-urmărirea-feței-și-motion-capture) |
| `mocap-retarget` | coace mocap ca canale adăugate | [10](#10-intrare-live-midi-urmărirea-feței-și-motion-capture) |
| `mouth` | importă indicii de gură (Papagayo/Rhubarb) | [8](#8-audio-lip-sync-și-sunete) |
| `onion` | onion-skin PNG sau SVG | [12](#12-efecte-profunzime-de-câmp-onion-skin-urme-și-cameră) |
| `parallax` | adâncime multiplan pentru straturile de nivel superior | [5](#5-rigging-skin-uri-ik-și-ajutoare-de-layout) |
| `parent-switch` | coace schimbarea animată a părintelui fără salt | [5](#5-rigging-skin-uri-ik-și-ajutoare-de-layout) |
| `physics` | coace o simulare de lanț | [7](#7-fizică-cloth-și-deformatoare) |
| `pick` | ce se află sub un punct al canvasului (cel mai de sus primul) | [3](#3-crearea-și-inspecția-proiectelor) |
| `pose-bench` | acuratețea pe date rezervate a metodelor de poze | [17](#17-bază-de-date-de-poze-sugestii-și-alte-ajutoare) |
| `pose-db` | bază de date de poze: build, info, train-mlp | [17](#17-bază-de-date-de-poze-sugestii-și-alte-ajutoare) |
| `pose-match` | potrivește parametri astfel încât vârfurile oaselor să atingă ținte | [17](#17-bază-de-date-de-poze-sugestii-și-alte-ajutoare) |
| `pose-suggest` | sugestii de poză intermediară, de completare și de poză următoare | [17](#17-bază-de-date-de-poze-sugestii-și-alte-ajutoare) |
| `proxy-bench` | milisecunde pe cadru la fiecare nivel de proxy | [4](#4-randare-și-export) |
| `reduced-motion` | scrie o copie mai calmă a unui proiect | [14](#14-asigurarea-calității) |
| `reference` | marchează un strat ca referință de calcare (nu este exportat niciodată) | [5](#5-rigging-skin-uri-ik-și-ajutoare-de-layout) |
| `render` | randează un cadru în PNG sau SVG (proxy-uri de scrubbing, oase, scară) | [4](#4-randare-și-export) |
| `render-gpu` | randare GPU (doar build-uri wgpu) | [4](#4-randare-și-export) |
| `render-hdr` | randare în virgulă mobilă în OpenEXR sau PNG pe 16 biți | [4](#4-randare-și-export) |
| `reparent` | schimbă părintele unui os păstrând poza de repaus | [5](#5-rigging-skin-uri-ik-și-ajutoare-de-layout) |
| `retarget` | copiază animația între rig-uri pe perechi de lanțuri | [5](#5-rigging-skin-uri-ik-și-ajutoare-de-layout) |
| `safety` | raport despre flash-uri / flash-uri roșii / mișcarea camerei | [14](#14-asigurarea-calității) |
| `script` | rulează un script Rhai în sandbox care editează proiectul | [6](#6-animație-chei-timeline-și-generatoare) |
| `scroll` | regiuni de derulare: add, list, remove, eval | [11](#11-mașini-de-stări-legări-de-date-containere-și-regiuni-de-derulare) |
| `select-box` | selecție prin încadrare | [3](#3-crearea-și-inspecția-proiectelor) |
| `shot` | salvează un shot de cameră | [6](#6-animație-chei-timeline-și-generatoare) |
| `size-report` | unde se află octeții unui proiect/export | [13](#13-nivel-de-detaliu-rapoarte-de-dimensiune-și-export-fit) |
| `skin` | skin-uri denumite: seturi de straturi, oase doar pentru skin, constrângeri, lanțuri IK | [5](#5-rigging-skin-uri-ik-și-ajutoare-de-layout) |
| `skin-diffuse` | greutăți de skin prin difuzie termică | [7](#7-fizică-cloth-și-deformatoare) |
| `slider` | creează un os controler care amestecă poze stocate | [5](#5-rigging-skin-uri-ik-și-ajutoare-de-layout) |
| `snapshot` | istoric numerotat de revizii al unui proiect într-un folder (save, list, restore) | [3](#3-crearea-și-inspecția-proiectelor) |
| `soft-scale` | scalare proporțională a unei selecții | [7](#7-fizică-cloth-și-deformatoare) |
| `sound` | piste de sunet ale proiectului: list, add, edit, remove, mix | [8](#8-audio-lip-sync-și-sunete) |
| `speech-motion` | încuviințări și ridicări de sprâncene aditive, temporizate pe cuvintele aliniate | [8](#8-audio-lip-sync-și-sunete) |
| `spring-fit` | potrivește frecvența/amortizarea unui arc la o curbă | [6](#6-animație-chei-timeline-și-generatoare) |
| `state-bake` | coace o mașină de stări bazată pe cadre în chei | [11](#11-mașini-de-stări-legări-de-date-containere-și-regiuni-de-derulare) |
| `stats` | numărul de oase, straturi, canale, chei (JSON) | [3](#3-crearea-și-inspecția-proiectelor) |
| `stretch` | coace un urmăritor cu întindere pentru a ajunge la țintă | [6](#6-animație-chei-timeline-și-generatoare) |
| `studio` | editor din browser (server web local) | [16](#16-studio-colaborare-serverul-agent-mcp-și-registrul-ui) |
| `tag` | propune nume de roluri standard pentru oase și straturi | [3](#3-crearea-și-inspecția-proiectelor) |
| `take-apply` | aplică un take din bibliotecă cu cross-fade-uri | [6](#6-animație-chei-timeline-și-generatoare) |
| `take-list` | listează take-urile dintr-o bibliotecă | [6](#6-animație-chei-timeline-și-generatoare) |
| `take-record` | înregistrează un take într-o bibliotecă | [6](#6-animație-chei-timeline-și-generatoare) |
| `timecode` | cadru <-> timecode SMPTE (cu suport pentru drop-frame) | [3](#3-crearea-și-inspecția-proiectelor) |
| `timeline` | piste pe canal cu chei, blocări și culori | [3](#3-crearea-și-inspecția-proiectelor) |
| `trace` | vectorizează canalul alfa al unui PNG într-un traseu vectorial | [9](#9-import) |
| `trail` | urma de mișcare a unui os, strat sau punct | [12](#12-efecte-profunzime-de-câmp-onion-skin-urme-și-cameră) |
| `trim` | coace un trim path într-un strat vectorial | [5](#5-rigging-skin-uri-ik-și-ajutoare-de-layout) |
| `ui` | registrul de comenzi, scurtăturile, layout-urile și căutarea studioului | [16](#16-studio-colaborare-serverul-agent-mcp-și-registrul-ui) |
| `vcs` | control al versiunilor cu îmbinare semantică pentru un singur fișier de proiect | [15](#15-controlul-versiunilor-vcs) |
| `view-swap` | comandă un strat switch dintr-o proprietate de os (funcție în trepte) | [5](#5-rigging-skin-uri-ik-și-ajutoare-de-layout) |
| `walk` | generează un ciclu de mers | [6](#6-animație-chei-timeline-și-generatoare) |
| `warp-pivot` | rotește/scalează/mută o selecție de puncte de control | [7](#7-fizică-cloth-și-deformatoare) |
| `waveform` | vârfuri min/max pentru desenare | [8](#8-audio-lip-sync-și-sunete) |
