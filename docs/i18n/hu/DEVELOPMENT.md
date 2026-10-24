<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Fejlesztői útmutató

*Azoknak, akik az OSATURA-t építik, módosítják, tesztelik és kiadják. A dokumentáció CC-BY-4.0 licencű. Minden itt leírt tartalom a forrásfából, a `ROADMAP.md` 9. szakaszából, a [CI.md](CI.md), a [CLEANROOM.md](CLEANROOM.md), a [CONTRIBUTING.md](CONTRIBUTING.md) és a [LICENSING.md](LICENSING.md) fájlokból származik; a „run" jelölésű parancsokat az oldal írása közben futtattuk (Linux x86-64, rustc 1.97.0, offline regisztrum).*

Tartalom: [Környezet](#1-környezet) - [Build- és tesztparancsok](#2-build--és-tesztparancsok) - [A tesztcsomagok](#3-a-tesztcsomagok) - [Kódolási konvenciók](#4-kódolási-konvenciók) - [Ellenőrzőlista új modulhoz vagy parancshoz](#5-ellenőrzőlista-új-modulhoz-vagy-parancshoz) - [Clean-room szabályok és DCO](#6-clean-room-szabályok-és-dco) - [Kiadási ellenőrzőlista](#7-kiadási-ellenőrzőlista) - [Folyamatos integráció](#8-folyamatos-integráció) - [Hibakeresés](#9-hibakeresés) - [Profilozás](#10-profilozás)

## 1. Környezet

| Szükséges | Miért | Megjegyzés |
|---|---|---|
| Rust (stable, edition 2021 workspace) | minden | rustc 1.97.0-val fejlesztve; a `Cargo.lock` be van commitolva, a buildek `--locked` kapcsolóval is működnek |
| C fordító (`cc`) | a C ABI példa/teszt (opcionális) | lásd [C_ABI.md](C_ABI.md) |
| `ffmpeg` / `ffprobe` a `PATH`-on | videóexport, mp3/ogg hangdekódolás, `dof-export` .mp4/.webm formátumba, `soundtrack` | **soha nem része a csomagnak, és a buildhez sem kell**; az ezt igénylő parancsok ezt kiírják, és hiányában nem jeleznek sikert |
| Node.js >= 18, `playwright-core` és Chromium | az öt stúdió-smoke-teszt; a WASM-burkoló tesztje | alapértelmezések: `/opt/npm-tools/node_modules/playwright-core`, `/opt/pw-browsers/chromium-1194/chrome-linux/chrome`; felülírás: `PLAYWRIGHT_CORE`, `CHROME` |
| `cargo-deny` | licenc-, tiltás- és forráspolitika | a `scripts/licence-audit.sh` futtatja, ha telepítve van (`--no-deny` kihagyja) |
| `python3` | a `scripts/licence-audit.sh` egy beágyazott szkripttel generál dokumentumokat | `python3 -I` módban fut |
| Vulkan vagy GL adapter | csak az `osatura-gpu` valódi GPU-s méréseihez | a tesztek szoftveres adapterre térnek vissza; lásd a 2. szakaszt |
| `rustup target wasm32-unknown-unknown`, `wasm-opt` | csak a [WASM.md](WASM.md) miatt | a fejlesztői környezetben soha nem volt telepítve; a CI telepíti |

A workspace felépítése (a tervezést lásd az [ARCHITECTURE.md](ARCHITECTURE.md) fájlban):

| Crate | Licenc | Szerep |
|---|---|---|
| `osatura-core` | MPL-2.0 | motor, formátumok, eszközök, a stabil `api` homlokzat ([API.md](API.md)); feature-ök: `scripting` (alapértelmezett, Rhai) |
| `osatura-cli` | AGPL-3.0-or-later | az `osatura` binárisa, stúdiószerver, collab-szerver; feature-ök: `scripting` (alapértelmezett), `gpu` |
| `osatura-capi` | MPL-2.0 | C ABI ([C_ABI.md](C_ABI.md)) |
| `osatura-wasm` | MPL-2.0 | WebAssembly ABI + JS-burkoló ([WASM.md](WASM.md)) |
| `osatura-gpu` | MPL-2.0 | opcionális wgpu renderer; **nem** alapértelmezett tagja a workspace-nek |
| `osatura-gui` | AGPL-3.0-or-later | az `osatura-studio`, a stúdió natív ablakban (`tao` + `wry`, csak Windowson és macOS-en; máshol csonk); **nem** alapértelmezett tag, `publish = false` |

Repository-szintű fájlok: `ROADMAP.md` (a 9. szakasz a dátumozott funkció- és review-napló), `FEATURES.md`, `CLEANROOM.md`, `LICENSING.md`, `REUSE.toml`, `deny.toml`, `LICENSES/`, `scripts/`.

## 2. Build- és tesztparancsok

```sh
cargo build                                  # default members: core, cli, capi, wasm (gpu is opt-in)
cargo test --workspace                       # run: 1343 tests + 19 doc-tests without gpu and gui (section 3)
cargo test -p osatura-core                  # one crate (the engine and the facade tests)
cargo test -p osatura-core api::            # a module or a name filter
cargo test -p osatura-cli --test e2e        # one integration test file
cargo test --doc -p osatura-core            # doc-tests only (the facade examples)
cargo test -p osatura-cli --test spdx_audit # SPDX header audit (0.01 s)
cargo clippy --workspace --all-targets -- -D warnings   # zero warnings (gated in CI since wave 8)
scripts/licence-audit.sh                     # regenerate THIRD_PARTY*, SPDX check, cargo deny
scripts/e2e.sh                               # the end-to-end pipeline in release mode (--debug for debug)
```

Feature-ök és opcionális részek:

```sh
cargo build -p osatura-cli --no-default-features         # no Rhai (no `script` command, no smartstring)
cargo build -p osatura-core --no-default-features        # what capi, wasm and gpu use
cargo build -p osatura-cli --features gpu                # adds `render-gpu` (wgpu)
cargo test  -p osatura-gpu                               # GPU tests (31: 18 unit + 13 parity); NOT in the workspace default
OSATURA_GPU_BACKEND=vulkan osatura render-gpu ...     # restrict the wgpu backend: vulkan or gl
```

Alapszabályok: a tesztcsomagokhoz debug buildeket használj (a repository fejlesztésekor a lemezterületre is ügyeltek: ne másold a `target/` könyvtárat); `--release` csak időmérési kérdésekhez kell. A homlokzat pillanatképét az `OSATURA_BLESS=1 cargo test -p osatura-core api::` paranccsal fogadtatod el (5. szakasz). Az `OSATURA_UI_CONFIG` kijelöli a stúdió UI-beállításfájlját a tesztekben, az `OSATURA_AUTHOR` a szerző nevét a verziókezelési commitokban, az `OSATURA_E2E_TIMINGS=<file>` pedig arra készteti az e2e tesztet, hogy kiírja a szakaszonkénti időméréseket.

## 3. A tesztcsomagok

A számok a `ROADMAP.md` 9. szakaszából és egy teljes `cargo test --workspace --offline` futásból származnak, amelyet 2026. október 22-én futtattunk ehhez az oldalhoz (debug profil, más feladatokkal megosztott gép, ezért az időmérések tájékoztató jellegűek, és **lassabbak lesznek, mint egy üresjáratban lévő gépen**):

| Csomag | Tesztek | Idő (ez a futás) | Mit fed le |
|---|---:|---:|---|
| `osatura-core` unit (`src/lib.rs`) | 1167 | 349 s | minden modul; tartalmazza az `api::tests::api_surface_snapshot` tesztet |
| `osatura-core/tests/prelude_surface.rs` | 6 | 0.7 s | a homlokzat a crate-en kívülről (`#[non_exhaustive]`, hibaleképezés, a hívások reprezentatív részhalmaza) |
| `osatura-cli` unit (`src/usage_tests.rs`, `src/args.rs`, stúdió, collab, http) | 80 | 42 s | opciófeldolgozás, USAGE a parserrel szemben, HTTP-szerver, szabályzat, collab-napló |
| `tests/e2e.rs` | 2 | 214 s | a teljes pipeline a valódi binárison keresztül (a ROADMAP szerint kb. 19 s üresjáratban lévő debug buildben és 40 s release-ben; itt a gép terhelt volt) |
| `tests/fixes.rs`, `tests/wave4.rs`, `tests/robust.rs` | 7, 15, 10 | 19 s, 1.2 s, 1.2 s | a review-hullámok regressziói és a rosszindulatú bemenetek tesztjei |
| `tests/doc_mismatch.rs` | 20 | 1.8 s | dokumentáció/viselkedés regressziók: atomi kimenetek, szigorú opciók, a `--json`/`--quiet`/`--allow-empty` elfogadási mátrix, a `golden` skálázási szabályai |
| `tests/source_offer.rs` | 3 | 0.2 s | az AGPL 13. szakasza szerinti `/source` ajánlat |
| `tests/face_cli.rs` | 1 | 0.8 s | arckövetés CLI |
| `osatura-core/tests/wasm_safety.rs` | 2 | 0.1 s | a `clock.rs`-en kívül nincs közvetlen `Instant` / `SystemTime` / `process::id` / `thread::spawn` (forrásvizsgálat) |
| `osatura-cli/tests/wave7.rs` | 7 | 7 s | 7. hullám: alparancsonkénti opciók, `follow` / `make-loop`, `color-space`, `collab-token`, a stúdió titokfájlja, a `dof` / `render-hdr` / `fx` képkockatartománya |
| `tests/spdx_audit.rs` | 2 | 0.01 s | minden `.rs/.js/.mjs/.sh` fájlnak van a crate licencével megegyező SPDX sora; a REUSE.toml fedi a többit |
| `osatura-capi/tests/abi.rs` | 7 | 1.1 s | a C ABI a Rust oldalról (C smoke teszt: `osatura-capi/tests/smoke.c`, kézzel fordítva, lásd [C_ABI.md](C_ABI.md)) |
| `osatura-wasm` unit | 8 | 1.6 s | az ABI-függvények a gazdagépen |
| `osatura-gpu` unit + `tests/parity.rs` | 18 + 13 | 0.2 s + 6.5 s | GPU és CPU paritása (szoftveres adapter, ha nincs GPU) |
| Doc-tesztek (`osatura-core`) | 19 | 171 s (mindegyik egy programot fordít) | a homlokzat példái (`api/`, `lib.rs`); az `osatura-capi` és az `osatura-wasm` crate-nek nincs |

Összesen: 1343 teszt a unit- és integrációs binárisokban és 19 doc-teszt, 2026. október 24-én megszámolva a `cargo test --workspace --exclude osatura-gpu --exclude osatura-gui` paranccsal (a következő mondat 1368-as száma a GPU crate tesztjeit is tartalmazta; a 7. hullám után, 2026. október 22-én egy teljes `cargo test --workspace` futásból újraszámolva, a fenti csomagonkénti időmérések egy korábbi futásból származnak), plusz 19 doc-teszt; a teljes `cargo test --workspace --offline` 836 s-ig tartott a megosztott gépen.

A `cargo test`-en kívüli csomagok:

* **Stúdió-smoke-tesztek (Chromium).** Öt Node-szkript hajtja a valódi stúdiót headless Chromiumban a `playwright-core` segítségével: `studio_ui_smoke.js` (128 ellenőrzés), `studio_collab_smoke.js` (60, két oldal egy gépen), `studio_overlays_smoke.js` (89), `studio_suggest_smoke.js` (35) és `studio_demo_smoke.js` (25: a vezetett demó betölti mind a 13 lépést, soha nem ír, bájtra pontosan visszaállítja a projektet, és helyreáll egy bezárt lapból); mind az ötöt 2026. okt. 24-én futtattuk, és lefutottak (a számok a szkriptek kimenete szerintiek). A `cargo test` **nem** futtatja őket, és alapértelmezés szerint nem részei a CI-nek. Futtatás:

  ```sh
  cargo build -p osatura-cli
  node osatura-cli/tests/studio_ui_smoke.js        # same for collab / overlays / suggest / demo
  ```

  Környezet: `OSATURA_BIN` (alapértelmezés: `target/debug/osatura`), `PLAYWRIGHT_CORE`, `CHROME`, az overlay-szkriptnél pedig `SHOTS` (a képernyőképek könyvtára). A szkriptek eltakarítják az ideiglenes fájljaikat.
* **WASM-burkoló teszt.** `node osatura-wasm/js/test.mjs` (csak mock, hacsak nem adsz meg projektútvonalat, és nem építetted meg a modult; lásd [WASM.md](WASM.md)).
* **Licenc-audit.** `scripts/licence-audit.sh`: újraépíti a `docs/THIRD_PARTY.md` és a `docs/THIRD_PARTY_NOTICES.md` fájlt a `cargo metadata --locked --all-features` kimenetéből (normál és build-függőségek, a fejlesztői függőségek kizárva), minden forrásfájl SPDX sorát összeveti a crate `license` mezőjével, majd lefuttatja a `cargo deny check licenses bans sources` parancsot, hacsak nincs `--no-deny`. A `deny.toml` a megengedő licenceket engedi, az MPL-2.0-t pedig csak megnevezett crate-ekhez.
* **clippy.** A `cargo clippy --workspace --all-targets -- -D warnings` 0 kilépési kóddal és figyelmeztetés nélkül fut le (8. hullám), és a CI kapuja. Ahol egy lint az adott esetben téves, a kód szűk `#[allow(clippy::...)]` attribútumot visel indoklással (indexelő ciklusok párhuzamos tömbök fölött, `too_many_arguments` stabil szignatúráknál, `manual_clamp` ott, ahol a NaN-kezelés eltér, a publikus API-hoz tartozó saját `next` / `mul` nevek); ne adj hozzá crate-szintű allow-t.
* **CI-bekötés.** A repository az *animációk* ellenőrzésére szállítja az `osatura ci` parancsot ([CI.md](CI.md)); ez más dolog, mint a kód CI-je. A projekt forrásfájában nincs kód-CI konfiguráció; a 8. szakasz egy minimális receptet ad.

Tesztek hozzáadása: a motor tesztjeit a kód mellé tedd (`#[cfg(test)]`), a CLI viselkedését vizsgáló teszteket az `osatura-cli/tests/` könyvtárba, és minden javított hibához tartozik egy regressziós teszt (a [CHANGELOG.md](CHANGELOG.md) review-hullámai felsorolják, mit javítottak és hol). A tesztek nem hagyhatnak ideiglenes fájlokat (használj drop guardokat), és nem függhetnek az órától vagy a hálózattól. Az időkereteket determinisztikus teszt-órával teszteljük.

## 4. Kódolási konvenciók

* **Bemenetre nincs pánik.** Bármi, ami projektet, fájlt, argumentumot vagy hálózati bájtot olvas, hibát ad vissza. Az indexelés, szeletelés, `unwrap` és az aritmetikai túlcsordulás nem megbízható értékeken hibának számít. A homlokzat (`api/`) ezen felül elkapja a pánikokat, és `ApiError::Internal` hibát ad vissza (csak ott, ahol a cél támogatja az unwindingot), de ez az utolsó védvonal, nem tervezési elv.
* **Véges értékek szabálya.** A fájlokból és argumentumokból érkező számokat NaN-ra és végtelenre, valamint ésszerű nagyságrendre ellenőrizzük (a homlokzat tartományai az [API.md](API.md) fájlban vannak: nagyságrend legfeljebb 1e7, fps a (0, 1000] tartományban, képkockák 1..=200000, színpad legfeljebb 32768, 4096 csont, renderoldal legfeljebb 16384 px). Utasíts el, ne vágj le, hacsak a függvény dokumentáltan nem vág (a képkockákat az idővonalhoz vágjuk).
* **Foglalás előtt ellenőrizd a méreteket.** A pixelpuffereket, hálóméreteket és mintaszámokat ellenőrzött aritmetikával számoljuk, és a `vec![..]` előtt elutasítjuk.
* **Atomi írás.** Az eredményeket a `fsutil::write_atomic` függvénnyel írd (ideiglenes fájl ugyanabban a könyvtárban, `fsync`, átnevezés; szimbolikus linket csak `--follow-symlinks` esetén fogad el; ha a könyvtár nem írható, helyben, `fsync`-kel felülír, ami dokumentáltan nem atomi). A felhasználó fájlját soha ne írd közvetlenül `std::fs::write`-tal. Az ideiglenes könyvtárakat az `fsutil` adja (0700-as mód, a fájlok 0600-asak). A véletlen bájtokat az `fsutil::random_bytes` adja (`/dev/urandom`).
* **Az asseteket az `assets::resolve` oldja fel.** A relatív kép-, normáltérkép- és hangútvonalak a folyamatszintű asset-könyvtárhoz képest oldódnak fel (`assets::default_dir`, a bemeneti projekt mappájából beállítva); az abszolút útvonalakat és a `..` elemet elutasítja, hacsak nincs `--allow-external-assets`; a képfájlok legfeljebb 64 MB-osak, a hang legfeljebb 256 MB lehet. A származtatott projektet író parancsok a *bemeneti* projekt könyvtárához képest validálnak, és a relatív asset-útvonalakat átírják, vagy figyelmeztetnek rájuk (`write_json`).
* **Determinisztikus véletlen.** Minden véletlenszerű viselkedés (fuzzing, zaj, jitter, szimuláció) a motor saját, seedelt generátorát használja `--seed` opcióval, és bitre reprodukálható. Nincs `rand` crate, és a rendszeridő sem lehet seed.
* **Println és lezárt csövek.** A CLI a `println!`/`print!` makrókat olyan makrókkal árnyékolja, amelyek lezárt csőnél csendben kilépnek (az `osatura ... | head` nem pánikolhat). A makrók után deklarált modulokban a `main.rs`-ben továbbra is ezeket használd.
* **A hibák megmondják, mit tegyél.** Az üzenetek megmondják, mi volt a hiba, melyik érték, és mi a megengedett tartomány vagy a javítás (a homlokzat tesztjei ezt a stílust ellenőrzik).
<!-- REUSE-IgnoreStart -->
* **SPDX fejlécek.** Minden `.rs`, `.js`, `.mjs` és `.sh` fájl `SPDX-License-Identifier:` sorral kezdődik, amely megegyezik a crate licencével (MPL-2.0 a core/capi/wasm/gpu/scripts esetén, AGPL-3.0-or-later a CLI-nél), és egy szerzői jogi sorral; a Markdown dokumentumok CC-BY-4.0 fejléc-megjegyzést viselnek, ahogy ez a fájl is. A többi fájlt (JSON, képek, a licencszövegek) a `REUSE.toml` fedi; új útvonalmintánál oda is vedd fel. Ellenkező esetben a `cargo test -p osatura-cli --test spdx_audit` elbukik.
<!-- REUSE-IgnoreEnd -->
* **Függőségek.** Kevés, tiszta Rust, megengedő licenc. Egy új crate-hez `deny.toml`-ellenőrzés (`scripts/licence-audit.sh`), egy sor a `docs/THIRD_PARTY.md` fájlban (újragenerálva) és indoklás kell a ROADMAP-bejegyzésben. Az `osatura-core`-t a capi, a wasm, a gpu és a CLI alapértelmezett feature-ök nélkül építi, ezért minden nehéz dolog feature mögé kerül.
* **Először a homlokzat.** Az új, felhasználó számára látható motorképesség, amelyet más nyelveknek is el kell érniük, az `osatura-core/src/api/` fájlba kerül egy pillanatkép-sorral, egy doc-teszttel és egy unit-teszttel; a C és a wasm réteg csak a homlokzatot burkolja.

## 5. Ellenőrzőlista új modulhoz vagy parancshoz

1. **Modul** az `osatura-core/src/<name>.rs` fájlban, a `lib.rs`-ben regisztrálva; SPDX fejléc; unit-tesztek (érvényes, határérték, rosszindulatú bemenet, determinizmus); nincs pánik.
2. **CLI-parancs**: egy `USAGE` bejegyzés (minden opció pontos írásmódja) az `osatura-cli/src/usage.rs` fájlban, az elosztó ág az `osatura-cli/src/cmds_1.rs` és `cmds_6.rs` közötti fájlok egyikében (az `ui` esetén a `main.rs`-ben), és az opciótáblák az `osatura-cli/src/args.rs` fájlban: minden értéket váró opciót fel kell venni a `VALUE_OPTS`-ba; az értéket nem váró kapcsolókat a parancs `CMD_OPTS` bejegyzésébe, ahol a globális tábla téves lenne (például a `ci --prune` kapcsoló, az `export-oca --layers` és az `autotrace --png` értéket vár, a `keyedit --free` kapcsoló). A USAGE és a parser egybevetését végző unit-tesztek elbuknak, ha a szöveg és a parser eltér. Támogasd a globális kapcsolókat (`--json`, `--quiet`, `--allow-empty`, `--allow-external-assets`, `--follow-symlinks`, `--max-input-bytes`), a `<command> --help` formát és a `--key=value` alakot.
3. **A kimenet írása** csak a `fsutil::write_atomic` vagy a `write_json` segítségével; a semmit sem előállító parancsok nem nulla kóddal lépnek ki, hacsak nincs `--allow-empty`.
4. **Dokumentáció**: a parancs a [CLI_REFERENCE.md](CLI_REFERENCE.md) fájlban, ha újat adsz hozzá, egy említés az útmutatókban, ahol releváns, és a bejegyzés a `ROADMAP.md` 9. szakaszában (mi került be, mi *nincs* kész: ez a lista táplálja a [LIMITATIONS.md](LIMITATIONS.md) fájlt).
5. **CLEANROOM.md sor**: minden új algoritmushoz rögzítsd a nyilvános forrást, amelyből megvalósították (tankönyv, szabvány, tanulmány), és a független levezetést; sor nélkül nincs merge.
6. **A homlokzati felület újra elfogadtatása**, ha az `api/` változott: futtasd az `OSATURA_BLESS=1 cargo test -p osatura-core api::` parancsot, nézd át az `osatura-core/src/api_surface.txt` diffjét (egy eltávolított vagy módosított sor törést jelent, és a homlokzat új főverzióját igényli; egy hozzáadott sor alverzió), majd futtasd újra a teszteket a változó nélkül. Frissítsd az [API.md](API.md) fájlt, valamint a C/WASM burkolókat és fejléceiket, ha ki kell tenniük.
7. **End-to-end**: add hozzá a parancsot az e2e pipeline-hoz, ha a gyártási úthoz tartozik; adj hozzá smoke-ellenőrzést a stúdió-szkriptekhez, ha van UI-ja.
8. **Futtasd**: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/licence-audit.sh` (ha függőségek vagy fejlécek változtak).

## 6. Clean-room szabályok és DCO

A projekt független megvalósítás. Röviden (a kötelező érvényű szöveg a [CLEANROOM.md](CLEANROOM.md) és a [CONTRIBUTING.md](CONTRIBUTING.md)):

* Nyilvános szabványokból, tankönyvekből és tanulmányokból valósíts meg, és hivatkozz rájuk. Ne olvass, ne fordíts vissza, ne másolj és ne fejts vissza zárt forrású animációs eszközök kódját, fájljait vagy interfészeit, és funkció írása közben ne olvasd vagy illeszd be versenytárs eszközök forráskódját. Ne illessz be kódot összeegyeztethetetlen licencű projektekből.
* Minden új módszert vegyél fel a `CLEANROOM.md` táblázatába a nyilvános forrásával.
* A szabadalmi megjegyzések a [PATENT_NOTES.md](PATENT_NOTES.md) fájlban vannak; a szabadalmi szempontból érzékeny területet érintő funkciót ott kell előbb megvitatni. Ez nem jogi tanács.
* A hozzájárulásokat annak a résznek a licence alatt fogadjuk el, amelyet módosítanak (bejövő egyenlő a kimenővel): MPL-2.0 a könyvtárakra és szkriptekre, AGPL-3.0-or-later az `osatura-cli`-re, CC-BY-4.0 a dokumentációra és a szintetikus assetekre.
* **Sign-off.** Minden commithoz kell egy `Signed-off-by: Name <email>` sor (`git commit -s`), amely a Developer Certificate of Origin 1.1-et tanúsítja (a szöveg a CONTRIBUTING.md-ben van). A sign-off kizárólag a származást igazolja: **nem** ad átlicencelési jogot, és amíg nincs CLA, a hozzájárult kódhoz nem lehetséges kereskedelmi vagy kettős licenc.
* A nevet és a logót a [TRADEMARK.md](TRADEMARK.md) védi.

## 7. Kiadási ellenőrzőlista

Az első buildek a `v0.1.0-alpha.3` előzetes kiadás-vázlatként készülnek (minden crate továbbra is 0.1.0; a „v0.5/v0.6/v0.7" verziócímkék a README-ben, a FEATURES-ben és a ROADMAP-ben funkciószintek, nem crate-verziók; lásd [CHANGELOG.md](CHANGELOG.md)). Egy kiadás elkészítéséhez:

1. **Verzió.** Döntsd el az egyes crate-ek verzióját. A homlokzatnak saját verziója van (`API_VERSION`, ma 1.4.0: főverzió egy felületi sor eltávolításakor vagy módosításakor, alverzió hozzáadáskor), az `OSATURA_ABI_VERSION` / `arm_abi_version()` (mindkettő 1) pedig csak a C vagy wasm ABI törő változásakor módosul. Igazítsd egymáshoz a `Cargo.toml` verzióit, az `osatura --version` kimenetét, a README állapotdobozát és a CHANGELOG-ot.
2. **Zöld kapuk.** `cargo test --workspace`, `cargo test --doc -p osatura-core`, `cargo test -p osatura-cli --test spdx_audit`, `cargo clippy --workspace --all-targets -- -D warnings`, az öt Chromium smoke-teszt, `scripts/e2e.sh` (release), `cargo test -p osatura-gpu`, ha a GPU-kód változott.
3. **Harmadik fél közlemények.** Futtasd a `scripts/licence-audit.sh` szkriptet: újragenerálja a `docs/THIRD_PARTY.md` (leltár) és a `docs/THIRD_PARTY_NOTICES.md` (a szállított dolgok licencszövegei) fájlt, ellenőrzi az SPDX sorokat, és lefuttatja a `cargo deny` parancsot. Commitold az újragenerált fájlokat. A bináris terjesztésnek a közlemények csomagját a bináris mellé kell szállítania.
4. **Licencfájlok.** A `LICENSE`, a `LICENSES/` (minden használt licenc szövege) és a `REUSE.toml` szinkronban van a [LICENSING.md](LICENSING.md) fájllal; a dokumentáció CC-BY-4.0 attribúciója sértetlen.
5. **Nincs ffmpeg.** Ne csomagold, ne linkeld és ne töltsd le az ffmpeg-et a kiadási artefaktumokban. Futásidőben a `PATH`-on észleljük, és a dokumentáció kimondja, hogy a felhasználó telepíti.
6. **AGPL forrásajánlat.** Az `osatura-cli` binárisa a `/source` címen kiszolgálja a saját forrásajánlatát (AGPL 13. szakasz), és a `--version` kapcsolóval közleményt ír ki; a `release.yml` fordítási időben az `OSATURA_SOURCE_URL` értékét `<server>/<repository>/tree/<commit>` címre állítja, hogy a kiadási binárisok megnevezzék azt a pontos forrást, amelyből épültek (a munkafolyamat füsttesztje ellenőrzi ezt a `--version` kimenetében; egy sima `cargo build`-nek nincs beépített címe); a módosított stúdiószerverek üzemeltetői a `--source-url` opciót arra a helyre állítják, ahol a módosított forrásuk van. Tesztek: `tests/source_offer.rs`. Bináris kiadáshoz a megfelelő forrást (tag, tarball) a felhasználók által elérhető URL-en kell biztosítani.
7. **Dokumentáció.** Frissítsd a README állapotdobozát, a [LIMITATIONS.md](LIMITATIONS.md) fájlt és a CHANGELOG-ot (az *Unreleased* szakaszt told át az új verzió alá dátummal).
8. **A vázlat felépítése a GitHubon.** A `.github/workflows/release.yml` a `main` minden pushánál megépíti a Windows, macOS és Linux archívumokat és a webes modult; az a `main`-beli commit, amelynek üzenete tartalmazza a `[release vX.Y.Z-label]` jelölést (vagy egy pusholt `v*` tag), egy *előzetes kiadás-vázlatot* is létrehoz az archívumokkal és a `SHA256SUMS.txt` fájllal (az azonos című régebbi vázlat lecserélődik). Nézd át a vázlatot, majd tedd közzé kézzel.
9. **Címkézés és közzététel** aláírt taggel; csatolj ellenőrzőösszegeket. Ne tegyél közzé crate-eket olyan néven, amely nem a tiéd; a forkokra a TRADEMARK.md szabályai vonatkoznak.

## 8. Folyamatos integráció

Két különböző dolog osztozik ezen a szón (egy harmadik munkafolyamat, a `.github/workflows/release.yml` a kiadási archívumokat és a vázlatkiadást építi, lásd a 7. szakaszt):

* **Animációs CI** (`osatura ci`): *projektek* ellenőrzése (lint, golden renderek, teljesítmény, determinizmus). Teljes leírása a [CI.md](CI.md) fájlban: `osatura-ci.json` manifest, küszöbértékek, golden képek (`--update` kontra `bless`), jelentések, GitHub Actions és egyszerű shell példák (ott „example, not run" jelöléssel).
* **Kód-CI** ehhez a repository-hoz: a `.github/workflows/ci.yml` négyféle jobot tartalmaz: `test` (mátrix ubuntu / windows / macos: `cargo test --workspace --exclude osatura-gpu --locked` és clippy `-D warnings`-szal), `wasm32` (megépíti az `osatura-wasm`-ot `wasm32-unknown-unknown` célra, a modult az `osatura.js` mellé másolja, és lefuttatja a `node osatura-wasm/js/test.mjs` parancsot egy demóprojekttel, a valódi-modul részét is beleértve), `gpu parity` (tájékoztató jellegű; adapter hiányában kihagyja magát, a Windows szoftveres DX12 adapterén fut) és `licences` (`cargo-deny check` és `reuse lint`). A toolchain rögzített (1.97.0), hogy az új clippy-lintek ne törjék el a buildet. A `test` job az `osatura-gui`-t is megépíti, és lefuttatja az `osatura-studio --selftest` parancsot (Windowson és macOS-en valódi önteszt). **Az első teljesen zöld futás: 2026-10-24** GitHub-hosztolt runnereken. Hiba esetén a lépések a hibás teszteket workflow-annotációként írják ki (naplóhozzáférés nélkül is olvashatók). A Windows kihagyja a csak POSIX-en futó teszteket (symlinkek, jogosultságok, `/dev/zero`, POSIX útvonalelválasztók). A parancsok, amelyekből felépül, ahogy a dokumentáció írása közben futtattuk őket:

  ```sh
  cargo test --workspace --locked
  cargo test --doc -p osatura-core --locked
  cargo clippy --workspace --all-targets --locked -- -D warnings
  scripts/licence-audit.sh            # needs python3 and (optional) cargo-deny
  ```

  Cache-eld a `~/.cargo/registry` és a `target/` könyvtárat a futások között, az e2e tesztnek adj több percet debug buildben, a Chromium smoke-teszteket pedig csak olyan runnereken futtasd, amelyeken van Chromium és `playwright-core`.

## 9. Hibakeresés

* **Backtrace-ek.** `RUST_BACKTRACE=1 osatura ...` bármely pánikhoz (a bemenet okozta pánik hiba; kérjük, jelezd azzal a projekttel együtt, amely kiváltja).
* **Géppel olvasható kimenet.** `--json` a támogató parancsoknál; `--quiet` szkriptekhez. Kilépési kódok: 0 siker esetén, 1 hiba esetén (megbukott ellenőrzés, teljesítetlen keret, hibás használat; run: egy ismeretlen parancs és egy hiányzó argumentum egyaránt 1-gyel lép ki); az `osatura ci` megkülönbözteti az 1-et (egy ellenőrzés megbukott) a 2-től (hiba).
* **Projekt vizsgálata.** Az `osatura check`, az `osatura lint` (hibáknál 1-gyel lép ki) és az `osatura diff a.json b.json` renderelés nélkül ad képet a szerkezetről és a problémákról; az `osatura render project.json N out.png` egy képkockát renderel, az `out.svg` pedig vektoros kimenetet ad, amelyet böngészőben megnyithatsz (a `--bones` a vázat mutatja).
* **Stúdió.** Futtasd az `osatura studio project.json --port 0` parancsot (kiírja a tényleges portot). A böngésző fejlesztői eszközei megmutatják a JSON API-t: a kérések egyszerű `POST /api/...` hívások `Content-Type: application/json` fejléccel. A 421 olyan Host fejlécet jelent, amely nincs az `--allow-host` listában; a 403 azt, hogy egy Origin vagy `Sec-Fetch-Site` ellenőrzés elbukott (proxy mögött használd a `--public-origin` opciót); a 415 a tartalomtípusra utal; a 429 a token back-off.
* **Demóállapotok.** A Stúdió vezetett demóját (és a projektoldal lépésenkénti demóját) az `osatura demo-person` kimenetéből generálja a `scripts/build-studio-tour.mjs` és a `scripts/lib/person-stages.mjs`; a build elbukik, ha a kész demó már nem egyezik a szövegekben idézett számokkal (`EXPECT` a könyvtárban). Az `osatura-core/src/demo.rs` módosítása után futtasd újra a szkriptet, és építsd újra a CLI-t.
* **Collab-napló.** Egy megosztott munkamenet naplója (`--collab-log`) szöveg, műveletenként egy `<hash> <json>` sorral; a megszakadt láncot betöltéskor jelzi.
* **Egy teszt csak olykor bukik.** Nézd meg az órát, az ideiglenes könyvtárak neveit és a `HashMap` bejárási sorrendjét; a motorkód konstrukciója szerint determinisztikus, így az időszakos hibák szinte mindig a teszt higiéniájából erednek.
* **A homlokzati pillanatkép elbukik.** Egy publikus szignatúrát módosítottál: lásd az 5. szakasz 6. pontját.
* **GPU.** `osatura render-gpu <project> <out.png> --adapter software|auto --bench N` (csak `--features gpu` buildekben) összeveti a CPU-rendererrel (a szoftveres adapter kiírja, hogy az időmérései semmit nem jelentenek); az `OSATURA_GPU_BACKEND` kiválasztja a backendet.
* **Szanitizerek a C ABI-hoz.** Fordítsd a `smoke.c` fájlt vagy a saját hívódat `-fsanitize=address,undefined` kapcsolóval a statikus könyvtárhoz linkelve (a dokumentációs munka során megtörtént, tiszta; lásd [C_ABI.md](C_ABI.md)).

## 10. Profilozás

* **A gyártási pipeline szakaszonkénti időmérése:** a `scripts/e2e.sh` táblázatot ír ki (az `OSATURA_E2E_TIMINGS` nevezi meg a fájlt); a `ROADMAP.md` 9. szakaszának release-számai ebből származnak.
* **GPU a CPU ellen:** `osatura render-gpu <project> <out.png> --bench N`.
* **Méret- és sebességkeret:** `osatura size-report`, `osatura export-fit --max-seconds` (az időkeretet a jelöltek ciklusain belül ellenőrzi; egy ffmpeg- vagy kvantáló-hívás nem szakítható meg).
* **Egy projekt renderelési költsége:** az `osatura ci` teljesítményküszöbbel ([CI.md](CI.md)) rögzíti a képkockánkénti időt a te gépeden; az időmérések géptől függenek, ezért ugyanazon a gépen hasonlíts.
* **Általános eszközök:** építs `--release` móddal, és használd a `perf record`/`perf report` vagy a `cargo flamegraph` eszközt (nem része a projektnek; külön telepítendő). A debug buildek nagyjából egy nagyságrenddel lassabbak, és semmit nem mondanak a valódi sebességről.
