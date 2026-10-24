<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Biztonság

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/README.md) · [Română](https://github.com/chiuta/osatura/blob/main/README.ro.md) · [Français](https://github.com/chiuta/osatura/blob/main/README.fr.md) · [Italiano](https://github.com/chiuta/osatura/blob/main/README.it.md) · [Español](https://github.com/chiuta/osatura/blob/main/README.es.md) · [Português](https://github.com/chiuta/osatura/blob/main/README.pt.md) · [Deutsch](https://github.com/chiuta/osatura/blob/main/README.de.md) · [Magyar](https://github.com/chiuta/osatura/blob/main/README.hu.md)

*Fenyegetési modell, beépített védelmek, ismert hiányosságok, a probléma bejelentésének módja és a stúdió biztonságos üzemeltetése. A dokumentáció CC-BY-4.0 licencű. A forrásból (`osatura-core/src/{api,assets,fsutil,expr,script}.rs`, `osatura-cli/src/{http,studio,collab_srv}.rs`), a `ROADMAP.md` 9. szakaszából (1-4. review-hullám) és az alább megnevezett tesztekből származik. **Az OSATURA-n nem végeztek független biztonsági auditot.** A review-hullámokat maga a projekt végezte; ezt az oldalt a szándék és a tesztek által lefedett dolgok leírásaként kezeld, nem garanciaként.*

Tartalom: [1 Fenyegetési modell](#1-fenyegetési-modell) - [2 A stúdió HTTP-szervere](#2-a-stúdió-http-szervere) - [3 Hitelesítés és együttműködés](#3-hitelesítés-és-együttműködés) - [4 Assetfeloldás és bemeneti korlátok](#4-assetfeloldás-és-bemeneti-korlátok) - [5 Kifejezés- és szkript-sandbox](#5-kifejezés--és-szkript-sandbox) - [6 Fájlok, szimbolikus linkek és ideiglenes adatok](#6-fájlok-szimbolikus-linkek-és-ideiglenes-adatok) - [7 Külső programok (ffmpeg)](#7-külső-programok-ffmpeg) - [8 Könyvtárak (C ABI, WASM, homlokzat)](#8-könyvtárak-c-abi-wasm-homlokzat) - [9 Ismert hiányosságok](#9-ismert-hiányosságok) - [10 Sebezhetőség bejelentése](#10-sebezhetőség-bejelentése) - [11 Üzemeltetői keményítési ellenőrzőlista](#11-üzemeltetői-keményítési-ellenőrzőlista) - [12 Reverse proxy TLS-sel: nginx és Caddy](#12-reverse-proxy-tls-sel-nginx-és-caddy)

## 1. Fenyegetési modell

Amit az OSATURA feldolgoz: **projektfájlok** (JSON), képek, hang, Lottie / OCA / glTF-szerű importfájlok, leképezési fájlok, és a stúdióban a böngészőből érkező HTTP-kérések. A projektfájlok az alábbi modellben *nem megbízható adatok*: másoktól kapott projekt lehet ártalmas, és megnyitása nem futtathat kódot, nem olvashat a mappáján kívüli fájlokat, és nem meríthet ki egy gépet.

| Szereplő | Mit tehet | Feltételezett cél | Fő védelmek |
|---|---|---|---|
| Ártalmas projekt- / importfájl szerzője | fájlt adhat neked | fájljaid olvasása, az eszköz összeomlasztása vagy lefagyasztása, kód futtatása, a lemez megtöltése | szigorú feldolgozás, méret- és mélységkorlátok, asset-útvonalak elutasítása, bemenetre nincs pánik, kifejezés- és szkript-sandbox, atomi kimenet |
| Egy weboldal, amelyet akkor látogatsz, amikor a stúdió localhoston fut | a böngésződ kéréseket küldet a `127.0.0.1` címre (CSRF, DNS rebinding) | a projekted szerkesztése, olvasása, fájlok írása | Host allow-list, Origin / `Sec-Fetch-Site` ellenőrzések, JSON tartalomtípus, token `SameSite=Strict` sütivel, CSP |
| Másik felhasználó ugyanazon a gépen | csatlakozhat a porthoz, olvashatja a `/tmp`-t | munkamenet vagy ideiglenes fájlok olvasása vagy eltérítése | alapértelmezés szerint a `127.0.0.1`-re köt, privát ideiglenes könyvtárak (0700), 0600-as naplófájlok, máshová kötéshez token kell |
| Valaki a hálózatodon, ha a stúdió a localhoston túlra van kötve | elérheti a portot | a token használata vagy kitalálása, a kapcsolatok kimerítése | kötelező token (16+ karakter), állandó idejű összehasonlítás, back-off, kapcsolat- és méretkorlátok; **nincs TLS** |
| Közreműködő a tokennel | szerkesztheti a megosztott munkamenetet | a projekt megkárosítása | visszavonási előzmény, a hash-láncolt napló; a közreműködők megbízhatók a projekttel, nincsenek fiókok vagy szerepkörök |
| A könyvtárak ártalmas hívója | rossz argumentumokat adhat át a homlokzatnak, a C vagy WASM ABI-nak | memóriasérülés, a határon átterjedő pánik | argumentumvalidáció, foglalás előtti méretellenőrzés, elkapott pánikok (ahol a cél unwindol) |

Hatókörön kívül: védelem egy rosszindulatú helyi rendszergazda, egy kompromittált build-eszközlánc vagy függőség ellen (a szállított tartalomról lásd [THIRD_PARTY.md](THIRD_PARTY.md); a `cargo deny` licenceket, tiltásokat és forrásokat felügyel, sebezhetőségeket nem), oldalcsatornák, valamint maga az ffmpeg biztonsága.

## 2. A stúdió HTTP-szervere

Az `osatura studio` a szerkesztőt saját, kis HTTP/1.1-megvalósításból szolgálja ki (`osatura-cli/src/http.rs`, csak szabványos könyvtár, harmadik féltől származó webes keretrendszer nélkül). A viselkedés, a rögzítő tesztekkel együtt (`osatura-cli` unit-tesztek, `tests/robust.rs`, `tests/wave4.rs`, `tests/source_offer.rs` és a Chromium smoke-tesztek):

**Korlátok**

| Korlát | Érték | Túllépés esetén |
|---|---|---|
| A kérésfej fogadásának ideje | 10 s | 408 |
| A törzs fogadásának ideje | 20 s | 408 |
| Válasz írásának ideje | 10 s | a kapcsolat megszakad |
| A kérésfej mérete | 16 KB | 431 |
| Fejlécek száma | 100 | 431 |
| Törzsméret | 32 MB | 413 |
| Párhuzamos kapcsolatok | 128 | 503 |
| Chunked kéréstörzsek | nem támogatottak | 411 |
| Ismétlődő `Host` fejléc | elutasítva | 400 |
| Vezérlőkarakter (CR, NUL, DEL…) a fejléc értékében | elutasítva (a tabulátor megengedett) | 400 |

Minden válasz tartalmazza a `Connection: close` (nincs kimeríthető keep-alive állapot), a `Cache-Control: no-store`, az `X-Content-Type-Options: nosniff`, az `X-Frame-Options: DENY`, a `Referrer-Policy: no-referrer` és egy `Content-Security-Policy` fejlécet.

**Kérési szabályzat** (`Policy::check`, minden kérésre alkalmazva):

1. **Host allow-list.** A `Host` fejlécnek a kötött címnek kell lennie (`127.0.0.1`, `localhost`, a tényleges port), vagy egy `--allow-host` opcióval megadott névnek. Ellenkező esetben 421 Misdirected Request. Ez állítja meg a DNS rebindinget: egy `evil.example` oldal, amely a `127.0.0.1` címre oldódik fel, `Host: evil.example` fejlécet küld.
2. **Origin és `Sec-Fetch-Site`.** Az idegen oldalról származó `Origin` fejlécet, vagy `Sec-Fetch-Site: cross-site` / `same-site` fejlécet hordozó kéréseket 403-mal utasítja el. Az `Origin: https://<allowed host>` elfogadott az `--allow-host` listában szereplő hosztokra; ha egy reverse proxy nyilvános URL-en szolgálja ki az oldalt, add meg a `--public-origin URL` opciót, hogy a proxy origin-je elfogadott legyen. A cross-origin továbbra is elutasított.
3. **Tartalomtípus.** A `POST` `Content-Type: application/json` fejlécet igényel (különben 415), így egy egyszerű HTML-űrlap nem küldhet az API-nak.
4. **Token** (lásd a 3. szakaszt), ha be van állítva; kötelező, ha a `--bind` nem loopback cím.

**Forrásajánlat.** A stúdió kiszolgálja a `/source` címet (akkor is, ha token szükséges), és a `--source-url` állítja be a címet (a hivatalos kiadási binárisok alapértelmezése az a commit, amelyből épültek), ahogy az AGPL-3.0 13. szakasza megköveteli (lásd [LICENSING.md](LICENSING.md)).

Az **ismeretlen vagy hibás útvonalak és metódusok** sima hibaválaszt kapnak; a szerkesztőoldal a binárisba van fordítva (`studio.html`), és nem a fájlrendszerből olvassák, így statikus tartalomnál nincs útvonal-bejárási (path traversal) támadási felület.

## 3. Hitelesítés és együttműködés

A `--collab-token T` opció (vagy a nem helyi `--bind` által megkövetelt token) bekapcsolja a hozzáférés-vezérlést; a `--collab-log LOG` tartósítja a megosztott munkamenetet.

* **Tokenformátum.** Legalább 16 karakter, az `A-Za-z0-9._~+/=-` készletből. Bármi, amiben CR, LF, `;` vagy más karakter van, induláskor elutasított (fejléc- és sütiinjekció).
* **Bemutatás.** Az `X-Collab-Token: T` fejléc, vagy az `arm_token` süti (`HttpOnly`, `SameSite=Strict`, valamint `Secure`, ha a kérés `https://` `--public-origin` név alatt, https `Origin` mellett vagy `X-Forwarded-Proto: https` fejléccel érkezett), amelyet a `POST /api/login` állít be. A rossz süti ugyanúgy sikertelen kísérletnek számít, mint a rossz fejléc.
* **Összehasonlítás.** Állandó idejű (mindkét oldalt SHA-256-tal hashelik, és a kivonatokat hasonlítják össze), így a válaszidő nem árulja el az előtag-egyezéseket.
* **Back-off.** Kliens IP-címenként, az **5.** egymás utáni hibától: 2, 4, 8, 16, majd 30 másodpercnyi 429-es válasz; legfeljebb 512 kliens-címet követ. A helyes tokent mindig elfogadja, még back-off alatt is (így a támadó nem zárhatja ki a tulajdonost). A back-off lassítja a találgatást; elosztott támadást nem állít meg. **Reverse proxy mögött minden kliens a proxy címével jelenik meg**, ezért egy back-off számlálón osztoznak: egy látogató rossz kéréseinek áradata mindenki hibaszámlálóját késlelteti, de a helyes token ettől még működik.
* **Résztvevői titkok.** Minden közreműködő kap egy véletlen 128 bites titkot (`X-Collab-Secret`), amely az eseményfolyamát azonosítja; legfeljebb 32 résztvevő és 64 eseményfolyam.
* **Naplóintegritás.** A munkamenet-napló szövegfájl, műveletenként egy sorral: `<hash> <json>`, SHA-256-tal láncolva (2-es formátumverzió). Egy sor szerkesztése, törlése vagy átrendezése megszakítja a láncot, amit a napló betöltésekor jelez. Minden kérést `fdatasync` követ a válasz előtt, a fájl 0600-as móddal jön létre, és atomi módon íródik újra. A lánc a véletlen sérülést és az alkalmi manipulációt észleli; **nincs aláírva**, így aki írhatja a fájlt, újra tud építeni egy érvényes láncot.
* **Résztvevőnkénti tokenek (7. hullám, opcionális).** A `--collab-secret-file F` az egyetlen közös bearer tokent egy titokból származtatott tokenekre cseréli: `ap1.<name>.<expires>.<mac>`, ahol a `mac` = HMAC-SHA-256 (RFC 2104, saját SHA-256, RFC 4231 tesztvektorok a tesztekben) a név és a lejárat fölött. A gazda az `osatura collab-token NAME --secret-file F --ttl SECONDS` paranccsal állítja elő őket (alapértelmezés 8 óra, legfeljebb 30 nap), és mindenkinek a sajátját adja. A token egy névhez kötött (a csatlakozó résztvevő azt a nevet kapja, amit az oldal küld, így senki nem csatlakozhat másnak kiadva magát), magától lejár, újraszámolással ellenőrzik (a szerveren nincs tábla, nincs mit ellopni belőle a titkon kívül), és a sütije a hátralévő élettartammal egyenlő `Max-Age` értéket kap. A `--collab-revoke NAMES` a lejárat előtt elutasítja az adott nevek tokenjeit; a titok megváltoztatása mindent visszavon. A titokfájlnak privátnak kell lennie (Unixon 600-as mód, különben a stúdió nem indul el), legalább 16 karakteresnek, és parancssori argumentumként soha nem fogadja el. A `--collab-token` és a `--collab-secret-file` kombinálható; bármelyik teljesíti azt a szabályt, hogy a nem loopback kötéshez hitelesítő adat kell. A lejáratot **kérésenként** ellenőrzi: a már megnyitott eseményfolyam a végéig nyitva marad, a következő kapcsolatot pedig elutasítja. A lejárt vagy visszavont tokenre `401` a válasz az okkal, és nem táplálja a találgatás elleni back-offot.
* **Sebességkorlát (7. hullám).** A `--collab-rate N` másodpercenként N kérést enged kliens-címenként, 2N-es löketekkel (token bucket); ennél többre `429` a válasz `Retry-After` fejléccel. Az alapértelmezés 100 másodpercenként, ha hitelesítő adat szükséges, különben nincs korlát; a 0 kikapcsolja. Hitelesítés után, és **cím** (a kapcsolat forrása) szerint alkalmazza, nem TCP-kapcsolatonként, mert a kapcsolatonkénti korlátot több kapcsolat megnyitásával meg lehet kerülni; reverse proxy mögött minden kliens a proxy címén osztozik, ezért a proxynál is korlátozz (12. szakasz).
* **Kötési cím figyelmeztetések (7. hullám).** A nem loopback `--bind` induláskor figyelmeztetést ír ki (sima HTTP; a `0.0.0.0` / `::` esetén, hogy a stúdió minden interfészen figyel; a közös tokennél, hogy nincsenek nevei és soha nem jár le).
* **Nincs TLS, nincsenek fiókok.** A szerver sima HTTP-t beszél. A hálózaton utazó tokent bárki elolvashatja az úton. Tedd a stúdiót TLS-t lezáró reverse proxy vagy SSH / VPN alagút mögé (11. szakasz). Nincsenek felhasználói fiókok, szerepkörök vagy projektenkénti jogosultságok: a token teljes szerkesztési hozzáférést ad.

## 4. Assetfeloldás és bemeneti korlátok

* Az **assetek** (képfájlok, normáltérképek, a projekt által megnevezett hangok) az `assets::resolve` függvényen át oldódnak fel: a folyamatszintű asset-könyvtárhoz képest (`assets::default_dir`, általában a bemeneti projekt mappája). **Az abszolút útvonalakat és minden `..` elemet elutasít**, hacsak nincs megadva a globális `--allow-external-assets` kapcsoló. A képfájlok legfeljebb 64 MB-osak, a hangfájlok legfeljebb 256 MB-osak lehetnek.
* **Projektfájlok.** A `load_project` csak szabályos fájlokat olvas (eszközöket, csöveket nem), legfeljebb 256 MB-ig (módosítás: `--max-input-bytes N`). A homlokzat `MAX_JSON_BYTES` értéke 256 MiB.
* **Származtatott projektek.** A származtatott projektet író parancsok a *bemeneti* projekt könyvtárához képest validálnak; ha a kimenet máshol van, a meglévő fájlok relatív asset-útvonalait átírják, vagy figyelmeztetés jelzi, hogy a kimenet nem találja meg őket (az `--allow-external-assets` abszolúttá teszi őket).
* **Renderelés.** A homlokzat elutasítja a 16384 pixelnél nagyobb oldalú képeket (`MAX_RENDER_SIDE`) és a 0,05-4 tartományon kívüli skálákat; a C és a wasm réteg a 256 MiB fölötti kimenetet foglalás előtt elutasítja. További homlokzati korlátok: fps a (0, 1000] tartományban, képkockák 1-200000, színpad legfeljebb 32768, 4096 csont, nagyságrendek 1e7-en belül, `lod_levels` 1-12, `fuzz` minták legfeljebb 100000, `trace` színek 2-64 ([API.md](API.md)).
* **Bemenetre nincs pánik.** Az ártalmas bemenetet vizsgáló tesztek az `osatura-cli/tests/robust.rs` fájlban és a hullámtesztekben vannak; fuzz-jellegű tesztek fedik a parsereket. A bemenetből elérhető pánikot sebezhetőségnek tekintjük.
* **A stúdió törzsei** legfeljebb 32 MB-osak; a szerkesztések ugyanazon a validáción mennek át, mint a fájlok.

## 5. Kifejezés- és szkript-sandbox

* A **kifejezéseket** (értékkötések, kényszerek, driverek) saját parser dolgozza fel: **forrás legfeljebb 4096 bájt, beágyazási mélység 64**, aritmetika, összehasonlítás, a rögzített `frame`, `time`, `fps`, `frames`, `pi`, `t` változók, a driverekhez a `get("layer","property")`, valamint a beépített függvények rögzített táblája (`sin`, `abs`, `min`, ...); nincsenek ciklusok, I/O és felhasználó által definiált függvények, így a kiértékelés mindig befejeződik. A nem véges eredményeket a véges értékek szabálya elutasítja.
* A **szkriptek** (`script` parancs, `scripting` feature, alapból bekapcsolva) Rhai-ban futnak ezekkel a korlátokkal: **500 000 művelet, 32 hívási szint, kifejezésmélység (64 legfelső szinten, 32 függvényekben), sztringek 10 000 karakter, tömbök 10 000 elem, map-ek 1000 bejegyzés, 5000 API-kérés, szkriptforrás 64 KB**, és `no_module` (nincs `import`, tehát nincs fájlhozzáférés modulokon át). A szkriptek számára kitett gazda-API a projektszerkesztő függvények rögzített készlete. Építs `--no-default-features` kapcsolóval, hogy a szkriptmotort teljesen eltávolítsd (a `smartstring` függőséget is eltávolítja).
* A szkript továbbra is kód, amelyet te választasz futtatni egy projekten: maga nem tud fájlt olvasni vagy írni, de a fenti korlátokon belül nagyon naggyá teheti a szerkesztett projektet, és egy végtelen ciklus a műveletkorlátnál véget ér.

## 6. Fájlok, szimbolikus linkek és ideiglenes adatok

* **Atomi írás.** A kimeneteket az `fsutil::write_atomic` írja: ideiglenes fájl ugyanabban a könyvtárban (név: `.<name>.tmp-<pid>-<random>`), `fsync`, `rename`. Összeomlás esetén vagy a régi, vagy az új fájl marad. **A kimenetként megadott szimbolikus linkeket elutasítja**, megnevezve a `--follow-symlinks` kapcsolót (ez most minden felhasználó által megcímzett kimenetre kiterjed: `ci run` jelentések és golden képek, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, `face-*`, póz-fájlok, pillanatképek, `vcs`, a collab-napló és az ffmpeg által írt videófájlok, amelyeket előzetesen ellenőriz), így egy elhelyezett link nem tud átirányítani egy írást. Ha a könyvtár nem írható, de a fájl igen, a tartalék megoldás a helyben felülírás `fsync`-kel, ami **nem atomi** (dokumentált).
* **Privát adatok.** A collab-naplók és más, csak a tulajdonos által olvasható fájlok 0600-as móddal jönnek létre (`write_atomic_private`). Az ideiglenes könyvtárak kizárólagosan, 0700-as móddal jönnek létre (`private_temp_dir`), így egy megosztott `/tmp`-ben egy kiszámítható nevet nem hozhat létre előre másik felhasználó. A véletlen értékek (új résztvevők tokenjei, ideiglenes nevek) a `/dev/urandom`-ból jönnek (`fsutil::random_bytes`); a motor seedelt determinisztikus generátorát titkokhoz soha nem használják.
* **UI-beállítások.** A stúdió `ui.json` fájlja figyelmeztetéssel figyelmen kívül hagyja az ismeretlen kulcsokat, és az első mentés előtt átmásolódik az `ui.json.bak` fájlba (majd `.bak.1`, ...).
* **Windows / nem Unix.** A fenti jogosultsági módok Unixon érvényesek. Windowson és macOS-en a tesztkészlet sikeres a CI-ben (a csak POSIX-en futó ellenőrzéseket a Windows kihagyja), de a symlink-, jogosultsági és `/dev/urandom` ellenőrzések csak Linuxon és macOS-en futnak, és a Windows-viselkedést senki nem vizsgálta át kézzel.
* **Aláíratlan binárisok és az ablakos alkalmazás.** A kiadási futtatható fájlok nincsenek kódaláírással ellátva; ellenőrizd a letöltéseket a `SHA256SUMS.txt` fájllal. Az `osatura-studio` ugyanazt a helyi szervert indítja el (csak `127.0.0.1`, véletlenszerű szabad port), és a rendszer webnézetében (WebView2 / WKWebView) jeleníti meg; webnézete ugyanúgy ki van téve, mint egy böngészőfül azon a címen, és külön biztonsági felülvizsgálaton nem esett át.

## 7. Külső programok (ffmpeg)

Az ffmpeg-et és az ffprobe-ot külön folyamatként **csak** videóexporthoz, mp3 / ogg dekódoláshoz, hangsáv-keveréshez és néhány eszközhöz futtatjuk; soha nincsenek becsomagolva ([LICENSING.md](LICENSING.md)). Minden hívás:

* az argumentumokat tömbként adja át (nincs shell, nincs sztring-összefűzés parancssorba);
* a `-protocol_whitelist file` kapcsolót használja (egy belső csőhasználat hozzáadja a `pipe,fd` értéket), így egy médiafájl nem tudja rávenni az ffmpeg-et, hogy `http://`, `concat:` vagy más hálózati / speciális protokollt nyisson meg;
* a fájlneveket az `fsutil::file_arg` függvényen át adja át, amely `./` előtagot tesz a `-` karakterrel kezdődő, illetve az első útvonalelemben `:` vagy `|` karaktert tartalmazó nevek elé, így egy fájlnevet nem lehet opciónak vagy URL-nek érteni;
* a `-loglevel error` kapcsolót használja.

Maga az ffmpeg összetett médiaformátumokat dolgoz fel, és voltak saját sebezhetőségei: tartsd naprakészen, és titkokat tároló gépen ne adj neki olyan forrásból médiát, amelyben nem bízol. Az időkeretes parancsok (`export-fit --max-seconds`) nem tudnak megszakítani egy futó ffmpeg-hívást.

## 8. Könyvtárak (C ABI, WASM, homlokzat)

* A **homlokzat** minden argumentumot validál, `ApiError` hibát ad vissza, és a pánikokat `ApiError::Internal` hibává alakítja, ahol a cél unwindol. Az `Animation` és a `Player` `Send + Sync` (fordításkor ellenőrizve).
* A **C ABI** ([C_ABI.md](C_ABI.md)) ellenőrzi a null mutatókat, a hosszakat és az UTF-8-at, és kódokat ad vissza; a hívónak továbbra is érvényes mutatókat kell átadnia, és be kell tartania a pufferszabályokat. Az utolsó hiba sztringje szálanként külön van. Az ABI-t AddressSanitizer és UndefinedBehaviorSanitizer alatt próbálták ki a dokumentált példákkal (tiszta). Nincs megerősítve olyan hívó ellen, amely megsérti a dokumentált szerződéseket (dupla zárás, zárás utáni használat, rossz hosszak).
* A **WASM modul** ([WASM.md](WASM.md)) lefordul a CI-ben, és átmegy a wrapper tesztjén Node alatt, és egyszer futtatták headless Chromiumban (a hibás bemenet hibát ad, nincs szivárgás 60 nyitás/zárás ciklus alatt, a túlméretes stage-eket elutasítja), de **böngészőkomponensként nem vizsgálták, és Firefoxban vagy Safariban nem futtatták**, így biztonsági tulajdonságai csak a kód olvasásából és abból a tesztből ismertek. Konstrukciója szerint nincs fájlrendszer- vagy hálózati hozzáférése; egy pánik ott csapdába ejti a példányt.
* Egyik réteg sem sandboxolja a *projektet*: egy ártalmas projekt renderelését a fenti korlátok határolják, de a projekt a korlátokon belül is sok munkát kérhet (sok réteg, nagy képek, hosszú idővonalak).

## 9. Ismert hiányosságok

Őszinte lista; a funkcionális korlátok tágabb listáját lásd a [LIMITATIONS.md](LIMITATIONS.md) fájlban.

1. **Nincs TLS és nincsenek fiókok** a stúdióban. A 7. hullám óta a token lehet résztvevőnkénti és lejáró (3. szakasz), ami még nem fiókrendszer: nincsenek szerepkörök, nincs jelszó-visszaállítás, és nincs saját TLS. Natív TLS-t azért nem adtunk hozzá, mert a `Cargo.lock` nem tartalmaz licenc szempontjából összeegyeztethető TLS crate-et (egy ilyen hozzáadása nagy új függőséget és kriptográfiai kódot vinne egy auditálatlan AGPL/MPL projektbe); a 12. szakasz a támogatott módot mutatja, a TLS-t lezáró reverse proxyt.
2. **A back-off címenkénti**, és proxy mögött közös; lassítja, de nem akadályozza meg az online találgatást. Használj hosszú, véletlen tokent (a 16 karakteres minimum alsó határ, nem ajánlás).
3. **A collab-napló láncolt, nem aláírt.**
4. **Nem auditált kódbázis.** Nincs harmadik fél általi penetrációs teszt vagy formális fuzzing-kampány; az ellenőrzések a projekt saját tesztjei és review-hullámai.
5. **A függőségeket licencekre ellenőrzik, sebezhetőségekre nem.** A repositoryban nincs `cargo audit` lépés; telepítés előtt futtasd magad.
6. **Az asset-könyvtár folyamatszintű.** Több bemeneti könyvtár fölötti `merge` az assetekhez az utolsó könyvtárat használja; csak a `write_json` írja át az asset-útvonalakat (a többi író nem).
7. **Nem atomi tartalék**, ha a kimeneti könyvtár nem írható.
8. **Az útvonal-ellenőrzések kanonikus útvonalakat használnak.** Az `assets::resolve` kanonizálja a létező fájlokat, és elutasít minden fájlt, amelynek tényleges helye a projektkönyvtáron kívül van, így a projektmappán belüli, kifelé mutató szimbolikus linket elutasítja (az `assets.rs` tesztelte); a mappán belül maradó linkek működnek. Az ellenőrzés és a későbbi olvasás közötti versenyhelyzet (time-of-check to time-of-use) olyasvalaki által, aki egyidejűleg módosíthatja a projektmappát, nem kizárt.
9. **Az ffmpeg** kívül esik az ellenőrzésünkön (7. szakasz).
10. **A WASM** nincs tesztelve böngészőmotorban (a CI-ben csak Node alatt fut); az azt beágyazó oldal böngészős fenyegetési modelljét (CSP, memóriakorlátok) nem vizsgálták.
11. **Szolgáltatásmegtagadás mérettel.** A korlátok az egyes kéréseket és fájlokat határolják, nem az összes munkát: egy érvényes, de nagyon nehéz projekt hosszú ideig foglalhat egy CPU-t (a fenti kapcsolatkorlát a párhuzamosságot korlátozza, nem egy kérés költségét).

## 10. Sebezhetőség bejelentése

* **Sebezhetőség miatt ne nyiss nyilvános issue-t.** Küldj privát jelentést a karbantartónak: **`alexio@trom.tf`**. *(Helyőrző: a cím megjelenik a program `--version` közleményében és a licencdokumentumokban; a tulajdonosnak kell biztonsági elérhetőségként megerősítenie, és az első kiadás előtt lecserélheti dedikált címre vagy privát advisory csatornára.)*
* Tartalmazza: a verziót (`osatura --version`) és a platformot, a problémát kiváltó projektet vagy kérést (a minimális fájl a legjobb), hogy mit vártál és mi történt, és hogy kérsz-e említést.
* Mire számíthatsz *(szándék, nem szerződés: ez egy egyszemélyes projekt)*: észszerű időn belüli visszaigazolás, privátban kidolgozott javítás, a csomaghoz hozzáadott regressziós teszt, egy megjegyzés a [CHANGELOG.md](CHANGELOG.md) fájlban, és kívánságra említés. Kérjük, hagyj időt a javításra, mielőtt részleteket közzéteszel.
* Támogatott verziók: csak a legújabb előzetes kiadás (a `v0.1.0-alpha.3`, a közzététele után) és a `main` aktuális fája támogatott.
* A stúdió hálózati kitettségéről szóló jelentések, ha szándékosan nyilvános címhez van kötve reverse proxy nélkül, először dokumentációs problémák: lásd a 11. szakaszt.

## 11. Üzemeltetői keményítési ellenőrzőlista

Mindenkinek, aki az `osatura studio`-t egynél több személy számára vagy a localhoston kívül futtatja.

1. [ ] Inkább **ne** tedd ki. Egy felhasználónál hagyd az alapértelmezett kötést (`127.0.0.1`), és használj SSH-alagutat (`ssh -L 8080:127.0.0.1:8080 host`).
2. [ ] Ha ki van téve, **zárd le a TLS-t** egy előtte álló reverse proxyban (nginx, Caddy, ...); a sima HTTP-portot soha ne tedd ki az internetre.
3. [ ] Használj **hosszú, véletlen tokent** (például 32+ karakter az `openssl rand -base64 24` kimenetéből), a szolgáltatás védett fájlján vagy környezetén át add át, ne shell-előzményen keresztül, és cseréld, ha valaki távozik. Ne feledd, hogy ez teljes szerkesztési hozzáférés.
4. [ ] Indítsd a `--bind 127.0.0.1` (vagy a privát interfész) opcióval plusz a proxyval; állítsd be az `--allow-host your.domain` és a `--public-origin https://your.domain` opciót, hogy a Host és Origin ellenőrzések a proxyra átmenjenek, és semmi másra.
5. [ ] Egyetlen közös token helyett inkább **résztvevőnkénti tokeneket** használj (`--collab-secret-file`, 3. szakasz) rövid `--ttl` értékkel; a titokfájlt 600-as módban, a kiszolgált mappán kívül tartsd.
5a. [ ] Állíts be a proxyn **sebességkorlátot**, és ha lehet, korlátozz IP szerint, vagy adj hozzá egy második hitelesítési réteget (HTTP basic TLS-en, VPN, SSO proxy), mert a beépített back-off csak a proxy címét látja.
6. [ ] Futtasd **nem privilegizált felhasználóként**, külön könyvtárban, amely csak a kiszolgálandó projekteket tartalmazza; titkok és érzékeny szimbolikus linkek nélkül. Máshol használj csak olvasható fájlrendszert, konténert vagy systemd sandboxingot (`ProtectSystem`, `PrivateTmp`, `NoNewPrivileges`).
7. [ ] Az `--allow-external-assets` és a `--follow-symlinks` maradjon **kikapcsolva**.
8. [ ] Állítsd be a `--source-url` opciót arra a helyre, ahonnan a (esetleg módosított) forrás letölthető (AGPL 13. szakasz).
9. [ ] Tartósíts a `--collab-log` opcióval privát könyvtárban, készíts róla mentést, és tartsd meg a 0600-as módot. Kezeld a naplót érzékenyként: a teljes szerkesztési előzményt tartalmazza.
10. [ ] Korlátozd az erőforrásokat operációs rendszer szintjén (memória, CPU-idő, nyitott fájlok, lemezkvóta); a beépített korlátok az egyes kéréseket korlátozzák, nem a teljes terhelést.
11. [ ] Tartsd naprakészen az ffmpeg-et és a buildhez használt Rust eszközláncot; futtasd a `cargo audit` parancsot (vagy egyenértékűt) a kiadásod `Cargo.lock` fájlján.
12. [ ] Ne adj a szervernek nem megbízható forrásból származó projekteket, hacsak nem a fenti sandboxban fut. Szkriptelés nélküli telepítésekhez építs `--no-default-features` kapcsolóval.
13. [ ] Figyeld a naplókat a 421, 403, 429 és 503 válaszokra; a kiugrások próbálkozást vagy hibás konfigurációt jeleznek.

## 12. Reverse proxy TLS-sel: nginx és Caddy

A stúdió sima HTTP-t beszél. A saját gépeden túli bármihez futtasd loopbacken, és hagyd, hogy egy proxy zárja le a TLS-t. Ezek a receptek a proxyk nyilvános dokumentációjából és a fent leírt stúdióviselkedésből készültek; **a szerzői környezetben nem futtattuk őket** (nincs nginx vagy Caddy, nincsenek tanúsítványok, nincs hálózat): teszteld őket, mielőtt rájuk hagyatkozol.

Indítsd a stúdiót loopbacken, és add meg a nyilvános hosztját, hogy a Host és Origin ellenőrzések a proxyt fogadják el, és semmi mást:

```
osatura studio demo.json --port 8080 --bind 127.0.0.1 \
  --allow-host studio.example.org --public-origin https://studio.example.org \
  --collab-secret-file /etc/osatura/collab.secret --collab-log /var/lib/osatura/session.log
```

Egy `https://` `--public-origin` esetén a bejelentkezési süti megkapja a `Secure` attribútumot. Az eseményfolyam (`/api/collab/events`) Server-Sent Events: a proxy nem puffereli, és hosszú olvasásokat kell engednie.

**nginx** (tanúsítványok a CA-dtól vagy a `certbot`-tól):

```
limit_req_zone $binary_remote_addr zone=osatura:10m rate=20r/s;

server {
    listen 80;
    server_name studio.example.org;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    server_name studio.example.org;
    ssl_certificate     /etc/letsencrypt/live/studio.example.org/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/studio.example.org/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    add_header Strict-Transport-Security "max-age=31536000" always;
    client_max_body_size 64m;

    location / {
        limit_req zone=osatura burst=60 nodelay;
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Proto https;
        proxy_set_header Connection "";
    }
    location /api/collab/events {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Proto https;
        proxy_set_header Connection "";
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 1h;
    }
}
```

**Caddy** (a tanúsítványt maga szerzi be és újítja meg; `Caddyfile`):

```
studio.example.org {
    encode zstd gzip
    header Strict-Transport-Security "max-age=31536000"
    request_body {
        max_size 64MB
    }
    reverse_proxy 127.0.0.1:8080 {
        flush_interval -1
        transport http {
            read_timeout 1h
        }
    }
}
```

A `flush_interval -1` pufferelés nélkül küldi át az eseményfolyamot. A Caddy beállítja az `X-Forwarded-Proto` fejlécet, és továbbadja az eredeti `Host` értéket. Kérési sebességkorláthoz használd a `caddy-ratelimit` bővítményt, vagy tedd elé az nginx `limit_req` beállítását.

Ellenőrzések indítás után: a `curl -I https://studio.example.org/` `401`-gyel válaszol (bejelentkezési oldal), és token nélkül soha nem szolgálja ki a stúdiót; a `curl -I http://127.0.0.1:8080/ -H 'Host: evil.example'` `421`-gyel válaszol; a `/api/login` által beállított süti `Secure; HttpOnly; SameSite=Strict` jelzőket hordoz; a stúdió saját munkamenet-naplója privát marad (6. szakasz). Ha a proxy másik gépen fut, a stúdiót a proxy által elérhető privát interfészhez kösd, tűzfalazd a portot, és ne feledd, hogy maga a stúdió ezen a kapcsolaton is sima HTTP-t beszél.
