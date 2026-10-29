# OSATURA stúdió

A stúdió egy böngészős szerkesztő, amelyet maga az `osatura` bináris szolgál ki: `osatura studio project.json`.
Van benne vászon kijelölő és transzformáló gizmókkal, rétegek listája, csontok listája, dope sheet és görbegráf,
onion skin és mozgásnyomok, újrakötható gyorsbillentyűkkel ellátott parancspaletta, pózjavaslatok, élő bemeneti és
arcpanelek, GIF/HTML/Lottie exportgombok, valamint közös, többfelhasználós szerkesztési munkamenet. Nincsenek fiókjai, nincs TLS-e és
nincs adatbázisa: egy kis, csak a szabványos könyvtárat használó HTTP-szerver a parancssor által használt szerkesztő előtt.

Itt minden a valódi binárison (`osatura 0.1.0`, debug build, Linux) lett ellenőrizve, a felhasználói
felület pedig Playwright által vezérelt headless Chromiumban. A képernyőképek is így készültek (1400x900, a beépített demó
projekt). Csak **Chromiumot** teszteltünk; lásd a 11. szakaszt. Windowson és macOS-en a kiadási archívumok az `osatura-studio` programot is tartalmazzák, ugyanezt a stúdiót natív ablakban (a CI-ben lefordítva és önteszttel ellenőrizve, ember még nem használta; lásd [GETTING_STARTED.md](GETTING_STARTED.md) és [LIMITATIONS.md](LIMITATIONS.md) L-75), Windowson pedig az `osatura.exe` dupla kattintása elindítja a stúdiót, és megnyitja a böngészőt (L-76).

A dokumentum licence: CC-BY-4.0. A parancssorhoz lásd a [GETTING_STARTED.md](GETTING_STARTED.md) és a
[CLI_REFERENCE.md](CLI_REFERENCE.md) fájlt; a szerver biztonságához lásd még a [SECURITY.md](SECURITY.md) fájlt.

![A stúdió a demóprojekttel: fent az eszköztár, a vásznon egy kijelölt kar réteg a megrajzolt csontjaival, jobbra a Rétegek panel, alul az idővonal kulcsrombuszokkal.](../../img/studio-overview.png)

## 1. A stúdió indítása

```
osatura studio <project.json> [--port N] [--ui-config F] [--bind ADDR] [--allow-host NAMES]
                [--public-origin URL] [--source-url URL] [--collab-token T] [--collab-log LOG [--collab-nosync]]
```

```
$ osatura demo demo.json
$ osatura studio demo.json --port 8080
AGPL-3.0 source offer: http://127.0.0.1:8080/source
OSATURA studio on http://127.0.0.1:8080  (Ctrl-C to stop; saves go to demo.json)
UI settings (layout, shortcuts) in /home/you/.config/osatura/ui.json      (your own home directory)
```

Nyisd meg a címet egy böngészőben. A projektet induláskor ellenőrzi; a `check`-en át nem menő fájlt ugyanazzal az
üzenettel utasítja el. Az ismeretlen mezőkre figyelmeztet (`warning: unknown field bogus (ignored)`).

| Kapcsoló | Alapértelmezés | Jelentés |
|---|---|---|
| `--port N` | 8080 | TCP-port. Foglalt port esetén `cannot listen on 127.0.0.1:8861: Address already in use (os error 98)`, hibás portnál `cannot listen on 127.0.0.1:99999: invalid port value` az üzenet. |
| `--bind ADDR` | `127.0.0.1` | A figyelt interfész. Minden, ami nem loopback cím, **megköveteli a `--collab-token` megadását**. |
| `--ui-config F` | `$OSATURA_UI_CONFIG`, különben `$XDG_CONFIG_HOME/osatura/ui.json`, különben Windowson `%APPDATA%\OSATURA\ui.json` (az alpha.3 előtt nem volt ilyen: a beállítások csak a munkamenet idejére maradtak meg), különben `~/.config/osatura/ui.json` | Itt tárolódik az elrendezés, a gyorsbillentyűk, a legutóbbi parancsok és az overlay-beállítások. Felhasználónkénti fájl, soha nem része a projektnek. |

Az ablakos alkalmazás és a dupla kattintásos indítás az első projektet az `OSATURA_DOCUMENTS` könyvtárban hozza létre, ha ez a változó be van állítva, különben a `Documents\OSATURA` mappában (ezt használja a hordozható csomag).
| `--allow-host NAMES` | nincs | További elfogadott `Host` nevek (vesszővel elválasztva), DNS-néven való eléréshez. |
| `--public-origin URL` | nincs | A TLS fordított proxy által bemutatott puszta origin (`https://studio.example.org`, útvonal nélkül); ennek `Host` és `Origin` értékét ezután elfogadja. |
| `--source-url URL` | nincs; a hivatalos kiadási binárisokban az a commit, amelyből épültek (`https://github.com/chiuta/osatura/tree/<commit>`) | Innen tölthető le a *te* buildedhez tartozó forráskód (AGPL 13. szakasz); a `/source` oldalon és a `--version` kimenetében jelenik meg. Egy kifejezetten megadott érték mindig felülír. |
| `--collab-token T` | nincs | Közös titok, **legalább 16 karakter** (`openssl rand -hex 16`). Nem loopback kötésekhez kötelező. |
| `--collab-log LOG` | nincs | A közös szerkesztési munkamenetet fájlba menti, és újraindítás után folytatja. |
| `--collab-nosync` | ki | A `--collab-log` mellett: nem végez `fsync`-et minden szerkesztés után (gyorsabb; áramkimaradás elvesztheti az utolsó szerkesztéseket). `--collab-log` nélkül indításkor elutasítja (`--collab-nosync needs --collab-log`). |

**Mentés.** A szerkesztések a memóriában élnek. A lemezen lévő projektfájl csak a **Save** (`Ctrl+S`) megnyomásakor változik, amely atomi módon ír
(ideiglenes fájl, majd átnevezés, soha nem szimbolikus linken keresztül). Ez közös munkamenet közben is igaz (8. szakasz).

## 2. Biztonsági modell (olvasd el, mielőtt hálózatra kötöd)

A stúdiót egy saját gépén dolgozó személynek szánták. Ami védi:

* **Host engedélylista.** Csak azokra a kérésekre válaszol, amelyek `Host` fejléce a szerver saját nevei közül való; minden más **421**-et kap.
  Az `--allow-host` és a `--public-origin` további neveket fogad el.
* **Webhelyek közötti védelem.** Az idegen `Origin` fejlécű, vagy `Sec-Fetch-Site: cross-site` kérés **403**-at kap. Az a POST, amelynek törzse nem
  `application/json`, **415**-öt kap (a törzs nélküli POST-ok, például az `/api/undo`, nem igényelnek típust).
* **Biztonsági fejlécek** a válaszokon: szigorú `Content-Security-Policy` (`default-src 'self'` és keretezés tiltva), `X-Frame-Options: DENY`,
  `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `Cache-Control: no-store`.
* **Token minden, a gépen kívüli eléréshez.** A `--collab-token` mellett minden kérésnek tartalmaznia kell a tokent az `X-Collab-Token` fejlécben, vagy a
  `POST /api/login` által beállított `osatura_token` sütiben (HttpOnly, SameSite=Strict). A tokent az URL-ben **soha nem fogadja el**. A hibás tokeneket
  klienscímenként lassítja (az egymás utáni ötödik hibától az a cím **429**-et kap 2, 4, ... legfeljebb 30 másodpercre), de a késleltetés nem tudja megállítani azt a
  támadót, aki folyamatosan próbálkozik: használj hosszú, véletlen tokent.
* **Résztvevői titkok.** Egy közös munkamenethez való csatlakozás véletlen 128 bites titkot ad vissza; a résztvevőként tett minden hívásnak ezt kell küldenie
  `X-Collab-Secret` néven. A résztvevő számának ismerete nem elég.

Mindezt `curl`-lel ellenőriztem egy futó szerveren:

```
bad Host (evil.example):               421
allowed host (--allow-host):           200
cross-origin POST (Origin: evil.example): 403
POST from the --public-origin:         200
Sec-Fetch-Site: cross-site:            403
POST with Content-Type: text/plain:    415
no token, GET / or /api/state:         401
wrong X-Collab-Token:                  401
right X-Collab-Token:                  200
token in the URL (?token=...):         401
/source without a token:               200
unknown route:                         404   (GET /favicon.ico answers with a small SVG icon, without a token)
```

Indítási elutasítások (1-es kilépési kód, semmi sem indul el):

```
refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (and still sends everything in clear text)
--collab-token must be at least 16 characters (e.g. `openssl rand -hex 16`)
--source-url must be an http:// or https:// URL without spaces or quotes
--public-origin must be a bare origin (scheme, host and optional port, no path), e.g. https://studio.example.org
```

**Őszinte korlátok.** A forgalom sima HTTP: a token és minden más titkosítatlanul utazik, ezért ha a stúdió bárki más számára elérhető, tedd TLS-es fordított proxy mögé
(`--public-origin`). A token `Set-Cookie` fejléce megkapja a **`Secure` jelzőt**, ha a kérés `https://` `--public-origin` név alatt érkezik (vagy https `Origin` / `X-Forwarded-Proto: https` mellett); a sima HTTP loopbacken soha,
és proxy mögött minden kliens a proxy címével látszik, így a késleltetés mindenkit egyszerre fékez. A `/source` token nélkül is nyitott
(az AGPL ezt kéri). Nincsenek felhasználói fiókok: bárki, akinél a token van, módosíthatja a projektet és megadhatja a mentési útvonalat.

### Az AGPL forráskód-ajánlat

A stúdió AGPL-3.0-or-later szoftver: ha módosítod, és mások hálózaton keresztül használják, fel kell kínálnod nekik a saját
változatod forráskódját. A `/source` ez az ajánlat; állítsd a `--source-url` kapcsolót arra a helyre, ahol a builded forráskódja van. Enélkül (a sima `cargo build`-del magad készített buildnek nincs ilyen) az oldal ezt közli, és megnevezi a karbantartót. A `.github/workflows/release.yml` által épített kiadási binárisok a saját commitjuk címét hordozzák (`OSATURA_SOURCE_URL` fordítási időben; a saját munkafolyamatával építő fork önmagára mutat).

## 3. Az felület bejárása

### Eszköztár

Visszavonás, Újra, **Save**; jelölőnégyzetek: `bones` (a csontváz rajzolása), `auto-key`, `grid` (térköz vászonegységben, 0 = ki), `snap` (más rétegekhez és a vászonhoz),
`whole groups` (egy csoport egy részére kattintva a legfelső szintű csoportot jelöli ki), `points` (a kijelölt rétegek horgonyainak, fogantyúinak, cage/lattice/pin pontjainak és mesh-csúcsainak mutatása);
egy `box` választó arra, mit jelöl ki a húzott doboz (`touches`, `contains`, `bounds touch`; alapértéke `touches`); **Lock**; valamint a **Overlays**, **Suggest**, **Layout**,
**Commands** (paletta) és **Keys** (gyorsbillentyű-szerkesztő) gombok. A második sorban a neved és a **Collab** található, valamint az exportgombok: **GIF**, **HTML player** és **Lottie**,
amelyek letöltik azt, amit az `osatura export` írna ki (a HTML-lejátszó tartalmazza a kevert hangot, ha van hang, és az `ffmpeg` elő tudja állítani az mp3-at). A **Simple view** gomb elrejti a szerkesztési jelölőnégyzeteket, az Overlays és a Suggest gombot, valamint a parancspaletta gombját, így csak a visszavonás, a mentés, a panelek és az exportok maradnak; a választást ez a böngésző (vagy ablak) megjegyzi, és a teljes eszköztár a **Full view** gombbal tér vissza.

### Panelek és elrendezések

Az ablak panelek dokkja, mindegyiknek stabil azonosítója van: `stage` (a vászon), `layers`, `selection`, `bones`, `live` (Live input), `face` (Webcam face), `json`
(a teljes projekt szerkeszthető JSON-ként egy *Apply JSON* gombbal) és `timeline`. Négy elrendezés-előbeállítás létezik:

```
animate  stage over timeline on the left, all panels as tabs on the right
rig      stage with a folded timeline; layers+bones and selection+json on the right (live and face hidden)
compact  stage over a tabbed timeline and panels
live     stage and timeline on the left; live, face, bones and selection on the right (layers and json hidden)
```

Az előbeállításokat a **Layout** menüből vagy a palettáról válthatod; a fülek húzásával újradokkolhatsz vagy lebegtethetsz egy panelt; a „reset layout” visszaállítja az alapértelmezést.
Az elrendezés a UI-konfigurációban tárolódik, felhasználónként. Az `osatura ui layout presets|show NAME|validate FILE.json` ugyanezeket az adatokat írja ki és ellenőrzi
(az elrendezésnek minden panelt elérhetővé kell tennie; a hiányosat a szerver 400-zal utasítja el).

### Rétegek panel

![A Rétegek panel egy zárolt réteggel (sun), egy rejtett réteggel (hills), valamint együtt kijelölt kar és láb réteggel.](../../img/studio-layers.png)

Minden sorban két jelölőnégyzet van: **visible** és **lock** (*a projekttel együtt* mentődik; a zárolt rétegeket a kattintások és a dobozok átugorják, és elutasítják a szerkesztést).
A képernyőkép a zárolt `sun` réteget, a rejtett `hills` réteget és két, Shift-kattintással kijelölt réteget mutat.

### Selection panel

A szabályokat maga az oldal közli: *A kattintás a legfelső dolgot választja (a zárolt és rejtett dolgokat átugorja); a Shift-kattintás kapcsol; az üres helyen húzás
dobozt rajzol (Alt nyomva tartásával szabad formájú lasszót, a Shift hozzáad). A dobozt húzva mozgatsz, a négyzetet húzva skálázol (a Shift megtartja az arányokat, az Alt
a középpontból skáláz), a kerek gombbal forgatsz (a Shift 15 fokra illeszt). Húzás közben tartsd lenyomva a Ctrl-t az illesztés kikapcsolásához. A nyílbillentyűk 1 egységet léptetnek (Shift: 10),
a Delete törli a rétegeket, az Esc megszakítja a húzást vagy törli a kijelölést.*

### Idővonal: dope sheet és gráf

![A dope sheet három kulcs dobozos kijelölésével a 12. képkockánál (a kulcsok tömörek, a többi üreges).](../../img/studio-dopesheet.png)

![A görbegráf nézet: három bone_angle görbe (arm_up, arm_low, hand) különböző színekkel, három kijelölt kulccsal a 12. képkockánál, amelyek érintőfogantyúi látszanak; az állapotsor: „3 keys selected | frames 12”.](../../img/studio-graph.png)

A `Graph` (vagy `Alt+V`) vált a dope sheet és a görbegráf között. Az idővonal vezérlői: `Play`, `|<` (a tartomány első képkockája), `loop`, `range` a
`Set in` / `Set out` gombokkal (csak a be és ki pont között játszik), `snap` (a mozgatott kulcsokat egész képkockára kerekíti), `on collision` (`replace`, `merge`, `refuse`, ha a mozgatott kulcsok
meglévőkre esnek), `Copy` / `Paste` / `Duplicate` / `Delete`, `interp` + `Apply` (a kijelölt kulcsok easingjének beállítása), `Auto-smooth` (túllendülésmentes érintők bake-elése a
kijelölt kulcsokba), `ripple` N az `Insert` / `Remove` gombokkal (és `grow project` a projekt hosszának módosításához velük együtt), nagyítás `-` `+` `Fit`, és `shy` (a shy jelölésű csatornák mutatása).

Egér: kattintással kulcsot jelölsz ki, Shift-kattintással hozzáadsz, dobozhúzással kijelölsz, a kulcsok húzásával mozgatod őket; az **Alt-húzás vagy a középső gomb pásztáz**, a **Ctrl+görgő nagyít**,
a **Shift+görgő függőlegesen görget**; a gráfban a **Shift egy tengelyre korlátozza a kulcshúzást**. A vászonillesztés tűréshatára alapértelmezés szerint 6 vászonegység.
Minden kulcsszerkesztés ugyanazokon az ellenőrzött műveleteken megy át, mint az `osatura keyedit` (lásd [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)), így egyetlen visszavonási lépés, és a zárolt csatornák elutasítják.

### Overlayek: onion skin, nyomok, proxyk

![Az onion skin és a mozgásnyomok bekapcsolva a karnál a 14. képkockánál, nyitott Overlays panellel.](../../img/studio-overlays.png)

Az `Overlays` (`Alt+7`) tartalmazza az onion skint (`Alt+N`: a korábbi/későbbi képkockák piros/zöld árnyalattal, fix lépés vagy kulcskockák, körvonal mód, lecsengés, csak a kijelölt rétegek),
a mozgásnyomokat (`Alt+M`: csont-, réteg-pivot- vagy kontrollpont-útvonalak képkockánkénti jelölőkkel) és a **scrub proxykat** (`Alt+Q`: csökkentett minőségű képek a lejátszófej
húzása közben, képgyorsítótárral és opcionális részletességi szintű másolattal). A panel például ezt jelenti: `picture cache: 2 frames, 17 KiB, 0 hits / 7 misses, emptied 4x by edits`.
A beállítások a UI-konfigurációban tárolódnak, és túlélik az újraindítást. Az overlayek soha nem részei az exportoknak.

### Parancspaletta és gyorsbillentyűk

![A Ctrl+K-val megnyitott és a „toggle” szóra szűrt parancspaletta, a találó parancsokkal és gyorsbillentyű-jelzéseikkel.](../../img/studio-palette.png)

A `Ctrl+K` (vagy `Ctrl+Shift+P`) megnyitja a palettát: gépeléssel szűrsz (fuzzy, az egyező betűk kiemelve), a nyílbillentyűk mozognak, az Enter futtat, az Esc bezár. Ha semmit sem írsz,
a legutóbbi parancsokat listázza elöl. **84 parancs** van; az `osatura ui commands` felsorolja őket, az `osatura ui search onion` pedig pontosan úgy rangsorolja őket, mint a paletta.

![A Ctrl+vessző-vel megnyitott gyorsbillentyű-szerkesztő, kategóriák szerint listázva a parancsokat a billentyűkombinációikkal.](../../img/studio-shortcuts.png)

A `Ctrl+,` (vagy a **Keys**) megnyitja a gyorsbillentyű-szerkesztőt: kattints egy kombinációra, nyomd meg az újat. Ütközés esetén megnevezi a másik parancsot, és semmit sem változtat, amíg meg nem
erősíted a „Take it over” gombbal. A böngésző által lefoglalt kombinációkat elutasítja: `Ctrl+W`, `Ctrl+T`, `Ctrl+N`, `Ctrl+Q`, `Ctrl+R`, `Ctrl+Tab`, `Ctrl+Shift+Tab`, `Ctrl+Shift+W`, `Ctrl+Shift+N`, `F5`, `F11`, `F12`,
`Alt+F4`. Az újrakötés a UI-konfigurációban tárolódik, és túléli az újratöltést. A szerver 400-zal utasítja el az ütköző, ismeretlen parancsot tartalmazó, hiányos elrendezésű vagy nem objektum törzsű leképezést.

### Suggest

![A Suggest panel három rangsorolt köztes póz jelöltet mutat (linear, pca, manifold) a 12. és 24. képkocka között, mindegyik pontszámmal és Apply gombbal.](../../img/studio-suggest.png)

A `Suggest` (`Alt+8`) a projekt saját kulcspózaiból tanult pózokat javasol: két képkocka köztes pózát, egy részben rögzített póz kiegészítését vagy azt, hogy „mi szokott
következni”. A demóprojekten a 12. és 24. képkocka között a `linear` 1,00-t, a `pca` 1,00-t és a `manifold` 0,99-et kapta, 5 pózból, 2 tengelyen építve (a variancia 99 %-a).
Az `Apply` kulcsokként írja ki a pózt, egy visszavonási lépéssel. Ugyanez a motor az `osatura pose-suggest` és az `/api/suggest`. Ezek klasszikus módszerek (lineáris keverés, PCA, kernel-
sokaság, egy opcionális, általad tanított kis hálózat); néhány pózzal a három egyezik.

### Élő bemeneti és webkamerás arc panelek

* **Live input** (`Alt+4`): az egér a kép fölött, a billentyűk, a gamepad és a Web MIDI egy táblázat alapján tulajdonságokra képeződnek; a `Record` egy előadást kulcsokká bake-el egyetlen visszavonási
  lépésben. A Web MIDI-hez támogató böngésző kell (a Chromium az).
* **Webcam face** (`Alt+5`): a tereptárgypontok ugyanazt a leképezési táblázatot hajtják (forrás: `face`, azonosítók mint `mouth_open`, `eye_r_open`, `head_yaw`). A gombok:
  *Start camera*, *Synthetic face*, *Synthetic face (detector)*, *Calibrate (hold neutral)* és *Face example mapping*. A kamerás útvonalhoz általad megadott tereptárgy-szolgáltató függvény kell
  (`window.osaturaLandmarkProvider`, lásd az oldal forrásában a megjegyzést); a videó soha nem hagyja el az oldalt, és csak a tereptárgypontok mennek a helyi szerverre.
  A két szintetikus mód kapásból működik. A *Synthetic face (detector)* a szerveren rendereli a szintetikus fejet, és a beépített detektorral találja meg újra,
  amelyet **csak szintetikus arcokon tanítottak** (stilizált fejek, nem fényképek); lásd az [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) arcokról szóló szakaszát.

### Vezetett demó

A **Demo** gomb (`demo.panel` parancs) 13 lépéses vezetett demót nyit, amely felépíti az `osatura demo-person` sétáló személyét: díszlet, test, fej és arc, kezenként öt ujjal rendelkező karok, a váz, a lábak sétája, a karok lendítése, törzs és fej, pislogás és mosoly, a mozgó talaj, ellenőrzés és az export. Minden lépés egyetlen mindent-lecserélő művelettel tölti be a projekt adott állapotát a szerkesztőbe, így lejátszhatod, tekerheted, kijelölhetsz csontokat és rétegeket, és olvashatod az idővonalat. Az állapotok a kész projekt kivágásai; a panelben látható parancsokat és kimenetüket a valódi binárissal futtattuk.

* **Semmi sem íródik lemezre.** A Mentés (és a Ctrl+S) le van tiltva, amíg a demó be van töltve. A stúdió egyetlen írási útvonalát, a `POST /api/save` hívást a demó soha nem hívja.
* **A projekted visszajön.** A *Restore my project* visszaállítja az előzőleg megnyitott projektet (egyetlen mindent-lecserélő művelet, egy Undo-lépés). Ha a demó betöltött állapotában bezárod a lapot vagy újratöltöd, a következő oldalbetöltés észreveszi, hogy a szerver még a demót tartja, a Mentést kikapcsolva tartja, és újra felajánlja a *Restore my project* lehetőséget; az eredetit ehhez a böngésző helyi tárolója őrzi.
* A demó nem indul el együttműködési munkamenetben, mert az egész projektet lecseréli.
* Az állapotokat a `node scripts/build-studio-tour.mjs --bin target/release/osatura` paranccsal generálhatod újra, majd építsd újra a CLI-t (az oldal be van ágyazva a binárisba).

## 4. Gyorsbillentyűk (alapértelmezések)

Ez az `osatura ui shortcuts` kimenete friss konfigurációnál: **72 kötés, 84 parancs, 0 ütközés**. A gyorsbillentyűk egy **kontextushoz** tartoznak:
`global` (mindenhol), `canvas` (a képen van a fókusz) és `timeline` (az idővonalon van a fókusz). A szövegmezők megtartják a saját billentyűiket.

### Global

| Gyorsbillentyű | Parancs | Gyorsbillentyű | Parancs |
|---|---|---|---|
| `Space` | lejátszás / szünet (`playback.toggle`) | `Alt+B` | csontok kapcsolása |
| `Home` / `End` | első / utolsó képkocka | `Alt+G` | teljes csoportok kapcsolása |
| `,` / `.` | előző / következő képkocka | `Alt+K` | auto-key kapcsolása |
| `Shift+,` / `Shift+.` | 10 képkockával vissza / előre | `Alt+M` | mozgásnyomok kapcsolása |
| `[` / `]` | tartomány be / ki pontjának beállítása | `Alt+N` | onion skin kapcsolása |
| `Alt+R` | lejátszási tartomány kapcsolása | `Alt+O` | ciklikus lejátszás kapcsolása |
| `Alt+1` | Layers panel kapcsolása | `Alt+P` | kontrollpontok kapcsolása |
| `Alt+2` | Selection panel kapcsolása | `Alt+Q` | scrub proxyk kapcsolása |
| `Alt+3` | Bones panel kapcsolása | `Alt+S` | vászonillesztés kapcsolása |
| `Alt+4` | Live input panel kapcsolása | `Alt+T` | Timeline panel kapcsolása |
| `Alt+5` | Webcam face panel kapcsolása | `Alt+V` | dope sheet / gráf kapcsolása |
| `Alt+6` | JSON panel kapcsolása | `Alt+Shift+S` | idővonal-illesztés kapcsolása |
| `Alt+7` | Overlays panel | `Alt+=` / `Alt+-` | idővonal nagyítása / kicsinyítése |
| `Alt+8` | Suggest panel | `Ctrl+S` | mentés |
| `Ctrl+K` / `Ctrl+Shift+P` | parancspaletta | `Ctrl+,` | gyorsbillentyűk |
| `Ctrl+Z` | visszavonás | `Ctrl+Y` / `Ctrl+Shift+Z` | újra |

### Canvas

| Gyorsbillentyű | Parancs |
|---|---|
| `Arrow keys` | a kijelölés léptetése 1 egységgel (`Shift`: 10 egység) |
| `Delete` / `Backspace` | a kijelölés törlése |
| `Escape` | a kijelölés törlése |
| `Ctrl+A` | az összes kijelölése |
| `Alt+L` | a kijelölt rétegek zárolása / feloldása |
| `I` | kulcs hozzáadása a lejátszófejnél (`anim.add_key`) |

### Timeline

| Gyorsbillentyű | Parancs |
|---|---|
| `ArrowLeft` / `ArrowRight` | előző / következő képkocka (`Shift`: 10) |
| `-` / `=` | kicsinyítés / nagyítás |
| `F` | nagyítás a teljes tartományra |
| `Delete` / `Backspace` | a kijelölt kulcsok törlése |
| `Escape` | a kulcsok kijelölésének megszüntetése |
| `Ctrl+A` | az összes kulcs kijelölése |
| `Ctrl+C` / `Ctrl+V` / `Ctrl+D` | kulcsok másolása / beillesztése / duplikálása |
| `Alt+ArrowLeft` / `Alt+ArrowRight` | a kijelölt kulcsok mozgatása egy képkockával korábbra / későbbre |
| `Alt+ArrowUp` / `Alt+ArrowDown` | a kijelölt kulcsok (értékének) növelése / csökkentése |
| `I` | kulcs hozzáadása a lejátszófejnél |

Módosítók a vásznon: a **Shift** korlátoz (tengelyzár mozgatáskor, 15 fokos lépések forgatáskor, arányos skálázáskor, a csont csúcsa ekkor csak forog,
és a Shift-kattintás kapcsolja a kijelölést, míg a doboz hozzáad); az **Alt** a középpontból skáláz (és az üres helyen Alt-húzás lasszót rajzol); a **Ctrl** (macOS-en Cmd)
kikapcsolja az illesztést.

## 5. Kijelölés és gizmók

A kattintás a mutató alatti legfelső dolgot választja: egy réteget, egy csontot (a kapszuláját, ami kicsit szélesebb a megrajzolt vonalnál) vagy, bekapcsolt `points` mellett, egy kontrollpontot. A zárolt és rejtett
rétegeket átugorja, így a kattintás áthalad arra, ami alattuk van. A doboz a `box` választó módja szerint jelöl: `touches` (bármi, amit a doboz érint), `contains` (csak ami teljesen belül van) vagy `bounds touch` (a befoglaló dobozok átfedik egymást); az Alt-húzás helyette lasszót rajzol.

Egy réteg esetén a transzformációs doboz a réteg saját tengelyeit követi: 8 fogantyú skáláz (az élek egy tengelyt, a sarkok kettőt; az átellenes élen túli húzás negatív skálával tükröz),
a kerek gomb a középpont körül forgat, a belsejében húzás mozgat. Csontnál a fejkör mozgatja a csontot, a csúcs forgatja (és nyújtja, hacsak nincs nyomva a Shift).
A kontrollpontoknál (horgonyok, Bézier-fogantyúk, deformerpontok, mesh-csúcsok) ugyanaz a gizmo mozgatja a kijelölt pontokat. Az illesztés lefedi a rácsot (alapból ki), más rétegek éleit
és középpontjait, valamint a vászon széleit, 6 vászonegységen belül. **Egy húzás egy visszavonási lépés** (egy batch művelet).

## 6. Auto-key: pontosan mi kap kulcsot

Hogy egy vászonszerkesztés *statikus* értéket módosít vagy *kulcsokat* hoz létre, azt így döntik el. A HTTP API-val ellenőriztem a demóprojekten (minden szerkesztés `mode` értékét jelzi):

| Helyzet | Eredmény |
|---|---|
| Rétegszerkesztés, `auto-key` ki, a tulajdonságnak nincs kulcsolt csatornája | a réteg transzformációjának **statikus** módosítása (csatorna nem jön létre) |
| Rétegszerkesztés, `auto-key` be | a szerkesztett tulajdonságok **kulcsot** kapnak az aktuális képkockánál (új `layer_x` és `layer_y` csatornák jöttek létre: 14-ről 16 csatornára) |
| Olyan réteg- vagy ponttulajdonság, amelynek **már van kulcsolt csatornája**, `auto-key` ki | továbbra is **kulcsot** kap az aktuális képkockánál (a 9. képkockánál végzett szerkesztés második kulcsot adott az 5. képkockától induló csatornákhoz) |
| **Csont**szerkesztés | mindig **kulcsot** kap, az `auto-key` beállításától függetlenül (csont-eltolás csatornák jöttek létre: 16-ról 18-ra) |

Így egy animált tulajdonságot sosem lehet csendben statikus értékkel felülírni, és egy csontpóz sosem „vész el” a kapcsoló elfelejtése miatt. Kapcsold be az `auto-key` funkciót (`Alt+K`),
ha egy még statikus tulajdonságot akarsz animálni. Az `I` billentyű (`anim.add_key`, „Add key at playhead”) kulcsot ad hozzá az aktuális képkockánál.

## 7. Visszavonás és újra

A visszavonás és az újra (`Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`) teljes műveleteken dolgozik: egy gizmo-húzás, egy léptetés, egy kulcsszerkesztés és egy Apply mind egy lépés. Közös munkamenet nélkül
lineárisak. **Közös munkamenetben a visszavonás felhasználónkénti**: csak a saját utolsó szerkesztésedet vonja vissza, mások szerkesztéseit a helyükön hagyja (8. szakasz). A visszavonási előzmények nem mentődnek a
projekttel, és a szerver leállásakor elvesznek.

## 8. Közös szerkesztés (együttműködés)

### Végigvezetés: két ember, egy projekt

![Két böngészőfelhasználó egy munkamenetben: az eszköztáron két résztvevő-jelvény látható, Ana (te) a 14. képkockánál és Bo a 0.-nál, Bo kijelölődoboza és kurzora a láb réteg körül van megrajzolva a vásznon.](../../img/studio-collab.png)

1. Indítsd el a stúdiót. A más gépeken lévő embereknek tokent kell megadnod, és hálózati címre kell kötnöd:
   `osatura studio demo.json --bind 0.0.0.0 --port 8080 --collab-token $(openssl rand -hex 16) --collab-log session.log`,
   lehetőleg TLS-proxy mögött (2. szakasz). Egy gépen nincs szükség tokenre.
2. Az első böngészőben írd be a neved a **Collab** mellé („Ana”), és nyomd meg a **Collab** gombot. A gomb **Leave** lesz, és megjelenik egy jelvény neked. A második böngészőben
   írd be, hogy „Bo”, és csatlakozz. Minden jelvény a résztvevő aktuális képkockáját mutatja (`Ana (you) @14`, `Bo @0`).
3. Jelölj ki dolgokat. A másik személy látja a kijelölődobozodat és a kurzorodat a te színedben, a neveddel (Bo kijelölte a `leg` réteget, és mozgatta a mutatót; Ana egy lila dobozt lát
   „Bo” felirattal, és egy pontot a `leg` mellett a rétegek listájában).
4. Szerkesszetek egyszerre. A tesztben: Ana 10 egységgel megléptette a kar réteget (`Shift+ArrowRight`); Bo látta, hogy a `[10,0]` megjelenik az `/api/project`-ben. Bo ezután elrejtette a `sun` réteget, amit Ana
   azonnal látott.
5. Ana megnyomta a `Ctrl+Z`-t. **Csak Ana léptetése lett visszavonva** (a kar visszatért a `[0,0]`-ra); Bo elrejtett napja rejtve maradt.
6. A régebbi nézetből dolgozók szerkesztései összefésülődnek, nem utasítódnak el: lefuttattam a projekt saját szimulációját (`osatura collab-sim`, 6 replika, 600 véletlen párhuzamos
   szerkesztés, 12 másik kézbesítési sorrend), és minden replika azonos lett, az összefésült projekt pedig lefordult.
7. Nyomd meg a **Save** gombot (vagy `Ctrl+S`) bármelyik böngészőben, hogy a közös állapotot kiírd a projektfájlba; addig semmi sem íródik a fájlba.

Csak Chromiumot teszteltünk, két oldallal egy gépen. Nem teszteltem különböző gépeket, lassú kapcsolatokat vagy kettőnél több valódi böngészőt.

### Tartósság: `--collab-log`

```
$ osatura studio demo.json --collab-log session.log
...
$ # (an edit is made in a session, the server is stopped and started again)
collaboration log session.log: resumed at index 1 (1 envelopes after the checkpoint)
```

A napló soronkénti fájl (mód: `0600`, csak a tulajdonosé), amely **a teljes projekt** hash-láncolt ellenőrzőpontját (a demónál nagyjából 178 KB) tartalmazza, plusz egy sort minden elfogadott szerkesztéshez. A hozzáfűzések
kérésenként `fsync`-elnek, hacsak nincs `--collab-nosync`. Újraindítás után a munkamenet állapota folytatódik (megváltoztattam a projekt nevét egy munkamenetben; az újraindítás után az `/api/project` még mindig az
új nevet adta), miközben a lemezen lévő fájl változatlan maradt (`OSATURA demo`) a Save-ig. **A résztvevők és a visszavonási előzmények nem tartósak**: az oldalak új azonosítókkal csatlakoznak újra. A sérült naplót vagy egy másik projekt
naplóját a szerver hibával leállva utasítja el, nem írja felül. A hozzáfűzés közbeni összeomlás legfeljebb egy megszakadt utolsó sort veszít.

**A napló a teljes projektedet titkosítatlanul tartalmazza; védd úgy, mint magát a projektet.**

## 9. HTTP API referencia

A stúdió oldala ezeket az útvonalakat használja, és te is használhatod őket (szkriptekhez, tesztekhez). Minden törzs és válasz JSON, ha nincs másképp jelezve; a hibák alakja `{"ok": false, "errors": ["..."]}`, 400-as állapotkóddal (vagy 401, 403,
404, 415, 421, 429, 503 a 2. szakaszban leírtak szerint). A sikeres válaszok ott, ahol jelezve van, `"ok": true` értéket hordoznak. Az alábbi `state` a `GET /api/state` által visszaadott dokumentumot jelenti:
`{bones:[{name,rest_angle}], channels:[...], fps, frames, width, height, can_undo, can_redo, rev, ok}`; a `rev` minden módosításnál nő. Minden útvonalra érvényesek a 2. szakasz ellenőrzései.

### Olvasás

| Útvonal | Paraméterek | Válasz |
|---|---|---|
| `GET /` | | a stúdió oldala (HTML) |
| `GET /source` | | az AGPL-forrásajánlat oldala; token nélkül is megnyitható |
| `GET /api/state` | | `state` |
| `GET /api/project` | | a teljes projekt-JSON |
| `GET /api/layers` | | `{ok, layers:[{path, depth, kind, visible, locked, locked_eff, reference, ptr}]}` (a `locked_eff` a fölötte lévő zárolt csoportokat is számítja) |
| `GET /frame/N.svg` | a `bones=1` kirajzolja a csontvázat | egy képkocka SVG-ként (N lehet törtszám is); fejlécek: `X-Cache`, `X-Render-Ms` |
| `GET /api/frame` | `frame`, `proxy` 0-4 (0 = teljes minőség, 1-4 = görgetési proxyk), `kind` = `vector` (alapértelmezett) \| `raster` \| `png`, `bones=1`, `lod=1&screen=WxH&lodpx=P` (rajzolás csökkentett részletességű másolatból, legfeljebb P pixelen belül, 0-16) | a kép; fejlécek: `X-Cache` (hit/miss), `X-Render-Ms`, `X-Lod-Level`, `X-Lod-Bound-Px`, `X-Proxy-Level`. Tartományon kívüli képkocka: `{"errors":["frame 999 out of range"]}`; hibás kind: `bad kind 'x' (vector, raster, png)` |
| `GET /api/cache` | | `{ok, revision, entries, bytes, hits, misses, invalidations, lod:{builds, engines, invalidations, last_build_ms, levels, unavailable}}` |
| `GET /api/overlays` | `frame`, `sel` (vesszővel elválasztott réteg-nevek) | `{ok, frames, settings, ...}`, valamint szellemképek és a képkocka SVG-je, ha meg van adva egy |
| `GET /api/sounds` | | `{ok, total, sounds:[{name, start, gain, duration, peaks, error?}]}` hullámforma-csúcsok a vágás, a ki- és beúsztatás és az erősítés után |
| `GET /api/vcs/log` | `limit` (legfeljebb 1000), `rev` | `{ok, branch, commits:[{hash, author, time, message, parents}]}`; tárhely nélkül `{"ok":false,"errors":["no .osatura-vcs repository found ..."]}` (lásd `osatura vcs`) |
| `GET /api/vcs/diff` | `a` (alapértelmezés `HEAD`), `b` | `{ok, changes}` |
| `GET /api/timeline` | | `{ok, fps, frames, groups, rows, tracks}` a dope sheet modellje |
| `GET /api/graph` | `ch` (vesszővel elválasztott csatornaindexek), `f0`, `f1`, `n` (minták, 2-2000, alapértelmezés 200) | `{ok, channels, f0, f1, range, skipped}` görbeminták és érintőfogantyúk |
| `GET /api/live` | | `{ok, mapping:{params,triggers}, recorded, recording}` |
| `GET /api/ui` | | `{ok, layout, presets, preset_names, default_layout, shortcuts, commands, conflicts, reserved, overlays, recents, panels, persisted, path, warning, ...}` |
| `GET /api/ui/search` | `q`, `limit` (1-200, alapértelmezés 50) | `{hits:[{id, title, category, description, bindings, score, positions}]}` a parancspaletta rangsora |
| `GET /export.gif`, `/export.html`, `/export.html.nosound`, `/export.lottie.json` | | a fájl letöltésként (`animation.gif`, `player.html`, `animation.json`) |

### Szerkesztés

| Útvonal | Törzs | Válasz |
|---|---|---|
| `POST /api/op` | `{"op":"set","path":"/fps","value":30}` — az `op` értéke `set`, `insert` (tömbútvonal, a `.../-` hozzáfűz), `remove` vagy `batch` (`{"op":"batch","ops":[...]}`, egyetlen visszavonási lépés). Az útvonalak JSON Pointerek a projektbe. | `state`. Az érvénytelen eredményt elutasítja: `{"errors":["invalid project: invalid value: integer `-3`, expected u32"]}`, és semmi sem változik |
| `POST /api/undo`, `POST /api/redo` | nincs | `state` (megosztott munkamenetben felhasználónként) |
| `POST /api/key` | `{"target","prop","frame","value","ease"?}` (az `ease` például `{"type":"smooth"}`; alapértelmezés lineáris) | `state` |
| `POST /api/delete_key` | `{"target","prop","frame"}` | `state` |
| `POST /api/save` | nincs | `{"ok":true,"saved":"<path>"}`; atomikusan ír. Csak host végezheti, ha a stúdió `--collab-host` kapcsolóval fut (különben **403**) |
| `POST /api/suggest` | `{"kind":"between"\|"complete"\|"next","a","b","t","frame","constraints","free","top","snap","prior","apply":RANK, "library"?, "model"?}` | `{ok, suggestions}`; `apply` esetén a szerkesztés `state`-je is. A `kind` nélküli kérés eredménye: `bad request: missing field `kind`` |

### Kijelölés és gizmók (állapotmentes: a kijelölést az oldal tartja)

A **target** (cél) lehet `{"kind":"layer","path":"arm"}`, `{"kind":"bone","name":"hand"}` vagy `{"kind":"point","layer":"...","point":...}`.

| Útvonal | Törzs | Válasz |
|---|---|---|
| `POST /api/pick` | `{"frame","x","y","options"?}` (`options`: `tolerance`, `bone_radius`, `point_radius`, `layers`, `bones`, `point_layers`, `locked`, `locked_bones`) | `{ok, hit, hits:[{distance, target}]}` a legfelső az első |
| `POST /api/marquee` | `{"frame","rect":[x0,y0,x1,y1]` vagy `"lasso":[[x,y],...]` (3-5000 pont)`,"mode":"bbox"\|"contain"\|"intersect","options"?}` | `{ok, targets}` |
| `POST /api/select` | `{"frame","selection":[...],"targets":[...],"mode":"replace"\|"add"\|"toggle"\|"subtract"}` | `{ok, selection, bounds, gizmo:{corners, handles}, points}` |
| `POST /api/gizmo` | `{"frame","selection","handle","start":[x,y],"delta":[dx,dy],"modifiers":{"constrain","center","no_snap"},"snap":{"grid","objects","canvas","tolerance"},"auto_key","locked","pivot","commit":bool}` — a `handle` értéke `{"type":"move"}`, `rotate`, `{"type":"scale","at":"nw"}`, `bone_head` vagy `bone_tip` | `commit` nélkül: `{ok, drag:{changed, edits, guides, matrix, summary, box_after, outline}}` (előnézet); `commit:true` esetén: `state` plusz `drag` és `selection`, egyetlen visszavonási lépés |
| `POST /api/nudge` | `{"frame","selection","delta":[dx,dy],"auto_key","locked"}` | `state` + `drag` (a szerkesztések `mode` értéke `static` vagy `key`) + `selection` |
| `POST /api/delete` | `{"frame","selection"}` | `state` + `selection`; üres kijelölésnél `{"errors":["nothing selected"]}` |

### Idővonal

| Útvonal | Törzs | Válasz |
|---|---|---|
| `POST /api/timeline/select` | `{"selection":[{ch,frame}...],"mode":"replace"\|"add"\|"toggle"\|"subtract","all":true` vagy `"channel":N` vagy `"group":N` vagy `"box":{f0,f1,row0,row1}`, `"include_locked"}` | `{ok, selection:[{ch,frame}...]}` |
| `POST /api/timeline/op` | `{"op":{...},"preview":bool}` — az `op` a következők egyike: `move_keys`, `scale_keys`, `copy`, `paste`, `duplicate`, `delete`, `set_ease`, `set_value`, `add_key`, `ripple`, `offset`, `extrapolate`, `make_loop`, `handle`, `smooth`, `flags` (ismeretlen név esetén ezeket sorolja fel) | `preview:true` esetén: `{ok, preview:true, plan}` bármiféle módosítás nélkül; egyébként `state` + `plan:{changed, notes, op, clip}` |

Például a `{"preview":true,"op":{"op":"delete","keys":[{"ch":0,"frame":12}]}}` visszaadja a tervet (`changed: 1`), és békén hagyja a projektet.

### Overlayek, élő bemenet, arc, felület

| Útvonal | Törzs | Válasz |
|---|---|---|
| `POST /api/overlays` | `{"settings":{"onion":{"enabled":true,"before":3}},"frame","selection":[names],"reset":bool}` | `{ok, frames, ghosts, settings, svg}`; a beállítások a felületi konfigurációba kerülnek mentésre |
| `POST /api/trails` | `{"targets":[{"kind":"bone","name":"hand"}],"from","to"` vagy `"frame","before","after"`, `"smooth","svg":bool,"hit":{x,y,radius}}` | `{ok, from, to, trails:[{curve,frames,keys,...}], svg?, hit?, path_hit?}` |
| `POST /api/live/mapping` | `{"mapping":{params,triggers},"library"?}` (ugyanaz a formátum, mint a `live-map`-nél) | élő állapot |
| `POST /api/live/push` | `{"events":[{source,id,value,time}],"midi":[raw...]}` | élő állapot az események hatásával |
| `POST /api/live/record` | `{"on":true}` elindítja; `{"on":false,"tol":0.01}` leállítja, és **a produkciót beleégeti a projektbe** egyetlen visszavonási lépésként | állapot / `state` + `live` összegzés |
| `POST /api/face/config` | `{"config"?,"calibration"?}` | `{ok}` |
| `POST /api/face/calibrate` | `{"on":true}` / `{"on":false}` | `{ok, frames, calibration}` |
| `POST /api/face/push` | `{"frames":[...landmarks...],"layout"?,"size"?}` (egyszerre legfeljebb 10000 képkocka) | élő állapot |
| `POST /api/face/detect` | `{"image":"<base64 PNG/JPEG or data: URI>","track":bool,"push":bool,"t"}` | `{ok, found, layout:"osatura23", size, points, box, note:"detector trained on synthetic faces only"}` |
| `POST /api/face/synth` | `{"t","size":32-512,"seed"}` | `{ok, image:"data:image/png;base64,...", layout, size, points}` a szintetikus fej egy képkockája a pontos tájékozódási pontjaival |
| `POST /api/ui` | bármelyik a következők közül: `{"layout":{...}\|null,"shortcuts":{...}\|null,"recents":[...],"overlays":{...},"reset":"layout"\|"shortcuts"\|"overlays"\|"all"}` | a `GET /api/ui` dokumentuma; az ismeretlen mezőket (`unknown field 'bogus'`), az ütközéseket és a hiányos elrendezéseket elutasítja, és semmi sem változik |
| `POST /api/login` | `{"token":"..."}` | `{ok:true}` és `Set-Cookie: osatura_token=...; Path=/; HttpOnly; SameSite=Strict`; hibás token: 401 (az 5. sikertelen kísérlettől 429); beállított token nélkül `{"ok":true,"token_required":false}` |

### Együttműködés

A résztvevőként indított hívások elküldik az `X-Collab-Replica: <id>` és az `X-Collab-Secret: <secret>` fejlécet (mindkettőt a join adja vissza). Az a replikaazonosító, amelyhez rossz vagy hiányzó titok tartozik, **403**-at kap.

| Útvonal | Törzs / lekérdezés | Válasz |
|---|---|---|
| `POST /api/collab/join` | `{"name":"Ana","mode":"thin"\|"replica"}` | `{ok, replica, secret, colour, name, index, mode, clock, users, presence, doc}` (`snapshot` replica módban). Az első csatlakozás a szerkesztő dokumentumából indítja a munkamenetet |
| `POST /api/collab/leave` | `{"replica"}` | `{ok, left}` |
| `GET /api/collab/state` | `replica`, `doc=1`, `snapshot=1` | `state` + `collab_active` (+ users, presence) |
| `GET /api/collab/log` | `since=N` | `{ok, index, entries:[{index, by, envelope}]}` |
| `POST /api/collab/ops` | `{"replica","ops":[...]` vagy `"op":{...}`, `"base"?}` vagy `{"replica","envelopes":[...]}` | `state` + `applied`, `index`. Ismeretlen résztvevő: `unknown participant N: join first`; üres lista: `no ops`; az érvénytelen projektszerkesztéseket elutasítja |
| `POST /api/collab/undo`, `.../redo` | `{"replica"}` | `state` + `collab_can_undo`, `collab_can_redo` (csak a saját szerkesztéseidre) |
| `POST /api/collab/presence` | `{"replica","n","frame","cursor":[x,y],"selection":[names up to 64]}` | `{ok}` |
| `POST /api/collab/compact` | `{}` | tömörítési összegzés (`ok`, `logged`). Csak host végezheti, ha a stúdió `--collab-host` kapcsolóval fut (különben **403**) |
| `POST /api/collab/revoke` | `{"names":["ana","bob"]}` (1–64 név) | `{ok, revoked, revoked_total}`: ezeknek a neveknek a tokenjeit a továbbiakban elutasítja, és eltávolítja azokat a résztvevőket, akik ezek alatt csatlakoztak. Névhez kötött tokenek (`--collab-secret-file`) és host token (`--collab-host`) kell hozzá, vagy a megosztott `--collab-token`; különben **403**. A már megnyitott eseményfolyam akkor ér véget, amikor a kliense lecsatlakozik |
| `GET /api/collab/events` | `since=N` vagy `Last-Event-ID` | egy **server-sent-events** adatfolyam: visszajátszás a `since` ponttól, majd a felhasználók, a jelenlét és minden későbbi esemény; `resync`, ha a kliens túl sokat lemaradt; legfeljebb 64 adatfolyam (`503 too many event streams`) |

Aktív munkamenetben a szokásos szerkesztő útvonalak (`/api/op`, `/api/undo`, `/api/redo`, valamint a stúdió saját szerkesztései nudge, key vagy gizmo útján) a többi résztvevő számára is rögzítésre kerülnek.

## 10. Hibaelhárítás

| Tünet | Ok és javítás |
|---|---|
| A böngésző 421-et vagy „misdirected request” üzenetet mutat | A stúdiót olyan néven nyitottad meg, amelyet nem ismer (DNS-név, LAN-cím). Add hozzá az `--allow-host name` kapcsolóval, vagy nyisd meg a `http://127.0.0.1:PORT/` címet. |
| Minden kérésre 403 egy proxy mögött | A proxy módosítja az `Origin`/`Host` fejlécet. Add meg a nyilvános origint: `--public-origin https://studio.example.org`. |
| 401 | Be van állítva a `--collab-token`: nyisd meg az oldalt, és add meg egyszer a tokent, vagy küldd el az `X-Collab-Token` fejlécet. Az URL-ben megadott tokent nem fogadja el. |
| 429 | Öt hibás token egymás után a címedről; várj legfeljebb 30 másodpercet. Proxy mögött minden felhasználó ugyanazt a címet használja. |
| `refusing to listen on 0.0.0.0:...` | Nem loopback címre kötés token nélkül. Add hozzá a `--collab-token` kapcsolót (legalább 16 karakter). |
| `cannot listen on 127.0.0.1:8080: Address already in use` | Egy másik folyamat használja a portot; válassz másikat a `--port` kapcsolóval. Általában egy korábbi futásból megmaradt stúdió az oka. |
| Az oldal betöltődik, de a kép üres | Nézd meg a terminált; az a projekt, amely nem felel meg a `check` ellenőrzésnek, indításkor el lesz utasítva. A javítás után töltsd újra. |
| Böngészőkonzol-üzenet: `Failed to load resource ... 404` | A régebbi buildekben nem volt `/favicon.ico`; most már ki van szolgálva. Ha még mindig 404 jön, annak más az oka. |
| Az Arc panel szövege | A panel most azt írja, hogy a szintetikus adatokon tanított detektor benne van, fényképeken és webkamerákon nincs validálva, valódi webkamerához pedig saját tájékozódásipont-szolgáltató kell. |
| Az elrendezés elromlott, vagy hiányoznak panelek | `Layout` menü, elrendezés visszaállítása, vagy töröld az induláskor kiírt `ui.json` fájlt. Az a konfigurációs fájl, amely nem olvasható be teljesen, az alapértékek használata előtt átmásolódik a `ui.json.bak` fájlba (majd `.bak.1`, ...), és a `GET /api/ui` válaszában egy `warning` mező szól erről. |
| A szerkesztéseim újraindítás után eltűntek | Semmi sem íródik ki a **Save** előtt. A `--collab-log` kapcsolóval a *munkamenet* túléli az újraindítást, de a lemezen lévő fájlt továbbra is csak a Save írja. |
| A `Ctrl+W`, `Ctrl+T` és hasonlók nem köthetők le | A böngészők fenntartják őket (lista a 3. szakaszban). |
| A Ctrl+Z nem vonja vissza valaki más módosítását egy munkamenetben | Szándékosan így van: a visszavonás felhasználónkénti. |
| Az exportálás `ffmpeg`-et kér | Csak a HTML-beli hanghoz (mp3) és a parancssori mp4/webm-hez; a GIF, a Lottie és a hang nélküli HTML nem igényel semmit. |

## 11. Hozzáférhetőség és böngészőtámogatás

**Böngészők.** A felületet csak **Chromiumban** teszteltem (a Playwright mellékelt buildjében, fej nélkül futtatva, Linuxon). A Firefoxot és a Safarit nem próbáltam ki, működhetnek, de lehet, hogy nem.
A Web MIDI-hez olyan böngésző kell, amely megvalósítja (a Chromium igen); a kamerához `getUserMedia` és biztonságos környezet (HTTPS vagy `localhost`) szükséges. Az oldal server-sent eventset használ a
megosztott munkamenethez. A húzáshoz pointer eseményeket használ; az érintőképernyőket nem teszteltem.

**Hozzáférhetőség, őszintén.** A stúdiót nem auditáltam képernyőolvasóval vagy csak billentyűzettel végzett munkamenettel. Ami van: `lang` attribútum és viewport meta tag, címkézett jelölőnégyzetek, egy
`role="status"` toast, néhány ARIA-attribútum, **a gyorsbillentyűk teljes készlete és egy parancspaletta**, amelyek minden parancsot elérnek, átköthető gyorsbillentyűk, valamint az `osatura` parancssor, amely
mindent megtesz, amit a szerkesztő, mutató nélkül. Ami hiányzik: a vászon egy kép, és a vásznon végzett szerkesztés nagy része mutatóval történik (a nyílbillentyűs léptetés létezik), nincs világos téma, és nincs
`prefers-reduced-motion` vagy `prefers-color-scheme` kezelés, a fókuszkeretek a böngésző alapértelmezettjei, és nem ellenőriztem őket, a színkontrasztot pedig nem mértem meg. Ha segítő technológiára vagy utalva,
inkább a CLI-t és a JSON panelt használd, és jelezd a projektnek, ha valami nem működik.

## 12. Ismert esetlenségek

* Az első összefoglaló óta javítva: a Webcam Arc panel szövege, a `/favicon.ico` 404, a `--collab-nosync` a `--collab-log` nélkül, valamint a hiányzó `Secure` süti-jelző https `--public-origin` mögött.
* Az `osatura render p.json 99 o.png` egy 24 képkockás projekten mostantól szintén hiba (a legutolsó képkocka rendereléséhez add hozzá a `--clamp` kapcsolót); a stúdió `/api/frame` útvonala elutasítja a tartományon kívüli képkockát (`frame 999 out of range`), és mostantól csak a `0..frames-1` képkockákat fogadja el.
* Az elrendezés és a gyorsbillentyűk módosításai felhasználónként, a felületi konfigurációban tárolódnak, nem a projektben, ezért nem utaznak a fájllal.
