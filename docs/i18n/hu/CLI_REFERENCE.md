<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# Az OSATURA parancssori referenciája

Az `osatura` bináris referenciája (az `osatura-cli` crate, AGPL-3.0-or-later; az MPL-2.0 licenccel rendelkező `osatura-core` motort linkeli). Dokumentált verzió: **osatura 0.1.0**. Ez a szöveg CC-BY-4.0 licenc alatt áll.

Az alábbi szinopszisok mindegyike a program saját használati szövegéből (`osatura` argumentumok nélkül, vagy `osatura <command> --help`) van átmásolva. Minden példát a beépített demóprojekten futtattam le, és a bemutatott kimenet
a valódi kimenet megrövidített változata. Ahol a használati szöveg és a tényleges viselkedés eltér, ez a dokumentum a tényleges viselkedést írja le, az eltérést pedig a végén, az [Ismert dokumentáció/viselkedés eltérések](#ismert-eltérések-a-dokumentáció-és-a-viselkedés-között) között sorolja fel.

Kapcsolódó dokumentumok: [EXPORT_FORMATS.md](EXPORT_FORMATS.md) (mit őriz meg és mit dob el az egyes export-/importformátum),
[CI.md](CI.md) (animáció-CI manifesztek), [API.md](API.md) (könyvtár/C ABI), [PATENT_NOTES.md](PATENT_NOTES.md)
(kodek- és szabadalmi állapot, nem jogi tanács).

## Tartalom

1. [Az összes parancsra érvényes konvenciók](#1-az-összes-parancsra-érvényes-konvenciók)
2. [Első lépések: a demóprojekt](#2-első-lépések-a-demóprojekt)
3. [Projekt létrehozása és vizsgálata](#3-projekt-létrehozása-és-vizsgálata)
4. [Renderelés és exportálás](#4-renderelés-és-exportálás)
5. [Rigging, skinek, IK és elrendezési segédparancsok](#5-rigging-skinek-ik-és-elrendezési-segédparancsok)
6. [Animáció, kulcsok, idővonal és generátorok](#6-animáció-kulcsok-idővonal-és-generátorok)
7. [Fizika, ruha és deformerek](#7-fizika-ruha-és-deformerek)
8. [Hang, ajaksinkron és hangok](#8-hang-ajaksinkron-és-hangok)
9. [Importálás (SVG, OpenRaster, Lottie, BVH, GIF/APNG, PNG-sorozatok, bitképek)](#9-import)
10. [Élő bemenet, MIDI, arckövetés és mozgásrögzítés](#10-élő-bemenet-midi-arckövetés-és-mozgásrögzítés)
11. [Állapotgépek, adatkötések, konténerek és görgetési tartományok](#11-állapotgépek-adatkötések-konténerek-és-görgetési-területek)
12. [Effektek, mélységélesség, onion skin, nyomvonalak és kamera](#12-effektek-mélységélesség-onion-skin-nyomvonalak-és-kamera)
13. [Részletességi szint, méretjelentések és export-fit](#13-részletességi-szintek-méretjelentések-és-export-fit)
14. [Minőségbiztosítás: check, lint, ci, golden, fuzz, diff](#14-minőségbiztosítás)
15. [Verziókezelés (`vcs`)](#15-verziókezelés-vcs)
16. [Stúdió, együttműködés, MCP ügynökszerver és UI-regiszter](#16-stúdió-együttműködés-mcp-ügynökszerver-és-ui-regiszter)
17. [Pózadatbázis, javaslatok és egyéb segédparancsok](#17-póz-adatbázis-javaslatok-és-egyéb-segédek)
18. [Ismert dokumentáció/viselkedés eltérések](#ismert-eltérések-a-dokumentáció-és-a-viselkedés-között)
19. [Parancsok betűrendes mutatója](#abc-sorrendű-parancsindex)

---

## 1. Az összes parancsra érvényes konvenciók

### 1.1 A parancssor alakja

```
osatura <command> [sub-command] <positional arguments...> [options]
```

* Az opciók a sor **bármely pontján** állhatnak, az alparancs és a pozicionális argumentumok után is
  (az `osatura render demo.json 0 o.png --scale 2` és az `osatura --scale 2 render ...` ugyanaz).
* Az **értékes opciók** a következő szót veszik értékül (`--scale 2`), vagy a `--key=value` alakot (`--scale=2`). Az `=` alak
  minden értékes opciónál működik. Az olyan érték, amely maga is tartalmaz `=` jelet, a szóközös alakkal rendben van (`--remap a=b`).
* A **kapcsolók** (logikai opciók, például a `--bones`) nem vesznek értéket. A `--bones=1` elutasításra kerül:
  `--bones is a flag and takes no value: write --bones (without =), got '--bones=1'`.
* **Szigorú opciók.** Az az opció (kapcsoló vagy értékes opció), amelyet a parancs nem dokumentál, a figyelmen kívül hagyás helyett hibát ad
  „did you mean” javaslattal (kilépési kód 1):
  ```
  $ osatura render demo.json 0 o.svg --bone
  unknown option --bone for 'render' (did you mean --bones?); see: osatura render --help
  ```
  Az ellenőrzés minden olyan parancsra érvényes, amelynek van használati sora, így a `check`, `stats`, `lint`, `coach`, `contracts` és
  `machine list` parancsra is (a régebbi verziók ott csendben figyelmen kívül hagyták a felesleges kapcsolókat). A `--json`, `--quiet` és `--allow-empty` **nem**
  globális: minden olyan parancs elutasítja őket, amelynek a használati sora nem sorolja fel őket.
  **Az ellenőrzés alparancsonként (szinopszis-alakonként) történik, nem parancsonként.** Az egyik alakhoz felsorolt opciót a
  testvéralakok elutasítják, és a hibaüzenet megnevezi azt az alakot, amelyik elfogadja:
  ```
  $ osatura ci diff a.png b.png --quiet
  unknown option --quiet for 'ci diff' (--quiet applies to: ci run); see: osatura ci --help
  ```
  Így a `--quiet` csak a `ci run` parancshoz való, a `--json` csak a `vcs log|status|diff|show`, a `ui commands|shortcuts` és a
  `trail ... --target` alakhoz, a `--tol` csak a `golden check` (és a `ci diff`) parancshoz, a `--scale` csak a `golden write` parancshoz,
  és minden `keyedit` szerkesztés csak a szinopszisban a neve után írt opciókat fogadja el. (Az `export` továbbra is összevonja a két
  szinopszissorát, mert a formátumot a fájlkiterjesztés választja ki; a `--codec`, `--columns` és `--scale` opciókat, amelyek nem
  vonatkoznak a kiválasztott kiterjesztésre, maga az `export` jelzi.)
* Néhány régebbi parancs `--options` helyett `key=value` **szavakat** vesz (`cloth`, `physics`, `cage`, `warp-pivot`,
  `soft-scale`, `mesh-lock`, `glue`, `audio-react`, `beats`, `import-seq`, `import-anim`, `layer-pivot`, `chan-meta`,
  `waveform`). Ezek pozicionálisak, és pontosan úgy kell leírni őket, ahogy a szinopszis mutatja (`gravity=900`, nem `--gravity 900`).
* A **frame** (képkocka) egy szám (ahol van értelme, törtek is megengedettek). Az `frames = N` értékű projekt képkockái `0..N-1`.
  A rétegekre **útvonallal** (`group/child`), a csontokra **névvel** hivatkozunk; a kameránál a cél az üres `""`.
* A **tulajdonságnevek** (a `--prop`, `slider`, `drive`, leképezési fájlok stb. használják) snake_case nevek:
  `bone_angle bone_scale bone_offset_x bone_offset_y layer_opacity layer_x layer_y layer_rotation layer_scale_x
  layer_scale_y layer_skew layer_switch layer_wobble camera_x camera_y camera_zoom camera_rotation camera_focus
  camera_blur ik_mix constraint_weight path_morph point_dx point_dy lattice_dx lattice_dy spine_dx spine_dy pin_dx
  pin_dy cage_dx cage_dy fx_blur fx_dx fx_dy fx_soft fx_amount grad_dx grad_dy`.  Néhány parancs ugyanezeket a tulajdonságokat CamelCase alakban (`BoneAngle`) írja ki az üzeneteiben; az opcióértékek mindig snake_case alakúak.

### 1.2 Globális opciók

| Option | Meaning |
|---|---|
| `--allow-external-assets` | Megengedi, hogy a projekt abszolút útvonallal vagy a saját könyvtárán kívül hivatkozzon fájlokra (hangok, képek, normáltérképek...). Alapértelmezés: elutasítva. |
| `--follow-symlinks` | Az kimeneti fájlként megadott szimbolikus link **céljába** ír. Alapértelmezés: elutasítva. |
| `--max-input-bytes N` | A beolvasott legnagyobb projektfájl. Alapértelmezés `268435456` (256 MiB). Pozitív egész számnak kell lennie. |
| `<command> --help` | Kiírja egy parancs használati sorait a stdoutra (kilépési kód 0). |
| `<command> --version`, `--version` | `osatura 0.1.0`; parancs nélkül a licencsort és az AGPL-forrásajánlatot is. |
| `--json`, `--quiet`, `--allow-empty` | Parancsonkénti opciók, amelyeket azok a parancsok **elutasítanak** (`unknown option`), amelyek szinopszisa nem sorolja fel őket (`--json`: `pick`, `select-box`, `timeline`, `trail`, `size-report`, `collab-sim`, `vcs`, `ui`, `face-detect`, `pose-suggest`; `--allow-empty`: `physics`, `live-map`, `face-bake`; `--quiet`: `ci run`). Azoknak a szkripteknek, amelyek ezeket egységesen minden parancsnak átadták, ezt abba kell hagyniuk. |

Példák (ellenőrizve):

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

### 1.3 Bemeneti fájlok olvasása

* A bemeneti fájloknak **szabályos fájloknak** kell lenniük, amelyek nem nagyobbak a `--max-input-bytes` értéknél. Az eszközfájlokat, csöveket, könyvtárakat és
  a folyamathelyettesítéseket (`<(...)`) elutasítja: írd az adatot előbb fájlba.
* A projektfájlok JSON-ok. Az ismeretlen kulcsokat tartalmazó projektet **elfogadja**, az ismeretlen kulcsokat pedig a `check` (és a `lint`, `studio`)
  `warning: unknown field bogus_key (ignored)` formában jelzi. Az oldalfájlokat, amelyeket egy parancshoz te írsz
  (szerződések, leképezések, gépek, konténerek, görgetési tartományok, ci manifesztek, drive-térképek ...), **szigorúan** értelmezi: egy ismeretlen
  kulcs hiba, és semmi sem kerül alkalmazásra (`unknown field(s): elastic.damping (typo? nothing was applied)`).
* A projektet minden alkalommal érvényesíti, amikor rendereléshez vagy exportáláshoz betölti (`<file> is not a valid project:` után
  soronként egy probléma). A szerkesztő parancsok az írás előtt újrafordítják a projektet is, és megtagadják olyan írását, amely már nem érvényes.
* A projekten belüli **eszközútvonalakat** (hangfájlok, képfájlok, normáltérképek) a **projektfájl könyvtárához** képest oldja fel.
  Az abszolút útvonalakat és a `..` részt tartalmazó útvonalakat elutasítja, hacsak nincs megadva az `--allow-external-assets`:
  ```
  $ osatura sound add s.json o.json x /etc/passwd
  sound add: asset '/etc/passwd': absolute paths are not allowed (a project may only reference files inside its own directory; use --allow-external-assets to override) (nothing was added)
  ```
  A kapcsolót minden későbbi, az adott projektet betöltő parancsnál újra meg kell adni (a `check ext.json` nélküle hibázik, vele sikerül).

### 1.4 Kimeneti fájlok írása

* **Atomikus írás.** A legtöbb parancs ugyanabban a könyvtárban lévő ideiglenes fájlon (`.<name>.tmp-<pid>-<random>`) keresztül ír, `fsync`-eli azt,
  átmásolja a régi fájl jogosultságait, és `rename`-mel a cél fölé helyezi, így egy összeomlás vagy egy megtelt lemez sosem hagy félig megírt fájlt.
  Ha nem hozható létre ideiglenes fájl (csak olvasható könyvtár, de írható fájl), a fájlt a helyén írja felül, ami **nem** atomikus.
  A FIFO-kat és a `/dev/stdout`-szerű speciális fájlokat közvetlenül írja.
* A kimenetként megadott **szimbolikus linkeket** elutasítja, hacsak nincs megadva a `--follow-symlinks` (ekkor a link célját írja; a függő link továbbra is hiba).
  A kimeneti fájlként megadott könyvtár hiba (`is a directory`).
* **Sosem a bemenet fölé.** A `render`, `render-hdr`, `dof`, `dof-export`, `export` (a `.png` `.json` sprite-sheet kísérőfájlja és
  a `--report` is), `layout-preview`, `render-gpu`, `export-fit`, `fuzz-grid`, `onion`, `trail` és `golden` elutasítja azt a kimenetet, amely ugyanaz a fájl,
  mint a bemeneti projekt: `refusing to overwrite the input project demo.json with output of the same name`.
* A **szerkesztő parancsok** (amelyeknek `<project.json> <out.json>` párjuk van: `reparent`, `keys`, `physics`, ...) **a helyükön** is írhatnak
  (az `osatura reparent p.json p.json hand none` működik, és felülírja a `p.json` fájlt). Az `import-lottie` elutasítja azt a kimenetet (vagy `fx.json` fájlt), amely
  ugyanaz a fájl, mint a bemenete; a többi importparancs más fajta fájlt olvas, mint amit ír.
* A **meglévő kimeneteket** minden parancs csendben felülírja. Nincs `--force`.
* **Minden felhasználó által megadott kimenet** (a `ci run` jelentései és golden fájljai, az `export-fit`, `lod`, `fuzz`, `fuzz-grid`, a `face-*` fájlok,
  pózfájlok, pillanatképek, a `vcs` és a stúdió együttműködési naplója is) ugyanazon az atomikus, szimbolikus linkeket elutasító íráson megy át. Az ffmpeg által írt
  `.mp4`/`.webm` kimeneteknél az ffmpeg indítása előtt ellenőrzi, hogy nem szimbolikus link-e. Szülőmappák: a `lod`, `export-fit`, `ci` és `face-*`
  létrehozza a hiányzó mappákat; a `fuzz`, `fuzz-grid` és `export` nem (`No such file or directory`).

### 1.5 stdout, stderr és kilépési kódok

* Az eredmények („wrote out.json”, táblázatok, JSON) a **stdoutra** kerülnek. A hibák és a legtöbb figyelmeztetés a **stderrre**. Az `import-svg` a
  figyelmeztetéseit stdoutra írja; az `export` a Lottie/GIF/GLB figyelmeztetéseket stderrre.
* A **0** kilépési kód sikert jelent. Az **1** kilépési kód bármilyen hibát *vagy* sikertelen ellenőrzést jelent: olvashatatlan/érvénytelen fájl, hibás argumentumok,
  `lint` hibákkal, `diff` eltérésekkel, megsértett szerződés, nem egyező golden, a határt meghaladó fuzz-hibaarány,
  nem teljesíthető `export-fit` költségvetés, a korlátot meghaladó `lod --verify`, `collab-sim` divergencia, sikertelen `ci` ellenőrzés.
  A **2** kilépési kódot csak a `ci` használja (semmit nem lehetett kiértékelni: hiányzó projekt/manifest, meghamisított golden).
* Argumentumok nélkül, ismeretlen parancsnál, hibás számú pozicionális argumentumnál vagy az `osatura --help` esetén a teljes használati szöveg a
  **stderrre** kerül, és a kilépési kód **1**. Az `osatura <command> --help` a parancs használati sorait a stdoutra írja, és 0-val lép ki.
  **Kivétel (Windows, a 0.1.0-alpha.2 óta):** ha az `osatura.exe` fájlt dupla kattintással indítják az Intézőből (a konzolablakot neki hozták létre, és más folyamat nem osztozik rajta), és nincs argumentuma, nem írja ki a használati szöveget: létrehozza a `Documents\OSATURA\first-rig.json` fájlt (a `demo-person` parancsból), ha hiányzik, lefuttatja a `studio` parancsot ezen a fájlon a `127.0.0.1:8080` címen, megnyitja a böngészőt, és nyitva tartja a konzolt, amíg be nem zárod, vagy hiba után Entert nem nyomsz. Terminálból indítva a fent leírtak szerint viselkedik. A program többi része 256 MiB vermű szálon fut (a Windows főszálának csak 1 MiB-ja van).

### 1.6 Külső programok

Az `ffmpeg` (a `PATH`-on kell lennie) szükséges ehhez: `.mp4`/`.webm` export és `dof-export` ezekbe a formátumokba, `sound mix`,
hang a `.html`-ben (mp3), és `face-track` videófájlon. Szükséges kódolók: `libx264` (mp4), `libvpx-vp9` / `libaom-av1` (webm),
`aac` (mp4 hang), `libopus` (webm hang), `libvorbis`, `libmp3lame` (sound mix). Nem tartalmaz ffmpeg-kódot vagy -binárist; lásd
[PATENT_NOTES.md](PATENT_NOTES.md). Mást nem futtat.

### 1.7 Teljesítménymegjegyzés

A dokumentumban szereplő összes időmérés **debug** buildből származik, és 10-30-szor lassabb egy release buildnél. A képkocka-igényes parancsokat
(`safety`, `golden`, `ci`, `export-fit`, `proxy-bench`) a demó lecsupaszított, 8 képkockás másolatán (`small.json`, a
`"frames": 8` beállításával készült) futtattam.

---

## 2. Első lépések: a demóprojekt

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

A demóban az `arm_up arm_low hand thigh shin foot_ctrl` csontok, egy `leg` nevű IK-lánc, valamint a legfelső szintű rétegek
`sky sun hills ground shadow arm leg target blush mouth(switch: closed/open/smile) ink` találhatók. Ebben a referenciában minden példa ezt használja,
hacsak nem jelezzük másképp.

---

## 3. Projekt létrehozása és vizsgálata

### `demo`

**Synopsis:** `osatura demo <out.json>`

Kiírja a beépített rig-demót (640x480, 24 fps, 48 képkocka, 6 csont, IK-láb, 14 réteg) projektfájlként.
**Kimenet:** `wrote <out.json>`. **Hibák:** hiányzó argumentum (használat, kilépési kód 1); nem írható útvonal. **Kapcsolódó:** `demo-fx`, `check`.
**Korlátok:** a demó rögzített eszköz; ennek a dokumentumnak minden példája erre épül.

```
$ osatura demo demo.json
wrote demo.json
```

### `demo-person`

**Synopsis:** `osatura demo-person <out.json>`

Kiírja a sétáló személy demót (640x480, 24 fps, 32 képkocka, 42 csont, 45 réteg) projektfájlként: oldalnézetből látott személy fejjel, füllel és lófarokkal, nyakkal, törzzsel, két karral (mindegyiken kéz és öt ujj) és két lábbal lábfejjel és lábujjakkal. A járásciklus két lépés hosszú, és varratmentesen ismétlődik: a medence lengedezik és billeg, a törzs dől és ellenforog, a fej vízszintes marad apró, késő bólintással, a karok a lábakkal ellentétesen lengenek, a könyök az előrelengésnél jobban hajlik, a kezek, ujjak, fül és lófarok pedig kicsit később követik. Minden láb a sarkára érkezik, laposra gördül, és a lábujjpercen emelkedik el; a talaj, a kerítés és az árnyék pontosan a talajon álló láb sebességével gördül, így a lábak nem csúsznak. Ezt a projektet hozzák létre a Windows-indítók (`osatura.exe`, `osatura-studio.exe`) az első indításkor `first-rig.json` néven; az `osatura demo` marad a dokumentum példáinak kis alapja.
**Kimenet:** `wrote <out.json>`. **Hibák:** hiányzó argumentum (használat, kilépési kód 1); nem írható útvonal. **Kapcsolódó:** `demo`, `studio`.
**Korlátok:** a kulcsok kétképkockánként vannak mintavételezve, eggyel a 0. képkocka előtt és eggyel az utolsó után, hogy a hurok sima legyen; a `lint` ezt a két kulcsot az idővonalon kívülinek jelzi (információ, nem hiba). Az alak oldalnézet, ezért nincs fordulás és mélység.

```
$ osatura demo-person person.json && osatura check person.json
wrote person.json
ok: 'OSATURA walking person' 640×480 @ 24 fps, 32 frames, 42 bones, 0 ik, 45 layers, 48 channels
```

### `demo-fx`

**Synopsis:** `osatura demo-fx <out.json>`

Kiírja a „liquid / boolean / particles” demót (640x480, 72 képkocka, 3 csont, 3 réteg), amely a procedurális rétegeket mutatja be.

```
$ osatura demo-fx fx.json && osatura check fx.json
wrote fx.json
ok: 'OSATURA fx demo' 640×480 @ 24 fps, 72 frames, 3 bones, 0 ik, 3 layers, 3 channels
```

### `check`

**Synopsis:** `osatura check <project.json>`

Betölt és érvényesít egy projektet. `warning:` sorokat ír ki (ismeretlen mezők, színátmenet-problémák, switch-kulcs-problémák, valamint az L013 lint-kód szerkesztőadat-figyelmeztetései: kamera zoom <= 0, réteg-átlátszatlanság a 0..1 tartományon kívül, hibás hivatkozások vagy időpontok a skinekben, comp-okban, markerekben, eseményekben, shotokban, channel_meta-ban, bone_vis-ben, klipekben és lod-okban) és egyetlen `ok:` összegző sort. Csak azok az értékek hibák, amelyek egy mentett fájlban nem létezhetnek (NaN vagy végtelen ezekben a szerkezetekben), így a régebbi projektek továbbra is betölthetők.
A hangsávokat is megnyitja: egy olvashatatlan/használhatatlan hangfájl **hiba**. **Kilépés:** 0 érvényes; 1 érvénytelen.

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

**Kapcsolódó:** `lint` (rig-/animációminőség), `ci` (regressziókapu). **Buktató:** a `check` nem renderel semmit; egy projekt átmehet a `check`-en, és mégis
elhasalhat egy képkockán később (a `NaN` pózokat a `render`/`export` fogja meg, amelyek előbb kiértékelik az összes képkockát).

### `stats`

**Synopsis:** `osatura stats <project.json>`

Számokat ír ki JSON-ként: `name, size, fps, frames, seconds, bones, ik_chains, constraints, springs, layers, channels, keyed_channels, keys, markers`.
A kimenet mindig JSON; nincs `--json` opció (úgy utasítja el, mint bármely ismeretlen opciót).

```
$ osatura stats demo.json
{ "name": "OSATURA demo", "size": [640.0, 480.0], "fps": 24.0, "frames": 48, "seconds": 2.0, "bones": 6,
  "ik_chains": 1, "constraints": 0, "springs": 0, "layers": 14, "channels": 14, "keyed_channels": 8, "keys": 37, "markers": 0 }
```
(a valódi kimenet több sorra tördelve, szépen formázva jelenik meg.) Vedd figyelembe, hogy a `layers` minden csomópontot számol, a csoportok/switchek gyermekeit is, miközben a `check` ugyanezt a számot írja ki.

### `diff`

**Synopsis:** `osatura diff <a.json> <b.json>`

Két projekt szemantikai különbsége: `+` hozzáadott, `-` eltávolított, `~` módosított. A név szerint címzett listáknál sorrendfüggetlen.
**Kilépés:** 0 azonos (`no differences`); **1**, ha vannak különbségek (az `N difference(s)` darabszám a stderrre kerül), így szkriptet is kapuzhat.

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
**Kapcsolódó:** `vcs diff` (előzmények), `compare` (képponti különbség), `ci diff` (képkülönbség).

### `tag`

**Synopsis:** `osatura tag <project.json>`

Szabványos szerepneveket (`arm.upper.l`, `hand.l`, `face.mouth` ...) javasol a csontoknak és a rétegeknek a nevük alapján, megbízhatósággal, és felsorolja azokat a szabványos szerepeket, amelyekhez nem talált csontot. Csak olvas; a szerepeket útmutatóként használd a `mocap-match` vagy az `autorig` parancshoz.

```
$ osatura tag demo.json
bone  arm_up                   -> arm.upper.l    (45%)
bone  hand                     -> hand.l         (45%)
layer mouth                    -> face.mouth     (90%)
no bone found for: head, neck, torso, hips, face.eye.l, ...
```

### `explain`

**Synopsis:** `osatura explain <project.json> <bone-or-layer> <frame>`

„Miért van itt?” - felsorolja azokat a sorrendbe rendezett hozzájárulásokat (nyugalmi póz, az egyes csatornák módjukkal és súlyukkal, kényszerek, IK), amelyek egy csont vagy réteg transzformációját adják egy adott képkockán, valamint a kapott head/tip/világ-szöget.

```
$ osatura explain demo.json hand 12
- bone 'hand' (length 45), parent 'arm_low'
- ancestry: hand ← arm_low ← arm_up
- rest pose: offset (100.00, 0.00), angle 0.00°
- channel #2 BoneAngle: keys → 20.000 (5 key(s), Hold outside range); mode replace, weight 1.00
- channel #3 BoneAngle: expression `6 * sin(time * 2 * pi * 2)`; mode add, weight 1.00
- result at frame 12: head (174.6, 126.4), tip (203.5, 92.0), world angle -50.0°
```
**Hibák:** ismeretlen csont-/rétegnév; az animáción kívüli képkocka.

### `pick`

**Synopsis:** `osatura pick <project.json> --frame N --x X --y Y [--tol PX] [--root] [--bones] [--points a,b] [--locked a,b] [--json]`

Mi van egy vászonpont alatt, a legfelső elöl (ugyanúgy végzi az ütközésvizsgálatot, mint a stúdió).

| Option | Meaning |
|---|---|
| `--frame N`, `--x X`, `--y Y` | Kötelező: a képkocka és a vászonkoordináták. |
| `--tol PX` | Kiválasztási tűrés vászonpixelben (alapértelmezés 3). |
| `--root` | A legfelső szintű őst (`mouth`) jelzi a legbelső réteg (`mouth/closed`) helyett. |
| `--bones` | A csontokat is figyelembe veszi. |
| `--points a,b` | Azok a rétegek, amelyek vezérlőpontjai (horgonyok/fogantyúk/mesh-csúcsok) kiválaszthatók. |
| `--locked a,b` | Kihagyandó rétegek (zároltként kezeli őket). |
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
**Buktató:** a háttérréteg (`sky`) normál találati cél, ezért szinte mindig van legalább egy találat.

### `select-box`

**Synopsis:** `osatura select-box <project.json> --frame N (--rect x0,y0,x1,y1 | --lasso x,y;x,y;x,y) [--mode bbox|contain|intersect] [--root] [--bones] [--json]`

Téglalapos/lasszós kijelölés. A `--mode` dönti el, mikor számít egy réteg bent lévőnek: `bbox` (a befoglaló doboza érinti a tartományt), `contain` (teljesen bent van), `intersect` (alapértelmezés: a megrajzolt geometriája metszi a tartományt). A `--root` a legfelső szintű ősöket jelzi, a `--bones` a csontokat is belefoglalja.
A `--lasso` értékét idézőjelek közé kell tenni a shellben (a `;` választja el a pontokat).

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

**Synopsis:** `osatura timeline <project.json> [--json]`

A dope sheet nézete szövegként: csatornánkénti sávok csont/réteg szerint csoportosítva, a kulcsképkockákkal (`keys @ ...`), az `expr` / `driven` csatornákkal, zárakkal és színekkel. A `#N` számok a `keyedit --channels` és a `#N@F0..F1` kijelölések által használt csatornaindexek.

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
A `--json` csoportokat ad azonosítókkal, szülőkkel, gyermekekkel, sávokkal, `frames` és `locked` mezőkkel.

### `timecode`

**Synopsis:** `osatura timecode <frame|HH:MM:SS;FF> [--fps 24] [--drop] [--nodrop]`

SMPTE-időkód-átváltás. Egy puszta szám képkocka, és időkódot ír ki; a `:` vagy `;` jelet tartalmazó argumentumot értelmezi, és a képkocka számát írja ki.
A 29,97, 59,94 és 119,88 fps alapértelmezés szerint drop-frame időkódot használ (a `;` a képkockák előtt); a `--nodrop` ehelyett 30-ig (vagy 60/120-ig) számol; a `--drop` kikényszeríti a drop-frame-et (más képkockasebességeknél hiba).
A `--drop` és a `--nodrop` kizárják egymást.

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
Nem olvas projektet; az `--fps` alapértelmezése 24 (nem egy projekt fps értéke).

### `snapshot`

**Synopsis:**
```
osatura snapshot save <project.json> <dir> [--keep 20]
osatura snapshot list <project.json> <dir>
osatura snapshot restore <project.json> <dir> <out.json> [--number N]
osatura snapshot recover <project.json> <dir> <out.json>
```
A `--keep` csak a `save` parancshoz, a `--number` csak a `restore` parancshoz tartozik; minden más kombinációt elutasít (`unknown option --keep for 'snapshot list' (applies to: snapshot save)`), akárcsak a hiányzó vagy felesleges fájlargumentumot.

Egy projekt számozott előzményei egy könyvtárban (a `vcs`-től függetlenül). A `save` az aktuális projektet `rev-NNNNNN-<hash>.json` néven tárolja, és csak a legújabb `--keep` változatot tartja meg (alapértelmezés 20, a régebbi fájlokat törli); ha a legutóbbi változattal azonos projektet mentesz, `unchanged since the last revision` üzenetet ír, és nem tárol semmit. A `list` kiírja a változatokat. A `restore` a `--number N` változatot írja az `<out.json>` fájlba; a `recover` a legújabb változatot írja.

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
A `list` hiányzó könyvtáron nem ír ki semmit, és 0-val lép ki. **Kapcsolódó:** a `vcs` az ágakhoz és az egyesítésekhez.

---

## 4. Renderelés és exportálás

### `render`

**Synopsis:** `osatura render <project.json> <frame> <out.svg|out.png> [--bones] [--scale N] [--clamp] [--proxy N]`

A `--proxy N` görgetési proxyt renderel (N = 1..4): a `.png` bitkép 1/2^N felbontásban, az `.svg` egyszerűsített vektorgrafika; mindkettő eldobja a vékony vonalakat, az átmeneteket és a keverési módokat.

Egy képkockát renderel. A projekt hosszával megegyező vagy azt meghaladó `<frame>` hiba (`frame 99 is outside the animation ... valid frames are 0..N-1 ... pass --clamp`); a `--clamp` ehelyett az utolsó képkockát rendereli, és figyelmeztetést ír a stderrre. (Ezt az ellenőrzést csak a `render` végzi; a `dof`, a `render-hdr` és az `fx` nem.) A kiterjesztés választja ki a formátumot (`.svg` vektor, `.png` 8 bites RGBA raszter); bármely más kiterjesztést elutasít, **mielőtt** bármit írna.

| Option | Default | Meaning |
|---|---|---|
| `--bones` | off | Kirajzolja a csontváz-overlayt (a fejek pontokként, a csontok vonalakként). |
| `--scale N` | 1 | Kimeneti skála, a (0, 16] tartományba eső szám. A `.png` esetén a pixelméretet szorozza; az `.svg` dokumentum megtartja a viewBox értékét. |
| `--proxy N` | off | Görgetési proxy, N = 1..4: `.png` = bitkép 1/2^N felbontásban, `.svg` = egyszerűsített vektorgrafika; mindkettő eldobja a vékony vonalakat, az átmeneteket és a keverési módokat. Interaktív görgetésre szolgál, nem leszállítandó anyagokhoz. |

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
**Hibák:** hiányzó/érvénytelen projekt, nem véges póz azon a képkockán, a kimenet megegyezik a bemeneti projekttel. **Kapcsolódó:** `frames`, `render-hdr`, `onion`, `compare`, `fx`.
**Korlátok:** a PNG-rasztert a motor saját rasztere állítja elő (8 bites, sRGB; a keverés nem színkezelt, hacsak a projekt nem kéri a `color-space ... linear` beállítással). A rétegeffekteket, maszkokat és keverési módokat mindkettő kirajzolja.

### `frames`

**Synopsis:** `osatura frames <project.json> <out_dir> [--png] [--bones] [--scale N]`

**Minden** képkockát renderel egy könyvtárba (ha nem létezik, létrehozza) `frame_0000.svg` ... néven (vagy `.png`-ként a `--png` esetén). Előbb ellenőrzi, hogy minden képkocka pózai végesek-e.

```
$ osatura frames demo.json fr --png
wrote 48 frames to fr
$ ls fr | head -2
frame_0000.png
frame_0001.png
```
**Buktató:** az azonos nevű meglévő fájlokat felülírja; a könyvtárban lévő, nem kapcsolódó fájlokat békén hagyja. Használd bemenetként az `ffmpeg` vagy az `import-seq` parancshoz.

### `export`

**Synopsis:**
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
(formátumonként egy alak; a kimeneti fájl kiterjesztése, kis- és nagybetűre való tekintet nélkül, választja ki az alakot. Az olyan opciót, amelyet az alak nem sorol fel, elutasítja: `unknown option --scale for 'export .json' (applies to: export .gif, export .mp4, ...)`. Ismeretlen vagy hiányzó kiterjesztésnél helyette a formátumhibát jelzi.)

A kimeneti kiterjesztés választja ki a formátumot:

| Extension | Result | Needs ffmpeg | Options that apply |
|---|---|---|---|
| `.gif` | a projekt képkockasebességén ismétlődő GIF (lásd [EXPORT_FORMATS.md](EXPORT_FORMATS.md)) | nem | `--scale`, `--report` |
| `.html`, `.htm` | egyetlen önálló HTML-lejátszó (beágyazott SVG-képkockák, lejátszás/szünet, csúszka, opcionálisan beágyazott hang) | csak a hanghoz | - |
| `.json` | Lottie (Bodymovin) | nem | `--report` |
| `.lottie` | dotLottie konténer (zip: `manifest.json` + `animations/animation.json`) | nem | `--report` |
| `.mp4` | H.264 yuv420p (+ AAC, ha a projektnek vannak hangjai) | igen (`libx264`, `aac`) | `--scale` |
| `.apng` | animált PNG | nem | `--scale` |
| `.webm` | VP9 (alapértelmezés) vagy AV1 alfával (+ Opus hang) | igen (`libvpx-vp9` / `libaom-av1`, `libopus`) | `--scale`, `--codec vp9\|av1` |
| `.png` | sprite sheet **és** `<out>.atlas.json` metaadat (soha nem `<out>.json`) | nem | `--scale`, `--columns N` |
| `.glb` | glTF 2.0 bináris (csontok, animáció, skinelt kitöltések/vonalak, színátmenetek, maszkok, képek, morph targetek) | nem | `--report` |

Opciók:

| Option | Meaning |
|---|---|
| `--scale N` | Raszterskála (0, 16] a `.gif .mp4 .apng .webm .png` formátumokhoz. Más formátumoknál elutasítja (`unknown option --scale for 'export .json'`); a 7. hullám előtt figyelmeztetés volt. |
| `--report FILE` | JSON-ként kiírja azon dolgok listáját, amelyeket a formátum nem tudott átvinni (`.gif .json .lottie .glb`): `{"format","count","warnings":[{"code","layer","message"}]}`. Ugyanezek a figyelmeztetések mindig a stderrre is kikerülnek. A fájl nem lehet a bemeneti projekt. |
| `--codec vp9\|av1` | Csak `.webm` esetén (alapértelmezés `vp9`); bármely más érték: `unknown --codec 'foo' (vp9 or av1)`. |
| `--columns N` | Csak a `.png` sprite sheethez: csempék száma soronként (alapértelmezés: közel négyzetes rács, ceil(sqrt(frames)): 8 képkockánál 3 oszlop). |

Példák (mind ellenőrizve a demó 8 képkockás másolatán, `--scale 0.5`):

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

Hibák és elutasítások:

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
További elutasítások: `ffmpeg failed` (hiányzó kódoló), `export would write about N million values ...` (a Lottie/HTML/GLB méretvédelem: a határ 40 millió érték = képkockák x (horgonyok + csontok + rétegek); rövidítsd az animációt, vagy egyszerűsítsd az útvonalakat), nem véges pózok bármelyik képkockán, és `image too large for GIF`.

**Kimenetek / mellékhatások:** a `.png` a sprite sheet metaadatait is kiírja az `<out-name>.atlas.json` fájlba (pl. `out.png` -> `out.atlas.json`), így többé nem írhat felül egy `out.json` Lottie-fájlt; a kísérőfájlra ugyanazok a bemeneti/szimbolikus link védelmek vonatkoznak. Az `.mp4`/`.webm` a PNG-képkockákat egy privát ideiglenes könyvtárba rendereli, amelyet utána eltávolít.
**Hang:** az `.mp4` és a `.webm` összekeveri a projekt hangsávjait (kezdet, erősítés, vágás, ki- és beúsztatás; a projekt hosszánál elvágva); a `.html` egy összekevert mp3-at ágyaz be (3 MiB fölött figyelmeztet, 16 MiB fölött figyelmeztetéssel elhagyja); a `.gif` és az `.apng` `note: GIF cannot carry audio; ...` üzenetet ír, és néma marad; a Lottie `[audio]` figyelmeztetést ad.
**Átlátszóság:** a `"background": null` projekt átlátszó pixelekkel renderel mindenhol, ahová semmit sem rajzol. Az `.apng`, a `.png` sprite sheet és a `.webm` (`yuva420p`, konténercímke `alpha_mode=1`) megtartja az alfacsatornát; a `.gif` 1 bites átlátszóságot tart meg (ellenőrizve: az átlátszó szín jelzője be van állítva); az `.mp4`-nek nincs alfája. A háttérszínnel rendelkező projekt (a demóé `#f6f1e7`) átlátszatlan.
**Kapcsolódó:** `export-fit` (bájtkeretbe illesztés), `size-report`, `lod`, `dof-export`, `export-oca`, `render-hdr`. A formátumok részletei: [EXPORT_FORMATS.md](EXPORT_FORMATS.md).

### `export-oca`

**Synopsis:** `osatura export-oca <project.json> <out_dir> [--layers]`

**Open Cel Animation** mappát ír: `<out_dir>/<project name>.oca` (JSON, `ocaVersion` 1.1) és `layerN/fNNNN.png` PNG-cellákat. A `--layers` nélkül egyetlen `render` réteg van, amely az összeállított képkockákat tartalmazza; a `--layers` esetén minden **legfelső szintű** réteg külön, átlátszóságon renderelődik a saját OCA-rétegébe (a demóhoz 11 réteg). A `--scale N` szorozza a cellák méretét.

```
$ osatura export-oca small.json oca2 --layers
wrote oca2/small.oca
$ ls oca2
layer0 layer1 ... layer10 small.oca
```
**Korlátok:** a cellák rasztergrafikák (vektorok, csontok és csatornák nem maradnak meg); a réteg átlátszatlanságát/keverését nem írja ki (minden réteg `paintlayer`, átlátszatlanság 1); a hangokat nem exportálja; a mappát szükség esetén létrehozza, és a benne lévő fájlokat felülírja.

### `render-hdr`

**Synopsis:** `osatura render-hdr <project.json> <frame> <out.exr|out.png> [linear] [--precision half|float] [--float32] [--8bit] [--scale N] [--clamp]`

Lebegőpontos (32 bites) renderelés (a projekten kívüli `<frame>` hiba, a `--clamp` a legközelebbi érvényes képkockát veszi, mint a `render` esetén): az `.exr` egyrészes, soronkénti, tömörítetlen **OpenEXR**-t ír (alapértelmezés szerint HALF csatornák, FLOAT a `--precision float` vagy a `--float32` esetén), lineáris fényű és premultiplikált; a `.png` **16 bites RGBA PNG**-t ír, sRGB-kódolással, hacsak az ötödik argumentum nem a szó szerinti `linear`. A projekt hátterét **nem** rajzolja ki (a kép átlátszó ott, ahová semmi sincs rajzolva). A `--8bit` a régi 8 bites rasztert használja forrásként (nem kombinálható float32 kimenettel). A `--scale N` érvényes.

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
Az `ffprobe` az EXR-t `exr 640x480 gbrapf32le` néven olvassa. Az `.exr` esetén a `linear` szónak nincs hatása (az EXR mindig lineáris). **Korlátok:** nincs tömörítés, fájlonként egy képkocka, nincsenek rétegek/AOV-k; a forrásgrafika hétköznapi sRGB színű, így a fájl megjelenítési tartományú értékeket tartalmaz nagyobb pontossággal, nem jelenetvonatkoztatású HDR fényt. Az sRGB<->lineáris átváltáson túl nem színkezelt.

### `render-gpu`

**Synopsis:** `osatura render-gpu <project.json> <out.png> [--frame N] [--adapter software|auto] [--scale S] [--bench N]`

Egy képkocka GPU-s (wgpu) renderelése. Csak a `--features gpu` kapcsolóval készült buildekben érhető el; az alapértelmezett build ezt válaszolja:

```
$ osatura render-gpu small.json g.png
render-gpu needs a build with the gpu feature: cargo build -p osatura-cli --features gpu       [exit 1]
```
GPU-s buildben a nem támogatott jeleneteket (képek, csontok) hibaként jelzi, sosem rajzolja ki hibásan. A `--bench N` teljesítménymérési opció. Ezt a parancsot a tesztbuildben nem lehetett futtatni (nincs GPU-feature), ezért a kapcsolóit csak a használati szövegből vettem.

### `compare`

**Synopsis:** `osatura compare <a.json> <b.json> <frame> <out.png> [--gain 4 --scale N]`

Egyetlen PNG-t ír: **előtte | utána | különbség** egymás mellett (a harmadik panel hőtérkép, fekete = azonos, piros -> sárga = nagyobb változás, a `--gain` értékével szorozva, alapértelmezés 4). Mindkét projektnek ugyanakkora méretben kell renderelődnie (`frame sizes differ (...); compare projects of the same size`).

```
$ osatura compare small.json d2.json 3 cmp.png --gain 4 --scale 0.25
wrote cmp.png
```
**Kapcsolódó:** `diff` (szerkezet), `ci diff` (két PNG, percepciós metrikák), `golden`.

### `proxy-bench`

**Synopsis:** `osatura proxy-bench <project.json> [--frames N]`

Ezredmásodperc képkockánként, egy szálon, teljes minőségben és az egyes görgetési proxyszinteken (lásd `render --proxy`).

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
(debug build számai). 

---

## 5. Rigging, skinek, IK és elrendezési segédparancsok

Ebben a szakaszban minden parancs az `osatura <cmd> <project.json> <out.json> ...` mintát követi: beolvas egy projektet, módosítja, **újraérvényesíti**
és kiírja az `<out.json>` fájlt (amely megegyezhet a bemenettel). Hiba esetén semmi sem íródik ki.

### `autorig`

**Synopsis:** `osatura autorig <project.json> <out.json> [seed]`

Szabványos automatizmusokat ad hozzá a csontok/rétegek **nevei** alapján: IK-láncokat (mix 0, hogy az FK tovább működjön), légzési ciklust (torzó-csontot igényel) és pislogást (szemrétegeket igényel). Döntésenként egy sort ír ki: `+` hozzáadva, `-` kihagyva az okkal. Az `eye-clip` parancsot is futtatja.

```
$ osatura autorig small.json auto.json 7
+ IK arm_l -> arm_l_ctrl (mix 0, bend +1)
- arm_r: upper/lower bone not found
- leg_l: already has an IK chain
- breathing: no torso bone
- blink: no eye layers
wrote auto.json
```
A `seed` (opcionális pozicionális argumentum) a pislogás véletlenszerű időzítését magolja. **Korlátok:** tisztán névalapú (a `tag` mutatja, hogyan olvassa a neveket); sosem hoz létre csontokat.

### `slider`

**Synopsis:** `osatura slider <project.json> <out.json> <name> <poses>`, ahol a `poses` alakja `at:target.prop=val,target.prop=val;at:...`

Létrehoz egy `ctrl_<name>` **vezérlőcsontot**, amelynek `offset_x` értéke több tulajdonságot hajt meg a tárolt pózok között (póz-csúszka). Animáld a kiírt vezérlő `bone_offset_x` tulajdonságát.

```
$ osatura slider small.json sl.json pose "0:arm_up.bone_angle=0;24:arm_up.bone_angle=30"
controller bone: ctrl_pose (animate its offset_x)
wrote sl.json
```
A `target.prop` a snake_case tulajdonságneveket használja (az `arm_up.angle` hibát ad: `unknown property 'angle'`). Az utolsó pont választja el a célt a tulajdonságtól, így az olyan rétegútvonalak, mint a `grp/child.layer_opacity`, működnek.

### `view-swap`

**Synopsis:** `osatura view-swap <project.json> <out.json> <switch_layer> <bone.prop> <from:child,from:child,...>`

Egy switch réteget hajt meg egy csont tulajdonságával lépcsős függvénnyel: a `from:child` párokat a `from` szerint rendezi, és a switch az utolsó olyan pár gyermekét mutatja, amelynek `from` értéke nem nagyobb a meghajtó értékénél (`-40:0,0:1,40:2` = 0. gyermek 0 alatt, 1. gyermek 0-tól, 2. gyermek 40-től). Fordulónézetekhez (elöl/oldalt/hátul) használd, yaw-csonttal meghajtva. A gyermekindexeknek létezniük kell (`child index 5 is out of range (the layer has 3)`); a rétegnek switchnek kell lennie.

```
$ osatura view-swap small.json vs.json mouth foot_ctrl.bone_offset_x "-40:0,0:1,40:2"
wrote vs.json
```

### `align` és `distribute`

**Synopsis:**
`osatura align <project.json> <out.json> <mode> <layer,layer,...|layer> [stage]`  (mode: `left right top bottom center-x center-y`)
`osatura distribute <project.json> <out.json> <x|y> <gaps|centers> <layer,layer,...>`

Az `align` úgy mozgatja a rétegeket, hogy a kisimított görbéik befoglaló dobozai egy vonalba kerüljenek (egymáshoz képest, vagy egyetlen réteg esetén, ha a `stage` szó követi, a vászonhoz). A `distribute` három vagy több réteget oszt el egyenletesen egy tengely mentén, vagy a dobozok közötti **réseket** (gaps), vagy a **középpontokat** (centers) egyenlővé téve.

```
$ osatura align small.json al.json center-x sun,hills stage
wrote al.json
$ osatura align small.json al2.json top ground
select at least two layers (or one with --stage)                 [exit 1]
$ osatura distribute small.json di.json x gaps sun,ground,hills
wrote di.json
```
Ezek a rétegpozíciókat szerkesztik (statikus transzformáció), így a kulcsolt `layer_x`/`layer_y` csatornák továbbra is felülírják őket. (A hibaüzenet a `--stage` kapcsolót említi; a tényleges argumentum a puszta `stage` szó.)

### `trim`

**Synopsis:** `osatura trim <project.json> <out.json> <layer> <path_index> <start> <end> [offset]`

*Trim path*-t éget be: egy vektorréteg `path_index` sorszámú útvonalát csak a `start` és az `end` közötti résszel helyettesíti (az útvonal hosszának 0..1 törtrészei, mindkettő az `offset` értékével eltolva; a zárt útvonalak körbefordulnak). Az eredmény **nyitott töröttvonal ugyanazzal a vonallal és kitöltés nélkül** - destruktív, ezért tartsd meg az eredetit. Rajzolódó effekthez képkockánként égess be egy-egy vágott másolatot.

```
$ osatura trim small.json tr.json ink 0 0 0.5 0.25
wrote tr.json
```

### `reparent`

**Synopsis:** `osatura reparent <project.json> <out.json> <bone> <new_parent|none>`

Egy csont szülőjét változtatja meg **a csont elmozdítása nélkül**: a nyugalmi eltolást/szöget újraszámolja, hogy a nyugalmi póz változatlan maradjon (ellenőrizve: utána csak a `parent`, a `rest_angle` és a `rest_offset` különbözik). A `none` gyökérré teszi. Az olyan szülőt, amely ciklust okozna, elutasítja.

```
$ osatura reparent small.json rp.json hand none
wrote rp.json
$ osatura diff small.json rp.json
~ bone 'hand' changed (parent, rest_angle, rest_offset)
```

### `parent-switch`

**Synopsis:** `osatura parent-switch <project.json> <out.json> <bone> <frame:parent|none,...>`

Animált szülőváltást éget be (egy kéz, amely megfog egy kelléket): minden váltási képkockán rögzíti és megtartja a csont világpózát az új szülőhöz képest (merev követés), illetve `none` esetén befagyasztja világtérben. A csont gyökérré válik, a világpózát pedig minden képkockára egyszerű offset/szög kulcsokként írja ki; a régi csatornái a fájlban maradnak, de a beégetettek felülírják őket. **Korlátok:** elutasítja az IK-ban, kényszerekben vagy rugócsontokban részt vevő csontokat; a skálát nem viszi át.

```
$ osatura parent-switch demo.json ps.json hand 0:none,24:arm_up
wrote ps.json
```

### `skin`

**Synopsis (alparancsok):**
```
osatura skin <project.json> <out.json> add <group> <name> <layer,layer,...>
osatura skin <project.json> <out.json> rig <group> <name> <constraint,...|-> <ik,...|->
osatura skin <project.json> <out.json> bones <group> <name> <bone,...|->
osatura skin <project.json> <out.json> apply <group> <name>
osatura skin <project.json> <out.json> schedule <group> <frame:name,frame:name,...>
```

A *skin set*-et egy `<group>` címke azonosítja (egy „öltözet” vagy „arckifejezés” készlet; a példában véletlenül egy rétegcsoport nevével egyezik). Minden elnevezett skin rétegek egy készletét mutatja, **rétegútvonallal** megadva (pl. `skins/box`); egy skin alkalmazása megmutatja a rétegeit, és elrejti ugyanazon csoport minden más skinjének rétegeit.
Az `add` definiál egy skint; a `rig` kényszereket és IK-láncokat sorol fel, amelyek csak addig aktívak, amíg az adott skin az; a `bones` olyan csontokat sorol fel, amelyek csak az aktív skin alatt léteznek (különben 0 skálára összehúzva); az `apply` statikusan aktívvá tesz egy skint; a `schedule` az aktív skint időben kulcsolja lépcsős kulcsokként a **réteg átlátszatlanságán** (ezért ne animáld másképp ezeknek a rétegeknek az átlátszatlanságát). A `-` jelentése „nincs”.

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
**Korlátok (ROADMAP §9):** az a csont, amely csak egy skinben létezik, sosem kerül ki a kiértékelt rigből (az indexek maradnak; 0-ra skálázódik); az a réteg, amely súlyozva van egy csak az adott skinben létező csonthoz, de nincs benne abban a skinben, vele együtt eltűnik, és ezt nem ellenőrzi.

### `bone-vis`

**Synopsis:** `osatura bone-vis <project.json> <out.json> <bone> <from> <to>|clear`

A csontot a stúdió **overlayein** csak egy képkockatartományban mutatja (zsúfoltság-szabályozás). A renderelést/exportot nem változtatja. A `clear` eltávolítja a tartományt (`range removed`).

### `chan-meta`

**Synopsis:** `osatura chan-meta <project.json> <out.json> <target> <prop|*> [color=#rrggbb] [shy=on|off] [solo=on|off] [lock=on|off]`

Egy csatorna dope sheet metaadatai: megjelenítési szín, *shy* (elrejtve a listákban), *solo* (csak ezeket a csatornákat előnézi a stúdióban) és **lock** (megtagadja a szerkesztést: a zárolt csatornákat a `keys` kihagyja, a `keyedit`, az `audio-react` és a `live-map` elutasítja, a `clip bake` pedig soha nem érinti). A kiértékelés és az export mind a négyet figyelmen kívül hagyja. A `*` a cél minden tulajdonságát kijelöli.

```
$ osatura chan-meta small.json cm.json arm_up bone_angle 'color=#ff0000' shy=on lock=on
wrote cm.json
```

### `reference`

**Synopsis:** `osatura reference <project.json> <out.json> <layer> <on|off>`

Egy réteget **átrajzolási referenciaként** jelöl meg: látszik a stúdióban, de sosem exportálódik.

### `parallax`

**Synopsis:** `osatura parallax <project.json> <out.json> <far_layer,...,near_layer> <far_value> <near_value>`

Többsíkú mélységet (a `far_value` és a `near_value` között szétosztott `parallax` értékeket) rendel a felsorolt legfelső szintű rétegekhez, a megadott sorrendben (az első = a legtávolabbi). Kameramozgásnál a rétegek különböző mértékben tolódnak el; a `dof` a `|parallax - focus|` értékkel arányosan homályosít.

```
$ osatura parallax small.json pa.json sky,hills,ground 0.2 1.0
wrote pa.json
```

### `eye-clip`

**Synopsis:** `osatura eye-clip <project.json> <out.json>`

Minden írisz-/pupillarétegre rávet egy közönséges rétegmaszkot az ugyanazon az oldalon lévő szemforma-rétegből (név alapján találja meg, lásd `tag`), így a pupillák mozgás közben sem hagyják el a szemet. Az `autorig` is futtatja. Ha nem talál egyező neveket, a fájlt akkor is kiírja, változatlanul.

### `comp`

**Synopsis:**
```
osatura comp capture|apply <project.json> <out.json> <name>
osatura comp list <project.json>
```

Elnevezett rétegállapotok („comp-ok”: láthatóság, átlátszatlanság és a switch rétegek kiválasztott gyermeke rétegútvonalanként; **nem** transzformációk). A `capture` az aktuális állapotot tárolja `name` néven, az `apply` visszaállítja, a `list` kiírja a neveket (a `list` nem vesz kimeneti fájlt, és nem ír semmit; a régi negyedik szó, a `comp list p.json p.json`, még elfogadott, és figyelmen kívül marad).

```
$ osatura comp capture small.json c1.json pose1
wrote c1.json
$ osatura comp list c1.json c1.json
pose1 (14 layers)
$ osatura comp apply c1.json c2.json pose1
wrote c2.json
```

### `retarget`

**Synopsis:** `osatura retarget <source.json> <dest.json> <out.json> --pairs srcChain=dstChain[,..]`

*Szándék* szerint retargetel: az egyes forráslánc IK-**effektorának** (célcsontjának) animációját a két végtag elérésének (a csonthosszak összege) arányával skálázza, és átirányítja a célláncot célcsontjára. A nem kulcsolt effektorcsatornák nem skálázhatók; ezeket átnevezve továbbadja, és `note:` sorokként jelzi. A `--pairs srcChain=dstChain[,...]` kötelező. A példában a forrás és a cél ugyanaz a projekt, ami csak füsttesztnek jó.

```
$ osatura retarget demo.json demo.json rt.json --pairs leg=leg
wrote rt.json (+14 channels)
```
**Kapcsolódó:** `mocap-retarget` BVH-adatokhoz; `walk`, `mirror`.

---

## 6. Animáció, kulcsok, idővonal és generátorok

### `keys`

**Synopsis:**
```
osatura keys shift <project.json> <out.json> --by N [--from F --to F --target NAME --prop PROP]
osatura keys scale <project.json> <out.json> --factor X [--pivot F --from F --to F --target NAME --prop PROP]
osatura keys ease <project.json> <out.json> --ease linear|step|ease_in|ease_out|ease_in_out|smooth [--from F --to F --target NAME --prop PROP]
osatura keys reverse <project.json> <out.json> [--from F --to F --target NAME --prop PROP]
```
Minden művelet csak a saját opcióit fogadja el (a `keys shift --factor 2` elutasításra kerül); a `--by`, `--factor` és `--ease` kötelező a hozzájuk tartozó műveletnél (a kihagyott `--by` vagy `--factor` korábban csendes üres művelet volt).

Tömeges kulcsműveletek **az összes kulcsolt csatornán** (a `--target` / `--prop` szűri őket; a `--from/--to` tartomány korlátozza, mely kulcsokat érinti). A zárolt csatornákat `skipped locked channel ...` üzenettel kihagyja.

| Operation | Needs | Effect |
|---|---|---|
| `shift` | `--by N` | A kulcsokat N képkockával eltolja. |
| `scale` | `--pivot F --factor X` | A kulcsidőket a pivot képkocka körül skálázza. |
| `ease` | `--ease E` | Beállítja a kulcsok easingjét (elnevezett easingek; a `bezier:` alakhoz használd a `keyedit ease` parancsot). |
| `reverse` | - | Megfordítja a kulcsok sorrendjét a tartományon belül. |

```
$ osatura keys shift small.json k1.json --target arm_up --prop bone_angle --by 2
wrote k1.json (5 key(s) changed)
$ osatura keys scale small.json k2.json --pivot 0 --factor 0.5 --from 0 --to 48
wrote k2.json (37 key(s) changed)
$ osatura keys reverse small.json k4.json
wrote k4.json (37 key(s) changed)
```
A `0..frames-1` tartományon kívülre mozgatott kulcsok a fájlban maradnak (a lint `L007`-et jelent); ez a parancs nem hosszabbítja meg a projekt hosszát. **Kapcsolódó:** `keyedit` (pontos, kijelölésalapú), `loop-solve`.

### `keyedit`

**Synopsis:** `osatura keyedit <project.json> <out.json|-> <sub-command> [options]`

Dope sheet szerkesztések **egyetlen, visszavonásbiztos kötegként**. Ha a kimenet `-`, semmi sem íródik ki, és a tervezett műveletet JSON-ként írja ki (próbafutás). Alparancsok és opciók:

```
osatura keyedit <project.json> <out.json|-> move --sel SEL [--dt N] [--dv N] [--collision replace|merge|refuse] [--no-snap] [--free]
osatura keyedit <project.json> <out.json|-> scale --sel SEL --pivot F --factor X | dup --sel SEL --offset N | delete --sel SEL | ease --sel SEL --ease E | value --sel SEL --value V | smooth --sel SEL [--kind monotone|catmull_rom]
osatura keyedit <project.json> <out.json|-> copy --sel SEL --clip clip.json | paste --clip clip.json --at F [--remap from_target:prop=to_target:prop]
osatura keyedit <project.json> <out.json|-> add --target T --prop P --at F [--value V] [--ease E] | ripple --at F --by N --mode insert|delete [--scope 0,1] [--adjust-frames]
osatura keyedit <project.json> <out.json|-> offset --channels 0,1 --dt N | extrapolate --channels 0,1 --extrapolate loop | loop --channels 0,1 [--end F] | handle --sel SEL --side in|out --t F --v V [--mode free|aligned|auto] | lock --target T [--prop P] [--value on|off]
```

* A **SEL** `;`-vel összefűzött `TARGET:PROP[@F0..F1]` elemekből, vagy `#CHANNEL[@F0..F1]` (csatornaindex a `timeline` parancsból), vagy `all` értékből áll. Tedd idézőjelek közé (`;`, `@`, `#`).
* Az **E** (easing) értéke `linear`, `step`, `ease_in`, `ease_out`, `ease_in_out`, `smooth`, `bounce`, `bezier:x1,y1,x2,y2` vagy egy JSON-objektum.
* A `--collision` dönti el, mi történik, ha egy mozgatott kulcs egy meglévő kulcsra érkezik (`replace` az alapértelmezés, `merge`, `refuse`); a `--no-snap` megtartja a tört képkockákat; a `--free` feloldja az egész képkockákra vonatkozó korlátozást.
* Kimeneti sor: `<op>: N keys changed, wrote <out>`. A `copy` valódi kimenettel `copy: 0 keys changed, project unchanged, clip saved` üzenetet ír.
* Azokat a műveleteket, amelyek egy kulcsot a `0..=frames` tartományon kívülre tennének, pontos üzenettel elutasítja (`duplicate: frame 52 is outside the project range 0..=48`), hacsak a `ripple --adjust-frames` nem hosszabbítja meg a projektet.

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
**Buktatók:** a `delete` a `#0@12..12` kijelöléssel pontosan egy kulcsot távolít el; a `ripple --mode delete --scope 0,1` az eltolást a 0. és az 1. csatornára korlátozza; a `copy` és a `paste` egy klipfájlt cserél, nem a rendszer vágólapját. A zárolt csatornák elutasítják a `move/scale/...` műveleteket (előbb használd a `lock --value off` parancsot).

### `marker`

**Synopsis:** `osatura marker <project.json> <out.json> <name> <frame> [--end F]`

Idővonal-markert ad hozzá (elnevezett pont, vagy `--end` esetén tartomány). Az azonos nevű markert lecseréli. A markerek képkocka szerint rendezettek.

```
$ osatura marker small.json m1.json hit 4 --end 6
wrote m1.json (marker 'hit' at 4 = 00:00:00:04)
```

### `event`

**Synopsis:** `osatura event <project.json> <out.json> <name> <frame> [--data JSON]`

Elnevezett eseményt (a lejátszók számára futásidejű triggert) ad hozzá egy képkockához, opcionális JSON-adattal. Érvénytelen JSON esetén: `--data is not valid JSON: ...`.

```
$ osatura event small.json ev1.json boom 3 --data '{"power":2}'
wrote ev1.json (1 events)
```

### `idle`

**Synopsis:**
```
osatura idle blink <project.json> <out.json> --target T --prop P [--interval S --jitter X --closed F --seed N]
osatura idle breath <project.json> <out.json> --target T --prop P [--amp A --period F]
osatura idle sway <project.json> <out.json> --target T --prop P [--amp A --freq HZ --seed N]
```
Minden generátor csak a saját opcióit fogadja el (az `idle breath --seed 2` elutasításra kerül).

Procedurális idle csatornát generál, és **hozzáfűzi** (a meglévő csatornák megmaradnak).

| Generator | Options (defaults) | Typical target |
|---|---|---|
| `blink` | `--interval` a pislogások közötti átlagos másodpercek, `--jitter`, `--closed` a zárt képkockák száma, `--seed` | egy `layer_switch` (szem nyitva/zárva) vagy `layer_scale_y` |
| `breath` | `--amp` (0.03), `--period` képkockákban (3 s a projekt fps értékén) | egy torzó `bone_scale` tulajdonsága |
| `sway` | `--amp` (2), `--freq` Hz (0.3), `--seed` (1) | `bone_angle` |

```
$ osatura idle blink small.json i1.json --target mouth --prop layer_switch --interval 1 --seed 3
wrote i1.json
$ osatura idle sway small.json i3.json --target arm_up --prop bone_angle --amp 3 --freq 0.5
wrote i3.json
```
A `--target` és a `--prop` kötelező (`--target is required`).

### `follow` (két alak)

**Follow-through (opciók):** `osatura follow <project.json> <out.json> --target SRC --prop P --chain a,b,c [--delay F --decay X]`
Egy meglévő csatorna késleltetett, csillapodó másolatait adja hozzá a lánc csontjaihoz (a `--target`/`--prop` nevezi meg a forrást): átfedő akció / follow-through (a `--delay` láncszemenként képkockákban, alapértelmezés 3; a `--decay` láncszemenként, alapértelmezés 0.7). A késleltetett másolatot a projekt utolsó képkockájánál elvágja (a `frames` képkocka után nem ír kulcsot, így a `lint` L007 nem jelez); az utolsó megtartott kulcs és az elvágás közötti képkockákat mintavételezi, hogy a késleltetett görbe minden egész képkockán pontos legyen.

```
$ osatura follow small.json f1.json --target arm_up --prop bone_angle --chain arm_low,hand --delay 2 --decay 0.6
wrote f1.json (+2 channels)
```

**Útvonalkövetés (pozicionális):** `osatura follow <project.json> <out.json> <layer|bone:NAME> <layer_with_path> <start_frame> <end_frame> [no-orient]`
Beéget egy réteget (vagy a `bone:NAME` gyökércsontot), amely két képkocka között egy másik réteg első útvonala mentén halad, és az útvonal érintőjéhez forog, hacsak nincs megadva a `no-orient`.

```
$ osatura follow small.json fp.json target ink 0 7
wrote fp.json
$ osatura follow small.json fp2.json bone:hand ink 0 7 no-orient
path following needs a root bone (no parent)                      [exit 1]
```

### `take-record`, `take-apply`, `take-list`

**Synopsis:**
```
osatura take-record <project.json> <library.json> <name> <from> <to> [target_prefix,...]
osatura take-apply  <project.json> <out.json> <library.json> <name> <at> [from=to,...] [speed] [--blend FRAMES]
osatura take-list   <library.json>
```
A **take** az animáció egy újrafelhasználható szelete, amelyet egy könyvtárfájl tárol. A `take-record` rögzíti azon csatornák `from..to` képkockáit, amelyek célja valamelyik előtaggal kezdődik (elhagyva minden csatornát); a könyvtárat létrehozza, ha hiányzik, az azonos nevű take-et pedig lecseréli. A `take-apply` a take-et az `at` képkockára illeszti, opcionálisan **retargetelve** (a `from=to` párok átnevezik a célokat) és a `speed` módosításával; az argumentumokat szigorúan ellenőrzi: az `=` nélküli elem, az olyan forrásnév, amelyhez a take-nek nincs sávja, vagy a nem pozitív szám sebesség hiba, az olvashatatlan könyvtár pedig megnevezi a fájlt (`cannot read the take library ...`); a `--blend N` a meglévő animációval N képkockán át, mindkét végén átúsztat.

```
$ osatura take-record demo.json lib.json wave 0 24 arm_
recorded 2 tracks into lib.json
$ osatura take-list lib.json
wave (24 frames, 2 tracks)
$ osatura take-apply demo.json ta.json lib.json wave 10 arm_up=arm_low 1.5 --blend 4
applied 2 tracks; wrote ta.json
```

### `clip`

**Synopsis:**
```
osatura clip add <project.json> <out.json> <library.json> <take> <clip> [--at F --speed S --loops N --mode replace|additive|multiply --weight W
                   --include a_*,b --exclude c --track-weight arm_*=1,leg_*=0.2 --rename a=b --fade-in F --fade-out F]
osatura clip list <project.json>
osatura clip remove <project.json> <out.json> <clip>
osatura clip set-weight <project.json> <out.json> <clip> [W] [--keys frame:w,frame:w] [--track PATTERN[:prop]]
osatura clip bake <project.json> <out.json>
```
Rétegzett **klipköteg** (nemlineáris animáció): a klipek egy könyvtár take-jeire hivatkoznak, az `--at` pontnál játszódnak le `--speed`/`--loops` beállítással, és `replace`, `additive` vagy `multiply` móddal keverednek egy súllyal (állandó, időben kulcsolt, vagy sávminta szerinti). A `clip add/set-weight/remove` csak a köteget szerkeszti; a **`clip bake` újragenerálja a köteg tulajdonában lévő csatornákat** (a kézzel készített és a zárolt csatornákhoz sosem nyúl).

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
(A `clip list` példa az azonos nevű második `clip add` eredményét mutatja, amely felülírta az elsőt.) **Korlátok (ROADMAP §9):** a beégetett csatornákat egész képkockánként mintavételezi; nincs `clip refresh`, amely újra behúzná egy szerkesztett könyvtári take-et.

### `make-loop`

**Synopsis:** `osatura make-loop <project.json> <out.json> [targets|all] [props] [--inclusive]`

A kézzel kulcsolt `replace` csatornákat úgy alakítja, hogy ott érjenek véget, ahol kezdődtek: minden görbét képkockánként mintavételez, kivonja belőle az első és az utolsó értéke közötti egyenest (sodródás eltávolítása), majd visszaadja az első értéket. Egy haladó csont X eltolására alkalmazva a sétát helyben járássá alakítja. A hurok periódusa N = a projekt `frames` értéke, így a `0..N-1` ismételt lejátszása nem mutat duplikált illesztési képkockát. Az `--inclusive` a régi konvenció, amelyben az utolsó képkocka megegyezik az elsővel. Argumentumok nélkül minden kulcsolt csatornát feldolgoz. Módosított csatornánként egy `removed drift` sort ír ki.

```
$ osatura make-loop demo.json ml2.json foot_ctrl bone_offset_x
foot_ctrl BoneOffsetX: removed drift 0.79
wrote ml2.json
$ osatura make-loop demo.json ml.json all bone_angle --inclusive
```
**Korlátok:** tankönyvi lineáris trendeltávolítás, nem a mozgás újraidőzítése; a kifejezéssel meghajtott csatornákhoz nem nyúl. **A haladást szándékosan eltávolítja** (a `0..N-1` tartományt ismételten lejátszó projekt nem haladhat örökké, ezért a hurok helyben van): az a csatorna, amelynek teljes változása haladás volt, állandóvá válik, és a parancs egy `note: ... is now constant` sort ír ki róla; a `lint` ezt a csatornát ezután L005-ként (információ) jelzi, ami várható. Nevezd meg a célokat és a tulajdonságokat, hogy a többi csatorna haladása érintetlen maradjon, vagy használd a `loop-solve` parancsot az eleve ciklikus mozgáshoz.

### `loop-solve`

**Synopsis:** `osatura loop-solve <project.json> <out.json> [--window 24] [--inclusive]`

Megoldja az **illesztést** (seam): a kulcsolt csatornák végét egy ablakon belül beleolvasztja az elejükbe, hogy az animáció ugrás nélkül ismétlődjön. Kiírja az illesztési hibát előtte és utána.

```
$ osatura loop-solve demo.json ls.json --window 24
wrote ls.json: 8 channel(s) adjusted, seam 0.000 -> 0.000
```
Az `--window` képkockákban értendő (alapértelmezés 24). Hasonlítsd össze a `lint` `loop` szerződésével (`contracts`).

### `walk`

**Synopsis:** `osatura walk <project.json> <out.json> --thigh-l B --thigh-r B [--shin-l B --shin-r B --arm-l B --arm-r B --hips B --period N --stride DEG --knee DEG --bob PX --swing DEG]`

Ismétlődő járásciklust generál kulcsolt csatornákként a megnevezett csontokon (legalább egy csontopció kötelező: `name at least one bone (--thigh-l, --thigh-r, ...)`). Alapértékek: `--period 24` képkocka, `--stride 28`, `--knee 40`, `--bob 4`, `--swing 22`. A csatornák hurok-extrapolációt használnak, és **hozzáfűzödnek**.

```
$ osatura walk small.json wk.json --thigh-l arm_up --thigh-r arm_low --period 8 --stride 20
wrote wk.json (+2 channels)
```

### `mirror`

**Synopsis:** `osatura mirror <project.json> <out.json>`

Az összes kulcsolt csatornát tükrözi a függőleges tengelyen: a célok oldalt cserélnek (`L_`/`R_` előtagok, `_L`/`_R` és `_l`/`_r` utótagok, a `left`/`right` szavak), a forgások és a vízszintes eltolások előjelet váltanak. A tükrözött csatornák **lecserélik** a projekt csatornáit (nem az eredetiek mellé kerülnek). Az eredménysor `wrote X (N of M channel target(s) swapped sides)`; ha egyik célnak sincs bal/jobb neve, vagy a projektnek nincsenek csatornái, a stderren egy figyelmeztetés jelzi ezt (csak az előjelek fordulnak meg).

### `foot-lock`, `stretch`, `magnet`

```
osatura foot-lock <project.json> <out.json> <ctrl_bone> <ground_y> [tolerance] [min_frames]
osatura stretch   <project.json> <out.json> <follower_bone> <leader_bone> [min=1] [max=3]
osatura magnet    <project.json> <out.json> <ctrl_bone> <bone:NAME|x,y> <radius> <strength 0..1>
```

* A **`foot-lock`** minden képkockán mintavételezi a vezérlő világpozícióját; azok a legalább `min_frames` képkockányi szakaszok, amelyeken a talajon van (a színpad y tengelye lefelé mutat: a láb lent van, ha `y >= ground_y - tolerance`), az *érintkezések*, és egy **additív** korrekciós csatorna odarögzíti a vezérlőt, ahol az egyes érintkezések kezdődtek. Az eredeti animáció érintetlen marad; a visszavonáshoz töröld a hozzáadott csatornákat.
  ```
  $ osatura foot-lock demo.json fl.json foot_ctrl 385 2 2
  contact frames 23..47 pinned at (479.2, 384.1)
  wrote fl.json
  ```
* A **`stretch`** a `follower_bone` csontot a `leader_bone` fejére irányítja, és (egyenletesen, a `min` és a `max` között, alapértékek 1 és 3) úgy skálázza, hogy a hegye elérje azt. Additív szögcsatornaként plusz szorzó skálacsatornaként van beégetve; a követő gyermekeit vele együtt skálázza. A vezető nem függhet a követőtől (`the leader depends on the follower (it is one of its descendants)`).
  ```
  $ osatura stretch p.json st.json flw lead 1 2
  corrected 8 frames
  wrote st.json
  ```
* A **`magnet`** egy vezérlő közelségi vonzását égeti be egy csont feje (képkockánként követve) vagy egy rögzített pont felé: sima lecsengés, a célnál teljes `strength` (0..1), a `radius` távolságnál nulla. Additív eltolás-csatornákként van beégetve.
  ```
  $ osatura magnet p.json mg.json lead 200,100 200 0.8
  pulled on 8 frames
  wrote mg.json
  ```
**Korlátok (ROADMAP §9):** mindhárom képkockánkénti mintavételezésből származó beégetett eredmény: a forrásanimáció szerkesztése után futtasd újra. A `stretch` egyenletes skálát használ. A `foot-lock` és a `magnet` gyökér vezérlőcsontot vár.

### `shot`, `camera-seq`

```
osatura shot       <project.json> <out.json> <name> <x> <y> <zoom> [rotation]
osatura camera-seq <project.json> <out.json> <frame:shot[:cut],...>
```
A `shot` elnevezett **kameraállást** ment (középpont x, y, zoom, opcionálisan forgás fokban). A `camera-seq` kulcsolja a kamerát a mentett állásokon át: minden `frame` képkockán a kamera (easinggel) az állásra mozog, vagy a `:cut` utótaggal odaugrik. `camera_x/y/zoom/rotation` csatornákat ír.

```
$ osatura shot small.json sh.json wide 320 240 1 15
wrote sh.json (1 shots)
$ osatura shot sh.json sh2.json close 400 200 2
wrote sh2.json (2 shots)
$ osatura camera-seq sh2.json cs.json 0:wide,4:close:cut
wrote cs.json
```

### `spring-fit`

**Synopsis:** `osatura spring-fit --start V --samples a,b,c,... [--fps 24]`

Rugó **frekvenciát (Hz) és csillapítási arányt** illeszt egy megrajzolt válaszgörbére (a `--start` értéktől az utolsó értékig tartó lépcső képkockánkénti mintái). Kiírja az illesztett párt és a projekt `springs` részébe illesztendő JSON-t. Nem olvas projektet.

```
$ osatura spring-fit --start 0 --samples 0.5,0.9,1.1,1.0,0.98,1.0 --fps 24
frequency 4.68 Hz, damping 0.61 (rms error 0.028 of the step size)
add to the project: {"bone": "<bone>", "frequency": 4.68, "damping": 0.61}
```

### `script`

**Synopsis:** `osatura script <project.json> <out.json> <script.rhai>`

**Homokozóban futó Rhai-szkriptet** futtat, amely érvényesített kéréseken keresztül szerkeszti a projektet (az elutasított kérés mindent változatlanul hagy:
`request 5 (box.LayerOpacity at 0) was rejected, nothing changed: ...`). Beépített függvények: `fps() frames() bones() layers() set_key(target, prop, frame, value[, ease]) add_bone(name, parent (empty = root), length, x, y, angle) add_rect add_circle add_polygon|add_blob add_line add_gradient_rect add_ik add_aim add_copy_rotation add_pin add_group(name[, [layers]]) add_switch move_layer set_parent delete_layer delete_bone set_layer set_bone add_channel_expr add_noise add_marker add_event bone_info layer_info world print`.
Az a szkript, amely egyetlen kérést sem alkalmaz, `warning: script: 0 requests were applied` üzenetet ír. A `scripting` feature nélküli buildek ezt válaszolják: `this build has no scripting support`.

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
A csoporton belüli rétegekre útvonallal hivatkozunk (`skins/box`). A `print` a stdoutra ír.

---

## 7. Fizika, ruha és deformerek

Ezek mind **beégetnek** egy szimulációt vagy geometriai eredményt közönséges kulcsolt csatornákba: a projekt determinisztikus marad, az eredmény pedig bárhol lejátszható (ahol értelmezhető, a Lottie/GLB formátumokban is). A forrásanimáció módosítása után futtasd újra a parancsot.
Bemenetük `<project.json> <out.json>` és `key=value` szavak (`--` nélkül).

### `physics`

**Synopsis:**
```
osatura physics <project.json> <out.json> <bone,bone,...> [gravity=900] [wind=x,y] [gust=0.5] [damping=0.985]
    [circle=x,y,r[@bone];...] [capsule=x1,y1,x2,y2,r[@bone];...] [follow=bone] [--allow-empty]
    [drive=bone:gain;...] [vortex=bone|x,y:swirl[:radial[:radius]];...] [preset=hair|tail|rope|antenna|cape]
    [merge=keep|replace] [lateral=0..1]
```
Verlet-féle **lánc-szimulációt** (haj, farok, kötél, antenna, köpeny) éget be a felsorolt csontokra (a lánc gyökere elöl), `bone_angle` kulcsokként, képkockánként egyet.

| Word | Meaning |
|---|---|
| `gravity=` | Lefelé irányuló gyorsulás (alapértelmezés 900 színpadegység/s^2). |
| `wind=x,y`, `gust=`, | Állandó szélvektor; a zajjal modulált szél széllökés-amplitúdója. |
| `damping=` | Sebességcsillapítás lépésenként (alapértelmezés 0.985). |
| `circle=x,y,r[@bone];...`, `capsule=x1,y1,x2,y2,r[@bone];...` | Ütközők, opcionálisan egy csonthoz csatolva (követik azt). A kapszulák elfogadottak (ellenőrizve a `capsule=100,300,300,300,20` értékkel; a ROADMAP §9 régebbi „capsule colliders NOT done” megjegyzése elavult). |
| `follow=bone` | A lánc gyökerét ennek a csontnak a mozgása viszi magával (tehetetlenség). |
| `drive=bone:gain;...` | A láncot toló testek (sebességük szorozva a gain értékkel). |
| `vortex=bone\|x,y:swirl[:radial[:radius]];...` | Örvénymezők egy csont vagy pont körül. |
| `preset=` | Kezdőértékek a `hair tail rope antenna cape` készletekhez; a `preset=` **után** írt szavak felülírják. |
| `merge=keep\|replace` | A `keep` (alapértelmezés) érintetlenül hagyja azokat a csontokat, amelyeknek már van animációjuk; a `replace` felülírja őket. |
| `lateral=0..1` | Több lánc esetén (`bone,bone/bone,bone`) összekapcsolja a szomszédos szálakat. |
| `--allow-empty` | Elfogad olyan futást, amely semmit sem égetett be. |

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
(a `p.json` a demó plusz három, kulcsolatlan `tail1..tail3` csont, amelyeket Rhai-szkripttel készítettem - lásd `script`.) **Buktató:** az a beégetés, amely semmit sem változtat, hiba, hacsak nincs `--allow-empty`.
**Korlátok (ROADMAP §9):** az ütközők figyelmen kívül hagyják a csont skáláját (csak merev transzformáció); a statikus körök csak pontokban ütköznek (a csatolt körök a szakaszok mentén is); nincs kapszula-kapszula és önütközés.

### `cloth`

**Synopsis:**
```
osatura cloth <project.json> <out.json> <layer> [pin=top,left@bone,row:N,col:N,v:N,corners,none] [gravity=900]
    [wind=x,y] [gust=0.5] [gust-hz=0.5] [gust-len=px] [stiffness=1] [shear=0.05] [bend=0.2] [max-stretch=1.1]
    [damping=0.99] [cols=N rows=N] [rect=x,y,w,h] [name=cloth] [substeps=4] [iterations=10] [preroll=frames]
    [circle=x,y,r[@bone];...] [capsule=x1,y1,x2,y2,r[@bone];...] [from=F] [to=F]
```
Determinisztikus **ruhafelületet** éget be egy *kép*réteg meshére, vagy egy *vektor*rétegre, ha meg van adva a `rect=x,y,w,h` (a vektorréteg rácsmeshet kap az adott téglalap fölött), csúcsonkénti deformcsatornákként. A `pin=` választja ki a rögzített csúcsokat (a `top`, `left`... élek, `corners`, `row:N`, `col:N`, `v:N`, a `@bone` a rögzítéseket egy csonthoz csatolja, `none`). A `from`/`to` korlátozza a beégetett képkocka-tartományt, a `preroll` a `from` képkocka előtt futtatja a szimulációt, hogy a ruha nyugalomból induljon.

```
$ osatura cloth small.json cloth.json sun rect=100,60,120,120 cols=6 rows=6 pin=top gravity=900 wind=40,0
baked cloth on 'sun' (84 channels); wrote cloth.json
$ osatura cloth small.json cloth.json sun
a vector layer needs rect=x,y,w,h                    [exit 1]
$ osatura cloth small.json cloth.json nosuch
no layer at 'nosuch'                                 [exit 1]
```
A `84 channels` a 7x7-es rács 42 nem rögzített pontjának `lattice_dx` / `lattice_dy` kulcsai (cél: `sun::cloth::<col>,<row>`; a felső sor rögzített). Valódi csatornák a fájlban, ezért a ruha beégetése megnöveli a projektet (itt 98 csatorna a 14 helyett) - a költséget a `size-report` mutatja. **Korlátok (ROADMAP §9):** a ruha síkbeli (nincsenek síkon kívüli ráncok, ezért a nyírás alapértelmezés szerint engedékeny); elutasítja a szabad formájú mesheket (csak rácsmeshek); nincs önütközés; egész képkockánként égeti be.

### `cage`

**Synopsis:** `osatura cage <project.json> <out.json> <layer> [rect=x,y,w,h|auto] [hull] [verts=N] [pad=P] [name=cage]`

**Mean-value-coordinate cage deformert** ad egy réteg köré: a cage-en belüli tartalom követi a cage csúcsait, a kívüli változatlan marad. Az `auto` (alapértelmezés, ha nincs `rect`) a kitöltött határokat használja, a `verts=N` az oldalankénti csúcsok száma, a `hull` a konvex burkot használja, a `pad=` kitölti a dobozt, a `name=` nevezi el a deformert (alapértelmezés `cage`).

```
$ osatura cage small.json cg.json sun auto verts=3
added cage 'cage' (12 vertices) to 'sun'; wrote cg.json
```
**Korlátok (ROADMAP §9):** nincs automatikus cage-illesztés a kitöltött doboznál vagy a konvex burkon túl (nincs konkáv követés); a cage-en kívül nincs sima lecsengés (látható lépcső jelenik meg, ahol a grafika átlépi a cage határát); a szabad formájú mesheknek nincsenek csúcsonkénti kulcsolt eltolásaik (használj cage-et).

### `cage-pose`

**Synopsis:** `osatura cage-pose <project.json> <out.json> <layer> <frame> [name=cage] [move=i:dx,dy;...] [all=dx,dy] [ease=linear]`

Cage-csúcseltolásokat kulcsol egy képkockán (`cage_dx` / `cage_dy` csatornák). A `move=` egyes csúcsokat tol el, az `all=` minden csúcsot. A zárolt csatornákat elutasítja.

```
$ osatura cage-pose cg.json cp.json sun 4 all=5,0
keyed cage 'cage' on 'sun' at frame 4; wrote cp.json
$ osatura cage-pose cg.json cp.json sun 4 move=0:5,5
keyed cage 'cage' on 'sun' at frame 4; wrote cp.json
```

### `warp-pivot` és `soft-scale`

**Synopsis:**
```
osatura warp-pivot <project.json> <out.json> <layer> <frame> [deformer=NAME] [verts=1,4-7] [region=x,y,w,h]
    [pivot=x,y|bone:NAME|centroid|center] [rotate=DEG] [scale=S|sx,sy] [move=dx,dy] [radius=R] [falloff=smooth] [ease=linear]
osatura soft-scale <project.json> <out.json> <layer> <frame> [deformer=NAME] [verts=..|region=..] scale=S|sx,sy
    [radius=R] [falloff=smooth] [pivot=..]
```
A `warp-pivot` vezérlőpontok egy **kijelölését** (cage, lattice, cloth, pin, spine deformerek; a rács képmesh csúcsonkénti warp-rácsot kap) forgatja/skálázza/mozgatja egy pivot körül, és az eredményt kulcsolja a képkockán. A kijelölést a `verts=` (indexek és tartományok) és/vagy a `region=` adja; az üres kijelölés hiba (`the selection is empty (give verts= and/or region=)`). A `radius=` **lágy lecsengést** ad hozzá (`linear smooth gauss sharp constant`). A `soft-scale` arányos (lágy kijelöléses) skálázás; `verts`/`region` nélkül az egész mesht átméretezi a befoglaló doboz középpontja körül a `scale=F`, `width=W`, `height=H` vagy `size=WxH` szerint, `aspect=keep|free` beállítással.

```
$ osatura warp-pivot cg.json wp.json sun 4 verts=0-3 rotate=15 pivot=centroid radius=30 falloff=gauss
warp-pivot on 'sun' keyed at frame 4; wrote wp.json
$ osatura soft-scale cg.json ss.json sun 4 scale=1.2 pivot=center
soft-scale on 'sun' keyed at frame 4; wrote ss.json
```

### `layer-pivot`

**Synopsis:** `osatura layer-pivot <project.json> <out.json> <layer> pivot=x,y|bone:NAME`

Egy réteg pivotját (horgonyát) mozgatja **a kép megváltoztatása nélkül**, így a forgás- és skálacsatornák az új pont körül dolgoznak.

```
$ osatura layer-pivot small.json lp.json sun pivot=100,100
moved the pivot of 'sun'; wrote lp.json
```

### `mesh-lock`

**Synopsis:** `osatura mesh-lock add <project.json> <out.json> <name> host=LAYER[:path] follower=LAYER[:path] [verts=1,4-7] [strength=1]`
és `osatura mesh-lock list|remove <project.json> [<out.json> <name>]`

A követő csúcsait minden képkockán a **gazdamesh felületéhez** köti (baricentrikus elmozdulás-átvitel), például egy inget, amely követi a testet. A ciklusokat elutasítja; a gazdának és a követőnek azonos `parallax` értékűnek kell lennie (`mesh lock 'lock1': host and follower must have the same parallax`).

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

**Synopsis:** `osatura glue add <project.json> <out.json> <name> a=LAYER[:path] b=LAYER[:path] pairs=i:j;i:j|auto=RADIUS [mode=rigid|snap|offset] [strength=1]`
és `osatura glue list|remove <project.json> [<out.json> <name>]`

A `b` réteg csúcsait az `a` réteg csúcsaihoz ragasztja (b követi a-t). A `pairs=i:j;...` kifejezetten felsorolja a csúcspárokat, az `auto=RADIUS` minden olyan `b` csúcsot párosít, amely egy `a` csúcs sugarán belül van. Módok: a `rigid` megtartja a nyugalmi eltolást az `a` keretében, a `snap` egybeejti őket, az `offset` állandó eltolást tart.

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

**Synopsis:** `osatura mesh-trace <project.json> <out.json> <layer> [density=24] [tolerance=1.5] [smooth=N] [hotspot=x,y,r;...] [edge=D/4] [grading=0.5] [lambda=0.5] [taubin=1] [smooth-boundary=0] [threshold=128] [bones=R] [hot-size=E] [skin=1]`

Egy **képréteg alfa-sziluettjét** változó sűrűségű, szabad háromszögmeshbe követi le (a körvonal mentén, a hot-spotok és a csontok körül sűrűbben), kisimítja, és a csontokra skineli. Kiírja a csúcsok/háromszögek számát és a minőséget.

```
$ osatura mesh-trace o.json mt.json top density=12 tolerance=2 bones=0
wrote mt.json: 9589 vertices (1494 on the outline), 17682 triangles, area 1227677 px2
  quality min 0.484 mean 0.890, min angle 18.3 deg, mean edge 3.0 px at the outline / 13.7 px inside
```
(az `o.json` egy 1280x960-as OpenRaster-import; az alacsonyabb `density` kevesebb, nagyobb háromszöget ad. A futás 5 másodpercig tartott debug buildben.) **Korlátok:** képréteget igényel (alfa-sziluett); az ilyen sűrű mesh nagyra duzzasztja a projektfájlt - ellenőrizd a `size-report` paranccsal. A szabad meshek nem szimulálhatók ruhaként.

### `skin-diffuse`

**Synopsis:** `osatura skin-diffuse <project.json> <layer> <out.json> [--influences 4]`

**Hődiffúziós skin-súlyokat** számol egy meshelt rétegre a projekt csontjaiból, csúcsonként legfeljebb `--influences` (alapértelmezés 4) csontot megtartva. Figyeld az argumentumok sorrendjét: a **réteg a kimenet előtt** áll. A rétegnek meshes képrétegnek kell lennie.

```
$ osatura skin-diffuse demo.json arm sd.json
layer 'arm' is not an image layer                          [exit 1]
$ osatura skin-diffuse o.json top sd.json
layer 'top' has no mesh                                    [exit 1]
$ osatura skin-diffuse mt.json top sd.json               # mt.json from mesh-trace
wrote sd.json
```

---

## 8. Hang, ajaksinkron és hangok

Az elemző parancsok (`lipsync`, `lipsync-text`, `speech-motion`, `audio-react`, `beats`, `waveform`) hangbemenetének **WAV**-nak kell lennie (8/16/24/32 bites egész vagy 32 bites lebegőpontos PCM, tetszőleges csatornaszám monóra keverve, legfeljebb 256 MiB). Az MP3/OGG fájlokat előbb át kell konvertálni (`ffmpeg -i in.mp3 in.wav`). A projektben tárolt hang**sávok** (`sound`) bármilyen, az ffmpeg által olvasható formátumúak lehetnek.

### `lipsync`

**Synopsis:** `osatura lipsync <project.json> <audio.wav> <switch_layer> <out.json> [--map 0,1,2,3,4]`

Egy **switch réteget** (szájformák) hajt meg a hangerőből és a spektrális fényességből: amplitúdó- / nullátmenet-alapú heurisztika, amely képkockánként kiválaszt egyet az öt forma közül (`rest, small, open, round, wide`), és eltávolítja a villogást. **Nem fonémafelismerés.** A `--map` megadja az egyes öt formához tartozó switch-gyermekindexet; az alapértelmezés a switch gyermekeinek számát követi: 1 gyermek -> mind; 2 -> zárt + nyitott; 3 -> zárt + nyitott + kerek/széles; 4 -> zárt + kicsi + nyitott + kerek/széles; 5+ -> a fenti sorrendben. Lecseréli az adott réteg meglévő kulcsolt `layer_switch` csatornáját. A projekt hosszát **nem** változtatja (egy 2 s-os WAV egy 8 képkockás projekten 8 képkockát kulcsol).

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

**Synopsis:** `osatura lipsync-text <project.json> <audio.wav> <transcript.txt> <switch_layer> <out.json> [--map 0,1,2,3,4] [--lang ro|en] [--words-out timing.json|timing.srt]`

Egy **átiratot** igazít a hanghoz (energia + szövegigazítás: a szavakat és szótagokat a hangban talált beszédszakaszokra helyezi), és a szótagokból szájformákat származtat. A `--lang` (alapértelmezés `ro`) választja ki a magánhangzó-/szótagszabályokat; csak a `ro` és az `en` létezik. A `--words-out` JSON-ként írja ki a szóidőzítéseket (szavak szótagokkal), `.srt` név esetén pedig feliratként.

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
**Korlátok (ROADMAP §9):** nincs betűnkénti (fonémaszintű) időzítés, nincsenek beszélőspecifikus sebességmodellek, nincs a ro/en-en kívüli nyelv. A szintetikus hangot beszédként igazítja - használj valódi felvételeket.

### `speech-motion`

**Synopsis:** `osatura speech-motion <project.json> <audio.wav> <transcript.txt> <out.json> [--head bone] [--brow bone_or_layer] [--nod deg] [--fall deg] [--tilt deg] [--brow-px px] [--seed n] [--lang ro|en] [--words-out timing.json|timing.srt]`

**Additív** fejbólintásokat, eséseket, megdöntéseket és szemöldökemeléseket ad hozzá, az igazított szavakhoz időzítve. A `--head` a fej csontját nevezi meg, a `--brow` a szemöldökökhöz tartozó csontot vagy réteget; a szögopciók fokban értendők, a `--brow-px` pixelben, a `--seed` rögzíti a variációt.

```
$ osatura speech-motion small.json speech.wav transcript.txt sm.json --head arm_up --lang en
wrote sm.json (1 additive channel(s) from 3 words)
```

### `mouth`

**Synopsis:** `osatura mouth <project.json> <cues.txt> <switch_layer> <out.json> [--map 0,1,2,3,4] [--rhubarb]`

Kész száj-**cue**-kat importál hang elemzése helyett: Papagayo/Moho-switch fájlokat (soronként `frame PHONEME`, opcionális első sor `MohoSwitch1`) vagy Rhubarb TSV-t (`seconds<TAB>shape`, a tizedesponttal ismerhető fel; a másodperceket a projekt fps értékén váltja át). Az üres sorokat és a `#` megjegyzéseket figyelmen kívül hagyja. A fonémákat az öt formaosztályra képezi le (`rest/MBP/X/A` nyugalom; `etc/FV/L/B/G/H` kicsi; `AI/D` nyitott; `O/U/WQ/F` kerek; `E/C` széles); az ismeretlen formák `small` értékűnek számítanak. A `--rhubarb` a Rhubarb `E` jelét *kerekként* kezeli (a Papagayo `E` jele széles). Az alapértelmezett `--map` `0,1,2,3,4`.

```
$ printf '0 rest\n2 open\n4 round\n6 wide\n' > cues.txt
$ osatura mouth small.json cues.txt mouth mo.json --rhubarb
wrote mo.json (4 cues)
```

### `audio-react`

**Synopsis:** `osatura audio-react <project.json> <out.json> <audio.wav> <target> <prop> [gain=1] [offset=0] [floor=0.05] [attack=1] [release=4] [add=on]`

A hangerő hajt meg egy tulajdonságot: képkockánként egy kulcs, amely a simított szintet követi (RMS a leghangosabb képkockához viszonyítva). Kimenet = (a `floor` által kapuzott szint) x `gain` + `offset`; az `attack`/`release` a felfutási/lecsengési idő képkockákban; az `add=on` a csatornát felülírás helyett additívvá teszi. A zárolt csatornát elutasítja. Az opciók `key=value` szavak; az ismeretlen `unknown option 'foo'` hibát ad.

```
$ osatura audio-react small.json ar.json speech.wav ink layer_rotation gain=10
wrote ar.json
$ osatura audio-react small.json ar.json speech.wav arm_up bone_angle foo=1
unknown option 'foo'                                                [exit 1]
```

### `beats`

**Synopsis:** `osatura beats <project.json> <out.json> <audio.wav> [sensitivity=1.5] [gap=4] [as=markers|events] [name=beat]`

Felismeri a hangerő ugrásait a közelmúlt átlaga fölött (`sensitivity` 1.2-3, minél magasabb, annál kevesebb ütem), egymástól legalább `gap` képkockányira, és idővonal-**markerként** (alapértelmezés) vagy `name` nevű **eseményként** rögzíti őket. Kiírja a megtalált képkockákat.

```
$ osatura beats small.json bt.json speech.wav as=events name=hit sensitivity=1.2
1 beats at frames [1]; wrote bt.json
$ osatura beats small.json bt.json speech.wav
0 beats at frames []; wrote bt.json
```

### `waveform`

**Synopsis:** `osatura waveform <audio.wav> <out.json> [buckets=800]`

Min/max csúcsok hullámforma rajzolásához: `{"sample_rate","duration","peaks":[[min,max],...]}`. Nem olvas projektet.

```
$ osatura waveform speech.wav wf.json buckets=50
wrote wf.json (50 buckets)
```

### `sound`

**Synopsis:**
```
osatura sound list   <project.json>
osatura sound add    <project.json> <out.json> <name> <file.wav|ogg|mp3|...> [start_frame] [gain] [trim/fade options]
osatura sound edit   <project.json> <out.json> <name> [--start F] [--gain G] [trim/fade options]
osatura sound remove <project.json> <out.json> <name>
osatura sound mix    <project.json> <out.wav|out.ogg|out.mp3>
    trim/fade options (seconds, or frames with an f suffix such as 36f): --trim-in T --trim-out T --fade-in T --fade-out T
```
A hangsávok **hivatkozások** (a projekthez viszonyított fájlútvonal, kezdő képkocka, gain 0..4, vágási ablak a fájlon belül, ki- és beúsztatás). A projektbe semmi sem ágyazódik be; a hangfájlnak együtt kell utaznia vele, és a projekt könyvtárán belül kell lennie, hacsak nincs megadva az `--allow-external-assets`. Az `edit` csak a megadott mezőket módosítja (a `0` törli a vágást). A `--trim-out 0` azt jelenti: „a végéig”. A `sound mix` csak az összekevert hangot rendereli a **projekt hosszára** (az ffmpegen keresztül: wav = 44,1 kHz PCM 16 bit, ogg = Vorbis q4, mp3 = 96 kb/s).
Az összekevert hangot az `.mp4` (AAC) és a `.webm` (Opus) is muxolja, a `.html`-be pedig beágyazza; a `.gif`/`.apng` nem tudja hordozni.

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
A `check` hibát jelez, ha egy hivatkozott hangfájl hiányzik vagy olvashatatlan. Az ffprobe nélkül a nem WAV fájloknál csak a létezést ellenőrzi (azt nem, hogy a fájl hang-e). **Korlátok (ROADMAP §9):** nincs sávonkénti pan/EQ/ducking, csak a lineáris fade-görbe, nincs klip ismétlése, nincs újraidőzítés.

---

## 9. Import

### `import-svg`

**Szintaxis:** `osatura import-svg <file.svg> <out.json> [--split]`

Használati sor: `osatura import-svg <file.svg> <out.json> [--split]  SVG subset -> one vector layer, or one layer per shape named by its id (warnings for the rest)`

Új projektet hoz létre (24 fps, 24 képkocka; a méret a `viewBox`-ból vagy a `width`/`height` értékekből származik, egyébként 512x512) **egyetlen `svg` vektorréteggel**, amely egy SVG-részhalmaz alakzatait tartalmazza: `path` (minden parancs), `rect` (a lekerekített sarkokat figyelmen kívül hagyja), `circle`, `ellipse`, `line`, `polyline`, `polygon`, `g`/`a` `transform` attribútummal, valamint egyszínű `fill`/`stroke`/`stroke-width`. Minden mást kihagy, és egy `warning:` sort ír **a stdout-ra** (ha az elem a `<defs>` kívül van; a `<defs>` belüli `<filter>`, `<clipPath>`, `<mask>` vagy `<pattern>`, valamint a `filter`, `clip-path` és `mask` attribútumok figyelmeztetés nélkül kimaradnak): `<text>`, `<filter>`, `<clipPath>`, `<mask>`, `<pattern>`, `<linearGradient>`/`<radialGradient>`, `<image>`, `<use>`, `<foreignObject>`. Az átmenetes kitöltés (`fill="url(#g)"`) szürkére cserélődik. Korlátok: 8 MB SVG, 200 000 horgonypont, az elemek számára felső korlát.

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
A `--split` kapcsolóval minden alakzat **saját réteg** lesz, az alakzat `id`-jéről elnevezve, a fájlbeli sorrendben. Az id nélküli alakzat a címkéjéről és a pozíciójáról kapja a nevét (`rect1`, `circle6`), az ismétlődő id `_1`, `_2`... utótagot kap, a rétegnévben nem megengedett karakterek (`/`, `::`, vezérlőkarakterek) `_`-ra cserélődnek; a nevek 100 karakteresre vannak vágva. Ezt az alakot használd, ha a rajzot részenként fogod riggelni; lásd [TUTORIAL_SVG.md](TUTORIAL_SVG.md).

```
$ osatura import-svg pip.svg pip.json --split
wrote pip.json (16 paths, 16 layers)
```
Importálás, majd egyesítés: nincs olyan parancs, amely a réteget egy meglévő projektbe egyesítené; használd a `script` parancsot, vagy szerkeszd a JSON-t.

### `import-ora`

**Szintaxis:** `osatura import-ora <file.ora> <out.json> [--fps N] [--frames N]`

OpenRaster -> projekt: **rasztereként egy képréteg** (beágyazott PNG), az alulról felfelé haladó sorrend megmarad, a rejtett rétegek megmaradnak, de láthatatlanok, a csoportok ellaposodnak, rétegenként átveszi az `opacity`, `visibility` és pozíció (`x`,`y`) értékeket, valamint a gyakori `svg:` összetételi műveleteket (multiply, screen, overlay, darken, lighten, color-dodge/burn, hard/soft-light, difference, exclusion, plus -> add). A projekt `--frames` képkockából áll (alapértelmezés: 1) `--fps` sebességgel (alapértelmezés: 24). Egy beágyazott köteg átlátszósága összeszorzódik a rétegeivel, és a keverési módját örökli, de a áttetsző köteg belsejében átfedő rétegek **nem** lapulnak ki előbb (átlátszanak egymáson). Az azonos nevű rétegek `_1`, `_2` utótagot kapnak.

```
$ osatura import-ora t.ora ora.json
wrote ora.json (2 layers)
$ osatura import-ora t.ora ora2.json --fps 12 --frames 24
$ osatura import-ora layer.png bad.json
not a zip/ORA file: invalid Zip archive: Could not find EOCD       [exit 1]
```
Korlátok: 2000 archívumbejegyzés, bejegyzésenként 128 MiB, 64 megapixeles rétegek.

### `import-lottie`

**Szintaxis:** `osatura import-lottie <in.json> <out.json> [fx.json]`

Egy Lottie (Bodymovin) fájlt - szándékosan **részhalmazt** - importál projektként. Kiírja: `imported N layers; wrote <out>`, valamint minden kihagyott funkcióhoz egy `warning:` sort.

* **Importált:** `sh` útvonalú alakzatrétegek, `rc`, `el`, egyszínű és átmenetes kitöltések (az animált átmenet végpontjai/középpontja/sugara `grad_dx/grad_dy` csatornákká válnak), körvonalak, csoportok, réteg-transzformációk (statikus vagy kulcsolt pozíció/skála/forgatás/átlátszóság, a görbe pozícióútvonalak képkockánként beégetve), keverési módok, egyszínű háttérréteg, beágyazott PNG/JPEG data URI-kat tartalmazó képrétegek, null rétegek szülőként (a szülő transzformációi beégetődnek a gyerekekbe), réteg-maszkok (add/subtract/intersect átlátszósággal, invertálással, kulcsolt alakzatokkal), luma matte-ek (közelítve), előkompozíciók, a réteg be/ki tartományai (átlátszósági lépcsős sávként), kulcs-easing (hold -> step; bezier fogantyúk -> cubic-bezier; fogantyú nélkül -> lineáris).
* **Animált útvonalak:** az az útvonal, amelynek alakkulcsai azonos topológiájúak (azonos horgonyszám, nyitott/zárt), egy alap-útvonalat kap, plusz minden további eltérő kulcshoz egy **morph target**-et (`k1`, `k2`, ...), `path_morph` súlycsatornákkal, amelyek az easinget hordozzák. Eltérő topológia esetén az első alakot használja, és figyelmeztetést ír ki.
* **Effektek (`ef`):** a Gauss-elmosás, az árnyék, a tint és a fill **rétegenkénti effektekké** válnak (`effects` a rétegen, bármilyen rétegfajtán), az animált számok `fx_*` csatornák lesznek; az alakzatrétegeken lévő statikus tint/fill/tritone/levels színekbe égetődik. Az effektszínek nem animálhatók (az első értéket használja); más effektek, valamint a nem alakzatrétegeken elmosás/árnyék után álló színeffektek figyelmeztetést okoznak. Rétegenként csak az első néhány effekt marad meg.
* **Figyelmeztetéssel kihagyva:** szöveg és más rétegtípusok, hangeszközök, animált horgonypontok (az első érték), a pozíciótól eltérő tulajdonságok görbe mozgásútvonalai (egyenes szakaszok), track-matte forráson lévő maszkok, a beégetéshez túl hosszú szülőláncok, 2000-nél több réteg, 20 000 képkockánál hosszabb animáció.
* Az `fx.json` kompatibilitási okból megmaradt: üres tömb íródik bele (az elmosás és az árnyék ma már rétegenkénti effekt a projekten belül).

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
**Figyelem:** az `import-lottie in.json in.json` parancsot elutasítja (`refusing to overwrite the input ...`), az `fx.json` esetén is. **Korlátok:** a `.lottie` (zip) bemenetet nem olvassa (`stream did not contain valid UTF-8`) - előbb csomagold ki az `animations/animation.json` fájlt; az alakzat-morphok négyzetes munkáját csak a méretkorlátok szabják meg.

### `import-bvh`

**Szintaxis:** `osatura import-bvh <project.json> <file.bvh> <out.json> --joint-map Joint=bone,.. [--axis Zrotation --tol 0.5]`

A kiválasztott BVH-ízület forgatáscsatornáit rig-csontokra képezi le, kulcsolt `bone_angle` csatornákként, a projekt fps-én és képkockaszámán mintavételezve. A `--joint-map` kötelező; az `--axis` (alapértelmezés: `Zrotation`) azt választja ki, melyik BVH-forgatáscsatornát (`Zrotation`, `Xrotation`, `Yrotation`) olvassa; a `--tol` (alapértelmezés: 0,5 fok) egyszerűsíti a kulcsokat. Automatikus párosításhoz, gyökérmozgáshoz, nézetekhez és skálázáshoz használd a `mocap-retarget` parancsot (10. szakasz).

```
$ osatura import-bvh demo.json m.bvh ib.json --joint-map LeftUpLeg=thigh,LeftLeg=shin --axis Zrotation --tol 0.5
wrote ib.json (+2 channels from 4 BVH frames)
```

### `import-seq`

**Szintaxis:** `osatura import-seq <project.json> <out.json> <layer_name> <folder> [hold=1] [loop=on|off] [start=0] [x=0] [y=0]`

Egy **PNG-sorozatot** (a mappa összes PNG-je fájlnév szerinti sorrendben) importál képkockánkénti rétegként: egy `<layer_name>` nevű kapcsolórétegként, képkockánként egy képréteggel és egy step csatornával, amely mindegyiket `hold` képkockán át mutatja a `start` képkockától kezdve, az `(x, y)` pozícióban; a `loop=on` megismétli. Minden képnek azonos méretűnek kell lennie (`img.png: size 640x480 differs from the first image (160x120)`).

```
$ osatura import-seq demo.json is.json frames seq hold=2 loop=on start=4 x=10 y=20
imported 3 images; wrote is.json
```

### `import-anim`

**Szintaxis:** `osatura import-anim <project.json> <out.json> <layer_name> <file.gif|file.apng|file.png> [loop=on|off] [start=0] [x=0] [y=0]`

Egy **animált GIF-et vagy APNG-t** importál képkockánkénti rétegként, a fájl saját képkocka-késleltetéseit használva (projekt-képkockákra átváltva). Egy sima PNG egy képkockát ad.

```
$ osatura import-anim demo.json ia.json gifl dof.gif loop=off start=0
imported 4 frames as layer 'gifl'; wrote ia.json
$ osatura import-anim demo.json ia2.json apngl dof.apng
imported 3 frames as layer 'apngl'; wrote ia2.json
```
A képkockák beágyazott PNG-kké válnak: egy hosszú GIF nagy projektet eredményez.

### `trace`

**Szintaxis:** `osatura trace <image.png> <out.json> [--threshold 128 --tolerance 1.0 --min-area 12 --fill #000000]`

Egyszerű bittérkép -> vektor **átlátszóságot** tartalmazó képekhez: a PNG **alfacsatornáját** követi (marching squares a `--threshold` küszöbnél, 0..255, lineáris élinterpolációval), a kontúrokat `--tolerance` pixelre egyszerűsíti (Ramer-Douglas-Peucker), eldobja a `--min-area` alatti régiókat, a lyukakat tartalmazás alapján ismeri fel, és új projektet ír (neve `traced`, a vászon a kép mérete), a kontúrokkal kitöltött útvonalakként `--fill` színnel. A teljesen átlátszatlan kép ezért egyetlen, az egész képet körbevevő kontúrt ad (mint a példában). Kiírja: `wrote <out> (N contours)`. Színes képekhez használd az `autotrace` parancsot.

```
$ osatura trace img.png tr.json --threshold 128 --tolerance 1.0 --min-area 12 --fill '#112233'
wrote tr.json (1 contours)
```

### `autotrace`

**Szintaxis:** `osatura autotrace <image.png|jpg> <out.json|out.svg> [--mode fill|line] [--colors N=8] [--min-area A=8] [--tol T=1] [--smooth N=3] [--corner DEG=55] [--denoise N=0] [--quant kmeans|mediancut] [--seed S] [--fit bezier|polygon|vw] [--zorder area|luminance] [--threshold T] [--invert] [--prune N=4] [--into project.json] [--name L] [--fit-stage] [--png preview.png]`

Helyi bittérkép-követés - **modell és hálózat nélkül**: színkvantálás (`--colors`, k-means vagy median cut), kontúrkinyerés, Schneider-féle köbös Bezier-illesztés (`--fit`; alternatíva a `polygon` és a Visvalingam-Whyatt `vw`), lyukak páros-páratlan szabállyal; a `--mode line` a vonalakat középvonalakra vékonyítja (Zhang-Suen, a `--prune` eltávolítja a rövid ágakat). A kimenet egy új projekt, egy új `.svg`, vagy - a `--into project.json` esetén - egy meglévő projekt másolata egy új `--name` nevű (alapértelmezés: `trace`) rétegcsoporttal; a `--fit-stage` (az `--into` mellett) egyenletesen átméretezi az új csoportot, hogy ráférjen a projekt színpadára, és középre igazítja rajta, a `--png` előnézeti képet ír. Kiírja a darabszámokat és a **forráshoz mért hibát** (átlagos abszolút hiba, RMSE, PSNR, SSIM).

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
**Korlátok (ROADMAP §9):** klasszikus követő - a fényképekből színsávok lesznek; nincs tanult vagy szemantikus követés. A `--png` itt értékkel rendelkező opció (az előnézeti fájl neve), a többi `--png` kapcsolóval ellentétben.

### `autotrace-bench`

**Szintaxis:** `osatura autotrace-bench`

Beépített szintetikus alakzatokat, átmeneteket és vonalakat követ, és hűségtáblázatot ír ki (MAE, RMSE, PSNR, SSIM, régió-/horgonyszám, ezredmásodpercek). Nem fogad argumentumot; csak olvas.

```
$ osatura autotrace-bench
image          mode  colors  tol     MAE    RMSE   PSNR   SSIM regions strokes  anchors      ms
shapes         fill       6    1    2.19   12.10   26.5  0.975      11       0       33    13.6
...
```

---

## 10. Élő bemenet, MIDI, arckövetés és mozgásrögzítés

### `live-map`

**Szintaxis:** `osatura live-map <project.json> <events.json|song.mid> <mapping.json> <out.json> [--takes lib.json] [--tol 0.01] [--allow-empty]`

Egy **rögzített élő előadást** (MIDI, egér, gamepad, billentyűk, arcjelek) közönséges kulcsolt csatornákká, eseményekké és take-ekké alakít (nem romboló, mindent vagy semmit, a zárolásokat tiszteletben tartja). A bemenet vagy egy `.mid` Standard MIDI File, vagy egy események JSON-ja: `{"events":[{"source","id","value","time"}]}` (a `value` 0..1-re normalizált, a `time` másodpercben; `source` = `midi_cc midi_note midi_pitch_bend mouse_x mouse_y mouse_button gamepad_axis gamepad_button key face`; `id` = MIDI esetén `"<channel 1-16>:<number>"`, egérgomb esetén `"0"`, gamepad esetén `"<pad>:<index>"`, billentyűnél DOM-kód, például `KeyA`).

A leképezőfájl `params` részt (source/id -> `target`+`prop`, a `in_min in_max out_min out_max deadzone invert curve smooth additive` mezőkkel) és `triggers` részt (küszöbátlépés -> `jump_to_frame`, `play_take`, `set_key`, ... művelet) tartalmaz; az ismeretlen kulcsok hibák. A `--takes` a `play_take` műveletekhez használt take-könyvtár (léteznie kell), a `--tol` egyszerűsíti a kulcsokat.

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
**Figyelem:** ha az eseményeket beolvasta, de **semmi sem keletkezett** (elírt id, rossz source), a parancs hibával leáll, és nem ír semmit, hacsak nincs `--allow-empty`; nulla bemeneti esemény esetén egyszerűen a változatlan projektet írja ki. Az ugró triggerek egy `jump` nevű eseményként rögzülnek `data.to` mezővel (az időtengely nem íródik át).
**Korlátok (ROADMAP §9):** ez a parancs nem biztosít webkamerás/hangszintes élő bemenetet a stúdióban (az `audio-react` offline); az interaktív oldalt a stúdió adja.

### `drive`

**Szintaxis:** `osatura drive <project.json> <signals.csv> <maps.json> <out.json> [--tol 0.5]`

Előadás -> kulcsok egy **CSV**-ből: a fejléc sor nevezi meg a jeleket, minden további sor egy képkocka (sorindex = képkocka). A `maps.json` `{input, target, prop, in_min, in_max, out_min, out_max, deadzone, invert, curve, smooth}` objektumok tömbje; a `--tol` (alapértelmezés: 0,5) egyszerűsíti a kulcsokat.

```
$ printf 'jaw,speed\n0,0\n0.5,0.2\n1,0.4\n0.5,0.6\n' > sig.csv
$ echo '[{"input":"jaw","target":"arm_low","prop":"bone_angle","out_min":0,"out_max":40,"smooth":0.2}]' > maps.json
$ osatura drive demo.json sig.csv maps.json dr.json --tol 0.5
wrote dr.json (+1 channels)
```

### `face-synth`, `face-train`, `face-eval`, `face-detect`, `face-track`, `face-calibrate`, `face-bake`

Arc által vezérelt folyamat, amely **teljesen helyi, és kizárólag szintetikus arcokon tanított** (stilizált/rajzfilmfejek, 23 landmark, `osatura23` elrendezés). A binárisba egy kicsi, szintetikusan tanított modell van beépítve; a `--model` felülbírálja. **A fényképek, webkamerák és valódi videók nincsenek validálva** (ROADMAP §9; lásd még: [PATENT_NOTES.md](PATENT_NOTES.md): az arc által vezérelt bábozás nem esett át szabadalmi vizsgálaton).

| Parancs | Szintaxis | Mit csinál |
|---|---|---|
| `face-synth` | `osatura face-synth <out-dir> [--frames 90] [--size 160] [--seed 7] [--fps 30]` | Szintetikus fejanimáció: `000000.png...` PNG-képkockák, `truth.ndjson` (valódi landmarkok) és `params.json` (valódi jelek); az `--fps` az a képkockasebesség, amelyhez a jelek időzítve vannak (az NDJSON fejlécébe íródik). |
| `face-train` | `osatura face-train --samples N --seed S --out model.bin [--mine-rounds 1] [--scene N] [--reg-win N] [--reg-cell N] [--det-win N] [--det-cell N] [--ridge-lambda X] [--stages N] [--refine-win N] [--stage-patch N] [--stage-cell N] [--stage-lambda X]` | A saját detektort tanítja kizárólag szintetikus arcokon (letöltés nélkül). A `--scene` a tanítókép mérete (64..512); a `--reg-win`, `--reg-cell`, `--det-win`, `--det-cell`, `--ridge-lambda`, `--stages`, `--refine-win`, `--stage-patch`, `--stage-cell` és `--stage-lambda` a modellméretet és a regressziót hangoló paraméterek (az alapértékekkel tanították a beépített modellt). |
| `face-eval` | `osatura face-eval [--model m.bin] [--eval-n 200] [--seed 9] [--scene 128] [--report r.json] [--json]` | Kihagyott szintetikus pontosság, hamis pozitívok és robusztussági átvizsgálások (JSON; a `--report` fájlba írja, a `--json` a `--report` mellett is kiírja; a `--scene` a tesztkép mérete). |
| `face-detect` | `osatura face-detect <image> [--model m.bin] [--json] [--overlay o.png] [--threshold T] [--max-faces N] [--min-side N] [--max-side N] [--max-dim N]` | Arcdobozok és 23 landmark; az `--overlay` egy PNG-re rajzolja őket; a `--min-side` / `--max-side` az arcméretet korlátozza pixelben, a `--max-dim` pedig előbb lekicsinyíti a nagy képet. |
| `face-track` | `osatura face-track <video-or-frame-dir> <out.ndjson> [--model m.bin] [--fps 30] [--max-dim 256]` | Landmark-NDJSON a `face-bake` számára. A képek könyvtárát közvetlenül olvassa; a videó az ffmpegen keresztül megy. |
| `face-calibrate` | `osatura face-calibrate <landmarks.ndjson> <out.json> [--calibrate-frames N] [--map mapfile.json] [--fps F] [--layout L]` | Kiszámítja a semleges arc kalibrációját az első N képkockából. |
| `face-bake` | `osatura face-bake <project.json> <landmarks.ndjson> <mapfile.json> <out.json> [--calibrate-frames N\|0] [--calib calib.json] [--fps F] [--layout L] [--tol T] [--allow-empty]` | Az arcjeleket (`mouth_open`, `eye_l_open`, `eye_r_open`, `brow_raise`, `yaw`, `pitch`, `roll`, ...) csatornákba égeti egy `live-map` stílusú leképezőfájllal, amelynek `source` értéke `face`. |

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
Ezek a parancsok a modell-/jelentésfájlokat az atomi, szimbolikus linket elutasító íróval írják (a szülőmappák létrejönnek). A `face-bake` és a `live-map` a `+N channels (K keys in them), M trigger keys set` összesítést adja, és a stderr-en figyelmeztet, ha a felvétel hosszabb a projektnél (a többletképkockák nem égetődnek be; a projekt hossza nem változik).

### `mocap-info`, `mocap-match`, `mocap-retarget`

```
osatura mocap-info     <file.bvh>
osatura mocap-match    <project.json> <file.bvh> [--map map.json]
osatura mocap-retarget <project.json> <file.bvh> <out.json> [--view front|side|top|yaw=DEG] [--map map.json] [--scale auto|N] [--fps F]
    [--smooth N] [--start F] [--end F] [--rest-frame F] [--min-conf C] [--weight W] [--mirror] [--no-root] [--in-place] [--keep-length] [--footlock BONE --ground Y]
```
* A **`mocap-info`** kiírja egy BVH-fájl hierarchiáját, csatornáit, képkockasebességét és kiterjedését.
  ```
  $ osatura mocap-info m.bvh
  4 joints (2 end sites), 15 channels per frame
  4 frames at 0.0417 s (24.000 fps), duration 0.125 s
  Hips  offset (0.000, 0.000, 0.000)  [Xpos Ypos Zpos Zrot Xrot Yrot]
    LeftUpLeg  offset (5.000, 0.000, 0.000)  [Zrot Xrot Yrot]
  ...
  ```
* A **`mocap-match`** kiírja az **automatikus ízület -> csont párosítási táblázatot** (slot, BVH-ízület, rig-csont, megbízhatóság, bizonyíték `via`, hosszarány, megjegyzés), majd a nem párosított ízületeket/csontokat és a figyelmeztetéseket. A nevek és a topológia egyaránt számítanak. A `--map` egy `{"bvh joint": "rig bone" | null}` JSON-objektum (a `null` vagy `""` kizár egy ízületet; a `{"map": {...}}` burkolás is elfogadott). Mindig 0-s kilépési kóddal tér vissza - ha semmi sem párosodott, csak ezt figyelmezteti: `no bones matched: supply a --map file`.
  ```
  $ osatura mocap-match demo.json m.bvh --map map2.json         # {"Hips":null,"LeftUpLeg":"thigh","LeftLeg":"shin"}
  slot         bvh joint                  rig bone                conf  via         len x  note
  shin.l       LeftLeg                    shin                    1.00  manual      12.00  manual
  thigh.l      LeftUpLeg                  thigh                   1.00  manual      12.00  manual
  unmatched BVH joints: Hips, RightUpLeg
  ...
  ```
* A **`mocap-retarget`** a mozgást **hozzáfűzött** csatornákként égeti be: a 3D-s forgatásokat a 2D `--view` nézetre vetíti (`front` az alapértelmezés, `side`, `top` vagy `yaw=DEG`), skálázza a gyökérmozgást (`--scale auto` vagy egy szám), a `--fps` értékre (alapértelmezés: a projekté) újramintavételez, simít (`--smooth` képkocka), a `--start..--end` BVH-képkockákat veszi, a `--rest-frame` képkockát használja nyugalmi pózként, eldobja a `--min-conf` alatti párosításokat, a `--weight` súllyal kever, a `--mirror` tükröz, a `--no-root` eldobja a gyökérmozgást, az `--in-place` a haladó klipet helyben járóvá alakítja, a `--footlock BONE --ground Y` pedig egy lábat rögzít. Kiírja a táblázatot, a megjegyzéseket és a következőt: `N bones, F frames at R fps, root scale S, +C channels; wrote <out>`. A projektnél hosszabb klip **megnyújtja a projekt hosszát**; az összegzés után egy megjegyzés közli, mekkora hosszról mekkorára. A `--keep-length` ehelyett megtartja a projekt hosszát, és az utolsó képkockánál levágja a kulcsokat (oda egy interpolált kulcs kerül; egy megjegyzés megszámolja a be nem égetett kulcsokat). Ha semmi sem éri el a megbízhatósági küszöböt: `no bone matches to retarget (all below the confidence threshold?)` (1-es kilépési kód). Megjegyzés: ennél a parancsnál a `--scale` értéke `auto|N`, nem a szokásos renderelési skála.
  ```
  $ osatura mocap-retarget demo.json m.bvh mr.json --map map2.json --view side --smooth 1 --in-place --no-root
  ... (table)
  2 bones, 4 frames at 24 fps, root scale 12.0000, +2 channels; wrote mr.json
  ```
**Korlátok (ROADMAP §9):** valódi mozgásrögzítési fájlt nem teszteltek (nincs mellékelve: a példák egy kézzel írt, 4 képkockás BVH-t használnak); a 2D-s vetítés eldobja a mélységet (a kamera felé mutató csont az utolsó szögét tartja).

---

## 11. Állapotgépek, adatkötések, konténerek és görgetési területek

Ezek a parancsok a projekt **futásidejű interakcióval** kapcsolatos részeit kezelik: állapotgépeket (`machines`), adatkötéseket
(`bindings`), elrendezési konténereket (`containers`) és görgetési területeket (`scrolls`). Mindegyik a projektfájlon
belül tárolja az adatait; az `add` alakok egy kis külön JSON-fájlt olvasnak be, ellenőrzik, és új projektet írnak.
Magát a futásidejű modellt a [PROJECT_FORMAT.md](PROJECT_FORMAT.md) dokumentálja.

### `machine` (add, list, remove, validate)

**Szintaxis:**
```
osatura machine add      <project.json> <out.json> <machine.json>
osatura machine list     <project.json>
osatura machine remove   <project.json> <out.json> <name>
osatura machine validate <project.json>
```

Egy **gépnek** elnevezett `inputs` bemenetei (`bool`, `number` vagy `trigger`) és egy vagy több `layers` rétege van; minden
rétegnek van egy `entry` belépő állapota, `states` állapotai és `transitions` átmenetei. Az állapot egy **forrást** játszik le:
`{"kind":"segment","start":F,"end":F}` (a projekt kulcsolt csatornáinak képkockái), `{"kind":"clip","clip":NAME}`,
`{"kind":"region","marker":NAME}` vagy egy 1D blend tree: `{"kind":"blend","input":NAME,"children":[{"threshold":T,"source":{...}}]}`.
Az állapot mezői: `name`, `source`, `mode` (`loop` az alapértelmezés, `once`, `ping_pong`), `speed` (1), `blend_in` (másodperc).
Az átmenet mezői: `from` (egy állapot vagy `"*"`), `to`, `when` (`{"input":NAME,"op":eq|ne|lt|le|gt|ge,"value":V}` elemek listája,
mindnek teljesülnie kell; a puszta `{"input":"go"}` azt jelenti: "egyenlő 1-gyel"), `exit_time` (0..1), `duration` (átúsztatás másodpercben) és `ease`.
Az átmenetben az ismeretlen kulcsok **hibát** jelentenek (egy elírás, például a `conditions`, egy feltételes átmenetet egyébként feltétel nélkülivé tenne).

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

A `machine remove` nem távolít el olyan gépet, amelyet még egy kötés vezérel
(`binding 'goBind' still drives machine 'm'`, 1-es kilépési kód): előbb a kötést távolítsd el.
**Hibák:** az érvénytelen gépet (ismeretlen állapot, ismétlődő nevek, hibás bemenet-hivatkozás) üzenettel elutasítja, és
semmit sem ír. **Kapcsolódó:** `machine-sim`, `bind`, `state-bake` (a régebbi, képkocka-alapú gépformátum).
**Korlátok:** a gépeket a futtatókörnyezet és a `machine-sim` értékeli ki; az exportálók ebből semmit sem égetnek be (a Lottie, a GIF
és a többi fájl csak képkockákat tartalmaz, lásd [EXPORT_FORMATS.md](EXPORT_FORMATS.md)).

### `machine-sim`

**Szintaxis:** `osatura machine-sim <project.json> <script.json> <out_dir|out.json> [--fps N --svg-frames N]`

Időzített bemeneti eseményeket játszik vissza egy gépen, és naplózza minden tick állapotát. A szkript
`{"machine": NAME?, "duration": SECONDS?, "events": [{"t": SECONDS, "set": {"input": value}, "trigger": ["name"],
"value": {"external.path": number}}]}`. A `machine` alapértelmezése a projekt egyetlen gépe, a `duration` alapértelmezése az utolsó
esemény plusz egy másodperc; a `value` az adatkötéseket táplálja (lásd `bind`). Az alapértelmezett `--fps` a projekt fps-e.
Az `--svg-frames N` ezenfelül N SVG-képkockát renderel, egyenletesen elosztva a futáson.

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
A `sim.json` tartalma: `machine`, `fps`, `ticks[]` (`tick`, `time`, `states` `[layer,state]` párokként, `fading`) és az
eseménynapló. **Kapcsolódó:** `bind`, `machine`.

### `state-bake`

**Szintaxis:** `osatura state-bake <project.json> <machine.json> <events.json> <out.json> [--tol 0.01]`

Egy **képkocka-alapú** állapotgépet (a `machine`-nél más, régebbi formátumot) közönséges kulcsolt csatornákba éget.
A `machine.json` alakja: `{"initial":NAME,"states":[{"name","length","looped","channels":[...]}],"transitions":[{"from":NAME
or "*","to","when":COND,"fade":FRAMES,"ease"}]}`, ahol a `COND` a következők egyike: `{"type":"event","name"}`,
`{"type":"above"|"below"|"equals","input","value"}`, `{"type":"finished"}`, `{"type":"after","frames"}`.
Az `events.json` `{"frame":N,"name":NAME,"value":V}` elemek listája. Az eredmény a projekt csatornái után kerül hozzáfűzésre
(a `--tol` a kulcscsökkentés tűrése), a projektet lefordítja és kiírja.

```
$ echo '[{"frame":10,"name":"go"}]' > ev.json
$ osatura state-bake demo.json sm.json ev.json baked.json --tol 0.01
wrote baked.json (+1 channels)
```
**Figyelem:** a beégetett csatorna a meglévők után kerül; a csatornák a lista sorrendjében futnak, ezért töröld vagy némítsd
ugyanazon célpont eredeti csatornáját, ha a beégetettnek kell érvényesülnie. A `machine` formátumú (`layers` mezős)
gépfájlt elutasítja: `invalid JSON in mach.json: missing field 'initial'`.

### `bind` (add, list, remove) és `bind-eval`

**Szintaxis:**
```
osatura bind add    <project.json> <out.json> <name> <value.path> <target:prop|@machine:input>
                     [--expr E --remap a,b,c,d --no-clamp --smooth S --mode replace|add|multiply --weight W]
osatura bind list   <project.json>
osatura bind remove <project.json> <out.json> <name>
osatura bind-eval   <project.json> <values.json> [--data out.json]
```

Az **adatkötés** egy külső értékútvonalat (például `hud.size`) képez le egy projekttulajdonságra (`sun:layer_opacity`,
a tulajdonságnevek az 1. szakasz snake_case nevei) vagy egy gép bemenetére (`@m:go`). A `--remap a,b,c,d` az `a..b`
bemeneti tartományt a `c..d` tartományra képezi (korlátozva, hacsak nincs `--no-clamp`); az `--expr` az értéken végzett kifejezés;
a `--smooth S` S másodpercen át aluláteresztő szűrést végez; a `--mode` és a `--weight` azt szabályozza, hogyan kombinálódik az eredmény a csatornákkal.

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
A `bind-eval` kiírja az egyes kötések kimenetét a megadott értékekre (a `--data` JSON-ként is kiírja). A `list` a tulajdonságneveket
CamelCase alakban írja ki; a parancssor mindig snake_case alakot vár. **Kapcsolódó:** `machine`, `machine-sim` (a `value` térképe).

### `container` (add, list, remove, solve)

**Szintaxis:**
```
osatura container add    <project.json> <out.json> <container.json>
osatura container list   <project.json>
osatura container remove <project.json> <out.json> <id>
osatura container solve  <project.json> [--size WxH --data out.json]
```

A **konténer** egy flexbox-szerű elrendezési doboz: `id`, `members`, `axis` (`row`|`column`), `gap`, `cross_gap`, `padding`
(`[left,top,right,bottom]`), `justify` (`start|center|end|space_between|space_around`), `align`
(`start|center|end|stretch`), `wrap` (`none|wrap`) és `rect`, a saját téglalapjának szabálya a vásznon:
`{"kind":"fixed",...}`, `{"kind":"pin","left","right","top","bottom","width","height"}` (távolságok a vászon széleitől; tengelyenként bármely kettő),
`{"kind":"aspect",...}` vagy `{"kind":"hug"}`. A **tag** pontosan az `{"layer":PATH}`,
`{"container":ID}` vagy `{"spacer":true}` egyike, a `grow`, `shrink`, `width`, `height`, `min_w/max_w/min_h/max_h` mezőkkel és
egy `align` felülbírálattal; a `"resize":true` engedi, hogy a megoldó a réteget a helyéhez méretezze.

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
A `--size` nélkül a projekt saját méretét használja. A `solve` kiírja az egyes konténerek téglalapját, rétegenként pedig az
eltolást és a skálát, amelyet az elrendezés az animáció tetejére alkalmaz. **Hibák:** az ismeretlen réteget, a hiányzó beágyazott konténert vagy a
ciklust az `add` elutasítja (semmi sem íródik). **Kapcsolódó:** `scroll`, `layout-preview`.
**Korlátok:** az elrendezést a futtatókörnyezet/előnézet alkalmazza; az exportált fájlok a projekt saját méretére megoldott elrendezést tartalmazzák.

### `scroll` (add, list, remove, eval)

**Szintaxis:**
```
osatura scroll add    <project.json> <out.json> <scroll.json>
osatura scroll list   <project.json>
osatura scroll remove <project.json> <out.json> <id>
osatura scroll eval   <project.json> <id> [--offset X,Y --size WxH --velocity VX,VY --time S]
```

A **görgetési területnek** van `id`-je, `content` tartalma (egy konténer id-je, amely nem tagja másik konténernek), `viewport` nézete
(téglalapszabály, mint fent), `axis` tengelye (`x`, `y` vagy mindkettő), `offset` eltolása, `elastic` rugalmassága (`{"limit","stiffness","damping_ratio"}`; gumiszalag
a kemény korlátozás helyett), `snap_x`/`snap_y` (nyugalmi eltolások), `fling` (az elengedési sebesség vetítésének másodpercei a
leérkezési pont megtalálásához, alapértelmezés: 0,15), `clip_layer` (a tartalom maszkjaként használt téglalaprétég) és `input` (egy külső értékútvonal, amely
az eltolást vezérli).

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
`--velocity` nélkül az `eval` a kért eltolás korlátozott (vagy gumiszalagszerűen megnyújtott) értékét mutatja. `--velocity` esetén
az elengedést (lendület, snap, rugó) `--time` másodpercig szimulálja, és minden 0,1 s-nál kiír egy sort. A `--size WxH` másik vászonméretre értékel.
A példában a tartalom éppolyan széles, mint a nézet, így minden 0 feletti érték túlgörgetés, és a gumiszalag-hatást mutatja.

### `layout-preview`

**Szintaxis:** `osatura layout-preview <project.json> <out.svg> --size WxH [--frame N --scroll id=X,Y]`

Egy képkockát renderel **másik vászonméretre újramegoldott** elrendezéssel, és megrajzolja a konténerek körvonalait, így
ellenőrizheted, hogyan rendeződik át egy terv telefonon vagy széles képernyőn. A `--scroll id=X,Y` görgetési eltolást állít be (több területhez ismételd).

```
$ osatura layout-preview d5.json lp.svg --size 800x480 --scroll sc1=50,0
wrote lp.svg (800x480, 1 containers, 2 layers placed)
```

## 12. Effektek, mélységélesség, onion skin, nyomvonalak és kamera

### `fx`

**Szintaxis:** `osatura fx <project.json> <frame> <chain.json> <out.png> [--scale N] [--layer-normals] [--clamp]`

A projekten kívüli `<frame>` hiba; a `--clamp` a legközelebbi érvényes képkockát rendereli.

Renderel egy képkockát, és **utófeldolgozó effektláncot** futtat rajta. A `chain.json` effektek listája, mindegyiknek van `type` típusa:
`blur {radius}`, `drop_shadow {dx,dy,blur,color,opacity}`, `glow {radius,strength,color?}`, `outline {width,color}`,
`posterize {levels}`, `pixelate {size}`, `levels {in_black,in_white,gamma,out_black?,out_white?}`, `fade {amount}` és
`relight {ambient,bevel,height,lights:[{x,y,z,color?,intensity?,radius?,cast_shadow?,spot?,falloff?}],ground_shadow?,
normal_map?,ambient_color?}`. `normal_map` nélkül a relight az elmosott alfacsatornából készült domborulatot árnyalja.

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
A `--layer-normals` minden réteget a saját normáltérképével árnyal (lásd `layer-normal`), nem egyetlen térképpel az egész képre.
A `z`-t elhagyó fényt elutasítja (`missing field 'z'`). **Korlátok:** a lánc csak PNG-t előállító állókép-folyamat; ezen
effektek egyike sem kerül a Lottie-ba, a GIF-be vagy a HTML-lejátszóba (a Lottie-exportálónak saját effekt-részhalmaza van, lásd EXPORT_FORMATS.md).

### `color-space`

**Szintaxis:** `osatura color-space <project.json> <out.json> srgb|linear`

Beállítja a projekt `color_space` értékét, azt a teret, amelyben a raszteres kompozitor (PNG, GIF, APNG, videó, sprite sheet, `compare`, `ci`) és a lebegőpontos kompozitor (`render-hdr`) kever. Az `srgb` (az alapértelmezés, és ezzel rögzítettek minden golden képet) a tárolt, sRGB-kódolt értékeket keveri; a `linear` minden színt (kitöltések, körvonalak, színátmenet-megállók, háttér, képpixelek) lineáris fényre alakít, ott f32-ben komponál, majd a végén egyszer sRGB-re alakít. Egy 50%-os fekete réteg fehéren így 128/255 helyett 188/255-öt ad, az átmenetek pedig lineáris fényben interpolálnak. `srgb` esetén a kulcs nem íródik ki, így a meglévő fájlok és golden képek nem változnak.

```
$ osatura color-space demo.json lin.json linear
wrote lin.json (color_space: linear)
$ osatura render lin.json 0 lin.png
wrote lin.png
$ osatura color-space lin.json x.json p3
unknown colour space 'p3' (srgb or linear)
```
**Korlátok:** a 8 bites renderelő lineáris projektekhez a lebegőpontos kompozitorra vált (saját lefedettség-rasztererrel, így az élsimított élek kissé eltérnek az sRGB-kimenettől); a rétegeffektek színeit (árnyék, glow, tint) úgy használja, ahogy meg vannak adva, nem alakítja át; a `dof` és az utófeldolgozó lánc (`fx`) sRGB marad; az SVG, a Lottie és a GPU-renderelő figyelmen kívül hagyja a beállítást; a képek újramintavételezése az átalakítás előtt sRGB-ben történik. A keverési módokat a lineáris értékeken alkalmazza.

### `layer-normal`

**Szintaxis:** `osatura layer-normal <project.json> <out.json> <layer> <normal.png|none> [--strength S] [--fit stretch|tile|exact] [--green-down]`

Tangens terű normáltérképet csatol egy képréteghez (a `none` eltávolítja) az `fx ... --layer-normals` számára.
A `--fit` azt adja meg, hogyan illeszkedik a más méretű térkép (`stretch` az alapértelmezés, `tile`, `exact`); a `--green-down` megfordítja a zöld
csatornát (DirectX-stílusú térképek); a `--strength` a dőlést skálázza (1 = ahogy tárolva van, 0 = lapos).

```
$ osatura layer-normal demo.json dn.json sun n.png --strength 1 --fit stretch
wrote dn.json
$ osatura fx dn.json 12 chain.json fxn.png --scale 0.5 --layer-normals
wrote fxn.png (2 effect(s))
$ osatura layer-normal dn.json dn2.json sun none
wrote dn2.json
```

### `onion` (két alak)

**Szintaxis:**
```
osatura onion <project.json> <frame> <out.png> [--before 2 --after 1 --step 1 --scale N]
osatura onion <project.json> <out.svg> --frame N [--before B --after A --step S --keyed --outline --all-parts
               --layers a,b --tint-before C --tint-after C --opacity X --falloff X --ghosts-only]
```
Az első alak PNG-t ír a megelőző és a követő képkockák halványított szellemképeivel. A második SVG állóképet ír: a
képkockát, rajta színezett onion skin szellemképekkel. A `--keyed` rögzített lépések helyett a legközelebbi **kulcsképkockákat** használja;
az `--outline` a szellemképeket körvonalként rajzolja; az `--all-parts` minden részről szellemképet készít (alapértelmezés: a mozgó részekről), a `--layers` korlátozza őket,
a `--ghosts-only` magát a képkockát kihagyja.

```
$ osatura onion demo.json 12 on.png --before 2 --after 1 --step 2 --scale 0.5
wrote on.png
$ osatura onion demo.json on.svg --frame 12 --before 2 --after 2 --step 3 --outline --opacity 0.4
wrote on.svg (4 ghost(s) around frame 12)
$ osatura onion demo.json on2.svg --frame 12 --keyed --ghosts-only
wrote on2.svg (3 ghost(s) around frame 12)
```
Az alakot a második argumentum dönti el: a szám a PNG-alakot, a `.svg`-re végződő név az SVG-alakot jelenti.

### `trail` (két alak)

**Szintaxis:**
```
osatura trail <project.json> <bone> <out.svg> [--from F --to F]
osatura trail <project.json> [out.svg] --target layer:PATH|bone:NAME[:head|tip]|point:LAYER:KIND:A[:B] [--from F --to F --smooth N --json]
```
Megrajzolja egy csont végének (első alak) vagy bármely célpontnak (második alak: réteg, csont feje/vége vagy
deformer-pont) a **mozgásnyomvonalát**, képkockánként egy osztással és a kulcsoknál jelölőkkel. A `--smooth N` simítja a görbét; a `--json` JSON-ként
kiírja a mintavételezett görbét (ilyenkor az `out.svg` elhagyható).

```
$ osatura trail demo.json hand tr.svg --from 0 --to 24
wrote tr.svg (25 points)
$ osatura trail demo.json tr3.svg --target layer:sun --from 0 --to 24
wrote tr3.svg (25 ticks, 0 key marker(s))
$ osatura trail demo.json tr2.svg --target bone:hand:tip --from 0 --to 24 --smooth 2 --json
{ "curve": [ [212.8, 83.84], [213.77, 83.99], ...
```

### `dof` és `dof-export`

**Szintaxis:**
```
osatura dof        <project.json> <frame> <out.png> [--focus 1.0 --strength 12 --max-radius 24 --scale N --clamp]
osatura dof-export <project.json> <out.gif|out.apng|out.mp4|out.webm|out-folder> [--from F --to F --focus N --strength N --max-radius N --scale N --codec vp9|av1]
```
A projekten kívüli `<frame>` a `dof` (valamint a `render-hdr` és az `fx`) esetén hiba; a `--clamp` helyette a legközelebbi érvényes képkockát rendereli, mint a `render`.

Mélységélesség: minden **legfelső szintű réteget** `|parallax - focus|` szorozva a `strength` értékkel (pixel a parallax-távolság egységére, a `--max-radius` korlátozza)
mos el. `--focus` / `--strength` nélkül az értékek a kulcsolt
`camera_focus` / `camera_blur` csatornákat követik (alapértelmezett fókusz: 1.0). A rétegek mélységüket a `parallax` értékből kapják (lásd az 5. szakaszt).
A `dof-export` a `--from`..`--to` tartományt (egész képkockaszámok; alapértelmezés: 0-tól a végéig) animált GIF-be, animált
PNG-be, `.mp4`/`.webm` videóba (`ffmpeg` kell hozzá; a `--codec` alapértelmezése `vp9`; hang nem kerül bele) vagy, kiterjesztés nélküli név esetén, számozott PNG-k
mappájába (`f0000.png`...) rendereli. Bármely más kiterjesztést elutasít (`dof-export writes .gif, .apng,
.mp4, .webm or a folder (no extension)`, semmi sem íródik).

```
$ osatura dof demo.json 12 dof.png --focus 1.0 --strength 12 --max-radius 24 --scale 0.5
wrote dof.png
$ osatura dof-export demo.json dof.gif --from 0 --to 6 --scale 0.25
wrote dof.gif
$ osatura dof-export demo.json dofdir --from 0 --to 2 --scale 0.25      # dofdir/f0000.png ... f0002.png
wrote dofdir
```
**Kapcsolódó:** `parallax`, `render --proxy`, `shot`/`camera-seq` a kameramozgásokhoz. **Korlátok:** az elmosás
rétegenkénti közelítés, nem lencsemodell (nincsenek bokeh-alakzatok, nincs takaráskezelés); nem exportálódik Lottie-ba.

## 13. Részletességi szintek, méretjelentések és export-fit

### `size-report`

**Szintaxis:** `osatura size-report <project.json> [--tol T] [--json]`

Megmutatja, **hol vannak a bájtok**: a projektfájlban, a beégetett Lottie-ban és a HTML-oldalban, réteg, kép és csatorna szerint
(kulcsok, és közülük hány távolítható el a `--tol` tűrésnél, alapértelmezés: 0,05).

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
A `--json` ugyanezeket az adatokat JSON-ként írja ki (`html.defs_bytes_total`, `frame_bytes_max`, ...). Az `export-fit` vagy a `lod` előtt használd,
hogy lásd, melyik réteget érdemes egyszerűsíteni.

### `lod`

**Szintaxis:** `osatura lod <project.json> --tolerance-px 0.5 [--scale S | --size WxH] [--levels 4] [--ratio 2] [--out-dir D] [--report r.json] [--verify] [--frames N]`

**Részletességi szinteket** (LOD) épít L1..Ln, mindegyiket *kimeneti pixelben megadott hibakorláttal*: útvonalegyszerűsítés, kulcscsökkentés,
mesh-összevonás és kiselejtezés. A k. szint tűrése `--tolerance-px x ratio^(k-1)`. A `--scale` vagy a `--size` megadja,
mekkorán jelenik meg a kimenet (alapértelmezés: egységenként 1 px). Az `--out-dir` (alapértelmezés: `.`) kapja a `<name>_L1.json` ... fájlokat, a
`--report` pedig egy JSON-jelentést. A `--verify` `--frames N` képkockát renderel, és **megméri** a tényleges eltérést (1-es kilépési kód, ha a korlát fölött van).

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
**Figyelem:** egy kicsi, kézzel épített demón a megtakarítás csekély (4 px-nél kb. 4%); a LOD a követett vagy mesh-nehéz grafikán térül meg.
A jelentést és a szintfájlokat egyszerű (nem atomi) írással írja.

### `lod-pick`

**Szintaxis:** `osatura lod-pick <project.json> --screen-px WxH [--max-px 0.5] [--tolerance-px 0.25] [--levels 4] [--out chosen.json]`

Kiválasztja a **legdurvább szintet**, amelynek megadott korlátja a `--max-px` pixelen belül marad, ha a grafika egy
`--screen-px` méretű képernyőn jelenik meg (a skála a képernyőméret osztva a projektmérettel). Az `--out` kiírja ezt a szintet.

```
$ osatura lod-pick small.json --screen-px 320x240 --out chosen.json
level  tol_px  anchors ...
screen 320x240 -> scale 0.5000 px/unit; budget 0.5 px -> level L3 (declared 0.4638 px, 69 anchors, 0 mesh vertices, 37 keys)
wrote chosen.json
$ osatura lod-pick small.json --screen-px 64x48 --max-px 2 --out chosen2.json
screen 64x48 -> scale 0.1000 px/unit; budget 2 px -> level L4 (declared 0.5310 px, 62 anchors, 0 mesh vertices, 37 keys)
```

### `export-fit`

**Szintaxis:** `osatura export-fit <project.json> <out> --max-size 500KB [--format lottie|html|gif|apng|webm|project] [--min-ssim X] [--max-seconds 60] [--measure raw|gzip] [--frames N] [--search ladder|descent|grid] [--report out.json] [--pareto out.csv] [--audit] [--no-baseline]`

Megkeresi a **bájtkeretbe még beleférő legjobb minőségű exportot**: végigkeresi az exportáló minőségi paramétereit (Lottie: `decimals`,
`path_eps`, `key_tol`, `frame_step`; más formátumoknak saját van), és minden jelöltet SSIM-mel pontoz az eredeti
rendereléshez képest. A `--max-size` értéke lehet `500KB` (1000 bájt), `1.5MB`, `64000`, `200KiB` (1024 bájt). A formátum a
`--format` opcióból vagy a kimenet kiterjesztéséből adódik (`.json` Lottie, `.html`, `.gif`, `.apng`, `.webm`); a `project` karcsúsított
projekt-JSON-t ír. A `--measure gzip` a gzip-méretre keretez (tömörítő szerverekhez). A `--min-ssim` minőségi alsó korlát.
A `--max-seconds` **valós falióra-keret** (alapértelmezés: 60; `0` = korlátlan): lejártakor a addigi legjobb eredmény marad.
A `--frames N` azt adja meg, hány képkockát mintavételez az SSIM-hez (alapértelmezés: 8). A `--search` a stratégiát választja (alapértelmezés: `descent`).
A `--report` / `--pareto` a keresés jegyzőkönyvét JSON / CSV formában írja; az `--audit` minden jelöltet kiír; a `--no-baseline` kihagyja
az alapbeállításokkal való összehasonlítást.

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
**Kilépési kódok:** 0, ha az eredmény belefér a keretbe (akkor is, ha az időkeret megszakította a keresést); 1, ha a keret
nem tartható (a legkisebb export ilyenkor is kiíródik), vagy hibás argumentumok esetén. **Figyelem:** a beégetett kimeneten dolgozik,
ezért többet nem tud eltávolítani, mint amennyit a paraméterek engednek (egy 14 rétegű rignél az 1 KB-os Lottie lehetetlen); minden jelölt teljes
export, ezért nagy projektekhez bőkezű `--max-seconds` kell (a debug build jóval lassabb a release-nél). A jelentésfájlokat
egyszerű írással írja. **Kapcsolódó:** `size-report`, `lod`, `export` (4. szakasz), EXPORT_FORMATS.md (méretezési tippek).

## 14. Minőségbiztosítás

Korábban már dokumentált, kapcsolódó parancsok: `check` (ellenőrzés, 3. szakasz), `diff` (szemantikus diff, 3. szakasz), `compare` (két
render képi diffje, 4. szakasz).

### `lint`

**Szintaxis:** `osatura lint <project.json>`

Statikus rig- és animációellenőrzések. Találatonként egy sort ír: `Severity CODE [target:property] message`. 1-es kilépési kód, ha bármely
találat súlyossága Error vagy annál magasabb; az Info és a Warning 0-val lép ki.
```
$ osatura lint small.json
Info L007 [:CameraX] key at frame 24 lies outside the 0..8 timeline
Info L007 [arm_low:BoneAngle] key at frame 12 lies outside the 0..8 timeline
```
Az `L007` az idővonalon kívüli kulcs (itt azért, mert a `small.json` a demo 8 képkockára vágott változata).

### `contracts`

**Szintaxis:** `osatura contracts <project.json> <contracts.json>`

**Mozgásszerződések**: egységtesztek az animációhoz. A `contracts.json` szerződések listája, mindegyiknek van `type` típusa:
`loop_seamless {tol}`, `tip_range {bone,axis,min,max}`, `max_speed {bone,px_per_frame}` (további típusok is léteznek a
motor `Contract` enumjában). A megsértések a stdout-ra, a darabszám a stderr-re kerül, a kilépési kód pedig 1.
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

**Szintaxis:** `osatura coach <project.json>`

Időzítési és animációs alapelvekre vonatkozó javaslatok (átfedés, előkészítés, easing). Csak tanács, mindig 0-val lép ki.
```
$ osatura coach small.json
C003 [, arm_low, arm_up, foot_ctrl, hand, mouth, target] frame 0: 7 parts start moving on frame 0; offset them by a few frames (overlap) for a more natural feel
```

### `safety`

**Szintaxis:** `osatura safety <project.json> [--scale N]`

Villanás-, vörös villanás- és kameramozgás-jelentés (a WCAG 2.3.1 közelítése; mérnöki segédeszköz, nem műsorszórási
tanúsítás). Minden képkockát `--scale` skálán renderel (hosszú projektekhez használj kis értéket).
```
$ osatura safety small.json --scale 0.25
flashes      ok    worst 0 transitions/s (frame 0)
red flashes  ok    worst 0 transitions/s (frame 0)
camera       pan 0.06 widths/s, rotation 0°/s, zoom ×1.00/s
```

### `reduced-motion`

**Szintaxis:** `osatura reduced-motion <project.json> <out.json> [--factor 0.25]`

Nyugodtabb másolatot ír: a kameramozgást és a remegést a `--factor` értékkel (alapértelmezés: 0,25) skálázza.
```
$ osatura reduced-motion small.json calm.json --factor 0.25
wrote calm.json
```

### `golden`

**Szintaxis:**
```
osatura golden write <project.json> <golden.json> [--scale N]
osatura golden check <project.json> <golden.json> [--tol T]
```
A `write --scale N` esetén az N a (0, 1] tartományból való: a nagyobb értéket **elutasítja** (`golden write: --scale 2 is above 1 ...`, semmi sem íródik; korábban csendben 1-re vágta). A `check` a golden fájlban tárolt skálán renderel újra, és nincs `--scale` opciója: a `golden check ... --scale N` ismeretlen opcióként elutasításra kerül.


Könnyű regressziós teszt: a `write` minden képkockáról hash-t és 16x16-os bélyegképet rögzít a `golden.json` fájlba; a `check`
újrarenderel és összehasonlít. **Alapértelmezésben pontos** (`--tol 0`): a pixel-hash bármilyen eltérése hibát jelent. A `--tol T` (T > 0) lazítja
az ellenőrzést: a megváltozott hash is megfelel, ha a bélyegkép-távolság legfeljebb T. A negatív vagy nem véges `--tol` hiba.
Észleleti metrikájú, kép-alapú golden képekhez használd a `ci` parancsot.
```
$ osatura golden write small.json g.json --scale 0.25
wrote g.json (8 frames)
$ osatura golden check small.json g.json
all 8 frames match exactly
$ osatura golden check small.json g.json --tol 0.01
all 8 frames match (within thumbnail tolerance 0.01)
```
Az eltérő képkockákat felsorolja (hash-változás, bélyegkép-távolság, a 256 bélyegképcella közül hány tér el, a legnagyobb csatornaeltérés
255-ből), és a kilépési kód 1:
```
$ osatura golden check changed.json g.json
frame 1 differs: pixel hash changed, thumbnail distance 0.003, 12 of 256 thumbnail cells differ, max channel difference 31/255
2 frame(s) differ (exact comparison; --tol T accepts small differences)       [stderr, exit 1]
```
A csak az egyik oldalon meglévő képkockát `one side only` jelzéssel közli. **Korlátok:** a golden fájl csak a hash-t és a
bélyegképet őrzi, így a jelentett számok bélyegkép-szintűek, nem pixelszintűek. A rendereben láthatatlan változás (például
egy hátterszín, amelyet egy átlátszatlan réteg teljesen eltakar, mint a demo égboltját) jogosan hagyja változatlanul a hash-t.

### `ci` (init, run, bless, diff)

**Szintaxis:**
```
osatura ci init <project.json> [--out osatura-ci.json] [--no-goldens]
osatura ci run  [manifest] [--update] [--bless] [--prune] [--only a,b] [--junit f.xml] [--html f.html] [--json f.json] [--artifacts dir] [--quiet]
osatura ci bless [manifest]
osatura ci diff a.png b.png [--metric ssim|de2000|de76|mse] [--tol T] [--ignore x,y,w,h;...] [--aa N] [--out heat.png] [--side s.png] [--overlay o.png] [--flicker f.gif]
```
**Animáció-CI**: golden képkockák összehasonlítása észleleti metrikával (luma SSIM, CIEDE2000), valamint szerkezeti ellenőrzésekkel és
keretekkel. A manifest formátumát és a munkafolyamatot a [CI.md](CI.md) írja le. `exit 0` = sikeres, `1` = egy ellenőrzés megbukott,
`2` = hiba (olvashatatlan manifest, ...).
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
Az `init` kiírja a manifestet, és - ha nincs `--no-goldens` - a golden PNG-ket a mellette lévő `goldens/` mappába. A `run
--update` újraírja a hiányzó vagy megváltozott golden képeket, a `--bless` elfogadja az új rendereket golden képként, a `--prune` eltávolítja azokat a
golden fájlokat, amelyekre már egyetlen ellenőrzés sem hivatkozik, az `--only a,b` a megnevezett ellenőrzéseket futtatja. A `bless` a `run --bless`. A `diff` két PNG-t hasonlít össze: a `--tol` a metrika
küszöbe, az `--ignore` téglalapokat maszkol (`x,y,w,h;...`), az `--aa N` megbocsátja az élsimításból adódó eltéréseket, az `--out`,
`--side`, `--overlay`, `--flicker` pedig hőtérképet, egymás melletti képet, átfedést és villogó GIF-et ír.
**Figyelem:** a jelentéseket (`--junit`, `--html`, `--json`) és a golden fájlokat az atomi, szimbolikus linket elutasító író írja (a szülőmappákat létrehozva); a
szimbolikus linkként megadott jelentésútvonalat az ellenőrzések lefutása után utasítja el (az ellenőrzések lassúak, ezért valódi útvonalat adj meg). A debug build lassú: a fenti 8 ellenőrzés egy 8 képkockás projekten 15 másodpercet vett igénybe.

### `fuzz`

**Szintaxis:** `osatura fuzz <project.json> [--samples N] [--seed S] [--strategy random|lhs|halton|sweep|corners|pca] [--frame F] [--report out.json] [--heatmap out.svg] [--minimize] [--gate] [--max-fail-rate R] [--props angle,scale,offx,offy] [--only BONES] [--margin DEG] [--db posedb.json] [--ignore KINDS] [--no-deformers]`

**Póz-fuzzolás**: a csontkorlátokon belül pózokat mintavételez, és jelzi, hol törik el a rig (összecsukódás, nyúlás, tüskék,
önmetszés, elérhetetlen IK, korlátsértések, NaN, vásznon kívüli helyzet). A `--strategy` a mintavételezőt választja (a `pca` a projekt saját kulcspózainak, vagy a `--db`-vel megadott
póz-adatbázis PCA-tengelyei mentén mintavételez), a `--minimize` a hibás pózt minimális reprodukálóvá zsugorítja, a `--heatmap` feszültségtérképet ír,
az `--ignore` kihagy hibafajtákat, az `--only` csontokra szűkít. Kapu nélkül a `fuzz` egy **jelentés**: kiírja az összegzést, és 0-val lép ki (`(report only: exit 0; add --gate, ...)`). A `--gate` hibával (1-es kilépési kód) leáll, ha bármely minta megbukik; a `--max-fail-rate R` (0..1 közötti **tört**) ezt a rátát jelentő kaput is bekapcsolja. (A `ci` saját `fuzz` ellenőrzésének saját `gate` beállítása van, amelyet ez nem érint.)
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
A demo IK-célpontja szándékosan csak kis tartományban érhető el, így szinte minden véletlen póz "elérhetetlen":
a rátát regressziós számként kezeld, nem sikeres/sikertelen mércéként ennél a rignél. A jelentést és a hőtérképet atomi módon írja, a szimbolikus linkként megadott útvonalat elutasítja (kivéve `--follow-symlinks` esetén); a szülőmappának léteznie kell.

### `fuzz-grid`

**Szintaxis:** `osatura fuzz-grid <project.json> --x PARAM --y PARAM [--res N] out.svg|out.png|out.json`

Két paraméter (`bone:NAME:PROP`, például `bone:arm_up:angle`) súlyosságtérképe `N x N` rácson (alapértelmezés: 32).
A kimenet típusát a kiterjesztés dönti el.
```
$ osatura fuzz-grid small.json --x bone:arm_up:angle --y bone:arm_low:angle --res 8 grid.svg
wrote grid.svg (8x8 cells, 59.4% failing; x = bone:arm_up:angle, y = bone:arm_low:angle)
```
Ugyanaz az egyszerű írásra vonatkozó megszorítás, mint a `fuzz` esetén.

## 15. Verziókezelés (`vcs`)

**Szintaxis:**
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
Az opciók alparancsonként eltérnek: a `--json` csak a `log`, `status`, `diff` és `show` parancsnál van; a `--force` csak a `checkout`-nál; az `--ours`, `--theirs`, `--value` (pontosan egy) csak a `resolve`-nál.

Kis, tartalom-címzett előzmény **egyetlen projektfájlhoz**, **szemantikus összefésüléssel**: a változásokat
dokumentumútvonalak (JSON pointerek) alapján hasonlítja össze, nem szövegsorok szerint, így két ember, aki különböző kulcsokat szerkeszt, sosem ütközik. Az `init` létrehoz
egy `.osatura-vcs/` mappát a projekt mellett; az alapértelmezett ág a `main`. A revíziók (`REV`) ágnevek, címkenevek vagy
commit-azonosító előtagok lehetnek.

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

**Ütközések.** Ha mindkét oldal ugyanazt az értéket változtatta, a `merge` megáll, a projektfájlban *a mi* oldalunkat hagyja, felsorolja az
ütközéseket, és 1-gyel lép ki:
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
Amíg egy merge, cherry-pick vagy revert nincs feloldva, a `checkout`, a `revert` és a `commit` elutasításra kerül ezzel:
`a merge / cherry-pick / revert is in progress: resolve the conflicts and commit, or run 'vcs merge --abort'`.
Egyéb tudnivalók: a `diff` argumentum nélkül a munkafájlt hasonlítja a HEAD-hez, a `diff A B` két revíziót (a `--json` egy `{kind,path,description}`
elemekből álló listát ír ki); a `show` kiír egy commitot és a változáslistáját; a `branch` argumentum nélkül kilistázza az ágakat (a `*` jelöli
az aktuálisat); a `branch -d NAME` megtagadja a kicsekkolt ág törlését; a `checkout --force` eldobja a nem commitolt
változásokat; a `revert REV` és a `cherry-pick REV` új commitot hoz létre (vagy ütközésekkel megáll, mint a `merge`); a `--repo DIR`
a jelenlegitől eltérő repó-mappát jelöl ki; az `--author` alapértelmezése `unknown`; a `log --json` a commit-
objektumokat írja ki. **Korlátok:** repónként egy fájl, csak helyi (nincsenek távoli repók, nincs push/pull), a JSON-útvonalakon túl nincs átnevezéskövetés;
a projekt által hivatkozott bináris erőforrások nincsenek verziózva.

## 16. Stúdió, együttműködés, MCP ügynökszerver és UI-regiszter

### `studio`

**Szintaxis:** `osatura studio <project.json> [--port N] [--ui-config F] [--bind ADDR] [--allow-host NAMES] [--public-origin URL] [--source-url URL] [--collab-token T (16+ characters)] [--collab-secret-file F [--collab-revoke NAMES]] [--collab-rate N] [--collab-log LOG [--collab-nosync]]`

Elindítja a **böngészős szerkesztőt** (visszavonás/újra, idővonal, élő és arc panelek, VCS és ügynökstílusú műveletek) helyi webszerverként, és
Ctrl-C-ig blokkol. Alapértelmezések: `--port 8080`, `--bind 127.0.0.1`. A projektet indításkor ellenőrzi; a mentés ugyanabba a fájlba történik
(`POST /api/save`).
```
$ osatura studio demo.json --port 18931 --ui-config ui.json
AGPL-3.0 source offer: http://127.0.0.1:18931/source
OSATURA studio on http://127.0.0.1:18931  (Ctrl-C to stop; saves go to demo.json)
UI settings (layout, shortcuts) in ui.json
```
Opciók: a `--ui-config F` az elrendezést és a gyorsbillentyűket őrzi (alapértelmezés: `~/.config/osatura/ui.json`, Windowson `%APPDATA%\OSATURA\ui.json`); a `--source-url URL` a
`/source` címen és a `--version` kimenetében megjelenített, közzétett forráshoz vezető hivatkozást állítja be (AGPL-3.0 13. szakasz; a hivatalos kiadási binárisok alapértelmezése az a commit, amelyből épültek); az `--allow-host NAMES` és a `--public-origin URL` elfogadja a
nyilvános név alatt, TLS fordított proxy mögül érkező kéréseket; a `--collab-token T` (legalább 16 karakter, például
`openssl rand -hex 16`) bekapcsolja a többfelhasználós munkamenetet, és **minden nem loopback `--bind` esetén kötelező**; a `--collab-nosync` csak a `--collab-log` mellett érvényes (egyébként hiba); a `--collab-log LOG`
tartósítja a megosztott munkamenetet, és újraindítás után folytatja (a `--collab-nosync` kihagyja az indításkori szinkronizálást).

**Keményítési opciók** (7. hullám; a részletek és a fenyegetésmodell a [SECURITY.md](SECURITY.md) fájlban): a `--collab-secret-file F` az egyetlen megosztott bearer helyett
**résztvevőnkénti tokeneket** vezet be: az F egy véletlen titkot tartalmaz (első sor, legalább 16 karakter, Unixon 600-as mód, különben elutasítja), és mindenki az `osatura collab-token NAME` paranccsal kap egy tokent; a token egyetlen névhez kötött (a belépés ezt a nevet használja, senki sem léphet be más néven), lejár, és a HMAC-SHA-256 újraszámításával ellenőrzik, így a
szerver nem tart fenn táblázatot. A `--collab-revoke NAMES` (vesszővel elválasztva, a `--collab-secret-file` kell hozzá) a megnevezett nevek tokenjeit lejárat előtt is elutasítja.
A `--collab-token` és a `--collab-secret-file` kombinálható (ilyenkor a megosztott token admin kulcsként működik); bármelyik kielégíti azt a szabályt, hogy a nem loopback bind hitelesítő adatot kíván. A `--collab-rate N` minden kliensciméhez
másodpercenként N kérést enged (2N-es löketekkel; a válasz `429` `Retry-After` fejléccel); az alapértelmezés 100, ha hitelesítő adat
kötelező, egyébként nincs korlát; a 0 kikapcsolja. A nem loopback bind figyelmeztetéseket ír ki (sima HTTP; a `0.0.0.0` / `::` esetén minden interfész; a megosztott token nem hordoz nevet és nem jár le).

**Biztonsági szabályok** (ellenőrizve): a szerver csak azokra a kérésekre válaszol, amelyek `Host` értéke a saját nevei egyike
(a `curl -H 'Host: evil.example' .../api/state` 421-et ad); egy másik oldalról érkező `Origin` és `Sec-Fetch-Site` 403-at ad; a POST-törzsnek
`application/json` típusúnak kell lennie (a `curl -X POST -d '{}'` 415-öt ad). A rövid tokent elutasítja
(`--collab-token must be at least 16 characters`), és a token nélküli `--bind 0.0.0.0` is elutasításra kerül
(`refusing to listen on 0.0.0.0:PORT: the studio has no accounts and no TLS, ...`). A szervernek nincsenek fiókjai és nincs TLS-e: tegyél
elé TLS proxyt, ha nem a saját gépedről használod; ha a `--public-origin` egy `https://` URL, a belépési süti `Secure` attribútumot kap az ezen név alatt érkező kéréseknél (vagy `Origin` https / `X-Forwarded-Proto: https` esetén). Az oldal kis SVG favicont szolgál ki a `/favicon.ico` címen. A HTTP API (például `GET /api/state`, `GET /api/project`,
`POST /api/op`, `POST /api/save`, `GET /frame/N.svg`) magának az oldalnak készült, és nem stabil nyilvános API.

### `collab-token`

**Szintaxis:** `osatura collab-token <name> [--secret-file F] [--ttl 28800]`

Résztvevőnkénti együttműködési tokent ír ki egy `--collab-secret-file F` opcióval indított stúdióhoz: `ap1.<name>.<expires>.<mac>`,
ahol a `mac` a név és a határidő hexadecimális HMAC-SHA-256 értéke a titokkal. A `<name>` 1-32 karakter: betűk,
számjegyek, `_` és `-`. A `--ttl` az élettartam másodpercben (alapértelmezés: 28800 = 8 óra, legfeljebb 2592000 = 30 nap). A titkot az
`F` fájlból (első sor) olvassa, vagy `--secret-file` nélkül az `OSATURA_COLLAB_SECRET` környezeti változóból; argumentumként soha
nem fogadja el, mert ott más felhasználók láthatják. A token a stdout-ra, a határidő a stderr-re kerül.

```
$ openssl rand -hex 32 > collab.secret && chmod 600 collab.secret
$ osatura studio demo.json --collab-secret-file collab.secret
$ osatura collab-token ana --secret-file collab.secret --ttl 3600
ap1.ana.1790000000.3f0c...
token for 'ana', valid until Unix time 1790000000 (in 3600 s); give it to that person only; ...
$ osatura collab-token ana --secret-file collab.secret --ttl 0
--ttl needs a whole number of seconds (1 to 2592000)
```
A résztvevő beilleszti a tokent a stúdió belépőoldalán (ezután egy HttpOnly sütiben tárolódik, amely a tokennel együtt lejár), vagy
`X-Collab-Token` fejlécben küldi. A lejárt tokenre `401` a válasz, az üzenetben a határidővel; a résztvevő nevét a
tokenből veszi. **Kapcsolódó:** `studio`.

### `collab-sim`

**Szintaxis:** `osatura collab-sim <project.json> [--clients N] [--ops M] [--seed S] [--undo-pct P] [--replays K] [--require-valid] [--raw] [--json]`

Tesztkeret az együttműködési motorhoz: N replika egyidejűleg szerkeszti a projektet, véletlen kézbesítési sorrenddel és
duplikátumokkal, majd ellenőrzi, hogy mindegyik ugyanazt a dokumentumot tartja-e (ha nem, 1-es kilépési kód). Az alapértelmezett szerkesztések érvényes projekt-
szerkesztések (számok, kulcsértékek, láthatóság, saját rétegek); a `--raw` tetszőleges JSON-szerkesztéseket használ (általában nem érvényes projekt);
a `--require-valid` a le nem forduló eredményt is 1-es kilépési kóddal jelzi.
```
$ osatura collab-sim small.json --clients 3 --ops 20 --seed 2 --json
{"buffered":12,"clients":3,"clocks_identical":true,"converged":true,"converged_ok":true,"deliveries":34,"duplicates_dropped":0,"edits":20,"envelopes":17,...}
```
A `converged_ok` pusztán a konvergenciát jelenti; az `ok` ezenfelül azt is megköveteli, hogy az összefésült dokumentum lefordítható legyen.

### `mcp`

**Szintaxis:** `osatura mcp <project.json>`

**Ügynökszerver**: JSON-RPC 2.0 (a Model Context Protocol) a stdin/stdout-on, soronként egy JSON-üzenettel. Betölti a
projektet, és a következő eszközöket teszi elérhetővé: `get_project`, `stats`, `lint`, `coach`, `safety`, `apply_ops`, `set_key`, `undo`, `redo`,
`render_frame` és `save`. A `save` meghívásáig semmi sem íródik ki. A stdout csak protokollüzeneteket hordoz.
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
A szerver (0-s kóddal) kilép, amikor a stdin bezárul.

### `ui`

**Szintaxis:**
```
osatura ui commands|shortcuts [--ui-config F] [--json]
osatura ui layout presets|show <name>|validate <file.json>
osatura ui search <query...> [--ui-config F]
```
A stúdió **parancsregisztere**: a `commands` minden parancsot kilistáz kategóriájával, címével és billentyűkötéseivel;
a `shortcuts` a érvényes gyorsbillentyűket (az alapértelmezettek plusz a `--ui-config` fájlban lévők) és az esetleges ütközéseket listázza; a `layout presets` a
dokkolt elrendezések előbeállításait listázza, a `layout show NAME` egyet JSON-ként kiír, a `layout validate FILE` pedig ellenőriz egy elrendezésfájlt;
a `search` úgy rangsorolja a parancsokat, mint a parancspaletta.
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

## 17. Póz-adatbázis, javaslatok és egyéb segédek

Ezek a parancsok **póz-adatbázist** építenek (csontforgatások egy projekt és egy könyvtár minden take-jének kulcsképkockáin),
és a megtanult szerkezetét (PCA-tengelyek, opcionális kis neurális háló) használják pózok javaslatához. Ez *a saját* pózaidon tanított statisztikai
segédeszköz, nem általános modell. A modell- és jelentésfájljaikat egyszerű írással írják.

### `pose-db` (build, info, train-mlp)

**Szintaxis:**
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
A `--project` hozzáadja a projekt kulcsképkockáin lévő pózokat, a `--library` egy take-könyvtár take-jeit (`take-record`, 6. szakasz),
a `--step N` ezenfelül minden N-edik képkockát mintavételez. A kis háló (sima SGD) opcionális; az alábbi `--model` opcióval használd.

### `pose-suggest` (between, complete, next)

**Szintaxis:**
```
osatura pose-suggest between  <project.json> <db.json|-> --a F1 --b F2 [--t 0.5] [--snap 0.5] [--top 5] [--apply RANK --out out.json] [--json]
osatura pose-suggest complete <project.json> <db.json|-> --frame F [--fix bone=angle,...] [--reach bone=x,y,...] [--free b1,b2] [--prior 1] [--model m.json] [--apply RANK --out out.json]
osatura pose-suggest next     <project.json> <db.json|-> --frame F [--k 8]
```
Az adatbázis helyén álló `-` magából a projektből tanul. A `between` két képkocka közötti átmeneti pózt javasol, amely a megtanult
sokaságon marad (`linear`, `pca`, `manifold` jelöltek); a `complete` megtalálja a legvalószínűbb teljes pózt rögzített
szögek (`--fix`), végpont-célok (`--reach`) és szabad csontok mellett; a `next` megmutatja, mit tett a könyvtár a képkockán lévőhöz hasonló pózok után.
Az `--apply RANK` (az `--out` opcióval) a kiválasztott jelöltet kulcsokként írja ki.
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

**Szintaxis:** `osatura pose-bench`

Kiírja a javaslatmódszerek kihagyott (held-out) pontosságát egy **szintetikus, járásszerű könyvtáron** (24 tanítóciklus, 6
kihagyott; a kiegészítés a 10 csontból 3-at lát; a kis hálót is betanítja). Az alacsonyabb a jobb. Nem olvas projektet. **Debug buildben
lassú** (percek); egy release build számai:
```
completion  mean pose                                   20.96 deg RMS
completion  nearest library pose                         1.77 deg RMS
completion  PCA least squares                            0.51 deg RMS
completion  MLP                                          1.64 deg RMS
between     linear blend                                 1.83 deg RMS
next        Markov mean                                  0.72 deg RMS
```

### `pose-match`

**Szintaxis:** `osatura pose-match <project.json> <params.json> <goals.json> <out.json> [--generations N --seed N]`

Keres (evolúciós optimalizálóval, alapértelmezés: 80 generáció, seed 1, érvényes tartomány: 1..100000) egy paraméterhalmaz olyan értékeket, amelyek a csontvégeket
a színpadtérben a célpozíciókra viszik. A `params.json` `{"target","prop","min","max","start"}` elemek listája; a `goals.json` `{"bone","at":[x,y],"weight"}` elemek listája. Az eredmény csatornákként kerül hozzáfűzésre;
a veszteség a végpont-hibák négyzeteinek súlyozott összege px²-ben.
```
$ echo '[{"target":"arm_up","prop":"bone_angle","min":-60,"max":40,"start":0},{"target":"arm_low","prop":"bone_angle","min":-60,"max":60,"start":0}]' > params.json
$ echo '[{"bone":"hand","at":[260,200],"weight":1.0}]' > goals.json
$ osatura pose-match demo.json params.json goals.json out.json --generations 30 --seed 3
wrote out.json (loss 3756.156 px²)
```
**Figyelem:** a példában a veszteség nagy, mert a tartományok nem érik el a célt: a számot is nézd, ne csak a
kilépési kódot (eredmény akkor is íródik, ha az illesztés gyenge).
## Ismert eltérések a dokumentáció és a viselkedés között

Ahol a program saját használati szövege (amelyet az `osatura` argumentumok nélkül kiír) vagy egy szomszédos dokumentum mást állít,
mint amit a program tesz, a fenti szakaszok azt írják le, amit a program valójában csinál. Az összes példa ellenőrzése
közben talált eltérések, abban a sorrendben, ahogy valószínűleg megtévesztenek:

1. **Javítva - ismeretlen kapcsolók.** Minden használati sorral rendelkező parancs elutasítja a nem dokumentált `--words` ("did you mean" javaslattal),
   köztük a `check`, `stats`, `lint`, `coach`, `contracts` és `machine list` parancsot is. A `--json`, `--quiet` és `--allow-empty`
   parancsonként érvényes, máshol elutasítja (törő változás azoknak a szkripteknek, amelyek mindenhol átadták őket). Az 5. hullám óta az ellenőrzés **alparancsonként** történik (a `ci diff ... --quiet`, `trail <bone> ... --json`, `ui layout presets --json`, `golden write --tol` elutasításra kerül), és az értékes opciókra is kiterjed. Tesztek: `osatura-cli/tests/doc_mismatch.rs` (elfogadási mátrix) és `options_are_validated_per_sub_command` a `main.rs` fájlban.
2. **Javítva - sima `fs::write` kimenetek.** A `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, `face-*`, a póz-fájlok, a pillanatképek,
   a `vcs` és az együttműködési napló az atomi, szimbolikus linket elutasító írót használja; az ffmpeg-kimeneteket előre ellenőrzi. Egy táblázatvezérelt teszt
   mintegy 24 parancsot fed le. Végponttól végpontig nem lefedett: a `face-train`/`face-eval` (csak az íróikra van egységteszt).
3. **Javítva - `golden check`** alapértelmezésben pontos; a `--tol` lazítja; kiírja az eltérések számát és a legnagyobb eltérést.
   A "háttérszín-változás átmegy" állítás a demo műterméke volt (az égboltréteg eltakarja a hátteret), nem hiba.
4. **Javítva - `fuzz` kilépési kód.** Az alapértelmezés csak jelentés (0-s kilépés); a `--gate` vagy a `--max-fail-rate` kapuz. Lásd `fuzz`.
5. **Javítva - `dof-export`** ír `.mp4` és `.webm` fájlt (ffmpegen keresztül, a `--codec` opcióval, hang nélkül); a használati szöveg most felsorolja őket. A más kiterjesztésű
   nevet elutasítja, és semmit sem ír.
6. **Javítva - `export` `.png`** a `<name>.atlas.json` fájlt írja, soha nem a `<name>.json` fájlt; az ismeretlen formátum hibaüzenete felsorol minden kiterjesztést.
7. **A használati szöveg által nem közölt alapértékek:** a `studio` a `127.0.0.1:8080` címen figyel; a `fuzz-grid --res` értéke 32; az `export-fit`
   8 képkockát mintavételez, `descent` stratégiával keres, és 60 s után leáll (`--max-seconds 0` = korlátlan); a `reduced-motion --factor`
   0,25; a `size-report --tol` 0,05; a `pose-match` 80 generációt futtat 1-es seeddel; a `lod --out-dir` az aktuális
   mappa.
8. **A `machine remove` elutasíthat** (`binding 'goBind' still drives machine 'm'`); a használati szöveg nem említi a
   függőségellenőrzést. A `machine` és a `state-bake` két különböző gépformátumot olvas (`layers`/`inputs`, illetve
   `initial`/`states`); a `state-bake` használati sorához nincs leírás, és formátumot sem ad meg.
9. **Javítva - `stats`** már nem fogadja el a `--json` opciót (a kimenet mindig JSON; a használati sor ezt közli).
10. **`vcs`:** az alapértelmezett ág a `main` (a `checkout master` ezzel bukik: `unknown revision 'master'`); a `vcs resolve`
    az ütközés útvonalát a *kezdő perjellel* együtt várja, pontosan úgy, ahogy a `merge` kiírja (`/background`), a puszta `background`
    ezt adja: `no conflict at 'background'`; a használati szöveg azt írja: "see: osatura vcs", de az argumentum nélküli `osatura vcs` egysoros
    használati leírást ír ki, és 1-gyel lép ki.
11. **Leírás nélküli használati sorok.** A `state-bake`, `pose-match`, `pose-bench`, `follow`, `keys`, `event`, `marker`,
    `idle`, `mouth`, `beats`, `export`, `frames`, `onion` és mások parancsainak van szintaxisa, de magyarázata nincs, vagy a leírás
    egy folytatósorban van; viselkedésük ebben a dokumentumban a futtatásukból származik.
12. **Tulajdonságok elnevezése.** Az üzenetek és a `list` kimenetek a tulajdonságokat CamelCase alakban írják ki (`LayerOpacity`, `BoneAngle`),
    de minden opció, leképezőfájl és mellékfájl a snake_case neveket várja (`layer_opacity`, `bone_angle`); a CamelCase alakot
    elutasítja (a `bind add ... sun:LayerOpacity` ezzel bukik: `unknown property 'LayerOpacity'`).
13. **`cloth`:** a használati szöveg "csúcsonkénti deformációról" beszél; a beégetett eredmény `lattice_dx` / `lattice_dy`
    csatornákként tárolódik a ruha rácspontjain (az alapértelmezett 7x7-es rácsnál a 42 rögzítetlen ponton), lásd a 7. szakaszt.
14. **A `render-gpu`** csak a `--features gpu` opcióval készült buildekben létezik; a tesztbuildben nem lehetett futtatni, ezért az opciói
    a használati szövegből származnak, és itt nincsenek ellenőrizve.
15. **Debug buildekben lassú.** A `pose-bench`, `face-train`, `ci run` és `export-fit` percekig tart optimalizálatlan buildben.
    Ennek a dokumentumnak az időmérései a debug binárisból származnak, és release buildben jóval rövidebbek.
16. **Javítva (5. hullám) - `golden write --scale`** korábban az 1 feletti értéket csendben 1-re vágta; mostantól elutasítja (a skála a (0, 1] tartományban van).
    A `golden check` figyelmen kívül hagyta a `--scale` opciót, és a tárolt értéket használta; mostantól elutasítja az opciót.
17. **Javítva (5. hullám) - szintaxisok.** A parancsok használati szövegének minden opciója szerepel az itteni szintaxisban, és fordítva (teszt
    `cli_reference_synopses_match_usage`), és minden alternatíva nélküli használati sor szó szerint megismétlődik itt (teszt
    `cli_reference_repeats_usage_lines_verbatim`). A `render-gpu --features gpu` és a `layer-normal ... relight --layer-normals`
    a használati leírásokban egy build-funkciót és egy másik parancs opcióját nevezi meg, nem az adott parancsok opcióit.
18. **Más dokumentumok két helyen régebbiek a programnál** (lásd az [EXPORT_FORMATS.md](EXPORT_FORMATS.md) 17. szakaszát):
    a `docs/LIMITATIONS.md` L-29 azt állítja, hogy a Lottie-import nem kezel képeket, effekteket, előkompozíciókat vagy matte-eket (az importáló ma már kezeli
    őket, a 9. szakaszban felsorolt figyelmeztetésekkel), az L-32 pedig az OCA-t az importformátumok között sorolja fel, miközben a CLI-ben csak az
    `export-oca` van meg.
## Abc-sorrendű parancsindex

Minden parancs egysoros összefoglalóval és a dokumentáló szakasszal. Az alparancsok (`machine add`, `vcs commit`, ...) a szülőparancsuk szakaszában találhatók.

| Parancs | Összefoglaló | Szakasz |
|---|---|---|
| `--version` | kiírja a verziót, a licencet, a forrásajánlatot és a támogatási hivatkozásokat | [1](#1-az-összes-parancsra-érvényes-konvenciók) |
| `align` | rétegek igazítása egymáshoz vagy a színpadhoz | [5](#5-rigging-skinek-ik-és-elrendezési-segédparancsok) |
| `audio-react` | a hangerő vezérel egy tulajdonságot | [8](#8-hang-ajaksinkron-és-hangok) |
| `autorig` | IK, légzés és pislogás beállítása csont- és rétegnevek alapján | [5](#5-rigging-skinek-ik-és-elrendezési-segédparancsok) |
| `autotrace` | helyi bittérkép-követés (színsávok, Bezier-illesztés) | [9](#9-import) |
| `autotrace-bench` | a követő hűségtáblázata szintetikus alakzatokon | [9](#9-import) |
| `beats` | hangerőugrások észlelése, markerek vagy események rögzítése | [8](#8-hang-ajaksinkron-és-hangok) |
| `bind` | adatkötések: add, list, remove | [11](#11-állapotgépek-adatkötések-konténerek-és-görgetési-területek) |
| `bind-eval` | kötések kiértékelése megadott értékekre | [11](#11-állapotgépek-adatkötések-konténerek-és-görgetési-területek) |
| `bone-vis` | egy csont megjelenítése csak az átfedésekben egy képkocka-tartományra | [5](#5-rigging-skinek-ik-és-elrendezési-segédparancsok) |
| `cage` | középérték-koordinátás kalitka-deformer hozzáadása | [7](#7-fizika-ruha-és-deformerek) |
| `cage-pose` | kalitkacsúcs-eltolások kulcsolása egy képkockán | [7](#7-fizika-ruha-és-deformerek) |
| `camera-seq` | a kamera kulcsolása mentett felvételeken át | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `chan-meta` | csatorna színe, shy, solo és zárolási jelzők | [5](#5-rigging-skinek-ik-és-elrendezési-segédparancsok) |
| `check` | projekt ellenőrzése (1-es kilépés, ha érvénytelen) | [3](#3-projekt-létrehozása-és-vizsgálata) |
| `ci` | animáció-CI: golden képek, észleleti diff, szerkezeti ellenőrzések | [14](#14-minőségbiztosítás) |
| `clip` | take rétegezése a klipkötegre | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `cloth` | determinisztikus ruhaszimuláció beégetése | [7](#7-fizika-ruha-és-deformerek) |
| `coach` | időzítési/alapelv-javaslatok | [14](#14-minőségbiztosítás) |
| `collab-sim` | együttműködési konvergenciateszt | [16](#16-stúdió-együttműködés-mcp-ügynökszerver-és-ui-regiszter) |
| `collab-token` | résztvevőnkénti együttműködési token kiállítása (HMAC-SHA-256, lejár) | [16](#16-stúdió-együttműködés-mcp-ügynökszerver-és-ui-regiszter) |
| `color-space` | sRGB vagy lineáris fényű kompozíció választása a projekthez | [12](#12-effektek-mélységélesség-onion-skin-nyomvonalak-és-kamera) |
| `comp` | elnevezett rétegállapotok (láthatóság, átlátszóság, kapcsolóválasztás) | [5](#5-rigging-skinek-ik-és-elrendezési-segédparancsok) |
| `compare` | két projekt előtte / utána / különbség képe | [4](#4-renderelés-és-exportálás) |
| `container` | elrendezési konténerek: add, list, remove, solve | [11](#11-állapotgépek-adatkötések-konténerek-és-görgetési-területek) |
| `contracts` | mozgásszerződések (egységtesztek animációhoz) | [14](#14-minőségbiztosítás) |
| `demo` | a beépített rig-demoprojekt kiírása | [3](#3-projekt-létrehozása-és-vizsgálata) |
| `demo-fx` | a folyadék / boolean / részecske demo kiírása | [3](#3-projekt-létrehozása-és-vizsgálata) |
| `diff` | két projekt szemantikus különbségei | [3](#3-projekt-létrehozása-és-vizsgálata) |
| `distribute` | három vagy több réteg egyenletes elosztása | [5](#5-rigging-skinek-ik-és-elrendezési-segédparancsok) |
| `dof` | egy képkocka mélységélesség-renderelése | [12](#12-effektek-mélységélesség-onion-skin-nyomvonalak-és-kamera) |
| `dof-export` | animált mélységélesség (GIF/APNG/videó/mappa) | [12](#12-effektek-mélységélesség-onion-skin-nyomvonalak-és-kamera) |
| `drive` | CSV-jelek -> kulcsok leképezőfájllal | [10](#10-élő-bemenet-midi-arckövetés-és-mozgásrögzítés) |
| `event` | elnevezett futásidejű esemény hozzáadása (opcionális JSON-adattal) | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `explain` | miért van egy tulajdonságnak az értéke: rendezett hozzájárulások | [3](#3-projekt-létrehozása-és-vizsgálata) |
| `export` | GIF, APNG, WebM/MP4, PNG-lap, HTML, Lottie, .lottie, glTF/GLB, ... exportálása | [4](#4-renderelés-és-exportálás) |
| `export-fit` | a bájtkeretbe belefér a legjobb minőségű export | [13](#13-részletességi-szintek-méretjelentések-és-export-fit) |
| `export-oca` | Open Cel Animation mappa kiírása | [4](#4-renderelés-és-exportálás) |
| `eye-clip` | írisz-/pupillarétegek maszkolása a szem alakjával | [5](#5-rigging-skinek-ik-és-elrendezési-segédparancsok) |
| `face-bake` | landmarkok -> kulcsok | [10](#10-élő-bemenet-midi-arckövetés-és-mozgásrögzítés) |
| `face-calibrate` | semleges arc kalibrációja | [10](#10-élő-bemenet-midi-arckövetés-és-mozgásrögzítés) |
| `face-detect` | arcdobozok és 23 landmark egy képen | [10](#10-élő-bemenet-midi-arckövetés-és-mozgásrögzítés) |
| `face-eval` | a detektor kihagyott pontossági jelentése | [10](#10-élő-bemenet-midi-arckövetés-és-mozgásrögzítés) |
| `face-synth` | szintetikus fejanimáció | [10](#10-élő-bemenet-midi-arckövetés-és-mozgásrögzítés) |
| `face-track` | landmark-NDJSON videóból vagy képkockákból | [10](#10-élő-bemenet-midi-arckövetés-és-mozgásrögzítés) |
| `face-train` | az arcdetektor tanítása szintetikus arcokon | [10](#10-élő-bemenet-midi-arckövetés-és-mozgásrögzítés) |
| `follow` | átfedő mozgás / utánmozgás generátorok | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `foot-lock` | vezérlő rögzítése talajérintkezés alatt | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `frames` | minden képkocka renderelése egy mappába (SVG, vagy PNG a --png opcióval) | [4](#4-renderelés-és-exportálás) |
| `fuzz` | olyan pózok keresése, ahol a rig eltörik | [14](#14-minőségbiztosítás) |
| `fuzz-grid` | két paraméter súlyosságtérképe | [14](#14-minőségbiztosítás) |
| `fx` | képkocka renderelése utófeldolgozó effektlánccal | [12](#12-effektek-mélységélesség-onion-skin-nyomvonalak-és-kamera) |
| `glue` | mesh-csúcsok összeragasztása | [7](#7-fizika-ruha-és-deformerek) |
| `golden` | regressziós képkockák írása/ellenőrzése | [14](#14-minőségbiztosítás) |
| `idle` | procedurális üresjárati csatorna generálása | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `import-anim` | animált GIF/APNG -> képkockánkénti réteg | [9](#9-import) |
| `import-bvh` | BVH mozgásrögzítés -> csatornák | [9](#9-import) |
| `import-lottie` | Lottie-részhalmaz -> projekt | [9](#9-import) |
| `import-ora` | OpenRaster -> projekt | [9](#9-import) |
| `import-seq` | PNG-sorozat -> képkockánkénti réteg | [9](#9-import) |
| `import-svg` | SVG-részhalmaz -> vektorréteg | [9](#9-import) |
| `keyedit` | dope sheet szerkesztések egyetlen, visszavonásbiztos kötegként | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `keys` | tömeges kulcsműveletek az összes kulcsolt csatornán | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `layer-normal` | normáltérkép csatolása egy réteghez | [12](#12-effektek-mélységélesség-onion-skin-nyomvonalak-és-kamera) |
| `layer-pivot` | egy réteg pivotjának mozgatása a kép megváltoztatása nélkül | [7](#7-fizika-ruha-és-deformerek) |
| `layout-preview` | renderelés másik vászonmérethez újramegoldott elrendezéssel | [11](#11-állapotgépek-adatkötések-konténerek-és-görgetési-területek) |
| `lint` | statikus rig-/animációellenőrzések (1-es kilépés hibáknál) | [14](#14-minőségbiztosítás) |
| `lipsync` | szájkapcsoló-réteg vezérlése a hang hangerejéből (nem fonéma-alapú) | [8](#8-hang-ajaksinkron-és-hangok) |
| `lipsync-text` | átirat igazítása a hanghoz és szájformák levezetése | [8](#8-hang-ajaksinkron-és-hangok) |
| `live-map` | rögzített élő bemenet (MIDI, egér, gamepad, billentyűk) -> kulcsok, take-ek, események | [10](#10-élő-bemenet-midi-arckövetés-és-mozgásrögzítés) |
| `lod` | részletességi szintek megadott pixelhiba-korlátokkal | [13](#13-részletességi-szintek-méretjelentések-és-export-fit) |
| `lod-pick` | a legdurvább szint, amely belefér egy képernyőn a pixelkeretbe | [13](#13-részletességi-szintek-méretjelentések-és-export-fit) |
| `loop-solve` | kulcsolt csatornák zökkenőmentes hurkolása | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `machine` | futásidejű állapotgépek: add, list, remove, validate | [11](#11-állapotgépek-adatkötések-konténerek-és-görgetési-területek) |
| `machine-sim` | időzített bemenetek visszajátszása: állapotnapló és SVG-képkockák | [11](#11-állapotgépek-adatkötések-konténerek-és-görgetési-területek) |
| `magnet` | közelségi vonzás beégetése | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `make-loop` | ciklus zökkenőmentes hurkolása (opcionálisan helyben) | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `marker` | idővonal-marker vagy -tartomány hozzáadása | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `mcp` | ügynökszerver (JSON-RPC stdio-n) | [16](#16-stúdió-együttműködés-mcp-ügynökszerver-és-ui-regiszter) |
| `mesh-lock` | követők a gazda mesh felszínét követik | [7](#7-fizika-ruha-és-deformerek) |
| `mesh-trace` | kép alfa-sziluettjének követése skinelt háromszögmeshé | [7](#7-fizika-ruha-és-deformerek) |
| `mirror` | az összes kulcsolt csatorna tükrözése bal<->jobb | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `mocap-info` | egy BVH-fájl hierarchiája és kiterjedése | [10](#10-élő-bemenet-midi-arckövetés-és-mozgásrögzítés) |
| `mocap-match` | BVH-ízület és rig-csont párosítása | [10](#10-élő-bemenet-midi-arckövetés-és-mozgásrögzítés) |
| `mocap-retarget` | mocap beégetése hozzáfűzött csatornákként | [10](#10-élő-bemenet-midi-arckövetés-és-mozgásrögzítés) |
| `mouth` | szájjelzések importálása (Papagayo/Rhubarb) | [8](#8-hang-ajaksinkron-és-hangok) |
| `onion` | onion skin PNG vagy SVG | [12](#12-effektek-mélységélesség-onion-skin-nyomvonalak-és-kamera) |
| `parallax` | multiplane mélység a legfelső szintű rétegekhez | [5](#5-rigging-skinek-ik-és-elrendezési-segédparancsok) |
| `parent-switch` | animált szülőváltás beégetése ugrás nélkül | [5](#5-rigging-skinek-ik-és-elrendezési-segédparancsok) |
| `physics` | láncszimuláció beégetése | [7](#7-fizika-ruha-és-deformerek) |
| `pick` | mi van egy vászonpont alatt (a legfelső elöl) | [3](#3-projekt-létrehozása-és-vizsgálata) |
| `pose-bench` | a pózmódszerek kihagyott pontossága | [17](#17-póz-adatbázis-javaslatok-és-egyéb-segédek) |
| `pose-db` | póz-adatbázis: build, info, train-mlp | [17](#17-póz-adatbázis-javaslatok-és-egyéb-segédek) |
| `pose-match` | paraméterek illesztése, hogy a csontvégek elérjék a célokat | [17](#17-póz-adatbázis-javaslatok-és-egyéb-segédek) |
| `pose-suggest` | átmeneti, kiegészítő és következő póz javaslatok | [17](#17-póz-adatbázis-javaslatok-és-egyéb-segédek) |
| `proxy-bench` | ezredmásodperc képkockánként minden proxyszinten | [4](#4-renderelés-és-exportálás) |
| `reduced-motion` | a projekt nyugodtabb másolatának kiírása | [14](#14-minőségbiztosítás) |
| `reference` | réteg megjelölése követési referenciaként (soha nem exportálódik) | [5](#5-rigging-skinek-ik-és-elrendezési-segédparancsok) |
| `render` | egy képkocka renderelése PNG-be vagy SVG-be (scrub proxyk, csontok, skála) | [4](#4-renderelés-és-exportálás) |
| `render-gpu` | GPU-renderelés (csak wgpu buildek) | [4](#4-renderelés-és-exportálás) |
| `render-hdr` | lebegőpontos renderelés OpenEXR-be vagy 16 bites PNG-be | [4](#4-renderelés-és-exportálás) |
| `reparent` | egy csont szülőjének cseréje a nyugalmi póz megtartásával | [5](#5-rigging-skinek-ik-és-elrendezési-segédparancsok) |
| `retarget` | animáció másolása rigek között lánc-párok szerint | [5](#5-rigging-skinek-ik-és-elrendezési-segédparancsok) |
| `safety` | villanás / vörös villanás / kameramozgás jelentés | [14](#14-minőségbiztosítás) |
| `script` | sandboxolt Rhai szkript futtatása, amely szerkeszti a projektet | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `scroll` | görgetési területek: add, list, remove, eval | [11](#11-állapotgépek-adatkötések-konténerek-és-görgetési-területek) |
| `select-box` | kijelölés keretezéssel | [3](#3-projekt-létrehozása-és-vizsgálata) |
| `shot` | kamerafelvétel mentése | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `size-report` | hol vannak egy projekt/export bájtjai | [13](#13-részletességi-szintek-méretjelentések-és-export-fit) |
| `skin` | elnevezett skinek: rétegkészletek, csak skinhez tartozó csontok, kényszerek, IK-láncok | [5](#5-rigging-skinek-ik-és-elrendezési-segédparancsok) |
| `skin-diffuse` | hődiffúziós skin-súlyok | [7](#7-fizika-ruha-és-deformerek) |
| `slider` | tárolt pózokat keverő vezérlőcsont létrehozása | [5](#5-rigging-skinek-ik-és-elrendezési-segédparancsok) |
| `snapshot` | egy projekt számozott revíziótörténete egy mappában (save, list, restore) | [3](#3-projekt-létrehozása-és-vizsgálata) |
| `soft-scale` | kijelölés arányos skálázása | [7](#7-fizika-ruha-és-deformerek) |
| `sound` | projekt hangsávjai: list, add, edit, remove, mix | [8](#8-hang-ajaksinkron-és-hangok) |
| `speech-motion` | additív bólintások és szemöldökemelések igazított szavakra időzítve | [8](#8-hang-ajaksinkron-és-hangok) |
| `spring-fit` | rugófrekvencia/csillapítás illesztése egy görbéhez | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `state-bake` | képkocka-alapú állapotgép beégetése kulcsokba | [11](#11-állapotgépek-adatkötések-konténerek-és-görgetési-területek) |
| `stats` | csontok, rétegek, csatornák, kulcsok darabszáma (JSON) | [3](#3-projekt-létrehozása-és-vizsgálata) |
| `stretch` | nyúlva elérő követő beégetése | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `studio` | böngészős szerkesztő (helyi webszerver) | [16](#16-stúdió-együttműködés-mcp-ügynökszerver-és-ui-regiszter) |
| `tag` | szabványos szerepnevek javaslata csontokhoz és rétegekhez | [3](#3-projekt-létrehozása-és-vizsgálata) |
| `take-apply` | könyvtári take alkalmazása átúsztatással | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `take-list` | egy könyvtár take-jeinek listázása | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `take-record` | take rögzítése egy könyvtárba | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `timecode` | képkocka <-> SMPTE időkód (drop-frame-tudatos) | [3](#3-projekt-létrehozása-és-vizsgálata) |
| `timeline` | csatornánkénti sávok kulcsokkal, zárolásokkal és színekkel | [3](#3-projekt-létrehozása-és-vizsgálata) |
| `trace` | PNG alfacsatornájának követése vektor útvonallá | [9](#9-import) |
| `trail` | csont, réteg vagy pont mozgásnyomvonala | [12](#12-effektek-mélységélesség-onion-skin-nyomvonalak-és-kamera) |
| `trim` | trim path beégetése egy vektorrétegbe | [5](#5-rigging-skinek-ik-és-elrendezési-segédparancsok) |
| `ui` | stúdió parancsregiszter, gyorsbillentyűk, elrendezések, keresés | [16](#16-stúdió-együttműködés-mcp-ügynökszerver-és-ui-regiszter) |
| `vcs` | szemantikus összefésüléses verziókezelés egy projektfájlhoz | [15](#15-verziókezelés-vcs) |
| `view-swap` | kapcsolóréteg vezérlése csont tulajdonságából (lépcsőfüggvény) | [5](#5-rigging-skinek-ik-és-elrendezési-segédparancsok) |
| `walk` | járásciklus generálása | [6](#6-animáció-kulcsok-idővonal-és-generátorok) |
| `warp-pivot` | vezérlőpontok kijelölésének forgatása/skálázása/mozgatása | [7](#7-fizika-ruha-és-deformerek) |
| `waveform` | min/max csúcsok rajzoláshoz | [8](#8-hang-ajaksinkron-és-hangok) |
