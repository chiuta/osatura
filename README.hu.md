<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# OSATURA

**Rustban írt 2D vektoros riggelő és animációs motor: csontok, IK, skinning, deformerek, állapotgép-futtatókörnyezet, parancssori eszköztár, böngészős stúdió és beágyazható könyvtárak egyetlen workspace-ben.** Román nyelvű változat: [README.ro.md](README.ro.md).

![Oldalnézetes figura (fej, fül, lófarok, törzs, ujjakkal ellátott kar, láb, talp) helyben jár az elúszó talajon; az osatura export valódi kimenete a beépített demóprojekten](docs/img/walk.gif)

*Mit készít: a beépített sétáló alak, az `osatura demo-person` és az `osatura export` parancsokkal exportálva (az egész projekt egyetlen JSON-fájl). Kétperces kipróbálás: [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md).*

## Összefoglalás és filozófia

Az OSATURA-val vektoros alakzatokból és képekből építhetsz karaktert vagy jelenetet, csontokkal és kényszerekkel riggelheted, kulcsokkal, klipekkel és fizikával animálhatod, majd GIF-ként, videóként, Lottie-ként, glTF-ként, önálló HTML-lejátszóként vagy interaktív futtatókörnyezetként exportálhatod. Kis, determinisztikus elemekből épül fel: ugyanaz a projekt és ugyanaz a seed ugyanazokat a pixeleket adja, minden eszköz egy dokumentált JSON-projektet olvas és ír, és minden parancs terminálból is működik, így az animációt úgy lehet szkriptelni, átnézni, diffelni és tesztelni, mint a forráskódot.

A projektet alakító alapelvek:

* **Független megvalósítás.** Nyilvános specifikációkból, tanulmányokból és tankönyvi matematikából íródott. A versenytársak funkciólistáit a *hatókör* meghatározásához vettük figyelembe; harmadik fél forráskódját nem másoltuk, és a projekt semmilyen zárt formátumot nem olvas, sem nem ír. Nem szigorú clean-room folyamat (lásd [CLEANROOM.md](docs/i18n/hu/CLEANROOM.md), a kutatási közzétételt is beleértve), és nem áll kapcsolatban a Moho-val vagy más kereskedelmi termékkel, nem származik azokból, és nem kompatibilis velük. A kód nagyrészt MI-modellel készült.
* **Őszinte állapotjelzés.** Minden funkciót azzal együtt írunk le, hogy mit *nem* tud ([docs/LIMITATIONS.md](docs/i18n/hu/LIMITATIONS.md), [ROADMAP.md](docs/i18n/hu/ROADMAP.md) 9. szakasz).
* **Alapértelmezetten biztonságos.** Bemenet hatására nincs pánik, a értékek végességét ellenőrzi, erőforrás-korlátok vannak, az írás atomi, az eszközútvonalak a projektmappába vannak zárva ([docs/SECURITY.md](docs/i18n/hu/SECURITY.md)).
* **Elsőbbség a nyílt formátumoknak.** OpenRaster, OCA, SVG, Lottie, glTF, APNG, WebM, EXR; tervezetten nincsenek zárt fájlformátumok.
* **Kis függőségi lábnyom**, a licencpolitikát tesztek és a `cargo deny` ellenőrzik.

## Állapot

*Név: a projektet 2026. okt. 24-ig ARMATURA-nak hívták, és egy védjegykeresés után nevezték át OSATURA-ra ([docs/LIMITATIONS.md](docs/i18n/hu/LIMITATIONS.md) L-78). A tárolót és az oldal címét is átneveztük (a régi tárolólinkek átirányítanak).*

> **Alfa. Az első buildek előzetes kiadás-vázlatként készültek: `v0.1.0-alpha.3`.** A Windows x86-64, macOS arm64 és Linux x86-64 futtatható fájlokat, valamint a WebAssembly modult a GitHub Actions építi, és egy *vázlat* előzetes kiadáshoz csatolja a <https://github.com/chiuta/osatura/releases> oldalon (a vázlatot csak a tulajdonos látja, amíg kézzel közzé nem teszi). Minden crate verziója továbbra is `0.1.0`; a „v0.5 / v0.6 / v0.7” a történetben *funkciószintet* jelöl, nem kiadási számot ([CHANGELOG.md](docs/i18n/hu/CHANGELOG.md)).
>
> | Ellenőrizve | Nem ellenőrizve |
> |---|---|
> | **Helyben (Linux x86-64, 2026. okt. 24.):** a `cargo test --workspace --exclude osatura-gpu --exclude osatura-gui` 1338 tesztje és 19 doc-tesztje sikeres (a GPU crate-hez adapter, az ablak crate-hez asztali környezet kell; korábban a GPU crate-et is beleszámítva 1368 volt). **A GitHub Actionsön (2026. okt. 24., eszközlánc 1.97.0):** a tesztkészlet és a `clippy -D warnings` sikeres Linuxon, macOS-en (arm64) és Windowson; a `cargo deny` és a `reuse lint` sikeres; a WebAssembly modul lefordul `wasm32-unknown-unknown` célra, és a wrapper tesztje a valódi modul ellen sikeres Node 20 alatt; a kiadási binárisok füstteszten átmennek (`--version`, `demo`, `check`, `render`), és az `osatura-studio --selftest` lefut Windowson és macOS-en. A stúdió négy headless Chromium füstteszje sikeres; a CLI-t, a Rust homlokzatot és a C ABI-t (ASan/UBSan alatt is) lefuttattuk a dokumentációban szereplő példákkal | **A futtatható fájlokat ember még nem használta valódi Windows vagy macOS asztalon**: a Windows dupla kattintásos indítóját és az `osatura-studio` ablakos alkalmazást (WebView2 / WKWebView) a CI-ben csak lefordították és önteszttel ellenőrizték; a binárisok **nincsenek kódaláírással ellátva** (SmartScreen „Ismeretlen közzétevő”, macOS Gatekeeper); a Linux binárist csak Ubuntu runneren építették; **a WebAssembly modult böngészőben még soha nem futtatták**, sebessége és memóriahasználata nincs megmérve; a stúdiót ember még nem használta éles környezetben, és Chromiumtól eltérő böngészőben sem; valódi GPU-val (csak szoftveres adapterrel), webkamerával, fényképpel vagy mozgásrögzítő fájllal még nem próbálták; nem volt biztonsági audit, jogi felülvizsgálat és szabadalmi keresés |
>
> Mielőtt bármire ráépítenél, olvasd el a [docs/LIMITATIONS.md](docs/i18n/hu/LIMITATIONS.md) fájlt (78 számozott korlát súlyossággal és kerülőmegoldással).

## Letöltés

Az előre elkészített archívumok – miután a tulajdonos közzétette a vázlat kiadást – a projekt [Releases oldalán](https://github.com/chiuta/osatura/releases) találhatók:

| Archívum | Tartalom |
|---|---|
| `osatura-windows-x86_64.zip` | `osatura.exe` (parancssor + stúdiószerver), `osatura-studio.exe` (ablakos alkalmazás), C könyvtár és fejléc, licencek, harmadik féltől származó közlemények |
| `osatura-macos-arm64.tar.gz` | `osatura`, `osatura-studio` (csak Apple silicon), C könyvtár és fejléc, licencek, közlemények |
| `osatura-linux-x86_64.tar.gz` | `osatura`, C könyvtár és fejléc, licencek, közlemények (nincs ablakos alkalmazás: használd az `osatura studio` parancsot és egy böngészőt) |
| `osatura-web.tar.gz` | `osatura_wasm.wasm`, `osatura.js`, demó `index.html` (böngészőkben nem ellenőrizve, lásd [docs/WASM.md](docs/i18n/hu/WASM.md)) |
| `SHA256SUMS.txt` | minden letöltést ellenőrizz ezzel |

* **Windows:** kattints duplán az `osatura.exe` fájlra, hogy létrejöjjön a `Documents\OSATURA\first-rig.json`, és megnyíljon a stúdió a böngésződben, vagy kattints duplán az `osatura-studio.exe` fájlra egy saját ablakért. Mindkét program aláíratlan: a SmartScreenben válaszd a *További információ*, majd a *Futtatás mindenképpen* lehetőséget. Lásd [docs/GETTING_STARTED.md](docs/i18n/hu/GETTING_STARTED.md).
* **macOS:** első alkalommal hagyd jóvá a programot a *Rendszerbeállítások > Adatvédelem és biztonság* alatt.
* **Forrás:** minden archívum megnevezi azt a commitot, amelyből készült; az AGPL alá tartozó részekhez a forrásajánlat tartozik (`osatura --version`, `/source`).

## Funkciók területenként

A részletek és a pontos opciók: [docs/CLI_REFERENCE.md](docs/i18n/hu/CLI_REFERENCE.md); ami részleges vagy hiányzik: [docs/LIMITATIONS.md](docs/i18n/hu/LIMITATIONS.md).

* **Rajzmodell.** Vektoros útvonalak, kitöltések, körvonalak (szaggatott, változó vastagságú), lineáris és radiális színátmenetek, maszkok, keverési módok, rétegenkénti effektek (elmosás, árnyék, színezés, kitöltés), képrétegek mesh warppal, folyékony metaballok, sokszög-boole-műveletek, determinisztikus részecskék, többsíkú kamera mentett beállításokkal, mélységélesség.
* **Riggelés.** Csontok előre kinematikával; kétcsontos, soft, stretch és FABRIK IK; korlátok, kényszerek, rugók; skinning súlyeszközökkel; lattice, spine, morph, wobble és cage deformerek; mesh-követés alfából; warp pivotok és glue; driverek (csúszkák, joystickok, nézetváltás); skin-készletek; automatikus riggelés réteg nevekből; szem-vágás; elrendezési konténerek és görgetési régiók.
* **Animáció.** Kulcsok Bézier/lépcsős/rugó/pattogó átmenettel, kifejezések és vezérelt csatornák, kapcsolórétegek, take-ek és rétegzett klipköteg, ciklus- és helyben történő átalakítás, lábrögzítés, mágnesek, útvonalkövetés, idle és overlap generátorok, lánc-/haj-/ruhafizika, lip-sync hangból vagy átiratból, beszédérzékeny fejmozgás, BVH-retargeting, élő bemenet (MIDI, egér, gamepad, általad megadott webkamerás landmarkok), hangsávok vágással, ki- és beúsztatással, keveréssel.
* **Futtatókörnyezet.** Állapotgépek 1D blend tree-kkel, adatkötés, görgetési fizika, részletességi szint hibahatárral, Rhai szkript-sandbox, patch-műveletek fölötti ágens (MCP) szerver.
* **Kimenet.** SVG, PNG, PNG-sorozatok és sprite sheetek, GIF, APNG, MP4/WebM (ffmpeg-gel), önálló HTML-lejátszó, Lottie és dotLottie, glTF 2.0 bináris, OpenEXR/16 bites PNG, OCA; méretre célzott export (`export-fit`). Import: SVG, OpenRaster, Lottie, BVH, PNG/GIF/APNG-sorozatok.
* **Minőségbiztosítási eszközök.** `check`, `lint`, szemantikus `diff`, póz-fuzzing hőtérképekkel, animációs CI észlelési diff-fel és referenciakockákkal ([docs/CI.md](docs/i18n/hu/CI.md)), verziókezelés szemantikus merge-dzsel (`vcs`), villogás- és mozgásbiztonsági ellenőrzések.
* **Stúdió.** A CLI által kiszolgált böngészős szerkesztő: vászon-gizmók, idővonal, dope sheet és görbeszerkesztő, overlayek, parancspaletta és gyorsbillentyűk, visszavonás/újra, valós idejű többfelhasználós szerkesztés opcionális tokennel és tartós naplóval ([docs/STUDIO.md](docs/i18n/hu/STUDIO.md)).
* **Könyvtárak.** Stabil Rust homlokzat ([docs/API.md](docs/i18n/hu/API.md)), C ABI ([docs/C_ABI.md](docs/i18n/hu/C_ABI.md)), WebAssembly modul és JavaScript wrapper (a CI-ben lefordítva és Node alatt tesztelve, böngészőben még nem futtatva; [docs/WASM.md](docs/i18n/hu/WASM.md)), opcionális wgpu renderer.

## Gyors kezdés

Követelmények: egy friss stabil Rust eszközlánc (vagy egy előre elkészített archívum a [Letöltés](#letöltés) szakaszból: hagyd ki a `cargo build` sort, és használd a kicsomagolt `osatura` fájlt). Az `ffmpeg` csak az MP4/WebM exporthoz és az mp3/ogg hanghoz kell; soha nem csomagoljuk vele. Az alábbi összes parancsot lefuttattuk ehhez a README-hez (debug build, Linux). A kiadási buildek (`cargo build --release`) renderelésnél sokkal gyorsabbak, és ugyanígy használhatók a `target/release` mappával.

```sh
cargo build -p osatura-cli                       # builds target/debug/osatura
export PATH="$PWD/target/debug:$PATH"             # so that `osatura` is found

osatura --version                                # version, licence, source offer, support links
osatura demo demo.json                           # write the built-in rig demo (640x480, 48 frames)
osatura check demo.json                          # validate: ok: 'OSATURA demo' 640x480 @ 24 fps, 48 frames, 6 bones...
osatura lint demo.json                           # static rig/animation checks: 0 finding(s), 0 error(s)
osatura render demo.json 12 frame12.png --bones  # one frame, with the skeleton overlay
osatura render demo.json 12 frame12.svg          # the same frame as vector output
osatura frames demo.json frames_out --png        # every frame as a PNG
osatura export demo.json demo.gif                # also .html (player), .json (Lottie), .lottie, .mp4 (ffmpeg)
osatura demo-fx fx.json                          # second demo: liquid, booleans, particles
osatura lipsync demo.json voice.wav mouth out.json   # mouth shapes from a WAV file (switch layer "mouth")
osatura studio demo.json --port 8080             # browser editor at http://127.0.0.1:8080
osatura ci init demo.json && osatura ci run     # animation CI: goldens + checks (about 40 s in a debug build)
```

A `voice.wav` bármilyen, általad megadott mono vagy sztereó WAV-fájl. A stúdió csak a `127.0.0.1` címen figyel, és más gazdanevekre érkező kéréseket elutasít; ha meg akarod osztani, előbb olvasd el a [docs/SECURITY.md](docs/i18n/hu/SECURITY.md) fájlt.

A könyvtárak használata:

```sh
cargo build -p osatura-capi                      # target/debug/libosatura_capi.so and .a
gcc -std=c99 -o smoke osatura-capi/tests/smoke.c -Ltarget/debug -losatura_capi
LD_LIBRARY_PATH=target/debug ./smoke demo.json    # abi 1, 48 frames, 320x240, rc=0, checksum=...
```

A Rust homlokzat az `osatura_core::prelude` ([docs/API.md](docs/i18n/hu/API.md)); további C- és Python-példák a [docs/C_ABI.md](docs/i18n/hu/C_ABI.md) fájlban vannak.

Tesztek (nagyjából 14 perc debug buildben egy megosztott gépen; lásd [docs/DEVELOPMENT.md](docs/i18n/hu/DEVELOPMENT.md)):

```sh
cargo test --workspace
cargo test --doc -p osatura-core
cargo test -p osatura-cli --test spdx_audit
```

## Tárolóstruktúra

| Útvonal | Tartalom | Licenc |
|---|---|---|
| `osatura-core/` | motor, formátumok, eszközök, a stabil `api` homlokzat | MPL-2.0 |
| `osatura-cli/` | az `osatura` bináris, a stúdió- és együttműködési szerver, végpontok közötti és füsttesztek | AGPL-3.0-or-later |
| `osatura-gui/` | az `osatura-studio`, a stúdió natív ablakban (Windows és macOS; Linuxon csonk) | AGPL-3.0-or-later |
| `osatura-capi/` | C ABI (`osatura.h`, megosztott és statikus könyvtár) | MPL-2.0 |
| `osatura-wasm/` | WebAssembly ABI, JavaScript wrapper, demóoldal (a CI-ben lefordítva) | MPL-2.0 |
| `osatura-gpu/` | opcionális wgpu renderer (nem alapértelmezett tag) | MPL-2.0 |
| `docs/` | dokumentáció (CC-BY-4.0) és generált harmadik féltől származó fájlok | CC-BY-4.0 |
| `.github/workflows/` | CI (`ci.yml`) és kiadási buildek (`release.yml`) | MPL-2.0 |
| `scripts/` | `e2e.sh`, `licence-audit.sh` | MPL-2.0 |
| `LICENSES/`, `REUSE.toml`, `deny.toml` | licencszövegek, licenc-metaadatok, függőségi szabályzat | - |
| `ROADMAP.md`, `FEATURES.md`, `CLEANROOM.md`, `LICENSING.md`, `TRADEMARK.md`, `CONTRIBUTING.md`, `CHANGELOG.md` | projektdokumentumok | CC-BY-4.0 |

## Dokumentációs térkép

Kezdd a [docs/INDEX.md](docs/i18n/hu/INDEX.md) fájlnál (minden dokumentum, célközönség, olvasási útvonalak).

| Oldal | Kinek / mire |
|---|---|
| [docs/GETTING_STARTED.md](docs/i18n/hu/GETTING_STARTED.md) | első projekt, renderelés, export |
| [docs/CLI_REFERENCE.md](docs/i18n/hu/CLI_REFERENCE.md) | minden parancs és opció |
| [docs/STUDIO.md](docs/i18n/hu/STUDIO.md) | a böngészős szerkesztő |
| [docs/RIGGING_GUIDE.md](docs/i18n/hu/RIGGING_GUIDE.md) | riggek építése |
| [docs/ANIMATION_GUIDE.md](docs/i18n/hu/ANIMATION_GUIDE.md) | animálás, klipek, fizika, hang |
| [docs/PROJECT_FORMAT.md](docs/i18n/hu/PROJECT_FORMAT.md) | a JSON projektformátum |
| [docs/EXPORT_FORMATS.md](docs/i18n/hu/EXPORT_FORMATS.md) | mit visz és mit hagy el az egyes formátum |
| [docs/ARCHITECTURE.md](docs/i18n/hu/ARCHITECTURE.md) | felépítés és modulok |
| [docs/GLOSSARY.md](docs/i18n/hu/GLOSSARY.md) | fogalmak |
| [docs/FAQ.md](docs/i18n/hu/FAQ.md) | gyakori kérdések |
| [docs/API.md](docs/i18n/hu/API.md) | Rust homlokzat 1.4 |
| [docs/C_ABI.md](docs/i18n/hu/C_ABI.md) | C interfész |
| [docs/WASM.md](docs/i18n/hu/WASM.md) | WebAssembly (a CI-ben lefordítva, böngészőben nem futtatva) |
| [docs/CI.md](docs/i18n/hu/CI.md) | animációs CI |
| [docs/DEVELOPMENT.md](docs/i18n/hu/DEVELOPMENT.md) | build, teszt, konvenciók, kiadás |
| [docs/SECURITY.md](docs/i18n/hu/SECURITY.md) | fenyegetésmodell és megerősítés |
| [docs/LIMITATIONS.md](docs/i18n/hu/LIMITATIONS.md) | ismert korlátok |
| [docs/PATENT_NOTES.md](docs/i18n/hu/PATENT_NOTES.md), [docs/THIRD_PARTY.md](docs/i18n/hu/THIRD_PARTY.md), [docs/THIRD_PARTY_NOTICES.md](docs/THIRD_PARTY_NOTICES.md) | szabadalmak, függőségek licencei |
| [CHANGELOG.md](docs/i18n/hu/CHANGELOG.md), [ROADMAP.md](docs/i18n/hu/ROADMAP.md), [FEATURES.md](docs/i18n/hu/FEATURES.md) | előzmények, terv és részletes állapot, funkciómátrix |
| [CLEANROOM.md](docs/i18n/hu/CLEANROOM.md), [LICENSING.md](docs/i18n/hu/LICENSING.md), [TRADEMARK.md](docs/i18n/hu/TRADEMARK.md), [CONTRIBUTING.md](docs/i18n/hu/CONTRIBUTING.md) | eredet, licencek, névhasználati szabályzat, közreműködési szabályok |

## Licencösszefoglaló

* **MPL-2.0** az `osatura-core`, `osatura-capi`, `osatura-wasm`, `osatura-gpu` könyvtárakra és a `scripts/` mappára: beleköthetők a saját programodba, nyílt vagy zárt forrásúba egyaránt; az ezekben a fájlokban végzett módosításokat MPL alatt kell közzétenni.
* **AGPL-3.0-or-later** az `osatura-cli`-re (a parancssor és a stúdió) és az `osatura-gui`-ra (az azt körülvevő ablak). Ha módosítod, és másoknak hálózaton keresztül hozzáférést adsz hozzá, a 13. szakasz előírja, hogy fel kell kínálnod nekik a forráskódodat: az `osatura studio --source-url URL` közzéteszi, hol található; a `/source` és az `osatura --version` tartalmazza az ajánlatot.
* **CC-BY-4.0** a dokumentációra és a szintetikus eszközökre (a mellékelt szintetikus arcfelismerő modellt is beleértve).
* **A projektjeid és a renderek a tieid maradnak.**
* A név és a logó nem tartozik ezen licencek hatálya alá ([TRADEMARK.md](docs/i18n/hu/TRADEMARK.md)). A szabadalmi helyzetet nem értékeltük ([docs/PATENT_NOTES.md](docs/i18n/hu/PATENT_NOTES.md)). A beküldött kód kereskedelmi vagy kettős licencelése közreműködői licencszerződés nélkül nem lehetséges, és ilyen nincs.
* Harmadik féltől származó crate-ek: 363 a 2026. okt. 24-i leltárban (minden platform és target együtt, ami az ablakos alkalmazás csak Windowson / macOS-en / Linuxon használt crate-jeit is tartalmazza; egy adott bináris ennek egy részhalmazát tartalmazza), mind megengedő licencű, kivéve hat gyenge copyleft MPL-2.0 crate-et: a `smartstring` az opcionális `scripting` funkció mögött, valamint öt (`cssparser`, `cssparser-macros`, `dtoa-short`, `selectors`, `option-ext`), amelyeket csak az `osatura-gui` ablakos alkalmazás húzhat be ([docs/THIRD_PARTY.md](docs/i18n/hu/THIRD_PARTY.md)). Az ffmpeg-et soha nem csomagoljuk.

Lásd: [LICENSE](LICENSE), [LICENSES/](LICENSES/) és [LICENSING.md](docs/i18n/hu/LICENSING.md). Mindez nem jogi tanács; a jogi felülvizsgálat még hátravan.

## Közreműködés

A közreműködést szívesen fogadjuk, annak a résznek a licence alatt, amelyet módosít. Olvasd el a [CONTRIBUTING.md](docs/i18n/hu/CONTRIBUTING.md) fájlt (minden commithoz kell egy `Signed-off-by` sor, DCO 1.1), a [CLEANROOM.md](docs/i18n/hu/CLEANROOM.md) fájlt (csak nyilvános forrásokból implementálj; ne másolj zárt vagy összeférhetetlen licencű kódból) és a [docs/DEVELOPMENT.md](docs/i18n/hu/DEVELOPMENT.md) fájlt (konvenciók, új modul ellenőrzőlistája, tesztek). Jó első hozzájárulások: egy javítás a [docs/LIMITATIONS.md](docs/i18n/hu/LIMITATIONS.md) listájából, a WebAssembly-ellenőrzőlista tényleges lefuttatása, a Windows / macOS futtatható fájlok tesztelése valódi asztali környezetben, valódi GPU-n végzett benchmark.

Biztonsági problémák: ne nyiss nyilvános issue-t; kövesd a [docs/SECURITY.md](docs/i18n/hu/SECURITY.md) 10. szakaszát.

## Támogatás

Az OSATURA-t Alexandru-Ionuț Chiuță (Alexio) fejleszti. Ha hasznosnak találod, támogathatod a munkát:

* Buy Me a Coffee: <https://buymeacoffee.com/echo.of.the.strings>
* Patreon: <https://patreon.com/c/alexio_tf>

A karbantartó elérhetősége (az `osatura --version` is kiírja): alexio@trom.tf.

## Köszönetnyilvánítás

* A nyilvános tanulmányok és tankönyvek szerzői, amelyeket az algoritmusok követnek, modulonként megnevezve a [CLEANROOM.md](docs/i18n/hu/CLEANROOM.md) fájlban (Bézier- és spline-matematika, inverz kinematika, lineáris keverésű skinning, mean-value koordináták, görbeillesztés és mások).
* A nyílt specifikációk, amelyeket az OSATURA olvas és ír: SVG, OpenRaster, OCA, Lottie/dotLottie, glTF, APNG, WebM, OpenEXR, BVH, PNG, GIF.
* A Rust crate-ek, amelyekre épít, licenceikkel együtt felsorolva a [docs/THIRD_PARTY.md](docs/i18n/hu/THIRD_PARTY.md) fájlban, különösen a `tiny-skia`, `serde`, `png`, `zip`, `rhai` és `wgpu`.
* Mindenki, aki a fenti hivatkozásokon keresztül támogatja a projektet.
