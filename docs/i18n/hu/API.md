# OSATURA Rust API és stabilitási szabályzat

*Facade 1.4.0 verzió (`osatura_core::api::API_VERSION`). Célközönség: a motort beágyazó Rust-fejlesztők és a facade-et módosító karbantartók. A dokumentáció licence CC-BY-4.0; a kódminták licence MPL-2.0.*

Ez az `osatura-core` **támogatott Rust API-jának** referenciája: az `osatura_core::api` homlokzat-modul (facade) és annak egysoros importja, az `osatura_core::prelude`. Felsorol minden nyilvános elemet, az API-t stabilan tartó szabályokat és tesztelt példákat. A fájl minden példáját lefordítjuk és lefuttatjuk az aktuális forrásokon, mielőtt a fájl megjelenik (lásd [Hogyan ellenőrizzük a példákat](#hogyan-ellenőrizzük-a-példákat)).

Tartalom: [Mit fed le](#mit-fed-le) - [A crate beszerzése](#a-crate-beszerzése) - [Stabilitási szabályzat](#stabilitási-szabályzat) - [Semver-szabályok](#semver-szabályok) - [Feature flag-ek](#feature-flag-ek) - [Szálbiztonság](#szálbiztonság) - [Determinizmus](#determinizmus) - [Korlátok](#korlátok-és-argumentumtartományok) - [Hibák](#hibák) - [Referencia: `Animation`](#referencia-animation) - [Referencia: `Player`](#referencia-player) - [Referencia: önálló függvények](#referencia-önálló-függvények-eszközök-verziókezelés-ci) - [Referencia: típusok](#referencia-típusok) - [Példák](#példák) - [Más nyelvek](#más-nyelvek) - [Függelék: gépi és binding JSON](#függelék-gép-json-és-kötés-json)

## Mit fed le

Csak az `osatura_core::api` és az `osatura_core::prelude` a támogatott nyilvános Rust API. A facade a következőkből áll:

* `Animation`: egy validált projekt és annak visszavonási előzménye (betöltés, lekérdezés, renderelés, exportálás, szerkesztés, eszközök);
* `Player`: egy futó állapotgép elrendezéssel, görgetéssel és részletességi szinttel (1.1, 1.2);
* önálló függvények a verziókezeléshez és az animáció-CI-hoz (1.2 olvasás, 1.3 írás), valamint a bitmap-vektorizáláshoz (tracing);
* egyszerű értéktípusok (`AnimationInfo`, `RenderSettings`, `Image`, `LodLevel`, `SizeFit`, `MocapReport`, `FuzzSummary`, `VcsCommit`, `VcsConflict`, `CiOutcome`, `CiInit`, ...) és az `Interpolation`, `ExportFormat`, `FitFormat`, `CiMode`, `VcsMergeOutcome`, `VcsChoice` enumok;
* az `ApiError` hibatípus (megvalósítja a `std::error::Error` és a `Display` trait-et).

A C ABI (`osatura-capi`, `osatura.h` header) és a WebAssembly modul (`osatura-wasm`) kizárólag erre a facade-re épül, és saját verzióval rendelkezik: [C_ABI.md](C_ABI.md), [WASM.md](WASM.md).

### Mit NEM fed le

Az `osatura-core` minden más modulját (`engine`, `project`, `edit`, `svg`, `raster`, `physics`, `vcs`, `ci`, ...) és a gyökér újraexportjait (`Engine`, `Project`, ...). Ezek csak azért `pub` és `#[doc(hidden)]`, mert a parancssori eszköz, a Studio, a C ABI tesztjei és az egységtesztek használják őket; bármelyik kiadásban megváltozhatnak vagy eltűnhetnek, és az `Err(String)` szövegeik sem szerződés. Ha olyasmire van szükséged, ami csak ott érhető el, kérj a facade-be egy kiegészítést (minor kiadás), ahelyett hogy a belső részekre támaszkodnál.

A projekt JSON-formátumának saját `format` verziómezője van (jelenleg 1); egy újabb build által írt projektet az `InvalidJson` hibával elutasítunk. Lásd [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

## A crate beszerzése

Az OSATURA nincs közzétéve semmilyen registryben; útvonallal (vagy git-tel) függj tőle a workspace-edből:

```toml
[dependencies]
osatura-core = { path = "../osatura/osatura-core", default-features = false }
```

A `default-features = false` elhagyja a Rhai szkriptmotort (lásd [Feature flag-ek](#feature-flag-ek)); a facade-nek nincs rá szüksége. Ezután `use osatura_core::prelude::*;`. A crate verziója (`0.1.0`) és a facade verziója (`API_VERSION`) független egymástól: a facade a semvert követi, a crate verziója egyelőre semmit sem ígér. Minimálisan támogatott Rust-verzió nincs megadva; a workspace-et Rust 1.97-tel (edition 2021) fejlesztjük és teszteljük.

## Stabilitási szabályzat

1. **A surface-fájl.** Az `osatura-core/src/api_surface.txt` felsorolja az `src/api/` minden nyilvános elemét (típusok, mezők, variánsok, konstansok és metódus-szignatúrák, soronként egyet). Az `api::tests::api_surface_snapshot` egységteszt újraépíti ezt a listát az `api/`-ből, és elbukik, ha eltér a beléptetett fájltól. A surface minden változtatása ezért mindkét fájl tudatos, átnézett módosítása.
2. **A prelude-teszt.** Az `osatura-core/tests/prelude_surface.rs` *külső* crate-ként fordul. A facade neveit kizárólag az `osatura_core::prelude`-on keresztül importálja, és egy reprezentatív részhalmazt hív meg (betöltés, renderelés, szerkesztés, exportálás, `Player`, az eszközök, a verziókezelő és CI burkolók, hibás bemenet kezelése). Ha egy általa használt név már nem exportálódik újra a prelude-ből, vagy egy szignatúra olyan módon változik, amelyet ezek a hívások észrevesznek, a teszt nem fordul le. Az `API_VERSION == "1.4.0"` értéket is ellenőrzi, így a verziólépés itt is tudatos módosítás. Nem sorol fel minden prelude-nevet, ezért egy név hozzáadása az újraexport-listához továbbra is kézi lépés.
3. **Az ellenőrzés korlátai.** A pillanatkép az `api/` egysoros szignatúráinak szövegbeli átvizsgálása, nem fordítószintű diff, mint a `cargo-public-api`: azt a szemantikai változást, amely megtartja a szignatúra szövegét (például ugyanarra a bemenetre más hiba), nem veszi észre, a többsoros szignatúrát pedig elmulasztaná (a fájl ezért tartja egy sorban a szignatúrákat). A prelude újraexport-listáját (`lib.rs`) a külső teszt ellenőrzi, nem a pillanatkép.
4. **A rejtett nem jelent támogatottat.** A `#[doc(hidden)] pub` modulok a workspace-en belüli fogyasztóknak léteznek. Az ígéreten kívül esnek, még ha le is fordulnak nálad.
5. **Hogyan változtasd a facade-et** (karbantartóknak; szintén a [DEVELOPMENT.md](DEVELOPMENT.md) fájlban):
   * szerkeszd az `api/` fájlt (egysoros szignatúrák, doc comment, doc-teszt az új önálló függvényekhez, egységtesztek);
   * add hozzá az új nyilvános neveket a `lib.rs`-beli `prelude` újraexport-listához és a `tests/prelude_surface.rs` fájlhoz;
   * emeld az `API_VERSION` értékét (és az `API_VERSION_MAJOR` értékét, ha törő a változás) az alábbi szabályok szerint, valamint a `prelude_surface.rs`-ben ellenőrzött verziót;
   * a surface-fájlt tudatosan generáld újra: `OSATURA_BLESS=1 cargo test -p osatura-core api::`, majd nézd át az `api_surface.txt` diffjét;
   * frissítsd ezt a fájlt, az `osatura-capi` / `osatura-wasm` crate-et pedig csak akkor, ha az új elemnek át kell lépnie ezeket a határokat (az ABI-verziójuk külön mozog, a kiegészítések megtartják).

## Semver-szabályok

Az `API_VERSION` kizárólag a facade szemantikus verziója.

| Változás | Kiadás |
|---|---|
| Hibajavítás, nincs szignatúraváltozás, nincs új viselkedési szerződés | patch (1.4.x) |
| Új függvény, metódus, típus, enum-variáns, struct-mező, konstans | minor (1.x.0) |
| Bármi eltávolítása vagy átnevezése; szignatúra megváltoztatása, dokumentált argumentumtartomány olyan módosítása, amely korábban érvényes bemenetet elutasít, vagy egy dokumentált eredmény jelentésének megváltoztatása | major (2.0.0) |

Következmények a hívók számára:

* **A facade nyilvános enumjai és structjai `#[non_exhaustive]` jelölésűek.** Adj hozzá joker ágat, amikor az `ApiError`, `Interpolation`, `ExportFormat`, `FitFormat`, `CiMode`, `VcsMergeOutcome` vagy `VcsChoice` típusra `match`-elsz; egy új variáns minor kiadás. A structokat a crate-en kívül nem hozhatod létre literállal (az `Image`, a `SizeFit` és a riport-structok eredmények, amelyeket csak olvasol). A beállításokat konstruktorokkal építsd: `RenderSettings::new(scale).with_bones(true)`, `MocapSettings::new().with_smooth(2)`.
* **Az eredményeken belüli sztringek adatok, nem szerződés.** A `VcsConflict::kind` ma a `"BothChanged"`, `"DeleteEdit"`, `"KeyRetimed"`, `"OrderConflict"`, `"ArrayConflict"` valamelyike, és *minor kiadásokban új értékeket kaphat*. A JSON-riportok (`FuzzSummary::json`, `CiOutcome::json`) saját séma-nevet hordoznak (`osatura-fuzz-report` v1, `osatura-ci/1`). A hibaüzenetek szövege embereknek szól.
* **Viselkedési stabilitás.** A renderelés és a kiértékelés determinisztikus (lásd alább), de a pixelkimenet egy minor kiadásban megváltozhat, ha a rasterizálóban vagy egy effektben hibát javítunk; az animáció-CI által rögzített golden-ek erre emlékeztetnek. Ne hashelj renderelt pixeleket OSATURA-verziók vagy gépek között tűrés nélkül.

A facade verziótörténete: 1.0 (betöltés, lekérdezés, renderelés, exportálás, szerkesztés, visszavonás) - 1.1 (`Player`, állapotgépek, adatkötések) - 1.2 (elrendezés, görgetés és részletességi szint a `Player`-en; `lod_levels`, `fit_to_size`, `retarget_bvh`, `fuzz`, `from_image`, `trace_image_svg`, `vcs_log`, `ci_run`) - 1.3 (verziókezelő és CI írási műveletek, `DirtyTree` / `MergeInProgress` / `Conflicts`, `CiOutcome::updated` / `blessed`) - 1.4 (réteg- és csontlétrehozás, `bake_physics`, APNG- és EXR-exportálás, `vcs_branches` / `vcs_branch_delete` / `vcs_status` / `vcs_diff` / `vcs_show` / `vcs_gc` / `vcs_fsck`).

## Feature flag-ek

Az `osatura-core` egyetlen Cargo feature-rel rendelkezik:

| Feature | Alapértelmezett | Hatás |
|---|---|---|
| `scripting` | be | Lefordítja a Rhai-alapú, sandboxolt szkriptmodult (`script.rs`), amelyet csak a parancssori `script` parancs használ. Behúzza a `rhai` crate-et és rajta keresztül a `smartstring` crate-et (MPL-2.0+, az egyetlen gyenge copyleftes harmadik féltől származó crate a fában). |

A facade-ben **nincs** egyetlen funkció sem feature mögött: minden itt dokumentált elem létezik `default-features = false` mellett is. Az `osatura-capi`, az `osatura-wasm` és az `osatura-gpu` a core-tól e feature nélkül függ, így nem tartalmaznak MPL-licencű harmadik féltől származó crate-et; az `osatura-cli --no-default-features` ugyanezt teszi, és ilyenkor a `script` parancsot elérhetetlennek jelzi. A renderelő háttérrendszerek nem feature-ök: a facade mindig a CPU-s rasterizálót használja; az opcionális GPU-renderelő külön crate (`osatura-gpu`) és CLI feature (`gpu`), és ezen az API-n keresztül nem érhető el.

## Szálbiztonság

* Az `Animation`, a `Player`, az `ApiError` és az `Image` `Send + Sync` (a dokumentumhoz fordítási időben ellenőrizve). A facade nem ír globális állapotot; csak két folyamatszintű asset-beállítást olvas (az alapértelmezett asset-könyvtárat és a külső assetek kapcsolóját), amelyeket a facade nem módosít.
* A `&self`-et váró metódusok (`info`, `bone_world`, `render_*`, `export`, `lod_levels`, `fit_to_size`, `fuzz`, `player`, `to_json`, ...) ugyanazon az `Animation`-ön egyszerre több szálon is futhatnak; a `&mut self`-et váró metódusok (`set_key`, `delete_key`, `undo`, `redo`, `retarget_bvh`, minden `Player`-bemenet és az `advance`) kizárólagos hozzáférést igényelnek, ahogy Rustban szokás. Az alábbi `render_in_parallel` példa minden képkockát a saját szálán renderel, és ellenőrzi, hogy a képek megegyeznek az egyszálas eredménnyel.
* Nincs belső zárolás és nincs háttérszál. A `Player` a projekt saját másolatán dolgozik, így az `Animation` későbbi szerkesztése sosem érinti a futó lejátszót.
* Az `Animation::from_json` a relatív képfájlokat a folyamat **aktuális könyvtárához** képest oldja fel, az abszolút útvonalakat és a `..` elemet pedig elutasítja (lásd [SECURITY.md](SECURITY.md)); ha egy projekt fájlokra hivatkozik, ágyazd be a képeket data URI-ként, vagy betöltés előtt állítsd be a munkakönyvtárat.
* A verziókezelő és CI függvények fájlokon dolgoznak: nem biztonságosak, ha egy másik folyamat ugyanabban a pillanatban szerkeszti ugyanazt a projektet (a facade előzetesen ellenőrzi a piszkos fát és a függő merge-et, de zár nincs).

## Determinizmus

Ugyanaz a projekt, ugyanazok az argumentumok, ugyanaz az OSATURA-build és gép: ugyanazok a bájtok. Ez érvényes a `render_rgba` / `render_png` / `render_svg`, a `bone_world`, minden exportálás, a `lod_levels` és a `fuzz` (adott seed mellett), a `fit_to_size`, valamint a `Player` esetén (ugyanazok a bemenetek és ugyanazok az `advance` hívások ugyanazt a pózt adják; az `advance` az időt legfeljebb 1/60 s-os darabokra bontja, így az időfelosztás csak e felbontás alatt számít). A motoron belüli véletlen döntések (zaj, részecskék, fuzzing, tracing-klaszterek) seedelt generátorokat használnak, sosem az órát. Ami *nem* garantált: azonos anti-aliasing élpixelek különböző operációs rendszerek, CPU-k, fordítók vagy `libm`-verziók között, 32 és 64 bites célok között, illetve OSATURA-verziók között. A bájtpontos golden-eket csak egyetlen Linux x86-64 gépen, folyamatok között ellenőriztük (lásd [LIMITATIONS.md](LIMITATIONS.md)); minden gépek közötti célra használj tűrést (az animáció-CI is így tesz).

## Korlátok és argumentumtartományok

| Elem | Szabály |
|---|---|
| `MAX_JSON_BYTES` | 256 MiB; egy ennél hosszabb szöveg `InvalidJson` |
| `MAX_RENDER_SIDE` | oldalanként 16384 kimeneti pixel (`render_*` és `Player::render_*`) |
| Render- / GIF-skála | véges és `0.05..=4` között, különben `InvalidArgument` |
| Képkocka-pozíciók | végesnek kell lenniük; az idővonalon kívüli értékeket `0..=frames-1` közé **szorítjuk** (nincs hiba). A tört képkockák rendben vannak. |
| Kulcsok (`set_key`, `delete_key`) | a `frame` és a `value` véges; a `prop` a projekt JSON-jából származó snake_case tulajdonságnév (`"bone_angle"`, `"layer_opacity"`, ...) |
| Projekt | `fps` a `(0, 1000]` tartományban, `frames` az `1..=200000` tartományban, a színpad oldalanként legfeljebb 32768 egység, legfeljebb 4096 csont, a +-1e7-nél nagyobb számokat betöltéskor elutasítjuk; a további funkciónkénti korlátokat betöltéskor validáljuk, és `InvalidProject` formájában térnek vissza |
| `Player::advance(dt)` | a `dt` véges, `0..=10` másodperc |
| `Player::set_canvas_size` | mindkét oldal véges, `> 0`, legfeljebb 32768 |
| `Player::set_screen_size` | a szélesség és a magasság véges és `> 0`; a `max_error_px` véges és `>= 0` |
| `Animation::lod_levels` | `tolerance_px > 0`; a `scale` a `(0, 64]` tartományban; a `levels` az `1..=12` tartományban |
| `Animation::fit_to_size` | a `max_bytes` az `1000..=1_000_000_000` tartományban |
| `Animation::fuzz` | a `samples` az `1..=100000` tartományban |
| `trace_image_svg`, `Animation::from_image` | a `colors` a `2..=64` tartományban; PNG- vagy JPEG-bájtok (méretkorlátok érvényesek) |
| `vcs_log` | a `limit` az `1..=10000` tartományban |

## Hibák

Minden hibázhatós függvény `Result<_, ApiError>` értéket ad vissza. A rossz bemenet hibaérték és **sosem pánikol**; a motoron belül mégis bekövetkező pánikot a facade határán elkapjuk, és `ApiError::Internal` formájában adjuk vissza (kérjük, jelentsd hibaként). Az `ApiError` megvalósítja a `Display` és a `std::error::Error` trait-et, így a `?` működik a `Box<dyn Error>` és az `anyhow`-szerű típusokkal.

| Variáns | Jelentés | Tipikus források |
|---|---|---|
| `InvalidJson(String)` | nem JSON, nem projekt, rossz szerkezet, túl nagy (> `MAX_JSON_BYTES`), vagy az ennél a buildnél újabb `format`; továbbá a `vcs_resolve`-nak átadott hibás JSON-érték | `Animation::from_json`, `validate_json`, `vcs_resolve` |
| `InvalidProject(Vec<String>)` | feldolgozva, de a validáció elbukott; problémánként egy üzenet (ismeretlen csont, rossz tartomány, ciklus, ...) | `from_json`, `from_image`, a `Player` motor-újraépítései |
| `InvalidArgument(String)` | a hívó egy argumentumát elutasítottuk: nem véges szám, tartományon kívüli érték, ismeretlen tulajdonság, ismeretlen gép / bemenet / scroll id, használhatatlan BVH vagy kép, rossz ágnév, nincs mit commitolni, nincs repository | szinte minden függvény |
| `EditRejected(Vec<String>)` | egy szerkesztést elutasítottunk (például zárolt csatorna vagy réteg, vagy érvénytelen eredmény); a projekt változatlan | `set_key`, `delete_key`, `retarget_bvh` |
| `Export(String)` | egy kódoló elbukott, vagy a formátum nem lehetséges ennél a projektnél | `export`, `render_png`, `fit_to_size` |
| `Internal(String)` | elkapott pánik | bármelyik |
| `DirtyTree(String)` | (1.3) a projektfájlban commitolatlan változások vannak, amelyeket a művelet felülírna | `vcs_checkout` (kivéve `force` esetén), `vcs_merge`, `vcs_cherry_pick`, `vcs_revert` |
| `MergeInProgress(String)` | (1.3) egy merge, cherry-pick vagy revert befejezetlen; oldd fel és commitold, vagy `vcs_abort` | ugyanezek a függvények |
| `Conflicts(Vec<String>)` | (1.3) `vcs_commit`, miközben konfliktusok maradtak; konfliktusonként egy mutató | `vcs_commit` |

Egy elbukó animáció-CI ellenőrzés **nem** hiba: a `ci_run` `Ok(CiOutcome)` értéket ad 1-es (egy ellenőrzés elbukott) vagy 2-es (egy ellenőrzést nem lehetett kiértékelni) `exit_code`-dal; csak a használhatatlan manifest `Err`. A merge-konfliktusok hasonlóan eredmények (`VcsMergeOutcome::Conflicted`), nem hibák.

## Referencia: `Animation`

Egy betöltött, validált projekt és annak visszavonási előzménye. Hozd létre a `from_json`, a `demo` vagy a `from_image` függvénnyel. A projektet olvasó összes metódus `&self`-et vár.

| Metódus | Visszatérés | Megjegyzések |
|---|---|---|
| `Animation::from_json(&str)` | `Result<Animation, ApiError>` | feldolgozás + validálás. `InvalidJson` / `InvalidProject`. Az ismeretlen JSON-mezőket itt csendben figyelmen kívül hagyja (a CLI `check` parancsa jelzi őket). |
| `Animation::validate_json(&str)` | `Result<(), ApiError>` | ugyanazok az ellenőrzések, semmit sem tart meg |
| `Animation::demo()` | `Animation` | kis beépített rig (6 csont, 48 képkocka 24 fps-sel, 640 x 480); tesztekhez és az első lépésekhez hasznos |
| `Animation::from_image(&[u8], colors, name)` | `Result<Animation, ApiError>` | a PNG/JPEG bájtokat kitöltött vektorrétegekké vektorizálja egy, a kép méretű színpadon |
| `to_json(&self)` | `String` | az aktuális projekt szépen formázott JSON-ként (körútra alkalmas a `from_json`-nal) |
| `info(&self)` | `AnimationInfo` | `name`, `width`, `height` (színpad-egységek), `fps`, `frames` |
| `frame_count(&self)`, `fps(&self)` | `u32`, `f32` | |
| `bone_names(&self)` | `Vec<String>` | rig-sorrend = a `bone_world` sorrendje |
| `layer_names(&self)` | `Vec<String>` | `group/child` rétegútvonalak, mélységi bejárásban |
| `bone_world(&self, frame)` | `Result<Vec<[f32; 6]>, ApiError>` | csontonkénti világmátrix `[a b c d tx ty]`; egy csont-lokális `(x, y)` pont a `(a*x + c*y + tx, b*x + d*y + ty)` pontra képződik le. A képkocka szorítva. Kiértékeli a teljes rigot (IK, kényszerek, rugók, kifejezések). |
| `machine_names(&self)` | `Vec<String>` | a projektben tárolt futásidejű állapotgépek |
| `player(&self, machine)` | `Result<Player, ApiError>` | a megnevezett gépet a 0. időpontban indítja a projekt egy másolatán |
| `render_rgba(&self, frame, &RenderSettings)` | `Result<Image, ApiError>` | egyenes alfás RGBA8, soronként, a felső sor elöl |
| `render_png(&self, frame, &RenderSettings)` | `Result<Vec<u8>, ApiError>` | egy PNG-fájl a memóriában |
| `render_svg(&self, frame, show_bones: bool)` | `Result<String, ApiError>` | egy SVG-dokumentum (a vektoros kimenetnek nincs skálája) |
| `export(&self, ExportFormat, scale)` | `Result<Vec<u8>, ApiError>` | `Gif` (ismétlődő, a teljes idővonal; a `scale` `0.05..=4` között), `Glb` (bináris glTF 2.0), `Lottie` (JSON-bájtok), `Html` (egyetlen önálló lejátszóoldal); a `scale` értékét a másik három figyelmen kívül hagyja. Hogy az egyes formátumok mit tartanak meg vagy hagynak el: [EXPORT_FORMATS.md](EXPORT_FORMATS.md). |
| `set_key(&mut self, target, prop, frame, value, Interpolation)` | `Result<(), ApiError>` | egy kulcs beállítása vagy hozzáadása a `target.prop` tulajdonságon; validált, visszavonható, tiszteletben tartja a csatorna- és rétegzárakat (`EditRejected`) |
| `delete_key(&mut self, target, prop, frame)` | `Result<bool, ApiError>` | `true`, ha egy kulcsot eltávolított |
| `add_rect_layer(&mut self, name, x, y, w, h, fill, parent)`, `add_ellipse_layer(&mut self, name, cx, cy, rx, ry, fill, parent)` | `Result<(), ApiError>` | (1.4) kitöltött alakzatréteg hozzáfűzése (a legfelső szintre, vagy a `parent` által megnevezett legfelső szintű csoportba); egyedi, `/` nélküli név, véges pozitív méretek, érvényes szín; visszavonható |
| `add_group_layer(&mut self, name)` | `Result<(), ApiError>` | (1.4) üres csoportréteg hozzáfűzése |
| `add_bone(&mut self, name, parent, length, x, y, angle)` | `Result<(), ApiError>` | (1.4) csont hozzáfűzése (a `parent`-nek léteznie kell, a szög fokban); visszavonható |
| `bake_physics(&mut self, chain, preset, overwrite)` | `Result<PhysicsBake, ApiError>` | (1.4) egy csontlánc szimulálása (a gyökér elöl) `hair`, `tail`, `rope`, `antenna` vagy `cape` előbeállítással, és a szögcsatornák eltárolása egyetlen visszavonási lépésként; `overwrite = false` esetén a kézzel kulcsolt csontokat megtartja, és a `skipped` alatt felsorolja |
| `render_exr(&self, frame, &RenderSettings, float32)` | `Result<Vec<u8>, ApiError>` | (1.4) OpenEXR-fájl (lineáris, premultiplikált; HALF vagy FLOAT) a lebegőpontos kompozitorból |
| `undo(&mut self)`, `redo(&mut self)` | `bool` | `false`, ha nincs mit visszavonni / újra végrehajtani |
| `lod_levels(&self, tolerance_px, scale, levels)` | `Result<Vec<LodLevel>, ApiError>` | a 0. szint az eredeti, a k >= 1 szint `scale` mellett `tolerance_px * 2^(k-1)` kimeneti pixelen belül marad; minden szint hordozza a deklarált korlátját és egy projekt-JSON-t. Determinisztikus; a korlát az idővonal-lejátszásra vonatkozik (az állapotgép-pózokra nem). Optimalizálatlan buildekben lassú (használj release-t). |
| `fit_to_size(&self, FitFormat, max_bytes)` | `Result<SizeFit, ApiError>` | a keretbe férő legkisebb exportálás a minőségi gombok lépésenkénti csökkentésével; ha lehetetlen, a legkisebb eredmény `met == false` értékkel és egy `reason`-nel |
| `retarget_bvh(&mut self, bvh_text, &MocapSettings)` | `Result<MocapReport, ApiError>` | BVH-retarget **egyetlen visszavonható szerkesztésként**; a csontokat név és topológia alapján párosítja; a zárolt csatornákat kihagyja és jelzi |
| `fuzz(&self, samples, seed)` | `Result<FuzzSummary, ApiError>` | póz-fuzzing; az animációt csak olvassa; a teljes riport JSON-ként |

A kulcsnál induló szakasz `Interpolation` értéke: `Linear`, `Step` (tartás), `EaseIn`, `EaseOut`, `EaseInOut`, `Smooth` (Catmull-Rom a szomszédos kulcsokon át). A projektformátum gazdagabb görbéit (köbös Bézier, bounce, back, spring) a CLI-vel és a Studióval lehet szerkeszteni, a `set_key`-jel nem.

## Referencia: `Player`

Egy futó állapotgép (interaktív lejátszás). Az `Animation::player` hozza létre; a projekt egy másolatán dolgozik; determinisztikus. Az állapotgépek és adatkötések egyszerű projekt-JSON-ok (`machines`, `bindings`), amelyeket a [függelék](#függelék-gép-json-és-kötés-json) foglal össze.

| Metódus | Visszatérés | Megjegyzések |
|---|---|---|
| `set_bool(name, bool)`, `set_number(name, f64)`, `fire(name)` | `Result<(), ApiError>` | egy bemenet beállítása név szerint; a rossz fajta bemenet vagy az ismeretlen név `InvalidArgument`. Egy trigger beállítva marad, amíg egy, őt tesztelő átmenet le nem fut (elfogyasztódik) vagy a következő `advance` végéig; a `hold: true` az elfogyasztásáig megtartja. |
| `set_value(path, f64)` | `Result<(), ApiError>` | külső érték közzététele az adatkötések számára (`"player.health"`). Az a scroll-régió, amelynek `input` értéke ez az útvonal, az értéket a főtengelye menti pixeles eltolásként veszi (a görgetés ekkor nyugalomban van). |
| `advance(dt)` | `Result<(), ApiError>` | a gép és a scroll-régiók előreléptetése `dt` másodperccel (`0..=10`) |
| `state(layer)` | `Option<String>` | egy gépréteg aktuális állapota; `None` ismeretlen rétegre |
| `events_json()` | `String` | függő események `[{"time":..,"layer":..,"kind":"entered"|"exited","state":..}]`, nem fogyasztódnak el |
| `take_events_json()`, `clear_events()` | `String`, `()` | a függő események elfogyasztása / eldobása |
| `render_rgba(&RenderSettings)`, `render_svg(show_bones)`, `bone_world()` | mint az `Animation`-on | az aktuális póz; a kép mérete a `set_canvas_size` által beállított vászonméret szorozva a skálával |
| `set_canvas_size(w, h)`, `canvas_size()` | `Result<(), ApiError>`, `(f32, f32)` | (1.2) reszponzív elrendezés: a konténerek újraszámolódnak, a görgetési határok követik |
| `scroll_ids()` | `Vec<String>` | a projekt scroll-régióinak id-i |
| `scroll_drag(id, x, y)` | `Result<(), ApiError>` | nyers mutató-eltolás pixelben (a tartalom ennyivel feljebb / balra mozdul); a végeken túl szorítva vagy gumiszalag-szerűen húzva |
| `scroll_release(id, vx, vy)` | `Result<(), ApiError>` | elengedési sebesség px/s-ban; az `advance` ezután lendít, beilleszt és rugóval visszaugrat |
| `scroll_offset(id)`, `scroll_max(id)`, `scroll_settled(id)` | `Option<[f32; 2]>`, `Option<[f32; 2]>`, `Option<bool>` | `None` ismeretlen id-ra |
| `set_screen_size(w_px, h_px, max_error_px)` | `Result<(), ApiError>` | (1.2) a nyolc előre épített részletességi szint közül a legdurvábbat választja, amelynek deklarált korlátja a vásznat az adott képernyőre illesztő skálán legfeljebb `max_error_px`. A szintek első használatkor épülnek (az 1. szintet 0,125 színpad-egység korlátozza, majd duplázódik) és gyorsítótárba kerülnek; `max_error_px` 0 esetén csak a veszteségmentes szintek jöhetnek szóba. |
| `clear_screen_size()`, `lod_level()`, `lod_bound_px()` | `()`, `usize`, `f32` | vissza az eredetihez; a használatban lévő szint (0 = eredeti); a deklarált korlátja pixelben |

Megjegyzések: a részletességi szint korlátja a geometriára és az idővonal-kulcsokra vonatkozik, ahogy a `lod/mod.rs` dokumentálja (`ASSUMPTIONS`, `LIMITS`); az állapotgép-pózokat és a kötéseket nem modellezi, és az olyan réteget célzó pózcsatornák, amelyet egy szint eltávolított, elmaradnak. A Studio scrub-proxyjai külön mechanizmust használnak saját szintrácssal. A scroll-régió `input` értéke a lejátszó értéktábláján megy át, nem az adatkötés-futtatókörnyezeten, így a kifejezések, az újraleképezés és a simítás nem vonatkoznak rá.

## Referencia: önálló függvények (eszközök, verziókezelés, CI)

| Függvény | Visszatérés | Feladata |
|---|---|---|
| `trace_image_svg(&[u8], colors)` | `Result<String, ApiError>` | egy bitmap vektorizálása kitöltött régiókból álló SVG-vé |
| `vcs_log(path, limit)` | `Result<Vec<VcsCommit>, ApiError>` | (1.2, csak olvasás) az előzmény a legújabbtól; a `path` a projektfájl vagy bármely útvonal a repository-ján belül; üres előzmény üres listát ad |
| `ci_run(manifest)` | `Result<CiOutcome, ApiError>` | (1.2) animáció-CI ellenőrző módban (semmit sem rögzít újra); a manifestben lévő relatív útvonalak mellette oldódnak fel |
| `vcs_init(project_file, author)` | `Result<(), ApiError>` | (1.3) előzmény indítása a fájl mellett a `main` ágon, még commit nélkül; elbukik, ha már van |
| `vcs_commit(path, message, author)` | `Result<String, ApiError>` | commit id; befejez egy merge-et, ha nem maradt konfliktus; `Conflicts`, amíg maradnak; a nincs mit commitolni vagy a nincs üzenet `InvalidArgument` |
| `vcs_branch(path, name, at)`, `vcs_tag(path, name, at)` | `Result<String, ApiError>` | ág / címke létrehozása (alapértelmezés: az aktuális commit); nem vált rá; a címkék sosem mozognak; a commit id-t adja vissza |
| `vcs_checkout(path, target, force)` | `Result<String, ApiError>` | a fájlt egy ágra (rajta marad) vagy bármely revízióra (detached) váltja; `DirtyTree`, hacsak nincs `force`; `MergeInProgress` |
| `vcs_merge(path, rev, no_ff, message, author)` | `Result<VcsMergeOutcome, ApiError>` | `UpToDate`, `FastForward(id)`, `Merged(id)` vagy `Conflicted(Vec<VcsConflict>)` |
| `vcs_cherry_pick(path, rev, author)`, `vcs_revert(path, rev, author)` | `Result<VcsMergeOutcome, ApiError>` | egy commit változásának újrajátszása / visszavonása |
| `vcs_conflicts(path)` | `Result<Vec<VcsConflict>, ApiError>` | a még függő konfliktusok (üres, ha nincs) |
| `vcs_resolve(path, conflict_path, &VcsChoice)` | `Result<usize, ApiError>` | egy konfliktus rendezése `Ours`, `Theirs` vagy `Value(json text)` értékkel; azt adja vissza, hány maradt |
| `vcs_abort(path)` | `Result<(), ApiError>` | egy befejezetlen merge / cherry-pick / revert elvetése és az utolsó commit visszaállítása |
| `vcs_branches(path)` | `Result<Vec<VcsBranch>, ApiError>` | (1.4) az ágak a commitjukkal és azzal, hogy ki van-e checkoutolva |
| `vcs_branch_delete(path, name)` | `Result<(), ApiError>` | (1.4) ág törlése; a checkoutolt ágat és az ismeretlen neveket elutasítja; az objektumok a `vcs_gc`-ig megmaradnak |
| `vcs_status(path)` | `Result<VcsStatus, ApiError>` | (1.4) ág, HEAD, nem commitolt szemantikai változások, befejezetlen merge, konfliktus-útvonalak |
| `vcs_diff(path, a, b)` | `Result<Vec<VcsChange>, ApiError>` | (1.4) szemantikai különbségek két revízió között; a `None` a munkafájl |
| `vcs_show(path, rev)` | `Result<VcsShow, ApiError>` | (1.4) egy commit és változásai az első szülőjéhez képest |
| `vcs_gc(path)` | `Result<VcsGc, ApiError>` | (1.4) az elérhetetlen objektumok törlése; törlés nélkül megszakad, ha az előzmény nem olvasható |
| `vcs_fsck(path)` | `Result<Vec<String>, ApiError>` | (1.4) integritási problémák, egészséges állapotban üres |
| `ci_init(project, manifest, write_goldens)` | `Result<CiInit, ApiError>` | alapértelmezett kalibrált manifest írása (meglévő fájlt sosem ír felül; alapértelmezett neve `osatura-ci.json` a projekt mellett); opcionálisan a golden-eket is rögzíti most |
| `ci_run_mode(manifest, CiMode)` | `Result<CiOutcome, ApiError>` | `Verify`, `Update` (a renderelő ellenőrzések összes golden-jének újrarögzítése), `Bless` (csak a bukó vagy hiányzók elfogadása) |
| `ci_bless(manifest)` | `Result<CiOutcome, ApiError>` | a `Bless` rövidítése |

Az előzmény a projektfájl mellett tárolódik (`.osatura-vcs/`); a commit-üzenetek és a szerzők egyszerű szövegek. Egy konfliktus tartalmazza az azonosító-mutatót (`path`), egy `kind` értéket, egy `description` értéket, valamint a `base` / `ours` / `theirs` értékeket JSON-szövegként (`None` = ott hiányzik). Egy konfliktusos merge után a projektfájl az összefésült állapotot tartalmazza, a konfliktusos elemek az aktuális ág értékén, és függő állapot van rögzítve, amíg minden konfliktust fel nem oldasz és `vcs_commit`-ot nem hívsz, vagy `vcs_abort`-ot. A projekt által hivatkozott bináris assetek nincsenek verziózva, csak az útvonal-sztringjeik. A szemantikai merge-et, annak konfliktusfajtáit és korlátait a [ROADMAP.md](ROADMAP.md) 9. szakasza (I-01 elem) és a [LIMITATIONS.md](LIMITATIONS.md) írja le.

Nincs a facade-ben: a mocap csontleképezés-felülbírálásai, a vektorizálás finomhangolása a színszámon túl, a fit opciói a formátumon és a kereten túl, a GPU-s renderelés, a Studio, az együttműködés, az ügynökszerver, a sok szerkesztő és baking parancs (fizika, foot lock, mágnesek, autorig, ...), az ágak törlése / listázása, diff, show, status, gc, fsck, valamint a CI-opciók, mint a prune, az `--only` és az artifactok. Ezekhez használd a CLI-t ([CLI_REFERENCE.md](CLI_REFERENCE.md)).

## Referencia: típusok

| Típus | Mezők / variánsok |
|---|---|
| `AnimationInfo` | `name: String`, `width: f32`, `height: f32` (színpad-egységek), `fps: f32`, `frames: u32` |
| `RenderSettings` | `scale: f32` (0.05..=4), `show_bones: bool`, `transparent: bool`; `new(scale)`, `with_bones(bool)`, `with_transparent(bool)`; `Default` = 1-es skála, mindkettő kikapcsolva |
| `Image` | `width: u32`, `height: u32`, `pixels: Vec<u8>` (RGBA8, egyenes alfa) |
| `ExportFormat` | `Gif`, `Glb`, `Lottie`, `Html`, `Apng` (1.4) |
| `PhysicsBake` (1.4) | `added: usize`, `skipped: Vec<String>` |
| `VcsBranch` (1.4) | `name`, `commit`, `current: bool` |
| `VcsChange` (1.4) | `kind` (`"Added"`, `"Removed"`, `"Changed"`, `"Moved"`, `"Retimed"`), `path`, `description` |
| `VcsStatus` (1.4) | `branch`, `head`, `changes: Vec<VcsChange>`, `merge`, `conflicts: Vec<String>` |
| `VcsShow` (1.4) | `commit: VcsCommit`, `changes: Vec<VcsChange>` |
| `VcsGc` (1.4) | `kept`, `removed` |
| `Interpolation` | `Linear`, `Step`, `EaseIn`, `EaseOut`, `EaseInOut`, `Smooth` |
| `LodLevel` | `level`, `declared_px: f64`, `bound_stage: f64`, `path_anchors`, `keys`, `layers`, `json: String`; `bound_at(screen_scale) -> f64` |
| `FitFormat` | `Lottie`, `Html`, `Gif`, `Apng`, `Project` |
| `SizeFit` | `met: bool`, `size: usize`, `bytes: Vec<u8>`, `ssim_min: f64`, `reason: Option<String>` |
| `MocapSettings` | `in_place`, `mirror`, `smooth` (mozgóátlag-sugár képkockákban); `new()`, `with_in_place`, `with_mirror`, `with_smooth`; `Default` |
| `MocapReport` | `matched: Vec<(String, String)>` (BVH-ízület, rig-csont), `unmatched_joints`, `unmatched_bones`, `frames`, `bones_used`, `blocked` (zárolt `target.prop`), `notes` |
| `FuzzSummary` | `samples`, `failures`, `failure_rate: f32`, `max_severity: f32`, `json: String` |
| `VcsCommit` | `id`, `author`, `time: u64` (Unix-másodpercek), `message` |
| `VcsConflict` | `path`, `kind`, `description`, `base`, `ours`, `theirs: Option<String>` |
| `VcsMergeOutcome` | `UpToDate`, `FastForward(String)`, `Merged(String)`, `Conflicted(Vec<VcsConflict>)` |
| `VcsChoice` | `Ours`, `Theirs`, `Value(String)` |
| `CiMode` | `Verify`, `Update`, `Bless` |
| `CiOutcome` | `total`, `passed`, `failed`, `errors`, `exit_code: i32` (0 jó, 1 elbukott, 2 nem kiértékelhető), `json: String`, `updated`, `blessed` |
| `CiInit` | `manifest: String`, `checks: Vec<(String, String)>` (fajta, név), `recorded: Option<CiOutcome>` |
| Konstansok | `API_VERSION: &str`, `API_VERSION_MAJOR: u32`, `MAX_RENDER_SIDE: u32`, `MAX_JSON_BYTES: usize` |

## Példák

Mindegyik teljes, és le lett futtatva. Feltételezik a `use osatura_core::prelude::*;` sort.

### Betöltés, kiértékelés, renderelés, exportálás

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Load a project, query it, render one frame and export the whole animation.
pub fn quick(project_path: &str) -> Result<(), Box<dyn Error>> {
    let json = std::fs::read_to_string(project_path)?;
    let mut anim = Animation::from_json(&json)?;           // parse + validate; typed error otherwise

    // Facts about the project.
    let info = anim.info();
    println!("{}: {}x{} @ {} fps, {} frames", info.name, info.width, info.height, info.fps, info.frames);

    // Evaluate: world matrices of every bone at frame 10 (six floats per bone: a b c d tx ty).
    let names = anim.bone_names();
    let world = anim.bone_world(10.0)?;
    let m = world[0];
    // A bone-local point (x, y) lands at (a*x + c*y + tx, b*x + d*y + ty).
    let (x, y) = (m[0] * 10.0 + m[2] * 0.0 + m[4], m[1] * 10.0 + m[3] * 0.0 + m[5]);
    println!("tip of {} (10 units along its x axis) is at ({x:.1}, {y:.1})", names[0]);

    // Edit through the undoable editor, then render.
    anim.set_key(&names[0], "bone_angle", 10.0, 25.0, Interpolation::EaseInOut)?;
    let png = anim.render_png(10.0, &RenderSettings::new(1.0).with_bones(true))?;
    std::fs::write("frame10.png", &png)?;
    let svg = anim.render_svg(10.0, false)?;
    std::fs::write("frame10.svg", svg)?;

    // Raw pixels: straight-alpha RGBA8, row-major, top row first.
    let img = anim.render_rgba(10.0, &RenderSettings::new(0.5).with_transparent(true))?;
    assert_eq!(img.pixels.len(), img.width as usize * img.height as usize * 4);

    // Whole-animation exports.
    let glb = anim.export(ExportFormat::Glb, 1.0)?;
    let gif = anim.export(ExportFormat::Gif, 0.5)?;        // `scale` only matters for GIF
    std::fs::write("anim.glb", &glb)?;
    std::fs::write("anim.gif", &gif)?;
    println!("{} bones, {} bytes of glb, {} bytes of gif", names.len(), glb.len(), gif.len());
    Ok(())
}
```

### Szerkesztés, visszavonás és hibakezelés

```rust
use osatura_core::prelude::*;

/// Edits are validated, undoable and all-or-nothing.
pub fn edit_and_errors() {
    let mut anim = Animation::demo();                       // small built-in project
    let bone = anim.bone_names()[0].clone();
    let before = anim.to_json();

    anim.set_key(&bone, "bone_angle", 4.0, 30.0, Interpolation::Smooth).unwrap();
    assert!(anim.delete_key(&bone, "bone_angle", 4.0).unwrap());      // true: a key was removed
    assert!(!anim.delete_key(&bone, "bone_angle", 77.0).unwrap());    // false: nothing there
    assert!(anim.undo() && anim.undo());                    // each successful edit is one undo step
    assert_eq!(anim.to_json(), before);
    assert!(anim.redo());

    // Errors are values. Match on them with a wildcard arm: the enum is #[non_exhaustive].
    match anim.set_key(&bone, "no_such_prop", 0.0, 1.0, Interpolation::Linear) {
        Err(ApiError::InvalidArgument(why)) => println!("rejected argument: {why}"),
        Err(ApiError::EditRejected(problems)) => println!("edit refused: {problems:?}"),
        Err(other) => println!("other failure: {other}"),   // Display is implemented for every variant
        Ok(()) => unreachable!(),
    }
    // A rejected edit leaves the project exactly as it was.
    let snapshot = anim.to_json();
    assert!(anim.set_key("no_such_bone", "bone_angle", 1.0, 1.0, Interpolation::Linear).is_err());
    assert_eq!(anim.to_json(), snapshot);

    // Bad input never panics.
    assert!(matches!(Animation::from_json("{ nope"), Err(ApiError::InvalidJson(_))));
    assert!(anim.render_rgba(f32::NAN, &RenderSettings::default()).is_err());
    assert!(anim.render_rgba(0.0, &RenderSettings::new(50.0)).is_err()); // scale must be 0.05..=4
}
```

### Interaktív lejátszás állapotgéppel

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Run a state machine of a project (machine.json has machine "hero", layer "body", bool input "excited").
pub fn play(project_path: &str) -> Result<(), Box<dyn Error>> {
    let anim = Animation::from_json(&std::fs::read_to_string(project_path)?)?;
    let machine = anim.machine_names()[0].clone();         // e.g. "hero"
    let mut player = anim.player(&machine)?;               // works on a copy of the project

    player.set_bool("excited", true)?;                     // also set_number, fire (trigger), set_value (binding path)
    for _ in 0..30 {
        player.advance(1.0 / 30.0)?;                       // deterministic; dt in seconds, 0..=10
    }
    println!("body is in state {:?}", player.state("body"));  // Some("wave")
    println!("{}", player.take_events_json());             // [{"time":..,"layer":"body","kind":"entered","state":..}, ..]

    let frame = player.render_rgba(&RenderSettings::new(0.5))?;
    println!("pose rendered at {}x{}", frame.width, frame.height);
    let svg = player.render_svg(false)?;
    assert!(svg.starts_with("<svg"));

    // Same inputs, same advance calls, same pose: a second player is bit-for-bit identical.
    let mut again = anim.player(&machine)?;
    again.set_bool("excited", true)?;
    for _ in 0..30 {
        again.advance(1.0 / 30.0)?;
    }
    assert_eq!(again.render_rgba(&RenderSettings::new(0.5))?, frame);

    // Wrong calls are typed errors, not panics.
    assert!(matches!(player.set_number("excited", 1.0), Err(ApiError::InvalidArgument(_))));
    assert!(matches!(anim.player("no such machine"), Err(ApiError::InvalidArgument(_))));
    Ok(())
}
```

### Elrendezés, görgetés és részletességi szint

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Layout, scrolling and level of detail on a Player (scroll.json has one scroll region "list").
pub fn layout_and_lod(project_path: &str) -> Result<(), Box<dyn Error>> {
    let anim = Animation::from_json(&std::fs::read_to_string(project_path)?)?;
    let mut player = anim.player(&anim.machine_names()[0])?;

    player.set_canvas_size(100.0, 120.0)?;                 // containers re-solve; renders get this size
    for id in player.scroll_ids() {
        let max = player.scroll_max(&id).unwrap();         // about [0, 80] here: 200 units of content in a 120 unit view
        player.scroll_drag(&id, 0.0, 50.0)?;               // raw pointer offset in px (clamped / rubber-banded)
        assert_eq!(player.scroll_offset(&id), Some([0.0, 50.0]));
        player.scroll_release(&id, 0.0, 900.0)?;           // fling at 900 px/s
        while player.scroll_settled(&id) == Some(false) {
            player.advance(1.0 / 60.0)?;                   // steps the machine and every scroll region
        }
        println!("{id}: max {max:?}, rests at {:?}", player.scroll_offset(&id).unwrap());
    }

    // Pick the coarsest level of detail whose declared error stays within 0.5 px at this screen size.
    player.set_screen_size(100.0, 120.0, 0.5)?;
    println!("level {} (declared bound {} px)", player.lod_level(), player.lod_bound_px());
    player.clear_screen_size();                            // back to the original
    assert_eq!(player.lod_level(), 0);

    // A scroll region whose `input` names a path follows set_value(path, px) instead of a drag.
    player.set_value("ui.scroll", 200.0)?;
    Ok(())
}
```

### Renderelés több szálon

```rust
use osatura_core::prelude::*;

/// `Animation` and `Player` are `Send + Sync`: read-only calls (`&self`) may run on several threads.
pub fn render_in_parallel() -> Result<(), ApiError> {
    let anim = Animation::demo();
    let frames = anim.frame_count();
    let settings = RenderSettings::new(0.25);
    let (a, s) = (&anim, &settings);                        // shared references are Copy: move them into the threads
    let images: Vec<Result<Image, ApiError>> = std::thread::scope(|scope| {
        let jobs: Vec<_> = (0..frames).map(|f| scope.spawn(move || a.render_rgba(f as f32, s))).collect();
        jobs.into_iter().map(|j| j.join().expect("render thread")).collect()
    });
    // Mutating calls (`&mut self`: set_key, undo, retarget_bvh ...) need exclusive access.
    for (f, img) in images.into_iter().enumerate() {
        assert_eq!(img?, anim.render_rgba(f as f32, &settings)?);   // threads do not change the result
    }
    Ok(())
}
```

### Verziókezelés és animáció-CI

```rust
use osatura_core::prelude::*;

/// Version control of a project file and animation CI, from Rust.
pub fn history(dir: &std::path::Path) -> Result<(), ApiError> {
    let file = dir.join("hero.json");
    std::fs::write(&file, Animation::demo().to_json()).unwrap();
    let path = file.to_str().unwrap();

    // A tiny helper: re-key the first bone at frame 5 and save the file.
    let edit = |angle: f32| -> Result<(), ApiError> {
        let mut a = Animation::from_json(&std::fs::read_to_string(path).unwrap())?;
        let bone = a.bone_names()[0].clone();
        a.set_key(&bone, "bone_angle", 5.0, angle, Interpolation::Linear)?;
        std::fs::write(path, a.to_json()).unwrap();
        Ok(())
    };

    vcs_init(path, Some("ana"))?;                           // branch `main`, no commit yet
    vcs_commit(path, Some("base"), None)?;
    vcs_branch(path, "other", None)?;                       // created, not switched to
    edit(20.0)?;
    vcs_commit(path, Some("main edit"), None)?;
    vcs_checkout(path, "other", false)?;                    // DirtyTree if the file has uncommitted changes
    edit(40.0)?;                                            // the same key, another value
    vcs_commit(path, Some("other edit"), None)?;
    vcs_checkout(path, "main", false)?;

    match vcs_merge(path, "other", false, None, None)? {
        VcsMergeOutcome::Conflicted(conflicts) => {         // a normal result, not an error
            for c in &conflicts {
                println!("{} ({}): {}", c.path, c.kind, c.description);
                vcs_resolve(path, &c.path, &VcsChoice::Theirs)?;
            }
            vcs_commit(path, None, None)?;                  // the merge commit
        }
        other => println!("merged without conflicts: {other:?}"),
    }
    for c in vcs_log(path, 10)? {
        println!("{} {} {}", &c.id[..8], c.author, c.message);
    }

    // Animation CI: write a calibrated manifest, record goldens, then verify.
    let init = ci_init(path, None, true)?;                  // never overwrites an existing manifest
    let run = ci_run(&init.manifest)?;                      // verify mode: nothing is re-recorded
    println!("{} checks, {} passed, exit code {}", run.total, run.passed, run.exit_code);
    if run.exit_code == 1 {
        // A failing check is a result, not an Err. After reviewing the report, accept the new look:
        ci_bless(&init.manifest)?;
    }
    Ok(())
}
```

### Mocap, fuzzing, vektorizálás, részletességi szint és méretcélzott exportálás

```rust
use osatura_core::prelude::*;

const RIG: &str = r#"{"format":1,"name":"rig","width":400.0,"height":400.0,"fps":30.0,"frames":10,"background":null,
 "camera":{"position":null,"zoom":1.0,"rotation":0.0},
 "bones":[{"name":"hips","parent":null,"length":20.0,"rest_offset":[200.0,250.0],"rest_angle":-90.0},
          {"name":"spine","parent":"hips","length":40.0,"rest_offset":[20.0,0.0],"rest_angle":0.0},
          {"name":"head","parent":"spine","length":30.0,"rest_offset":[40.0,0.0],"rest_angle":0.0}],
 "ik":[],"constraints":[],"springs":[],"layers":[],"channels":[]}"#;

fn mini_bvh() -> String {
    let mut s = String::from("HIERARCHY\nROOT Hips\n{\n OFFSET 0 100 0\n CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation\n");
    s.push_str(" JOINT Spine\n {\n  OFFSET 0 12 0\n  CHANNELS 3 Zrotation Xrotation Yrotation\n  JOINT Head\n  {\n   OFFSET 0 12 0\n   CHANNELS 3 Zrotation Xrotation Yrotation\n   End Site\n   {\n    OFFSET 0 8 0\n   }\n  }\n }\n}\nMOTION\nFrames: 4\nFrame Time: 0.0333333\n");
    for f in 0..4 {
        s.push_str(&format!("0 100 0 0 0 0 {} 0 0 0 0 0\n", f * 10));
    }
    s
}

/// Motion capture, pose fuzzing, tracing, level of detail and size-targeted export.
pub fn tools() -> Result<(), ApiError> {
    // BVH retargeting: one undoable edit; bones are matched by name and topology.
    let mut rig = Animation::from_json(RIG)?;
    let report = rig.retarget_bvh(&mini_bvh(), &MocapSettings::new().with_smooth(1))?;
    println!("matched {:?}; {} frames on {} bones", report.matched, report.frames, report.bones_used);
    assert!(rig.undo());                                    // the whole retarget is one step

    // Pose fuzzing: where does the rig break?  (deterministic for a given seed)
    let demo = Animation::demo();
    let fuzz = demo.fuzz(50, 1)?;
    println!("{} of {} samples failed (max severity {:.2})", fuzz.failures, fuzz.samples, fuzz.max_severity);
    let _full_report: &str = &fuzz.json;                    // schema "osatura-fuzz-report" v1

    // Bitmap tracing: PNG/JPEG bytes -> SVG, or straight into an animation.
    let png = demo.render_png(0.0, &RenderSettings::new(0.25))?;
    let svg = trace_image_svg(&png, 6)?;
    let traced = Animation::from_image(&png, 6, "traced")?;
    println!("traced SVG: {} bytes; project has {} layers", svg.len(), traced.layer_names().len());

    // Level of detail with a declared error bound, one project JSON per level (level 0 = original).
    for l in demo.lod_levels(0.5, 1.0, 3)? {
        println!("L{}: bound {:.3} px, {} anchors, {} keys", l.level, l.declared_px, l.path_anchors, l.keys);
        let _reduced = Animation::from_json(&l.json)?;
    }

    // Smallest export within a byte budget.
    let fit = demo.fit_to_size(FitFormat::Lottie, 250_000)?;
    println!("met={} {} bytes, worst SSIM {:.3}", fit.met, fit.size, fit.ssim_min);
    Ok(())
}
```

### Hogyan ellenőrizzük a példákat

A fenti kód az a kód, amely lefutott: ez a fájl a mintaforrásokból készül, amelyeket `cargo`-val lefordítottunk az `osatura-core` ellen, és végrehajtottunk (a használt projektfájlok az `osatura demo`, a `take-record`, a `clip add` és a `machine add` kimenetei, valamint egy konténert és egy scroll-régiót tartalmazó projekt). Az `api/` doc-tesztjei (`cargo test --doc -p osatura-core`: 19 doc-teszt, ebből 18 az `api/` facade-függvényein és a crate szintű példa a `lib.rs`-ben; debug buildben kb. 170 s, mert mindegyik egy programot fordít) magát a repository-ban lévő verziókezelő és CI burkolókat fedik le.

## Más nyelvek

A facade az igazság egyetlen forrása a többi bindinghoz. Lefedettség:

| Képesség | Rust facade | C ABI (`osatura.h`) | WebAssembly / JS |
|---|---|---|---|
| Betöltés, validálás, info, csontnevek / világmátrixok | igen | `osatura_open`, `_bone_*`, `_frame_count`, ... | `arm_open`, `load()` (csontmátrixok nélkül) |
| RGBA renderelés | igen | `osatura_render_rgba` | `arm_render_rgba`, `rgba()` |
| SVG / PNG renderelés / exportformátumok | igen | nem | SVG igen (`svg()`), PNG / exportálás nem |
| Kulcsok szerkesztése, visszavonás / újra | igen | nem | nem |
| `Player`: bemenetek, advance, állapot, események, renderelés | igen | `osatura_player_*` | `arm_player_*`, `startPlayer()`, ... |
| Elrendezés, görgetés, részletességi szint | igen | `osatura_player_set_canvas`, `_scroll_*`, `_set_screen`, `_lod_level` | `arm_player_set_canvas / layout / scroll_* / set_screen` |
| Eszközök, verziókezelés, CI | igen | nem | nem |

A C ABI verziója 1 marad, mert eddig minden kiegészítés additív volt. A wasm bináris a CI-ben épül és Node alatt tesztelt, de böngészőben még nem futtatták; lásd [WASM.md](WASM.md).

## Függelék: gép-JSON és kötés-JSON

Rövid összefoglalók a `Player` által futtatott két projektszakaszról (teljes formátum: [PROJECT_FORMAT.md](PROJECT_FORMAT.md)).

### Gép-JSON (összefoglaló)

```json
{"name":"hero",
 "inputs":[{"name":"moving","kind":"bool"},{"name":"speed","kind":"number"},{"name":"jump","kind":"trigger"}],
 "layers":[{"name":"body","mask":["arm_*"],"entry":"idle",
   "states":[{"name":"idle","source":{"kind":"clip","clip":"idle"},"blend_in":0.2},
             {"name":"loco","source":{"kind":"blend","input":"speed","children":[
                {"threshold":0,"source":{"kind":"clip","clip":"walk"}},{"threshold":1,"source":{"kind":"clip","clip":"run"}}]}},
             {"name":"jump","source":{"kind":"clip","clip":"jump"},"mode":"once"}],
   "transitions":[{"from":"idle","to":"loco","when":[{"input":"moving"}],"duration":0.25,"ease":{"type":"ease_in_out"}},
                  {"from":"*","to":"jump","when":[{"input":"jump"}],"duration":0.1},
                  {"from":"jump","to":"idle","exit_time":1.0}]}]}
```

A feltételek `{"input","op":"eq|ne|lt|le|gt|ge","value"}` alakúak (alapértelmezés `eq`, `1`; mindnek teljesülnie kell). A `from: "*"` a céltól eltekintve bármely állapotra illeszkedik. A deklarációs sorrendben az első illeszkedő átmenet nyer. Az `exit_time` az állapot lejátszott hányada (0..1). Források: `clip` (egy `Project.clips` bejegyzés), `segment` (a projekt kulcsolt replace-módú csatornáinak `start`, `end` képkockái), `region` (egy véggel rendelkező marker), `blend` (1D, két vagy több levélforrás növekvő küszöbökkel). Egy átmenet vagy feltétel ismeretlen kulcsai hibát jelentenek. CLI: `machine add|list|remove|validate`, `machine-sim <project> <script.json> <out_dir|out.json>` (szkript: `{"machine":"hero","duration":2,"events":[{"t":0.5,"set":{"moving":true},"trigger":["jump"],"value":{"player.health":40}}]}`).

### Kötés-JSON (összefoglaló)

```json
{"name":"hp","path":"player.health","expr":"value / 100","remap":{"in_min":0,"in_max":1,"out_min":-30,"out_max":30},
 "smooth":0.15,"target":{"kind":"prop","target":"arm_up","prop":"bone_angle"}}
```

Csővezeték: érték, majd `expr` (a `value` változó; a motor kifejezésfüggvényei, a forrás legfeljebb 4 KB, a beágyazás legfeljebb 64 mély), majd `remap` (alapértelmezetten szorítva; a `curve` lágyít), majd simítás (időállandó másodpercben), végül a cél: `{"kind":"prop",...}` (bármely csatorna-tulajdonság, beleértve a `layer_opacity` és a `layer_switch` tulajdonságot) vagy `{"kind":"input","machine","input"}` (egy trigger bemenet a felfutó élen sül el). CLI: `bind add|list|remove`, `bind-eval`.

### Elrendezés és rétegenkénti normálletérképek

Az elrendezési konténerek és a scroll-régiók a `containers` / `scrolls` projektmezők (üresen kihagyva): CLI `container add|list|remove|solve`, `scroll add|list|remove|eval`, `layout-preview <project> <out.svg> --size WxH`, valamint a `--size WxH` a `render`, a `frames` és az `export` parancson (a konténerek az adott vászonmérethez újraszámolódnak; a fájl változatlan). A `Layer.normal_map` (`data` base64 PNG vagy `path`, `fit` stretch / tile / exact, `strength`, `green_down`) a `relight` utófeldolgozó effektet táplálja azokra a pixelekre, amelyeket az a réteg, vagy egy saját térkép nélküli gyermeke lefed (CLI `layer-normal`, `fx ... --layer-normals`); nem része a facade-nek.
