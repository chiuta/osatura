<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Gyakran ismételt kérdések

*Rövid válaszok, mindegyik olyasmiből származik, amit lefuttattak, vagy amit a forrásfában és a többi dokumentumban elolvastak. A dokumentáció licence CC-BY-4.0. Ahol egy válasz valamilyen korláttól függ, jelzi ezt, és a [LIMITATIONS.md](LIMITATIONS.md) fájlra mutat. Az utolsó szakasz hibaüzenetei valódi buildből származnak; szó szerint idézzük őket, így az oldalon rákereshetsz a látott szövegre.*

Tartalom: [Telepítés és beállítás](#telepítés-és-beállítás) - [ffmpeg](#ffmpeg) - [Teljesítmény](#teljesítmény) - [Determinizmus](#determinizmus) - [Licencelés és a kimenet](#licencelés-és-a-kimenet) - [Fájlbiztonság és eszközútvonalak](#fájlbiztonság-és-eszközútvonalak) - [Miért kell a kimenetnek eltérnie a bemenettől](#miért-kell-a-kimenetnek-eltérnie-a-bemenettől) - [Riggelés és animáció](#riggelés-és-animáció) - [Stúdió](#stúdió) - [Arc, beszéd és mozgásrögzítés](#arc-beszéd-és-mozgásrögzítés) - [Hibák, biztonság és közreműködés](#hibák-biztonság-és-közreműködés) - [Korlátozások](#korlátozások) - [Összehasonlítás más eszközökkel](#összehasonlítás-más-eszközökkel) - [Hibaelhárítási táblázat](#hibaelhárítási-táblázat)

---

## Telepítés és beállítás

**1. Hogyan telepítsem az OSATURA-t?**
Előre elkészített archívumok (Windows x86-64, macOS arm64, Linux x86-64 és a WebAssembly modul) készülnek a `v0.1.0-alpha.3` előzetes kiadás-vázlatként a <https://github.com/chiuta/osatura/releases> oldalon; akkor tölthetők le, amikor a tulajdonos közzéteszi a vázlatot, és nincsenek kódaláírással ellátva (a SmartScreen „Ismeretlen közzétevőt” mutat; a macOS jóváhagyást kér). Nincs telepítő: csomagold ki az archívumot, és futtasd az `osatura` programot. Vagy fordítsd le forrásból egy friss stabil Rust eszközlánccal: a `cargo build -p osatura-cli` a `target/debug/osatura` fájlt adja; a `--release` hozzáadásával a `target/release/osatura` készül el. Lásd [DEVELOPMENT.md](DEVELOPMENT.md).

**2. Mely platformokon működik?**
Linux x86-64-en fejlesztették és próbálták ki a projektet. A GitHub Actionsön a tesztkészlet és a clippy macOS-en (arm64) és Windowson (x86-64) is sikeres, és ott épülnek a kiadási binárisok mindhárom rendszerre; de a Windows vagy macOS futtatható fájlokat ember még nem használta valódi asztali környezetben, ezért ezek „CI által tesztelt”, nem „támogatott” státuszúak ([LIMITATIONS.md](LIMITATIONS.md), 1. szakasz).

**3. Van grafikus telepítő vagy alkalmazás?**
Telepítő nincs. A termék a parancssor. A stúdió egy helyi weboldal, amelyet az `osatura studio` szolgál ki; Windowson az `osatura.exe` dupla kattintása elindítja, és megnyitja a böngésződet. A Windows és macOS archívumok tartalmazzák az `osatura-studio` programot is, egy kis natív ablakot ugyanezen stúdió körül (csak lefordították és önteszttel ellenőrizték a CI-ben); lásd [STUDIO.md](STUDIO.md) és [GETTING_STARTED.md](GETTING_STARTED.md).

**4. Hogyan kezdhetek gyorsan?**
Kövesd a [GETTING_STARTED.md](GETTING_STARTED.md) fájlt: egy üres projektből épít fel egy animált karaktert, minden lépést lefuttatva, és egy GIF-fel zárul.

**5. Fordíthatom a szkriptmotor nélkül?**
Igen: a `cargo build -p osatura-cli --no-default-features` kihagyja a Rhai-t, így a `script` parancs hiányzik (lásd [DEVELOPMENT.md](DEVELOPMENT.md)). A `scripting` funkció az egyetlen hely, ahol gyenge copyleft licencű crate (smartstring) bekerül; lásd [THIRD_PARTY.md](THIRD_PARTY.md).

**6. Kell hozzá hálózati kapcsolat vagy fiók?**
Nem. A program működéséhez nem kell sem fiók, sem hálózat (a stúdiónak nincsenek fiókjai). Alapértelmezés szerint a stúdió csak a saját gépeden figyel (lásd az 51. kérdést).

**7. Hol tartja a stúdió a beállításait?**
Az `$OSATURA_UI_CONFIG` fájlban, ha be van állítva, különben az `$XDG_CONFIG_HOME/osatura/ui.json` fájlban, különben Windowson a `%APPDATA%\OSATURA\ui.json` fájlban, különben a `~/.config/osatura/ui.json` fájlban. A projektjeidhez soha nem nyúl.

---

## ffmpeg

**8. Kell az ffmpeg?**
Csak az MP4- és WebM-exporthoz, a `sound mix` parancshoz, az mp3/ogg hanghoz és a `face-track` videófájlon való használatához. A GIF-, APNG-, HTML-, PNG-sorozat-, SVG-, glTF- és JSON-exporthoz nincs szükség másra. A teljes lista a [CLI_REFERENCE.md](CLI_REFERENCE.md) 1.6. szakaszában van.

**9. Az ffmpeg a csomag része?**
Soha. Az OSATURA az `ffmpeg`-et a `PATH`-ról, különálló programként indítja, és semmi mást nem hajt végre. Az ffmpeg-builded licencét és szabadalmi helyzetét neked kell ellenőrizned; lásd [PATENT_NOTES.md](PATENT_NOTES.md).

**10. Mit ír ki, ha hiányzik az ffmpeg?**
`cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead` (WebM esetén a tipp vége `or export an APNG instead`). Semmi sem íródik ki.

**11. Milyen enkóderekre van szüksége az ffmpeg-emnek?**
`libx264` az mp4-hez, `libvpx-vp9` vagy `libaom-av1` a webm-hez, `aac` és `libopus` az ezekben lévő hanghoz, `libvorbis` és `libmp3lame` a `sound mix` parancshoz.

---

## Teljesítmény

**12. Miért ilyen lassú?**
Valószínűleg debug buildet futtatsz. A dokumentumokban szereplő számok debug buildből származnak, és 10-30-szor lassabbak a release buildnél ([CLI_REFERENCE.md](CLI_REFERENCE.md), 1.7. szakasz). A 2 magos tesztgépen a debug build 46 másodperc alatt írta ki PNG-ként a demóprojekt 48 képkockáját, és 57 másodperc alatt exportálta GIF-be. Egy friss release build ugyanezt a két feladatot 1,5 és 2,2 másodperc alatt végezte el. Fordíts a `cargo build --release` paranccsal, és használd a `target/release/osatura` fájlt.

**13. Használja a GPU-t?**
Az alapértelmezett renderer a CPU. Létezik egy `gpu` crate és egy `render-gpu` parancs funkciókapcsoló mögött, de a fordítás során nem állt rendelkezésre valódi GPU: szoftveres GPU-n (llvmpipe) egy 640x480-as képkocka 253-295 ms-ig tartott, a CPU-s útvonalon 13,8 ms-ig. A GPU-s renderelést hardveren tesztelés nélkülinek tekintsd ([LIMITATIONS.md](LIMITATIONS.md)).

**14. Mennyibe kerül egy rig képkockánként?**
Debug buildben a `biped` példa mediánja 64,5 ms volt képkockánként a `ci` teljesítményellenőrzésben. Egy kis zászló ruha-bake-je 0,43 másodpercig tartott. Mérd meg a saját projektedet az `osatura stats` és az `osatura ci` paranccsal; a riggek nagyon különböznek.

**15. Hogyan tehetek könnyebbé egy nehéz projektet?**
Használd a `lod` parancsot olcsóbb változatok készítéséhez, az `export-fit` parancsot a bájtkeretet teljesítő exportbeállítások megtalálásához, és a `proxy-bench` parancsot a proxyk összehasonlításához. Lásd [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) és [EXPORT_FORMATS.md](EXPORT_FORMATS.md). Az `export-fit` a `--max-seconds` kapcsolóval korán megállhat, és ekkor jelzi, hogy kifutott az időből.

**16. Egy többezer hívásos szkript hibával leáll. Miért?**
Az `osatura script` futásonként legfeljebb 5000 kérést fogad el, és mindent vagy semmit elven működik: ha bármelyik kérést elutasítják, semmi sem íródik ki. Oszd fel a nagy feladatot több szkriptre, amelyek az előző kimenetet olvassák.

---

## Determinizmus

**17. Reprodukálható a kimenet?**
A renderelést erre tervezték: ugyanaz a projekt és ugyanaz a seed ugyanazokat a pixeleket adja, a fizika, a részecskék és a zaj seedeltek. Ez teszi lehetővé, hogy a `golden` és a `ci` renderek között hasonlítson. Csak egyféle gépen ellenőrizték (Linux x86-64); más CPU-kon vagy fordítókon a bitre azonos kimenetet nem ígérjük.

**18. Miért ad más eredményt egy fizikai bake, ha egyetlen számot megváltoztatok?**
A bake adott bemenetre determinisztikus, de szimuláció: a tömeg, a képkockasebesség, az ütköző vagy a seed bármilyen változása az egész futást megváltoztatja. Az összehasonlításhoz ugyanazokkal a beállításokkal és seeddel bake-eld. A bake alapértelmezés szerint összefésülődik a meglévő animációval (`merge=keep`: a meglévő csatornák megmaradnak, és a parancs ezt írja: `kept existing animation on 'b2' (use merge=replace to overwrite)`); felülíráshoz használd a `merge=replace` értéket.

**19. Hurkolódik a zaj?**
Nem. A `noise` csatorna magától nem ismétlődik, ezért egy hurok az illesztésnél ugrást fog mutatni. Használj helyette kulcsokat vagy olyan kifejezést, mint a `8 * sin(t * 2 * pi)`; a kifejezésekben a `t` a klip hossza alatt 0-tól 1-ig fut, nem képkockákban ([ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)).

**20. Ellenőrizhetem, hogy egy animáció tisztán hurkolódik?**
Futtasd a `lint` parancsot. A `make-loop` teljesen eltávolítja az elmozdulást: egy tesztben egy guruló labda állóvá vált, és a `lint` figyelmeztetett (L005). A `loop-solve` ciklikus mozgásnál lezárja az illesztést, de egy haladó objektumnál látható ugrást hagyott (280-ról 280-ra az elmozdulás csatornáján). Folytatódó elmozduláshoz használd az extrapolációs `loop_offset` módot.

---

## Licencelés és a kimenet

*Ez a [LICENSING.md](LICENSING.md) közérthető összefoglalója, nem jogi tanács. A licencelési megoldás jogi felülvizsgálata még hátravan.*

**21. Milyen licencek vonatkoznak rá?**
MPL-2.0 a magkönyvtárra, a C ABI-ra, a WASM- és GPU-crate-ekre; AGPL-3.0-or-later a parancssori programra és a stúdióra (`osatura-cli`); CC-BY-4.0 a dokumentációra, valamint a szintetikus példaeszközökre és modellekre.

**22. Kié az általam készített animáció és render?**
Tiéd. A projekt álláspontja szerint az OSATURA-val létrehozott projektek, animációk és renderek teljes egészében a tieid; a licencek a szoftverre vonatkoznak, nem a kimenetére. Egy GIF exportálása vagy egy parancs futtatása nem keletkeztet kötelezettséget.

**23. Használhatom a kimenetet kereskedelmi célra?**
Igen, a fenti álláspont értelmében. A kivétel mindaz, amit a példaeszközökből másolsz: azok CC-BY-4.0 licencűek, és forrásmegjelölést igényelnek.

**24. Közzé kell tennem a projektjeimet, ha az AGPL-es részeket használom?**
Nem. Az AGPL a programra vonatkozik. Ha egy módosított `osatura-cli`-t vagy stúdiót úgy futtatsz, hogy mások hálózaton keresztül használják, fel kell kínálnod nekik a forráskódját. A stúdió ezt egy `/source` oldallal és egy `--source-url` kapcsolóval teszi, amely a te másolatodra mutat.

**25. Köthetem a magkönyvtárat zárt forráskódú alkalmazásba?**
A mag MPL-2.0, ami fájlszintű copyleft: használhatod zárt kódból, és az MPL hatálya alá eső fájlok módosításait meg kell osztanod. A CLI és a stúdió AGPL, ami más kérdés.

**26. Mondhatom, hogy az eszközöm „együttműködik az OSATURA-val”?**
Igen: a „compatible with OSATURA” és a „made with OSATURA” megengedett. Módosított buildet nem szabad ugyanezen a néven terjeszteni ([TRADEMARK.md](TRADEMARK.md)).

**27. Kettős licenc alá helyezhetem a hozzájárulásomat?**
Nincs közreműködői licencszerződés. A hozzájárulásokat a projekt licence alatt fogadjuk el DCO-aláírással (`git commit -s`), így nincs olyan tulajdonos, aki újralicencelhetné őket. A részletek a [CONTRIBUTING.md](CONTRIBUTING.md) fájlban.

**28. Mi a helyzet a harmadik féltől származó kóddal?**
Nagyjából 360 harmadik féltől származó crate (az összes platformon együtt számolva; az ablakos alkalmazás hozzáadja a Windows / macOS eszközkészletet), mind megengedő licencű, kivéve hat MPL-2.0 licencűt: a smartstringet (MPL-2.0 vagy újabb, csak a `scripting` funkcióval) és ötöt, amelyeket csak az ablakos alkalmazás húzhat be. A listák a [THIRD_PARTY.md](THIRD_PARTY.md) és a [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md) fájlban vannak.

**29. Vannak szabadalmi aggályok?**
Lásd [PATENT_NOTES.md](PATENT_NOTES.md). Ezek mérnöki jegyzetek, nem jogi vélemény.

---

## Fájlbiztonság és eszközútvonalak

**30. Megrongálhatja egy parancs a projektemet?**
Az írások atomiak: az eredmény ugyanabban a könyvtárban egy ideiglenes fájlba kerül, szinkronizálódik, majd átnevezéssel a célfájl fölé kerül, így egy összeomlás a régi fájlt hagyja meg. A szerkesztő parancsok újra ellenőrzik a kiírni készülő projektet, és megtagadják érvénytelen kiírását (`output would be invalid: ...`). A `script` parancs semmit sem alkalmaz, ha bármelyik kérés meghiúsul.

**31. Biztonságos megnyitni valaki más projektjét?**
Észszerűen, de nem feltétel nélkül. A projekt adat, nem kód; az egyetlen külső program, amelyet az OSATURA elindít, az ffmpeg, a 8. kérdésben felsorolt formátumokhoz. A betöltő elutasítja az eszközöket, a csöveket és a könyvtárakat, korlátozza a bemenet méretét, és elutasítja az abszolút vagy `..` tartalmú eszközútvonalakat. A Rhai szkriptek sandboxban futnak. Ennek ellenére olvasd el a [SECURITY.md](SECURITY.md) fájlt, mielőtt nem megbízható fájlokat dolgozol fel egy szolgáltatásban.

**32. Hol keresi a projekt a hangokat és a képeket?**
A projektfájl mappájához képest. Az abszolút útvonalakat és minden `..` tartalmú útvonalat elutasítja, hacsak meg nem adod az `--allow-external-assets` kapcsolót, és ezt minden későbbi, a projektet betöltő parancsnál újra meg kell adnod. Példa a referenciából:
`sound add: asset '/etc/passwd': absolute paths are not allowed (a project may only reference files inside its own directory; use --allow-external-assets to override) (nothing was added)`.

**33. Mekkora projektet olvas be?**
Alapértelmezés szerint legfeljebb 256 MiB-ot. A `--max-input-bytes N` kapcsolóval módosíthatod. A nagyobb fájlok ezzel az üzenettel hibáznak: `cannot read demo.json: 51450 bytes exceed the input limit of 100 bytes (raise it with --max-input-bytes N)` (a számok a példáéi).

**34. Miért nem hajlandó szimbolikus linkbe írni?**
Hogy a kimeneted mellé tett link ne irányíthasson máshová egy írást. Az üzenet: `refusing to write through a symbolic link; pass --follow-symlinks to write to the link's target, or name the target file yourself`.

**35. Miért nem tudok `<(...)` vagy csővezeték felől olvasni?**
A bemenetnek szabályos fájlnak kell lennie: `cannot read /dev/fd/63: not a regular file (devices, pipes and directories are refused)`. Írd az adatot előbb fájlba.

**36. Elrontja a projektet egy ismeretlen mező?**
Nem: a `warning: unknown field bogus (ignored)` üzenet kiíródik, a kilépési állapot 0. Az egy parancshoz írt segédfájlok (szerződések, leképezések, gépek) szigorúbbak: egy ismeretlen kulcs ott hiba, és semmi sem érvényesül.

---

## Miért kell a kimenetnek eltérnie a bemenettől

**37. Miért mondja az `render in.png in.png` azt, hogy „refusing to overwrite”?**
Azoknál a parancsoknál, amelyek projektet olvasnak, és képet, videót vagy más, nem projekt fájlt írnak, a kimenet neve nem egyezhet a bemenetével. Az üzenet: `refusing to overwrite the input project X with output of the same name`. Ez megakadályozza azt a klasszikus botlást, hogy a projekt egyetlen példánya fölé exportálsz.

**38. Akkor miért írhat a `script` és az `autorig` a saját bemenete fölé?**
A védelem a nem projekt kimenetekre vonatkozik. Azok a parancsok, amelyek projektet olvasnak és projektet írnak (`script`, `autorig`, `loop-solve` és mások), megnevezhetik ugyanazt a fájlt bemenetként és kimenetként; az írás atomi, és ha a parancs meghiúsul, semmi sem változik. Ha meg akarod tartani az eredetit, adj új nevet, vagy használd az `osatura vcs` parancsot az előzmények megőrzésére ([CI.md](CI.md) és [CLI_REFERENCE.md](CLI_REFERENCE.md)).

**39. Miért utasítja el a `render` az `out.jpg` fájlt?**
A `render` csak `.png` vagy `.svg` fájlt ír: `render writes .png or .svg; 'out.jpg' has another extension (nothing was written)`. Más formátumokhoz használd az `export` parancsot; a lista az [EXPORT_FORMATS.md](EXPORT_FORMATS.md) fájlban van.

---

## Riggelés és animáció

**40. Merre van a nulla fok egy csontnál?**
A 90-es csontszög lefelé mutat; a szögeket nem úgy mérik az x tengelytől, ahogy egy matematikai tankönyv. A csatornák a nyugalmi pózhoz viszonyított különbségek, a `rest_offset` pedig a szülő keretében van megadva ([RIGGING_GUIDE.md](RIGGING_GUIDE.md)).

**41. Az IK-láncom nem nyúlik, vagy beugrik. Mi a `soft` és a `stretch`?**
A `soft` (0-tól 0,9-ig) simán vezeti a láncot a teljes kinyúlásba, a `stretch` (1-től 4-ig) pedig megengedi a megnyúlását. Mindkettő csak JSON-ban érhető el, Rhai-ban vagy CLI-kapcsolóként nem. Ha a `stretch` 1 fölötti, a soft zóna a megnyúlt elérés végén van. A kétcsontos láncokat analitikusan oldja meg; a három vagy több csontosak FABRIK-ot használnak.

**42. A kényszereim furcsa eredményt adnak.**
A kényszerek a lista sorrendjében futnak, és a sorrend számít. Az 0,5 súlyú aim a szög felét adja (a -74,7-es szögből -37,4 lesz). Ha egy későbbi kényszernek látnia kell egy korábbi eredményét, rendezd át a JSON-ban.

**43. Van automatikus skinner vektoros grafikához?**
A parancssorban nincs. A `Project::auto_skin` csak a könyvtárban létezik. Vektoros grafikánál rétegekhez szülőként rendelt csontokkal, lattice-ekkel, spine-okkal vagy pinekkel (csak JSON), vagy a `mesh-trace` parancs által követett meshekkel riggelsz.

**44. A `skin-diffuse` és a `cloth` panaszkodik a meshre.**
Mindkettőnek mesh kell, a cloth-nak pedig **rácsos** mesh: `cloth needs a grid mesh; the layer has a free-form (traced) mesh`. A mesh nélküli réteg ezt adja: `layer 'flag' has no mesh`.

**45. A `foot-lock` azt mondja, hogy gyökér kell neki.**
`foot lock needs a root controller bone (no parent)`. A gyökérvezérlő csontnak nem lehet szülőcsontja.

**46. A `scale` elutasítja a kulcskockáimat.**
A `scale: frame 27 is outside the project range 0..=24` azt jelenti, hogy egy eredménykulcs a projekten kívülre esne; a `keyedit scale` tartományon kívül elutasításra kerül, hacsak nem adod meg a `--free` kapcsolót. A próbafuttatás kimenetneve `-`.

**47. Miért lett hosszabb a projektem a `mocap-retarget` után?**
A `mocap-retarget` a projekt hosszát a rögzített mozgáshoz igazítva kiterjeszti. A `follow --chain` kulcsokat tolhat az utolsó képkocka mögé, amit a `lint` jelez (L007).

**48. Hogyan animálhatok egy réteget csont nélkül?**
Szerkeszd a réteg `layer_x` és `layer_y` csatornáit. A stúdióban kapcsold be az auto-key funkciót; kikapcsolt állapotban a réteg szerkesztése statikus marad, hacsak az adott tulajdonságnak nincs már kulcsolt csatornája. A csontszerkesztések mindig kulcsolnak. Lásd [STUDIO.md](STUDIO.md).

---

## Stúdió

**49. Hogyan nyitom meg a stúdiót?**
Az `osatura studio project.json` paranccsal, és nyisd meg a kiírt címet, ami általában `http://127.0.0.1:<port>`. Egyoldalas alkalmazás, amelyet az OSATURA saját kis HTTP-szervere szolgál ki (csak a szabványos könyvtárral); nincs szüksége internetre.

**50. Megváltoztatja a stúdió a fájlomat, miközben dolgozom?**
Nem. A lemezen lévő fájl csak a Save megnyomásakor változik. Az oldal mentés nélküli bezárása elveszíti a szerkesztéseket (az együttműködési napló, ha be van kapcsolva, az egész projektet megőrzi, de a visszavonási előzményeket nem).

**51. Csatlakozhatnak mások?**
Csak ha nem loopback címre kötöd, és ekkor token szükséges: `refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (and still sends everything in clear text)`. A tokennek legalább 16 karakteresnek kell lennie, például az `openssl rand -hex 16` kimenete. Nincs titkosítás: használj SSH-alagutat vagy TLS-sel ellátott fordított proxyt.

**52. Hová tegyem a tokent?**
Az `X-Collab-Token` fejlécbe vagy az `osatura_token` sütibe (HttpOnly, SameSite=Strict, és Secure, ha a kérés https `--public-origin` alatt érkezik). Az URL-ben soha nem fogadja el. Az ötödik sikertelen kísérlettől az adott klienscím HTTP 429-et kap növekvő késleltetéssel (2, 4 ... legfeljebb 30 mp).

**53. Mit jelentenek a 403, 415 és 421 HTTP-hibák?**
421: a Host fejléc nem az, amit a stúdió vár. 403: idegen `Origin` vagy `Sec-Fetch-Site`. 415: nem JSON POST. Ezek védelmet jelentenek a böngésződben lévő oldalak ellen, amelyek a helyi szervert hívnák; lásd [STUDIO.md](STUDIO.md) és [SECURITY.md](SECURITY.md).

**54. Minden böngészőben működik?**
Csak headless Chromiumban tesztelték. A többi nincs tesztelve. Nincs vászonnagyítási parancs, és létezik stúdióbeli panel a webkamerás arckövetéshez, de valódi kamera sosem állt rendelkezésre a teszteléshez ([LIMITATIONS.md](LIMITATIONS.md)).

---

## Arc, beszéd és mozgásrögzítés

**55. Követi az OSATURA az arcokat?**
Van egy kis beépített detektor, amelyet csak szintetikus arcokon tanítottak. 50 szintetikus tesztjeleneten (seed 9) minden arcot megtalált, az átlagos tereptárgyhiba az arcméret 0,028-szerese volt, és az arc nélküli 50 jelenetben 2 téves riasztást adott. Rajzfilmstílusú rajzokon gyengén teljesített: a 6 szögletes robotarcból 0-t talált meg, a kerek emoji és a manga-ovális eseteknél a hibák 0,164 és 0,166 voltak. Valódi fényképeken vagy valódi webkamerán még soha nem futtatták. Vedd figyelembe, hogy a [LIMITATIONS.md](LIMITATIONS.md) L-13 pontja és a stúdió webkamera-paneljének szövege azt mondja, hogy nincs arcdetektor a csomagban; ez ezzel a szintetikussal kapcsolatban elavult.

**56. Hogyan működik a lip-sync?**
A szájjelzések öt alak közül választanak (nyugalmi, kicsi, nyitott, kerek, széles). A tizedesponttal megadott jelzésidő másodperc; a puszta egész szám képkocka. A `--rhubarb` csak az E betű osztályozását változtatja (kerek a széles helyett). A projektnél hosszabb hangot levágja.

**57. Animálhatok videóból?**
A `face-track` videót olvas az ffmpeg-gel. A `face-bake` a jeleket kulcsolt csatornákként írja ki (az összegzés így szól: „+N channels (K keys in them), M trigger keys set”), és a projekt hosszára vágja, stderr-figyelmeztetéssel a nem bake-elt képkockák számáról. Ismeretlen jel esetén: `unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)`.

---

## Hibák, biztonság és közreműködés

**58. Hogyan jelentsek hibát?**
Nyiss egy issue-t a parancssorral, a pontos kimenettel (a stderr-rel is), a kilépési állapottal, a builddel (`osatura --version`) és, ha tudsz, egy minimális projekttel. Előbb futtasd az `osatura check project.json` és az `osatura lint project.json` parancsot: sok problémát ott jeleznek. Lásd [CONTRIBUTING.md](CONTRIBUTING.md).

**59. Hogyan jelentsek biztonsági problémát?**
Privátan, nem nyilvános issue-ban; lásd [SECURITY.md](SECURITY.md). A tárolóban megadott cím `alexio@trom.tf`; ez ugyanaz a cím, mint a karbantartó elérhetősége, és nem váltotta fel biztonsági csapat postafiókja.

**60. Van levelezőlista, chat vagy támogatási szerződés?**
Nincs. A dokumentumok egyetlen karbantartói elérhetőséget neveznek meg; az első kiadás még csak előzetes kiadás-vázlat.

**61. Hogyan járulhatok hozzá?**
Írd alá a commitokat (`git commit -s`), tartsd sikeresen futva a teszteket (`cargo test`, animációs munkánál az `osatura ci`), és olvasd el a [DEVELOPMENT.md](DEVELOPMENT.md) fájlt. A dokumentáció CC-BY-4.0 licencű, és szívesen fogad javításokat, főleg ott, ahol egy oldal és a program eltér egymástól.

---

## Korlátozások

**62. Mi az egyetlen legfontosabb korlát?**
Egyetlen projekt építette és tesztelte egyetlen környezetben. Bármi, ami nincs futtatottként megjelölve, „nincs tesztelve”. Olvasd el a [LIMITATIONS.md](LIMITATIONS.md) fájlt; vannak azonosítói, amelyeket idézhetsz az issue-kban.

**63. Mely exportformátumokra kell vigyázni?**
A videóhoz ffmpeg kell. A Lottie és a glTF az OSATURA által rajzolható és animálható dolgok egy részhalmazát fedi le, ezért az [EXPORT_FORMATS.md](EXPORT_FORMATS.md) fájlban nézd meg, mi vész el. Az ismeretlen kiterjesztésre adott `export` hiba a `.gif .html .json .mp4 .apng .webm .png .glb` listát adja, amely kihagyja a `.lottie`-t; a `.lottie` (dotLottie) érvényes cél, lásd [EXPORT_FORMATS.md](EXPORT_FORMATS.md), 6. szakasz.

**64. Használható a WASM build?**
A GitHub Actionsön lefordul `wasm32-unknown-unknown` célra, a wrapper tesztje pedig sikeres a valódi modul ellen Node 20 alatt, és `osatura-web.tar.gz` néven csatolva van a vázlatkiadáshoz. Böngészőben még nem futtatták, sebessége és memóriahasználata nincs megmérve ([WASM.md](WASM.md), [LIMITATIONS.md](LIMITATIONS.md)). Tekintsd részben ellenőrzöttnek.

**65. Importálja más eszközök projektfájljait?**
Importál SVG-t, OpenRastert, Lottie JSON-t, képeket, GIF/APNG-t, BVH mozgásrögzítést és MIDI-t ([EXPORT_FORMATS.md](EXPORT_FORMATS.md), 9. és 10. szakasz). PSD-t, `.lottie` zip-fájlokat és OCA mappákat nem importál. Az importok új projektet hoznak létre. Vedd figyelembe, hogy az `import-lottie in.json in.json` parancsnak nincs felülírásvédelme, és lecseréli a bemenetét. Ne várd, hogy egy importált fájl pixelre megegyezzen az eredetivel.

**66. Stabil a projekt JSON-ja?**
„1-es formátum”. Az újabb fájlt elutasítja: `project format 2 is newer than this build supports (1); update OSATURA`. Mivel csak alfa előzetes kiadás létezik, maga az 1-es formátum még változhat egy stabil kiadás előtt; lásd [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

---

## Összehasonlítás más eszközökkel

**67. Miben különbözik egy idővonalalapú asztali animációs szerkesztőtől?**
Azok interaktív, rajzközpontú programok. Az OSATURA parancsközpontú: minden művelet parancs vagy szkript, a projekt diffelhető JSON, a renderek determinisztikusak, így CI-ben tesztelhetők. A stúdió ugyanazon motor fölötti vékony webes nézet, és messze kevésbé kiforrott, mint egy érett szerkesztő.

**68. Miben különbözik egy játékokhoz készült 2D csontanimációs eszköztől?**
Azok az eszközök meshekre, skinekre és adott játékmotorokhoz készült futtatókörnyezeti könyvtárakra összpontosítanak. Az OSATURA-ban is vannak csontok, IK, kényszerek és meshek, de a futtatókörnyezeti kínálata kisebb: saját lejátszó HTML-ben, egy C ABI és egy lefordítatlan WASM-crate. A célozandó futtatókörnyezet alapján válassz.

**69. Miben különbözik egy motion graphics kompozitortól?**
A kompozitor effektekkel és kifejezésekkel ellátott rétegek köré épül videófelvételeken. Az OSATURA-ban vannak rétegeffektek és kifejezések, de a középpontja a karakterriggelés és a vektoros grafika, és nem végez videókompozitálást, 3D-t vagy felvételek színkorrekcióját.

**70. Mikor rossz választás az OSATURA?**
Ha ma valós idejű GPU-teljesítményre, kiforrott rajztábla-munkafolyamatra, képkockánkénti kézzel rajzolt animációra, stabil kiadással és támogatással rendelkező, gyártó által támogatott termékre vagy a Linux x86-64-en kívüli igazolt viselkedésre van szükséged.

---

## Hibaelhárítási táblázat

Az alábbi üzenetek mindegyikét valódi build állította elő, és 1-es kilépési állapottal végződik, hacsak nincs másként jelölve. Az üzenetek szövege egy kiadás előtt változhat.

| Üzenet | Ok | Megoldás |
|---|---|---|
| `cannot read nope.json: No such file or directory (os error 2)` | Rossz útvonal vagy név | Ellenőrizd az útvonalat és az aktuális könyvtárat |
| `invalid JSON in broken.json: EOF while parsing a value at line 2 column 0` | A fájl nem teljes JSON | Javítsd a szintaxist a megnevezett sorban; állítsd vissza biztonsági mentésből vagy az `osatura vcs` paranccsal |
| `badparent.json is not a valid project:` majd `bone 'a' has parent 'ghost' which is missing or listed after it` | Egy szülőcsont hiányzik, vagy a gyereke után szerepel | Definiáld a szülőket a gyerekek előtt; javítsd a nevet |
| `invalid JSON in fmt2.json: project format 2 is newer than this build supports (1); update OSATURA` | Újabb buildből származó fájl | Használj újabb buildet |
| `warning: unknown field bogus (ignored)` (kilépési kód 0) | Olyan kulcs, amelyet ez a build nem ismer | Távolítsd el vagy javítsd a kulcsot; ellenőrizd az elgépelést |
| `refusing to overwrite the input project X with output of the same name` | A kimeneti fájl neve megegyezik a bemenetével | Válassz másik kimeneti nevet |
| `render writes .png or .svg; 'out.jpg' has another extension (nothing was written)` | Rossz kiterjesztés a `render` parancsnak | Használj `.png`/`.svg` kiterjesztést, más formátumokhoz az `export` parancsot |
| `unknown option --bone for 'render' (did you mean --bones?); see: osatura (no arguments) for the usage` | Elgépelt kapcsoló | Használd a javasolt kapcsolót |
| `unknown export format '.bmp' (use .gif .html .json .mp4 .apng .webm .png .glb)` | A kiterjesztés nem exportálható | Válassz egy felsorolt formátumot (a `.lottie` is érvényes) |
| `--scale needs a number in (0, 16]` | A skála tartományon kívül van | Használj 0-nál nagyobb és legfeljebb 16 értéket |
| `bad frame number 'x'` | A képkocka-argumentum nem egész szám | Adj meg egész számot |
| `cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead` | Az ffmpeg nincs a `PATH`-on | Telepítsd az ffmpeg-et, vagy exportálj GIF-be, HTML-be vagy APNG-be |
| `the project has no sound tracks (see: sound add)` | Hangparancs hangtalan projekten | Adj hozzá sávot a `sound add` paranccsal |
| `channel hand_ctrl.BoneOffsetY is locked` | A csatorna zárolt | Oldd fel, vagy szerkessz másik csatornát |
| `request 1 (hand_ctrl.BoneOffsetY at 0) was rejected, nothing changed: channel hand_ctrl.BoneOffsetY is locked` | Ugyanez, a `script` parancsból | Előbb oldd fel; semmi sem érvényesült |
| `request 1 (channel head.BoneAngle) was rejected, nothing changed: ... expression error: unknown variable 'tt'` | Elgépelés egy kifejezésben | A változók: frame, time, fps, frames, pi, t |
| `... unexpected end of expression` | Befejezetlen kifejezés | Zárd le a zárójeleket, fejezd be a képletet |
| `points need an even number of coordinates: 3..1000 points` | Páratlan koordinátaszám vagy túl kevés pont | Adj meg x,y párokat: 3-tól 1000 pontig |
| `cannot read /dev/fd/63: not a regular file (devices, pipes and directories are refused)` | Process substitution vagy cső a bemenet | Előbb mentsd szabályos fájlba |
| `sound add: asset '/etc/passwd': absolute paths are not allowed (...)` | A projektmappán kívüli eszköz | Másold a projekt mellé, vagy add meg az `--allow-external-assets` kapcsolót |
| `cannot read demo.json: N bytes exceed the input limit of M bytes (raise it with --max-input-bytes N)` | A fájl meghaladja a bemeneti korlátot | Emeld a `--max-input-bytes` értékét, ha megbízol a fájlban |
| `refusing to write through a symbolic link; pass --follow-symlinks ...` | A kimenet szimbolikus link | Nevezd meg a célt, vagy add meg a `--follow-symlinks` kapcsolót |
| `cloth needs a grid mesh; the layer has a free-form (traced) mesh` | Cloth követett meshen | Építs rácsos meshet a réteghez |
| `layer 'flag' has no mesh` | Mesh-parancs sima rétegen | Előbb hozz létre meshet |
| `no bone 'b9'` | A csontnév nem létezik | Ellenőrizd a neveket a `stats` vagy az `explain` paranccsal |
| `unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)` | Rossz jelnév | Használj a listából egy nevet |
| `foot lock needs a root controller bone (no parent)` | A választott gyökérnek van szülője | Használj szülő nélküli vezérlőt |
| `scale: frame 27 is outside the project range 0..=24` | A skálázott kulcsok a projekten kívülre esnek | Hosszabbítsd meg a projektet, skálázz kevésbé, vagy használd a `--free` kapcsolót |
| `--extrapolate must be hold, linear, loop, ping_pong or loop_offset` | Rossz mód | Használd az öt közül az egyiket |
| `cannot listen on 127.0.0.1:8861: Address already in use (os error 98)` | A port foglalt | Válassz másik `--port` értéket, vagy állítsd le a másik folyamatot |
| `cannot listen on 127.0.0.1:99999: invalid port value` | A port 65535 fölötti | Használj 1-től 65535-ig terjedő értéket |
| `refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (...)` | Nyilvános kötés token nélkül | Add hozzá a `--collab-token` kapcsolót, vagy maradj a loopbacken |
| `--collab-token must be at least 16 characters (e.g. \`openssl rand -hex 16\`)` | Gyenge token | Generálj hosszabbat |
| `--source-url must be an http:// or https:// URL without spaces or quotes` | Hibás URL | Használj egyszerű URL-t |
| `--public-origin must be a bare origin (scheme, host and optional port, no path), e.g. https://studio.example.org` | Az origin útvonalat tartalmaz | Csak sémát, gazdát és portot adj meg |
| `output would be invalid: ...` | A szerkesztés érvénytelen projektet hagyna | Olvasd el a kettőspont utáni okokat; semmi sem íródott ki |
| `budget not met: the smallest export found is 14952 bytes (raw), above the budget of 8000 bytes ...` | Az `export-fit` nem tudja elérni a keretet | Emeld a keretet, vagy egyszerűsítsd a projektet (a legkisebb fájl ettől még kiíródik) |
| `time budget exhausted (60 s ...)` | Az `export-fit` kifutott az időből | Emeld a `--max-seconds` értékét |
| `fuzz: failure rate 98.00% exceeds --max-fail-rate 50.00%` | A pózok tönkretették a riget, és kapuzást kértél (`--gate` vagy `--max-fail-rate`) | Nézd meg a hibás eseteket; csak szándékosan emeld az arányt; a sima `fuzz` csak jelent (kilépési kód 0) |
| `no .osatura-vcs repository found from ... (run: osatura vcs init <project.json>)` | Még nincsenek előzmények | Futtasd a `vcs init` parancsot |
| `frame 999 out of range` (stúdió API) | A képkocka a projekten túl van | Használj a 0 és az utolsó képkocka közötti értéket |
| `bad kind 'x' (vector, raster, png)` (stúdió API) | Rossz fajta | Használj egy felsorolt fajtát |
| `nothing selected` (stúdió API) | Kijelölésparancs kijelölés nélkül | Előbb jelölj ki rétegeket vagy csontokat |

Az `osatura ci` kilépési kódjai: 0 sikeres, 1 sikertelen, 2 hiba.

---

*A dokumentáció licence CC-BY-4.0. Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio). Forrásmegjelöléssel megoszthatod és átdolgozhatod. Lásd még: [INDEX.md](INDEX.md), [GLOSSARY.md](GLOSSARY.md).*
