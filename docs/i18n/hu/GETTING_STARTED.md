# Első lépések az OSATURA-val

Ez az útmutató egy friss checkouttól elvezet egy kis animált karakterig, amelyet kizárólag a parancssor
használatával építesz fel, riggelsz, kulcsolsz, hurkolsz, renderelsz és exportálsz. Az alábbi parancsok
mindegyikét valóban lefuttattuk az oldal írása közben; a bemutatott kimenetek a ténylegesen kiírt kimenetek.

Hogyan ellenőriztük: Linux, `rustc 1.97.0`, `ffmpeg 6.1.1` és a bináris **debug** buildje
(`cargo build -p osatura-cli`). A release build ugyanazt a kódot futtatja gyorsabban. A tesztkészlet macOS-en és Windowson is sikeres a GitHub Actionsön; az ezen az oldalon szereplő parancsokat kézzel csak Linuxon futtattuk. A tárolóban nincs
megadva minimálisan támogatott Rust-verzió; az 1.97.0 egyszerűen az a fordító, amellyel teszteltünk.

A dokumentáció licence: CC-BY-4.0. Lásd még: [RIGGING_GUIDE.md](RIGGING_GUIDE.md),
[ANIMATION_GUIDE.md](ANIMATION_GUIDE.md), [STUDIO.md](STUDIO.md), [FAQ.md](FAQ.md), valamint a teljes
parancslista a [CLI_REFERENCE.md](CLI_REFERENCE.md) fájlban.

## 1. Mire van szükséged

| Követelmény | Mihez kell | Megjegyzések |
|---|---|---|
| Rust eszközlánc (`rustc`, `cargo`) | az OSATURA fordításához | telepítsd innen: <https://rustup.rs>; stabil csatorna |
| Egy böngésző | a stúdióhoz (`osatura studio`) | csak Chromiumot teszteltünk, lásd [STUDIO.md](STUDIO.md) |
| `ffmpeg` a `PATH`-on (opcionális) | `.mp4`, `.webm`, hang keverése videóba/HTML-be, `sound mix`, `face-track` videófájlon | minden más nélküle is működik |

`ffmpeg` nélkül az őt igénylő parancsok érthető üzenettel leállnak, és semmit sem írnak ki:

```
$ PATH=/nonexistent osatura export buddy-final.json o.mp4
cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead
```

GPU-t nem igényel. Az opcionális GPU-renderer (`render-gpu`) nem része az alapértelmezett buildnek.

**Nincs Rustod? Használj előre elkészített archívumot.** Amikor a tulajdonos közzétette a `v0.1.0-alpha.3` előzetes kiadás-vázlatot, a [Releases oldal](https://github.com/chiuta/osatura/releases) kínálja az `osatura-windows-x86_64.zip`, `osatura-macos-arm64.tar.gz`, `osatura-linux-x86_64.tar.gz` (glibc 2.35+; a későbbi kiadások régebbi disztribúciókhoz hozzáadják a statikus `osatura-linux-x86_64-musl.tar.gz` fájlt) és `SHA256SUMS.txt` fájlokat. Ellenőrizd az archívumot az összegekkel (`sha256sum -c SHA256SUMS.txt --ignore-missing`, Windowson `Get-FileHash`), csomagold ki, és futtasd az `osatura` (`osatura.exe`) programot a kicsomagolt mappából; a 2. szakasz fordítását hagyd ki, és olvasd el az oldal vége felé a „Windows: az `osatura.exe` dupla kattintása” és „A stúdió ablakban” részt. A programok nincsenek kódaláírással ellátva, ezért a rendszer az első alkalommal figyelmeztetni fog. Minden alább szereplőt ugyanennek a forrásnak helyi buildjével futtattunk.

## 2. Fordítás

A tároló gyökeréből:

```
cargo build --release -p osatura-cli
```

A bináris a `target/release/osatura`. Tedd a `PATH`-ra, vagy készíts hozzá aliast:

```
export PATH="$PWD/target/release:$PATH"      # or: alias osatura=$PWD/target/release/osatura
osatura --version
```

A `--version` kiírja a verziót, a licencet (AGPL-3.0-or-later a CLI-re és a stúdióra, MPL-2.0 a
motorkönyvtárakra) és a forráskód-ajánlatot. Az `osatura` argumentumok nélküli futtatása kiírja a teljes
súgót; az `osatura <command> --help` egyetlen parancs használatát írja ki.

Telepítés nélkül is futtathatod: `cargo run --release -p osatura-cli -- demo demo.json`.

A parancssori eszköz `--no-default-features` kapcsolóval történő fordítása elhagyja a `script` parancsot (a
Rhai szkriptmotort); ez a bemutató használja a `script` parancsot, ezért hagyd meg az alapértelmezett funkciókat.

## 3. Első lépések a beépített demóval

Az `osatura demo` egy teljes példaprojektet ír ki: egy skinnelt kart, egy IK-lábat, egy lip-sync
kapcsolóréteget, egy parallax hátteret és egy kameramozgást.

```
$ osatura demo demo.json
wrote demo.json
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels
```

A `check` pontosan úgy fordítja le a projektet, ahogy a renderer tenné, ezért az a projekt, amely átmegy a
`check`-en, renderelhető. Most rajzolj egy állóképet a csontok rávetítésével, és exportálj egy animációt:

```
$ osatura render demo.json 12 frame12.png --bones
wrote frame12.png
$ osatura render demo.json 12 frame12.svg
wrote frame12.svg
$ osatura export demo.json demo.gif
wrote demo.gif
$ osatura export demo.json demo.html
wrote demo.html
$ osatura export demo.json demo.lottie
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
wrote demo.lottie
```

Mit csináltál az imént:

* A `render <project> <frame> <out.png|out.svg>` egyetlen képkockát rajzol. A `--bones` ráveti a
  csontvázat, a `--scale N` a vászonméret N-szeresén renderel (0 < N <= 16).
* Az `export` a fájlkiterjesztésből választ formátumot: `.gif`, `.html` (önálló lejátszóoldal),
  `.json` (Lottie), `.lottie` (dotLottie), `.apng`, `.webm`, `.mp4`, `.glb`, `.png` (sprite sheet).
* A `warning:` sorok az őszinte rész: a Lottie nem tud mindent ábrázolni, amit az OSATURA. Minden
  korlátozást kiír az stderr-re, a `--report r.json` pedig ugyanezt a listát JSON-ként írja ki.
* Nyisd meg a `demo.html` fájlt böngészőben az animáció lejátszásához; nincs szükség szerverre és hálózatra.

Interaktív felfedezéshez:

```
osatura studio demo.json --port 8080
```

és nyisd meg a <http://127.0.0.1:8080/> címet. Lásd [STUDIO.md](STUDIO.md).

## 4. 15 perces bemutató: egy integető karakter

A „Buddy” karaktert egy üres projektből építed fel. A projektfájl JSON, de egy fejlécnél többet sosem kell
írnod: a `script` parancs egy kis, sandboxolt [Rhai](https://rhai.rs) szkriptet futtat ugyanazon az
ellenőrzött szerkesztőn keresztül, amelyet a stúdió is használ. Ha a szkript egy kérését elutasítják,
*semmi* sem érvényesül (mindent vagy semmit), így egy hibás szkript sosem hagy félig szerkesztett fájlt.

Dolgozz egy üres könyvtárban. Minden lépés új, számozott fájlt ír, így mindig visszaléphetsz.

### 1. lépés: üres projekt (1 perc)

```
cat > buddy.json <<'EOF'
{"format": 1, "name": "Buddy", "width": 320, "height": 240, "fps": 24, "frames": 24}
EOF
osatura check buddy.json
```
```
ok: 'Buddy' 320×240 @ 24 fps, 24 frames, 0 bones, 0 ik, 0 layers, 0 channels
```

Ez a hat kulcs a minimum. 24 képkocka 24 fps mellett egy másodperc, ez lesz a hurkunk.

### 2. lépés: alakzatok (2 perc)

A rétegek hátulról előre, a hozzáadás sorrendjében rajzolódnak. Hozd létre a `shapes.rhai` fájlt:

```
add_gradient_rect("sky", 0, 0, 320, 240, "#bfe3f5", "#fdf3d8");
add_rect("ground", 0, 200, 320, 40, "#7fa66b");
add_rect("torso", 140, 120, 40, 70, "#e07a5f");
add_circle("head", 160, 95, 24, "#f2cc8f");
add_rect("upper_arm", 160, 134, 40, 12, "#3d405b");
add_rect("forearm", 200, 134, 40, 12, "#81b29a");
```
```
$ osatura script buddy.json 01-shapes.json shapes.rhai
6 request(s) applied; wrote 01-shapes.json
```

A koordináták vászonegységek, az origó a bal felső sarokban van, az y lefelé mutat. A kar vízszintesen
van megrajzolva, a váll a (160, 140) pontban, mert ebben a pózban fogjuk felépíteni a csontjait.

### 3. lépés: csontok, szülőség és a grafika csatolása (3 perc)

Egy csontnak van neve, szülője (üres = gyökér), hossza, a szülője terében megadott fejpozíciója (`x`, `y`)
(gyökérnél: a vásznon) és a szülőjéhez viszonyított nyugalmi szöge fokban. A 0-s szög a +x irányába mutat,
a pozitív szögek a képernyőn az óramutató járásával megegyezően fordulnak, tehát a 90 lefelé, a -90 felfelé mutat.
Hozd létre a `bones.rhai` fájlt:

```
add_bone("body", "", 70, 160, 190, -90);
add_bone("neck", "body", 20, 70, 0, 0);
add_bone("arm_up", "body", 40, 50, 0, 90);
add_bone("arm_low", "arm_up", 40, 40, 0, 0);
add_bone("hand_ctrl", "", 10, 210, 165, 0);
set_layer("torso", "bone", "body");
set_layer("head", "bone", "neck");
set_layer("upper_arm", "bone", "arm_up");
set_layer("forearm", "bone", "arm_low");
```
```
$ osatura script 01-shapes.json 02-bones.json bones.rhai
9 request(s) applied; wrote 02-bones.json
$ osatura render 02-bones.json 0 rig.png --bones --scale 2
wrote rig.png
```

A csontokat így olvasd: a `body` egy gyökércsont a (160, 190) pontban, felfelé mutat (-90), és 70 hosszú.
A `neck` a gyereke; a feje a `body` mentén 70 egységre van (a tetején). Az `arm_up` a `body` másik gyereke,
50 egységre a gerinc mentén felfelé (a váll), +90-nel elforgatva, így oldalra mutat. Az `arm_low` az `arm_up`
gyereke: a szülőség azt jelenti, hogy a felkar elforgatása magával viszi az alkart. A `hand_ctrl` külön
gyökércsont, amelyet IK-célpontként fogunk használni; nincs grafikája.

A `set_layer(layer, "bone", bone)` egy réteget *mereven* egy csonthoz köt: követi a csont mozgását a
nyugalmi pózához képest. Ez a legegyszerűbb módja egy különálló darabokból álló rajzfilmfigura riggelésének.
(Hajlékony alakzatokhoz vektoros horgonyokat skinnelsz több csonthoz; lásd [RIGGING_GUIDE.md](RIGGING_GUIDE.md).)

### 4. lépés: inverz kinematika (1 perc)

A váll- és könyökszögek kulcsolása helyett hagyd, hogy a solver a `hand_ctrl` felé nyúljon. Hozd létre az `ik.rhai` fájlt:

```
add_ik("arm_ik", ["arm_up", "arm_low"], "hand_ctrl", 1);
```
```
$ osatura script 02-bones.json 03-ik.json ik.rhai
1 request(s) applied; wrote 03-ik.json
```

Az utolsó argumentum a hajlítás iránya: a +1 vagy -1 választja ki, merre hajoljon a könyök. A kétcsontos
lánc analitikus solvert használ; a három vagy több csontból álló láncok FABRIK-ot. Mostantól a `hand_ctrl`
mozgatása az egész kart mozgatja.

### 5. lépés: kulcsok és easing (3 perc)

A `set_key(target, property, frame, value, ease)` kulcsokat hoz létre. Csontnál a `bone_angle`, a `bone_scale`,
a `bone_offset_x` és a `bone_offset_y` a *nyugalmi pózhoz viszonyított különbségek*, ezért a 0 kulcsérték
„nyugalmat” jelent. Az easing neve arra a szakaszra vonatkozik, amely az adott kulcsnál *kezdődik*. Hozd létre a `keys.rhai` fájlt:

```
set_key("hand_ctrl", "bone_offset_y", 0, 0, "ease_in_out");
set_key("hand_ctrl", "bone_offset_y", 12, -75, "ease_in_out");
set_key("hand_ctrl", "bone_offset_y", 24, 0, "ease_in_out");
set_key("neck", "bone_angle", 0, 0, "smooth");
set_key("neck", "bone_angle", 16, 10, "smooth");
set_key("neck", "bone_angle", 24, 0, "smooth");
```
```
$ osatura script 03-ik.json 04-keys.json keys.rhai
6 request(s) applied; wrote 04-keys.json
$ osatura timeline 04-keys.json
timeline: 24 frames at 24 fps, 2 channels
Bones
  hand_ctrl
    #0 bone_offset_y (keys @ 0 12 24)
  body
    neck
      #1 bone_angle (keys @ 0 16 24)
```

A paraméter nélküli easing-nevek: `linear`, `step`, `ease_in`, `ease_out`, `ease_in_out`,
`smooth` és `bounce`. A `bezier:x1,y1,x2,y2` formát a `keyedit` fogadja el, a `spring` és a `back` pedig
paramétereket vesz fel (lásd [PROJECT_FORMAT.md](PROJECT_FORMAT.md)). A kéz és a fej szándékosan eltérő időzítést használ: az átfedő mozgás életszerűnek hat.

### 6. lépés: legyen hurok (1 perc)

Mindkét csatorna ugyanazzal az értékkel kezdődik és végződik (a 24. képkocka megismétli a 0.-t), és a hurokhoz
erre van szükség. A `loop-solve` ellenőrzi az illesztést, és a csatornákat hurkosra állítja:

```
$ osatura loop-solve 04-keys.json buddy-final.json
wrote buddy-final.json: 2 channel(s) adjusted, seam 0.000 -> 0.000
$ osatura lint buddy-final.json
0 finding(s), 0 error(s)
```

A „seam 0.000 -> 0.000” azt jelenti, hogy az utolsó póz már megegyezik az elsővel, így nem kellett semmit
korrigálni. Egy N képkockás projekt a 0..N-1 képkockákat játssza ismételten, tehát az N. képkocka ugyanaz a póz,
mint a 0., és sosem jelenik meg kétszer. Ha a hurkot úgy szerkeszted, hogy az *utolsó megjelenített* képkocka
egyezzen az elsővel, add meg az `--inclusive` kapcsolót (a régebbi konvenció). Ha az animációdnak nettó
elmozdulása van, a `make-loop` eltávolítja azt. Lásd [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).

A `lint` statikus rig- és animációellenőrzéseket futtat (elérhetetlen IK-célpontok, halott csatornák, hiányzó csontok...);
hiba esetén 1-gyel lép ki, így biztonságosan betehető szkriptbe vagy CI-feladatba.

### 7. lépés: nézd meg, renderelj (2 perc)

```
$ osatura stats buddy-final.json
```
JSON-összefoglalót ír ki (itt: 5 csont, 1 IK-lánc, 6 réteg, 2 csatorna, 6 kulcs). Renderelj állóképeket és
egy teljes képkockasorozatot:

```
$ osatura render buddy-final.json 12 frame12.png --scale 2
wrote frame12.png
$ osatura render buddy-final.json 12 frame12.svg
wrote frame12.svg
$ osatura frames buddy-final.json frames --png
wrote 24 frames to frames
$ ls frames | head -3
frame_0000.png
frame_0001.png
frame_0002.png
```

![A Buddy-animáció négy képkockája (0, 6, 12, 18), egyszer sima, egyszer a csontokkal együtt. A jobb kar könyökkel lefelé integetve emelkedik, miközben a fej enyhén bólint.](../../img/tutorial-buddy.png)

`--png` nélkül a `frames` SVG-fájlokat ír. A renderelés determinisztikus: ugyanaz a projekt és képkocka
ugyanazokat a bájtokat adja bármely, azonos buildű gépen, és ez teszi lehetővé az [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)
referenciakocka-ellenőrzéseit.

### 8. lépés: export (2 perc)

```
$ osatura export buddy-final.json buddy.gif
wrote buddy.gif
$ osatura export buddy-final.json buddy.html
wrote buddy.html
$ osatura export buddy-final.json buddy.json --report buddy-report.json
wrote buddy.json
$ cat buddy-report.json
{
  "count": 0,
  "format": "lottie",
  "warnings": []
}
$ osatura export buddy-final.json buddy.mp4        # needs ffmpeg
wrote buddy.mp4
$ osatura export buddy-final.json buddy.apng
wrote buddy.apng
```

Ennek a pici projektnek a fájlméretei: GIF 145 KB, Lottie JSON 21 KB, HTML-lejátszó 50 KB, APNG 75 KB, MP4 6 KB.
A `buddy.json` Lottie; a figyelmeztetések listája üres, mert a Buddy csak olyan funkciókat használ, amelyeket a
Lottie ki tud fejezni. A HTML-fájl önálló lejátszó (bármely böngészőben megnyitható).

Néhány megjegyezni való szabály:

* Az `export` és a `render` **megtagadja a bemeneti projekt felülírását** (`refusing to overwrite the input
  project buddy-final.json with output of the same name`), és szimbolikus linken keresztül sem ír, hacsak
  meg nem adod a `--follow-symlinks` kapcsolót. Az egyik projektet a másikba alakító parancsok
  (`script`, `loop-solve`, `walk`, `autorig`...) esetében *nincs* ilyen ellenőrzés: atomi módon írnak, így
  az `osatura script p.json p.json x.rhai` helyben szerkeszt. Használj eltérő neveket, amíg meg nem bízol egy szkriptben.
* A GIF 256 színű és 1 bites átlátszóságú; sima alfához használd az `.apng` vagy `.webm` formátumot.
* Futtasd az `osatura export-fit` parancsot (lásd [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)), ha bájtkeretet kell eltalálnod.

### 9. lépés (opcionális): szerkesztés a stúdióban

```
osatura studio buddy-final.json --port 8080
```

Húzd a rétegeket, jelölj ki egy kulcsot a dope sheetben, nyomd meg a `Space` billentyűt a lejátszáshoz. A fájlodba
semmi sem íródik, amíg meg nem nyomod a **Save** gombot (`Ctrl+S`).

## 5. Hogyan illeszkednek az elemek

| Fogalom | A fájlban | Így hozod létre |
|---|---|---|
| Réteg (vektor, kép, csoport, kapcsoló...) | `layers` | `add_rect`, `add_circle`, `add_polygon`, `add_line`, `import-svg`, `autotrace`... |
| Csont | `bones` (a szülők a gyerekek előtt) | `add_bone`, `set_parent`, `reparent` |
| IK-lánc / kényszer / rugó | `ik`, `constraints`, `springs` | `add_ik`, `add_aim`, `add_copy_rotation`, `add_pin`; a rugókat JSON-ban |
| Csatorna (animált tulajdonság) | `channels` | `set_key`, `keys`, `keyedit`, kifejezések, zaj, bake-ek |
| Klip, take, állapotgép, kötés | `clips`, `machines`, `bindings` | `take-record`, `clip`, `machine`, `bind` |

Minden egyszerű JSON, ezért működnek a diffek (`osatura diff a.json b.json`), a verziókezelés
(`osatura vcs`) és a szkriptek. Az ismeretlen mezőket figyelmeztetésként jelzi, és figyelmen kívül hagyja:

```
$ osatura check unk.json
warning: unknown field bogus (ignored)
ok: 'X' 100×100 @ 24 fps, 10 frames, 0 bones, 0 ik, 0 layers, 0 channels
```

Egy projekt csak a **saját könyvtárán belüli** kép- és hangfájlokra hivatkozhat. Az abszolút útvonalakat és a `..`
használatát elutasítja, hacsak meg nem adod az `--allow-external-assets` kapcsolót (lásd [FAQ.md](FAQ.md)).

## 6. Merre tovább

* Riggelj valami hajlékonyat (skinek, súlyok, lattice, cage, ruha, fizika): [RIGGING_GUIDE.md](RIGGING_GUIDE.md).
* Járásciklusok, klipkötegek, lip-sync, mocap, állapotgépek, minőségbiztosítás és méretkeretek:
  [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).
* A böngészős szerkesztő, a gyorsbillentyűi és a HTTP API: [STUDIO.md](STUDIO.md).
* Kérdések és hibaüzenetek: [FAQ.md](FAQ.md). Ismert korlátok: [LIMITATIONS.md](LIMITATIONS.md).
* Minden parancs és kapcsoló: [CLI_REFERENCE.md](CLI_REFERENCE.md); a fájlformátum: [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

## Windows: az `osatura.exe` dupla kattintása

Az `osatura.exe` parancssori program. A 0.1.0-alpha.2 óta az Intézőből megnyitva (dupla kattintással) létrehozza a `Documents\OSATURA\first-rig.json` fájlt a sétáló személy demóból (`osatura demo-person`; meglévő fájlt soha nem ír felül), elindítja a stúdiót a `http://127.0.0.1:8080` címen, és megnyitja a böngésződben; a konzolablak a stúdió futása alatt nyitva marad, a bezárásával állítod le. Terminálból argumentumok nélkül megnyitva, mint korábban, kiírja a használati szöveget. A 0.1.0-alpha.1-ben a dupla kattintás csak kiírta a használati szöveget, és az ablak azonnal bezárult.

A futtatható fájlok **nincsenek kódaláírással ellátva**: a Windows SmartScreen „Ismeretlen közzétevőt” mutat (válaszd a *További információ*, majd a *Futtatás mindenképpen* lehetőséget), a macOS Gatekeeper pedig kéri, hogy hagyd jóvá a programot a *Rendszerbeállítások > Adatvédelem és biztonság* alatt. Előbb ellenőrizd a letöltést a `SHA256SUMS.txt` fájllal. Ha a 8080-as port foglalt, futtasd terminálból az `osatura studio first-rig.json --port 8090` parancsot.

## A stúdió ablakban: `osatura-studio` (Windows és macOS)

A Windows és macOS archívumok tartalmaznak egy második programot, az `osatura-studio` (`osatura-studio.exe`) fájlt, egy kis natív ablakot a böngészős stúdió körül: ugyanabból a mappából elindítja az `osatura` motort `studio` módban egy szabad helyi porton, megjeleníti az oldalt egy WebView2 (a Windows 11-ben beépített; a régebbi Windowshoz az Evergreen futtatókörnyezet kell) vagy WKWebView (macOS) ablakban, és az ablak bezárásakor leállítja a szervert. Konzolablak nem jelenik meg. Tartsd ugyanabban a mappában, mint az `osatura(.exe)` fájlt.

* **Exportok** (GIF, HTML-lejátszó, Lottie): natív „Mentés másként” párbeszédablak nyílik, amely a projekt mappájában indul, és egy üzenet megnevezi a fájlt, amikor elmentődött. (Az alpha.2 némán a Letöltések mappába mentette őket; az alpha.3 rákérdez.)
* Kattints rá duplán, hogy a `Documents/OSATURA/first-rig.json` fájlon dolgozz (a sétáló személy demóból, az `osatura demo-person` parancsból jön létre, ha hiányzik), vagy indítsd `osatura-studio my-project.json` formában (például a „Megnyitás ezzel” menüből).
* Az `osatura-studio --selftest` ablak nélkül elindítja a szervert, lekéri az oldalt, és leállítja; a kilépési kód 0, ha ez sikerült (a CI Windowson és macOS-en lefuttatja).
* **Korlátok.** Az ablakot valódi Windows 11 asztalon használták (alpha.2: a stúdió és az exportok működtek); az alpha.3 Mentés másként párbeszédablaka le van fordítva és a CI-ben önteszttel ellenőrizve, de még senki nem próbálta ki, és macOS-en senki nem használta az ablakot. A többi, rendszert igénylő böngészőszerű funkció (húzás az Intézőből vagy a Finderből, fájlok megnyitása a stúdió saját választóiból) a rendszer webnézetének funkciója, és nincs tesztelve. Linuxos ablak nincs (GTK / WebKitGTK rendszercsomagokat igényelne): használd az `osatura studio` parancsot és egy böngészőt. A program nincs kódaláírással ellátva (lásd fent).
