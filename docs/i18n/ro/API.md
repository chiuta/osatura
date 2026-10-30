# API-ul Rust OSATURA și politica de stabilitate

*Versiunea fațadei 1.4.0 (`osatura_core::api::API_VERSION`). Public țintă: dezvoltatorii Rust care integrează motorul și mentenanții care modifică fațada. Licența documentației CC-BY-4.0; exemplele de cod MPL-2.0.*

Aceasta este referința pentru **API-ul Rust suportat** al `osatura-core`: modulul-fațadă `osatura_core::api` și importul său dintr-o singură linie, `osatura_core::prelude`. Enumeră fiecare element public, regulile care mențin API-ul stabil și exemple testate. Fiecare exemplu din acest fișier este compilat și rulat pe sursele curente înainte ca fișierul să fie publicat (vezi [Cum sunt verificate exemplele](#cum-sunt-verificate-exemplele)).

Cuprins: [Ce este acoperit](#ce-este-acoperit) - [Obținerea crate-ului](#obținerea-crate-ului) - [Politica de stabilitate](#politica-de-stabilitate) - [Reguli semver](#reguli-semver) - [Feature flags](#feature-flags) - [Siguranța la thread-uri](#siguranța-la-thread-uri) - [Determinism](#determinism) - [Limite](#limite-și-intervale-de-argumente) - [Erori](#erori) - [Referință: `Animation`](#referință-animation) - [Referință: `Player`](#referință-player) - [Referință: funcții libere](#referință-funcții-libere-unelte-control-al-versiunilor-ci) - [Referință: tipuri](#referință-tipuri) - [Exemple](#exemple) - [Alte limbaje](#alte-limbaje) - [Anexă: JSON de mașină și JSON de binding](#anexă-json-de-mașină-și-json-de-binding)

## Ce este acoperit

Doar `osatura_core::api` și `osatura_core::prelude` constituie API-ul Rust public suportat. Fațada constă din

* `Animation`: un proiect validat plus istoricul său de anulări (încărcare, interogare, randare, export, editare, unelte);
* `Player`: o mașină de stări în execuție, cu layout, derulare și nivel de detaliu (1.1, 1.2);
* funcții libere pentru controlul versiunilor și CI de animație (1.2 citire, 1.3 scriere) și pentru trasarea bitmap-urilor;
* tipuri simple de valori (`AnimationInfo`, `RenderSettings`, `Image`, `LodLevel`, `SizeFit`, `MocapReport`, `FuzzSummary`, `VcsCommit`, `VcsConflict`, `CiOutcome`, `CiInit`, ...) și enumurile `Interpolation`, `ExportFormat`, `FitFormat`, `CiMode`, `VcsMergeOutcome`, `VcsChoice`;
* tipul de eroare `ApiError` (implementează `std::error::Error` și `Display`).

ABI-ul C (`osatura-capi`, header-ul `osatura.h`) și modulul WebAssembly (`osatura-wasm`) sunt scrise exclusiv peste această fațadă și au propriile versiuni: [C_ABI.md](C_ABI.md), [WASM.md](WASM.md).

### Ce NU este acoperit

Toate celelalte module ale `osatura-core` (`engine`, `project`, `edit`, `svg`, `raster`, `physics`, `vcs`, `ci`, ...) și re-exporturile din rădăcină (`Engine`, `Project`, ...). Sunt `pub` și `#[doc(hidden)]` doar pentru că le folosesc instrumentul în linia de comandă, studioul, testele ABI-ului C și testele unitare; pot să se schimbe sau să dispară în orice versiune, iar textele lor `Err(String)` nu sunt nici ele un contract. Dacă ai nevoie de ceva accesibil doar acolo, cere o adăugare în fațadă (o versiune minoră) în loc să depinzi de elemente interne.

Formatul JSON al proiectului are propriul câmp de versiune `format` (în prezent 1); un proiect scris de un build mai nou este refuzat cu `InvalidJson`. Vezi [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

## Obținerea crate-ului

OSATURA nu este publicată într-un registru; depinde de ea prin cale (sau git) din workspace-ul tău:

```toml
[dependencies]
osatura-core = { path = "../osatura/osatura-core", default-features = false }
```

`default-features = false` elimină motorul de scripting Rhai (vezi [Feature flags](#feature-flags)); fațada nu are nevoie de el. Apoi `use osatura_core::prelude::*;`. Versiunea crate-ului (`0.1.0`) și versiunea fațadei (`API_VERSION`) sunt independente: fațada urmează semver, versiunea crate-ului nu promite încă nimic. Nu este declarată nicio versiune minimă suportată de Rust; workspace-ul este dezvoltat și testat cu Rust 1.97 (edition 2021).

## Politica de stabilitate

1. **Fișierul de suprafață.** `osatura-core/src/api_surface.txt` enumeră fiecare element public din `src/api/` (tipuri, câmpuri, variante, constante și semnături de metode, câte unul pe linie). Testul unitar `api::tests::api_surface_snapshot` reconstruiește această listă din `api/` și eșuează când diferă de fișierul din repository. Orice schimbare a suprafeței este deci o editare deliberată și revizuită a ambelor fișiere.
2. **Testul prelude.** `osatura-core/tests/prelude_surface.rs` este compilat ca un crate *extern*. Importă nume din fațadă doar prin `osatura_core::prelude` și apelează un subset reprezentativ (încărcare, randare, editare, export, `Player`, uneltele, wrapperele de control al versiunilor și de CI, tratarea intrărilor greșite). Dacă un nume pe care îl folosește nu mai este re-exportat din prelude, sau o semnătură se schimbă într-un mod pe care acele apeluri îl observă, testul nu se mai compilează. Mai verifică și `API_VERSION == "1.4.0"`, deci și o creștere de versiune este o editare conștientă acolo. Nu enumeră toate numele din prelude, deci adăugarea unui nume în lista de re-exporturi rămâne un pas manual.
3. **Limitele verificării.** Snapshotul este o scanare de text a semnăturilor pe o singură linie din `api/`, nu un diff la nivel de compilator precum `cargo-public-api`: o schimbare semantică ce păstrează textul semnăturii (de exemplu o altă eroare pentru aceeași intrare) nu este prinsă, iar o semnătură pe mai multe linii ar fi ratată (de aceea fișierul ține semnăturile pe o singură linie). Lista de re-exporturi din prelude (`lib.rs`) este verificată de testul extern, nu de snapshot.
4. **Ascuns nu înseamnă suportat.** Modulele `#[doc(hidden)] pub` există pentru consumatorii din workspace. Sunt în afara promisiunii, chiar dacă se compilează la tine.
5. **Cum se schimbă fațada** (pentru mentenanți; vezi și [DEVELOPMENT.md](DEVELOPMENT.md)):
   * editează `api/` (semnături pe o singură linie, comentariu de documentație, un doc-test pentru funcțiile libere noi, teste unitare);
   * adaugă numele publice noi în lista de re-exporturi `prelude` din `lib.rs` și în `tests/prelude_surface.rs`;
   * crește `API_VERSION` (și `API_VERSION_MAJOR` dacă schimbarea e incompatibilă) conform regulilor de mai jos, precum și versiunea verificată în `prelude_surface.rs`;
   * regenerează deliberat fișierul de suprafață: `OSATURA_BLESS=1 cargo test -p osatura-core api::` și revizuiește diferența din `api_surface.txt`;
   * actualizează acest fișier, iar `osatura-capi` / `osatura-wasm` doar dacă elementul nou trebuie să treacă prin acele granițe (versiunea lor de ABI evoluează separat, adăugările o păstrează).

## Reguli semver

`API_VERSION` este o versiune semantică doar a fațadei.

| Schimbare | Versiune |
|---|---|
| Corecție de bug, fără schimbarea semnăturii, fără un nou contract de comportament | patch (1.4.x) |
| Funcție, metodă, tip, variantă de enum, câmp de struct sau constantă nouă | minor (1.x.0) |
| Eliminarea sau redenumirea oricărui element; schimbarea unei semnături, a unui interval documentat de argumente astfel încât să respingă intrări valabile anterior, sau a semnificației unui rezultat documentat | major (2.0.0) |

Consecințe pentru apelanți:

* **Enumurile și structurile publice ale fațadei sunt `#[non_exhaustive]`.** Adaugă o ramură wildcard când faci `match` pe `ApiError`, `Interpolation`, `ExportFormat`, `FitFormat`, `CiMode`, `VcsMergeOutcome` sau `VcsChoice`; o variantă nouă înseamnă o versiune minoră. Nu poți construi structurile cu un literal în afara crate-ului (iar `Image`, `SizeFit` și structurile de raport sunt rezultate pe care doar le citești). Setările se construiesc cu constructori: `RenderSettings::new(scale).with_bones(true)`, `MocapSettings::new().with_smooth(2)`.
* **Șirurile din rezultate sunt date, nu contract.** `VcsConflict::kind` este astăzi unul dintre `"BothChanged"`, `"DeleteEdit"`, `"KeyRetimed"`, `"OrderConflict"`, `"ArrayConflict"` și *poate primi valori noi în versiuni minore*. Rapoartele JSON (`FuzzSummary::json`, `CiOutcome::json`) poartă propriile nume de schemă (`osatura-fuzz-report` v1, `osatura-ci/1`). Textele mesajelor de eroare sunt pentru oameni.
* **Stabilitatea comportamentului.** Randarea și evaluarea sunt deterministe (vezi mai jos), dar ieșirea în pixeli se poate schimba într-o versiune minoră când se corectează un bug al rasterizatorului sau al unui efect; golden-urile înregistrate de CI-ul de animație sunt o reamintire a acestui lucru. Nu calcula hash-uri ale pixelilor randați între versiuni OSATURA sau între mașini fără o toleranță.

Istoricul versiunilor fațadei: 1.0 (încărcare, interogare, randare, export, editare, anulare) - 1.1 (`Player`, mașini de stări, data bindings) - 1.2 (layout, derulare și nivel de detaliu pe `Player`; `lod_levels`, `fit_to_size`, `retarget_bvh`, `fuzz`, `from_image`, `trace_image_svg`, `vcs_log`, `ci_run`) - 1.3 (operații de scriere pentru controlul versiunilor și CI, `DirtyTree` / `MergeInProgress` / `Conflicts`, `CiOutcome::updated` / `blessed`) - 1.4 (crearea de straturi și oase, `bake_physics`, export APNG și EXR, `vcs_branches` / `vcs_branch_delete` / `vcs_status` / `vcs_diff` / `vcs_show` / `vcs_gc` / `vcs_fsck`).

## Feature flags

`osatura-core` are un singur feature Cargo:

| Feature | Implicit | Efect |
|---|---|---|
| `scripting` | activ | Compilează modulul de scripting în sandbox bazat pe Rhai (`script.rs`), folosit doar de comanda `script` din linia de comandă. Aduce `rhai` și, prin el, `smartstring` (MPL-2.0+, singurul crate terț cu copyleft slab din arbore). |

Fațada **nu** are nicio funcționalitate în spatele unui feature: fiecare element documentat aici există cu `default-features = false`. `osatura-capi`, `osatura-wasm` și `osatura-gpu` depind de core fără acest feature, deci nu conțin niciun crate terț MPL; `osatura-cli --no-default-features` face același lucru și apoi raportează comanda `script` ca indisponibilă. Back-endurile de randare nu sunt feature-uri: fațada folosește mereu rasterizatorul CPU; randerul GPU opțional este un crate separat (`osatura-gpu`) și un feature al CLI (`gpu`) și nu este accesibil prin acest API.

## Siguranța la thread-uri

* `Animation`, `Player`, `ApiError` și `Image` sunt `Send + Sync` (verificat la compilare pentru acest document). Fațada nu scrie nicio stare globală; citește doar două setări de resurse la nivel de proces (directorul implicit de resurse și comutatorul pentru resurse externe) pe care nimic din fațadă nu le modifică.
* Metodele care primesc `&self` (`info`, `bone_world`, `render_*`, `export`, `lod_levels`, `fit_to_size`, `fuzz`, `player`, `to_json`, ...) pot rula concurent pe mai multe thread-uri peste același `Animation`; metodele care primesc `&mut self` (`set_key`, `delete_key`, `undo`, `redo`, `retarget_bvh`, fiecare intrare a unui `Player` și `advance`) cer acces exclusiv, ca de obicei în Rust. Exemplul `render_in_parallel` de mai jos randează fiecare cadru pe propriul thread și verifică dacă imaginile sunt egale cu cele randate pe un singur thread.
* Nu există blocare internă și niciun thread în fundal. Un `Player` lucrează pe propria copie a proiectului, deci editarea ulterioară a `Animation` nu afectează niciodată un player aflat în execuție.
* `Animation::from_json` rezolvă fișierele de imagine relative față de **directorul curent** al procesului și refuză căile absolute și `..` (vezi [SECURITY.md](SECURITY.md)); încorporează imaginile ca data URI sau setează directorul de lucru înainte de încărcare, dacă un proiect face referire la fișiere.
* Funcțiile de control al versiunilor și de CI lucrează pe fișiere: nu sunt sigure dacă un alt proces editează același proiect în același moment (fațada verifică în prealabil dacă arborele are modificări necomise și dacă există un merge în curs, dar nu există nicio blocare).

## Determinism

Același proiect, aceleași argumente, același build OSATURA și aceeași mașină: aceiași octeți. Acest lucru este valabil pentru `render_rgba` / `render_png` / `render_svg`, `bone_world`, fiecare export, `lod_levels` și `fuzz` (pentru un seed dat), `fit_to_size` și pentru un `Player` (aceleași intrări și aceleași apeluri `advance` dau aceeași poziție; `advance` împarte timpul în bucăți de cel mult 1/60 s, deci felul în care împarți timpul contează doar sub această rezoluție). Alegerile aleatoare din motor (zgomot, particule, fuzzing, clustere de trasare) folosesc generatoare cu seed, niciodată ceasul. Ce *nu* este promis: pixeli identici la marginile anti-aliasate între sisteme de operare, procesoare, compilatoare sau versiuni `libm` diferite, între ținte pe 32 și 64 de biți, sau între versiuni OSATURA. Golden-urile identice octet cu octet au fost verificate doar între procese pe o singură mașină Linux x86-64 (vezi [LIMITATIONS.md](LIMITATIONS.md)); folosește toleranțe (CI-ul de animație o face) pentru orice comparație între mașini.

## Limite și intervale de argumente

| Element | Regulă |
|---|---|
| `MAX_JSON_BYTES` | 256 MiB; un text mai lung dă `InvalidJson` |
| `MAX_RENDER_SIDE` | 16384 de pixeli de ieșire pe latură (`render_*` și `Player::render_*`) |
| Scala de randare / GIF | finită și în `0.05..=4`, altfel `InvalidArgument` |
| Poziții de cadru | trebuie să fie finite; valorile din afara timeline-ului sunt **limitate** la `0..=frames-1` (fără eroare). Cadrele fracționare sunt acceptate. |
| Chei (`set_key`, `delete_key`) | `frame` și `value` finite; `prop` este un nume de proprietate în snake_case din JSON-ul proiectului (`"bone_angle"`, `"layer_opacity"`, ...) |
| Proiect | `fps` în `(0, 1000]`, `frames` în `1..=200000`, scenă de până la 32768 de unități pe latură, cel mult 4096 de oase, numere peste +-1e7 refuzate la încărcare; alte limite specifice funcționalităților sunt validate la încărcare și revin ca `InvalidProject` |
| `Player::advance(dt)` | `dt` finit, `0..=10` secunde |
| `Player::set_canvas_size` | ambele laturi finite, `> 0`, cel mult 32768 |
| `Player::set_screen_size` | lățime și înălțime finite și `> 0`; `max_error_px` finit și `>= 0` |
| `Animation::lod_levels` | `tolerance_px > 0`; `scale` în `(0, 64]`; `levels` în `1..=12` |
| `Animation::fit_to_size` | `max_bytes` în `1000..=1_000_000_000` |
| `Animation::fuzz` | `samples` în `1..=100000` |
| `trace_image_svg`, `Animation::from_image` | `colors` în `2..=64`; octeți PNG sau JPEG (se aplică limite de dimensiune) |
| `vcs_log` | `limit` în `1..=10000` |

## Erori

Fiecare funcție care poate eșua returnează `Result<_, ApiError>`. O intrare greșită este o valoare de eroare și **nu produce niciodată panic**; un panic care totuși apare în motor este prins la granița fațadei și returnat ca `ApiError::Internal` (te rugăm să-l raportezi ca bug). `ApiError` implementează `Display` și `std::error::Error`, deci `?` funcționează cu `Box<dyn Error>` și cu tipuri în stil `anyhow`.

| Variantă | Semnificație | Surse tipice |
|---|---|---|
| `InvalidJson(String)` | nu este JSON, nu este un proiect, formă greșită, prea mare (> `MAX_JSON_BYTES`) sau un `format` mai nou decât acest build; de asemenea, o valoare JSON greșită transmisă lui `vcs_resolve` | `Animation::from_json`, `validate_json`, `vcs_resolve` |
| `InvalidProject(Vec<String>)` | parsat, dar validarea a eșuat; un mesaj pe problemă (os necunoscut, interval greșit, ciclu, ...) | `from_json`, `from_image`, reconstruirile motorului în `Player` |
| `InvalidArgument(String)` | un argument al apelantului a fost respins: număr non-finit, valoare în afara intervalului, proprietate necunoscută, id necunoscut de mașină / intrare / derulare, BVH sau imagine inutilizabile, nume de branch greșit, nimic de comis, niciun repository | aproape orice funcție |
| `EditRejected(Vec<String>)` | o editare a fost refuzată (de exemplu un canal sau un strat blocat, ori un rezultat invalid); proiectul rămâne neschimbat | `set_key`, `delete_key`, `retarget_bvh` |
| `Export(String)` | un encoder a eșuat sau formatul nu este posibil pentru acest proiect | `export`, `render_png`, `fit_to_size` |
| `Internal(String)` | un panic prins | oricare |
| `DirtyTree(String)` | (1.3) fișierul proiectului are modificări necomise pe care operația le-ar suprascrie | `vcs_checkout` (dacă nu e `force`), `vcs_merge`, `vcs_cherry_pick`, `vcs_revert` |
| `MergeInProgress(String)` | (1.3) un merge, cherry-pick sau revert este neterminat; rezolvă și comite, sau `vcs_abort` | aceleași funcții |
| `Conflicts(Vec<String>)` | (1.3) `vcs_commit` cât timp mai există conflicte; un indicator pe conflict | `vcs_commit` |

O verificare eșuată a CI-ului de animație **nu** este o eroare: `ci_run` returnează `Ok(CiOutcome)` cu `exit_code` 1 (o verificare a eșuat) sau 2 (o verificare nu a putut fi evaluată); doar un manifest inutilizabil este `Err`. Conflictele de merge sunt la fel un rezultat (`VcsMergeOutcome::Conflicted`), nu o eroare.

## Referință: `Animation`

Un proiect încărcat și validat, plus istoricul său de anulări. Creează-l cu `from_json`, `demo` sau `from_image`. Toate metodele care citesc proiectul primesc `&self`.

| Metodă | Returnează | Note |
|---|---|---|
| `Animation::from_json(&str)` | `Result<Animation, ApiError>` | parsare + validare. `InvalidJson` / `InvalidProject`. Câmpurile JSON necunoscute sunt ignorate în tăcere aici (comanda CLI `check` le raportează). |
| `Animation::validate_json(&str)` | `Result<(), ApiError>` | aceleași verificări, nu păstrează nimic |
| `Animation::demo()` | `Animation` | un rig mic integrat (6 oase, 48 de cadre la 24 fps, 640 x 480); util pentru teste și primii pași |
| `Animation::from_image(&[u8], colors, name)` | `Result<Animation, ApiError>` | trasează octeți PNG/JPEG în straturi vectoriale umplute, pe o scenă de dimensiunea imaginii |
| `to_json(&self)` | `String` | proiectul curent ca JSON formatat (trece dus-întors prin `from_json`) |
| `info(&self)` | `AnimationInfo` | `name`, `width`, `height` (unități de scenă), `fps`, `frames` |
| `frame_count(&self)`, `fps(&self)` | `u32`, `f32` | |
| `bone_names(&self)` | `Vec<String>` | ordinea din rig = ordinea din `bone_world` |
| `layer_names(&self)` | `Vec<String>` | căi de straturi `group/child`, întâi în adâncime |
| `bone_world(&self, frame)` | `Result<Vec<[f32; 6]>, ApiError>` | matricea de lume pentru fiecare os `[a b c d tx ty]`; un punct local osului `(x, y)` se transformă în `(a*x + c*y + tx, b*x + d*y + ty)`. Cadrul este limitat. Evaluează întregul rig (IK, constrângeri, arcuri, expresii). |
| `machine_names(&self)` | `Vec<String>` | mașinile de stări de runtime stocate în proiect |
| `player(&self, machine)` | `Result<Player, ApiError>` | pornește mașina numită la momentul 0, pe o copie a proiectului |
| `render_rgba(&self, frame, &RenderSettings)` | `Result<Image, ApiError>` | RGBA8 cu alfa neprefixat (straight), pe rânduri, primul rând sus |
| `render_png(&self, frame, &RenderSettings)` | `Result<Vec<u8>, ApiError>` | un fișier PNG în memorie |
| `render_svg(&self, frame, show_bones: bool)` | `Result<String, ApiError>` | un document SVG (ieșirea vectorială nu are scală) |
| `export(&self, ExportFormat, scale)` | `Result<Vec<u8>, ApiError>` | `Gif` (în buclă, întreg timeline-ul; `scale` în `0.05..=4`), `Glb` (glTF 2.0 binar), `Lottie` (octeți JSON), `Html` (o pagină de player autonomă); `scale` este ignorat de celelalte trei. Ce păstrează sau elimină fiecare format: [EXPORT_FORMATS.md](EXPORT_FORMATS.md). |
| `set_key(&mut self, target, prop, frame, value, Interpolation)` | `Result<(), ApiError>` | setează sau adaugă o cheie pe `target.prop`; validat, anulabil, respectă blocările de canal și de strat (`EditRejected`) |
| `delete_key(&mut self, target, prop, frame)` | `Result<bool, ApiError>` | `true` dacă o cheie a fost eliminată |
| `add_rect_layer(&mut self, name, x, y, w, h, fill, parent)`, `add_ellipse_layer(&mut self, name, cx, cy, rx, ry, fill, parent)` | `Result<(), ApiError>` | (1.4) adaugă la sfârșit un strat de formă umplută (la nivelul superior sau în grupul de nivel superior numit de `parent`); nume unic fără `/`, dimensiuni finite și pozitive, o culoare validă; anulabil |
| `add_group_layer(&mut self, name)` | `Result<(), ApiError>` | (1.4) adaugă la sfârșit un strat-grup gol |
| `add_bone(&mut self, name, parent, length, x, y, angle)` | `Result<(), ApiError>` | (1.4) adaugă la sfârșit un os (`parent` trebuie să existe, unghi în grade); anulabil |
| `bake_physics(&mut self, chain, preset, overwrite)` | `Result<PhysicsBake, ApiError>` | (1.4) simulează un lanț de oase (rădăcina prima) cu `hair`, `tail`, `rope`, `antenna` sau `cape` și stochează canale de unghi ca un singur pas de anulare; cu `overwrite = false` oasele cu chei puse manual sunt păstrate și listate în `skipped` |
| `render_exr(&self, frame, &RenderSettings, float32)` | `Result<Vec<u8>, ApiError>` | (1.4) fișier OpenEXR (liniar, premultiplicat; HALF sau FLOAT) din compozitorul în virgulă mobilă |
| `undo(&mut self)`, `redo(&mut self)` | `bool` | `false` când nu e nimic de anulat / refăcut |
| `lod_levels(&self, tolerance_px, scale, levels)` | `Result<Vec<LodLevel>, ApiError>` | nivelul 0 este originalul, nivelul k >= 1 rămâne în limita a `tolerance_px * 2^(k-1)` pixeli de ieșire la `scale`; fiecare nivel poartă limita declarată și un JSON de proiect. Determinist; limita acoperă redarea timeline-ului (nu pozițiile mașinii de stări). Lent în build-uri neoptimizate (folosește release). |
| `fit_to_size(&self, FitFormat, max_bytes)` | `Result<SizeFit, ApiError>` | cel mai mic export încadrat în buget, prin coborârea treptată a parametrilor de calitate; dacă este imposibil, cel mai mic rezultat cu `met == false` și un `reason` |
| `retarget_bvh(&mut self, bvh_text, &MocapSettings)` | `Result<MocapReport, ApiError>` | retargetare BVH ca **o singură editare anulabilă**; oasele sunt potrivite după nume și topologie; canalele blocate sunt omise și raportate |
| `fuzz(&self, samples, seed)` | `Result<FuzzSummary, ApiError>` | fuzzing de poziții; animația este doar citită; raport complet ca JSON |

`Interpolation` a segmentului care începe la o cheie: `Linear`, `Step` (menținere), `EaseIn`, `EaseOut`, `EaseInOut`, `Smooth` (Catmull-Rom prin cheile vecine). Curbele mai bogate ale formatului de proiect (Bezier cubic, bounce, back, spring) se creează prin CLI și studio, nu prin `set_key`.

## Referință: `Player`

O mașină de stări în execuție (redare interactivă). Creat de `Animation::player`; lucrează pe o copie a proiectului; determinist. Mașinile de stări și data bindings sunt JSON simplu de proiect (`machines`, `bindings`), rezumate în [anexă](#anexă-json-de-mașină-și-json-de-binding).

| Metodă | Returnează | Note |
|---|---|---|
| `set_bool(name, bool)`, `set_number(name, f64)`, `fire(name)` | `Result<(), ApiError>` | setează o intrare după nume; un tip greșit de intrare sau un nume necunoscut dă `InvalidArgument`. Un trigger rămâne setat până când se declanșează o tranziție care îl testează (este consumat) sau până la sfârșitul următorului `advance`; `hold: true` îl păstrează până este consumat. |
| `set_value(path, f64)` | `Result<(), ApiError>` | publică o valoare externă pentru data bindings (`"player.health"`). O regiune de derulare al cărei `input` este această cale preia valoarea ca decalaj în pixeli de-a lungul axei sale principale (derularea este atunci în repaus). |
| `advance(dt)` | `Result<(), ApiError>` | avansează mașina și regiunile de derulare cu `dt` secunde (`0..=10`) |
| `state(layer)` | `Option<String>` | starea curentă a unui strat-mașină; `None` pentru un strat necunoscut |
| `events_json()` | `String` | evenimentele în așteptare `[{"time":..,"layer":..,"kind":"entered"|"exited","state":..}]`, neconsumate |
| `take_events_json()`, `clear_events()` | `String`, `()` | consumă / aruncă evenimentele în așteptare |
| `render_rgba(&RenderSettings)`, `render_svg(show_bones)`, `bone_world()` | ca pe `Animation` | poziția curentă; imaginea are dimensiunea canvasului stabilită de `set_canvas_size` înmulțită cu scala |
| `set_canvas_size(w, h)`, `canvas_size()` | `Result<(), ApiError>`, `(f32, f32)` | (1.2) layout adaptiv: containerele se rezolvă din nou, limitele de derulare urmează |
| `scroll_ids()` | `Vec<String>` | id-urile regiunilor de derulare ale proiectului |
| `scroll_drag(id, x, y)` | `Result<(), ApiError>` | decalaj brut al pointerului în pixeli (conținutul mutat în sus / spre stânga cu atât); dincolo de capete este limitat sau elastic (rubber-band) |
| `scroll_release(id, vx, vy)` | `Result<(), ApiError>` | viteza la eliberare în px/s; `advance` apoi lansează, aliniază și revine elastic |
| `scroll_offset(id)`, `scroll_max(id)`, `scroll_settled(id)` | `Option<[f32; 2]>`, `Option<[f32; 2]>`, `Option<bool>` | `None` pentru un id necunoscut |
| `set_screen_size(w_px, h_px, max_error_px)` | `Result<(), ApiError>` | (1.2) alege cel mai grosier dintre cele opt niveluri de detaliu precalculate a cărui limită declarată, la scala care încadrează canvasul în acel ecran, este cel mult `max_error_px`. Nivelurile se construiesc o singură dată la prima utilizare (nivelul 1 limitat la 0.125 unități de scenă, apoi dublând) și se păstrează în cache; cu `max_error_px` 0 se califică doar nivelurile fără pierderi. |
| `clear_screen_size()`, `lod_level()`, `lod_bound_px()` | `()`, `usize`, `f32` | revenire la original; nivelul folosit (0 = original); limita sa declarată în pixeli |

Note: limita nivelului de detaliu acoperă geometria și cheile din timeline, așa cum este documentat în `lod/mod.rs` (`ASSUMPTIONS`, `LIMITS`); pozițiile mașinii de stări și bindings nu sunt modelate de ea, iar canalele de poziție care vizează un strat eliminat de un nivel sunt abandonate. Proxy-urile de scrub ale studioului folosesc un mecanism separat, cu propria grilă de niveluri. `input`-ul unei regiuni de derulare trece prin tabelul de valori al player-ului, nu prin runtime-ul de data binding, deci expresiile, remaparea și netezirea nu i se aplică.

## Referință: funcții libere (unelte, control al versiunilor, CI)

| Funcție | Returnează | Ce face |
|---|---|---|
| `trace_image_svg(&[u8], colors)` | `Result<String, ApiError>` | trasează un bitmap într-un SVG de regiuni umplute |
| `vcs_log(path, limit)` | `Result<Vec<VcsCommit>, ApiError>` | (1.2, doar citire) istoricul, cel mai nou primul; `path` este fișierul proiectului sau orice cale din repository-ul său; un istoric gol dă o listă goală |
| `ci_run(manifest)` | `Result<CiOutcome, ApiError>` | (1.2) CI de animație în modul verify (nimic nu se reînregistrează); căile relative din manifest se rezolvă lângă el |
| `vcs_init(project_file, author)` | `Result<(), ApiError>` | (1.3) începe un istoric lângă fișier, pe branch-ul `main`, încă fără commit; eșuează dacă există deja unul |
| `vcs_commit(path, message, author)` | `Result<String, ApiError>` | id-ul commit-ului; încheie un merge când nu mai rămâne niciun conflict; `Conflicts` cât timp mai rămân; lipsa modificărilor de comis sau a mesajului dă `InvalidArgument` |
| `vcs_branch(path, name, at)`, `vcs_tag(path, name, at)` | `Result<String, ApiError>` | creează un branch / tag (implicit: commit-ul curent); nu se trece pe el; tag-urile nu se mută niciodată; se returnează id-ul commit-ului |
| `vcs_checkout(path, target, force)` | `Result<String, ApiError>` | trece fișierul pe un branch (rămâne pe el) sau pe orice revizie (detached); `DirtyTree` dacă nu e `force`; `MergeInProgress` |
| `vcs_merge(path, rev, no_ff, message, author)` | `Result<VcsMergeOutcome, ApiError>` | `UpToDate`, `FastForward(id)`, `Merged(id)` sau `Conflicted(Vec<VcsConflict>)` |
| `vcs_cherry_pick(path, rev, author)`, `vcs_revert(path, rev, author)` | `Result<VcsMergeOutcome, ApiError>` | reaplică / anulează modificarea unui commit |
| `vcs_conflicts(path)` | `Result<Vec<VcsConflict>, ApiError>` | conflictele încă în așteptare (gol când nu există) |
| `vcs_resolve(path, conflict_path, &VcsChoice)` | `Result<usize, ApiError>` | rezolvă un conflict cu `Ours`, `Theirs` sau `Value(json text)`; returnează câte au rămas |
| `vcs_abort(path)` | `Result<(), ApiError>` | abandonează un merge / cherry-pick / revert neterminat și restaurează ultimul commit |
| `vcs_branches(path)` | `Result<Vec<VcsBranch>, ApiError>` | (1.4) branch-urile cu commit-ul lor și dacă sunt active (checked out) |
| `vcs_branch_delete(path, name)` | `Result<(), ApiError>` | (1.4) șterge un branch; branch-ul activ și numele necunoscute sunt refuzate; obiectele rămân până la `vcs_gc` |
| `vcs_status(path)` | `Result<VcsStatus, ApiError>` | (1.4) branch, HEAD, modificări semantice necomise, merge neterminat, căile conflictelor |
| `vcs_diff(path, a, b)` | `Result<Vec<VcsChange>, ApiError>` | (1.4) diferențe semantice între două revizii; `None` este fișierul de lucru |
| `vcs_show(path, rev)` | `Result<VcsShow, ApiError>` | (1.4) un commit și modificările sale față de primul său părinte |
| `vcs_gc(path)` | `Result<VcsGc, ApiError>` | (1.4) șterge obiectele inaccesibile; se oprește fără a șterge dacă istoricul nu poate fi citit |
| `vcs_fsck(path)` | `Result<Vec<String>, ApiError>` | (1.4) probleme de integritate, gol când totul e sănătos |
| `ci_init(project, manifest, write_goldens)` | `Result<CiInit, ApiError>` | scrie un manifest implicit calibrat (nu suprascrie niciodată un fișier existent; nume implicit `osatura-ci.json` lângă proiect); opțional înregistrează acum golden-urile |
| `ci_run_mode(manifest, CiMode)` | `Result<CiOutcome, ApiError>` | `Verify`, `Update` (reînregistrează toate golden-urile verificărilor de randare), `Bless` (acceptă doar pe cele eșuate sau lipsă) |
| `ci_bless(manifest)` | `Result<CiOutcome, ApiError>` | prescurtare pentru `Bless` |

Istoricul este stocat lângă fișierul proiectului (`.osatura-vcs/`); mesajele de commit și autorii sunt text simplu. Un conflict conține indicatorul de identitate `path`, un `kind`, o `description` și valorile `base` / `ours` / `theirs` ca text JSON (`None` = absent acolo). După un merge cu conflicte, fișierul proiectului conține starea îmbinată, cu elementele în conflict la valoarea branch-ului curent, iar o stare în așteptare este înregistrată până când rezolvi toate conflictele și faci `vcs_commit`, sau `vcs_abort`. Resursele binare la care face referire un proiect nu sunt versionate, doar șirurile lor de cale. Merge-ul semantic, tipurile sale de conflict și limitele sale sunt descrise în [ROADMAP.md](ROADMAP.md) secțiunea 9 (elementul I-01) și în [LIMITATIONS.md](LIMITATIONS.md).

Nu sunt în fațadă: suprascrierile de mapare a oaselor pentru mocap, reglajele de trasare dincolo de numărul de culori, opțiunile de fit dincolo de format și buget, randarea GPU, studioul, colaborarea, serverul de agenți, numeroasele comenzi de creație și de baking (fizică, foot lock, magneți, autorig, ...), ștergerea / listarea branch-urilor, diff, show, status, gc, fsck, precum și opțiunile CI precum prune, `--only` și artefactele. Folosește CLI pentru acestea ([CLI_REFERENCE.md](CLI_REFERENCE.md)).

## Referință: tipuri

| Tip | Câmpuri / variante |
|---|---|
| `AnimationInfo` | `name: String`, `width: f32`, `height: f32` (unități de scenă), `fps: f32`, `frames: u32` |
| `RenderSettings` | `scale: f32` (0.05..=4), `show_bones: bool`, `transparent: bool`; `new(scale)`, `with_bones(bool)`, `with_transparent(bool)`; `Default` = scala 1, ambele dezactivate |
| `Image` | `width: u32`, `height: u32`, `pixels: Vec<u8>` (RGBA8, alfa neprefixat) |
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
| `MocapSettings` | `in_place`, `mirror`, `smooth` (raza mediei mobile, în cadre); `new()`, `with_in_place`, `with_mirror`, `with_smooth`; `Default` |
| `MocapReport` | `matched: Vec<(String, String)>` (articulație BVH, os din rig), `unmatched_joints`, `unmatched_bones`, `frames`, `bones_used`, `blocked` (`target.prop` blocat), `notes` |
| `FuzzSummary` | `samples`, `failures`, `failure_rate: f32`, `max_severity: f32`, `json: String` |
| `VcsCommit` | `id`, `author`, `time: u64` (secunde Unix), `message` |
| `VcsConflict` | `path`, `kind`, `description`, `base`, `ours`, `theirs: Option<String>` |
| `VcsMergeOutcome` | `UpToDate`, `FastForward(String)`, `Merged(String)`, `Conflicted(Vec<VcsConflict>)` |
| `VcsChoice` | `Ours`, `Theirs`, `Value(String)` |
| `CiMode` | `Verify`, `Update`, `Bless` |
| `CiOutcome` | `total`, `passed`, `failed`, `errors`, `exit_code: i32` (0 reușit, 1 eșuat, 2 neevaluabil), `json: String`, `updated`, `blessed` |
| `CiInit` | `manifest: String`, `checks: Vec<(String, String)>` (tip, nume), `recorded: Option<CiOutcome>` |
| Constante | `API_VERSION: &str`, `API_VERSION_MAJOR: u32`, `MAX_RENDER_SIDE: u32`, `MAX_JSON_BYTES: usize` |

## Exemple

Toate acestea sunt complete și au fost rulate. Presupun `use osatura_core::prelude::*;`.

### Încărcare, evaluare, randare, export

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

### Editare, anulare și tratarea erorilor

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

### Redare interactivă cu o mașină de stări

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

### Layout, derulare și nivel de detaliu

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

### Randare pe mai multe thread-uri

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

### Controlul versiunilor și CI de animație

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

### Mocap, fuzzing, trasare, nivel de detaliu și export cu țintă de dimensiune

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

### Cum sunt verificate exemplele

Codul de mai sus este codul care a rulat: acest fișier este generat din sursele exemplelor, care au fost compilate cu `cargo` față de `osatura-core` și executate (fișierele de proiect folosite sunt ieșirea comenzilor `osatura demo`, `take-record`, `clip add` și `machine add`, plus un proiect cu un container și o regiune de derulare). Doc-testele din `api/` (`cargo test --doc -p osatura-core`: 19 doc-teste, 18 pe funcții ale fațadei din `api/` și exemplul la nivel de crate din `lib.rs`; aproximativ 170 s într-un build de debug, deoarece fiecare compilează un program) acoperă wrapperele de control al versiunilor și de CI direct în repository.

## Alte limbaje

Fațada este unica sursă de adevăr pentru celelalte binding-uri. Acoperire:

| Capabilitate | Fațada Rust | ABI C (`osatura.h`) | WebAssembly / JS |
|---|---|---|---|
| Încărcare, validare, info, nume de oase / matrici de lume | da | `osatura_open`, `_bone_*`, `_frame_count`, ... | `arm_open`, `load()` (fără matrici de oase) |
| Randare RGBA | da | `osatura_render_rgba` | `arm_render_rgba`, `rgba()` |
| Randare SVG / PNG / formate de export | da | nu | SVG da (`svg()`), PNG / export nu |
| Editare chei, anulare / refacere | da | nu | nu |
| `Player`: intrări, advance, stare, evenimente, randare | da | `osatura_player_*` | `arm_player_*`, `startPlayer()`, ... |
| Layout, derulare, nivel de detaliu | da | `osatura_player_set_canvas`, `_scroll_*`, `_set_screen`, `_lod_level` | `arm_player_set_canvas / layout / scroll_* / set_screen` |
| Unelte, control al versiunilor, CI | da | nu | nu |

Versiunea ABI-ului C rămâne 1, deoarece fiecare adăugire de până acum a fost aditivă. Binarul wasm este construit și testat sub Node în CI, dar nu a fost rulat într-un browser; vezi [WASM.md](WASM.md).

## Anexă: JSON de mașină și JSON de binding

Scurte rezumate ale celor două secțiuni de proiect pe care le rulează `Player` (formatul complet: [PROJECT_FORMAT.md](PROJECT_FORMAT.md)).

### JSON de mașină (rezumat)

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

Condițiile sunt `{"input","op":"eq|ne|lt|le|gt|ge","value"}` (implicit `eq`, `1`; toate trebuie îndeplinite). `from: "*"` se potrivește cu orice stare în afară de țintă. Prima tranziție potrivită, în ordinea declarării, câștigă. `exit_time` este fracțiunea redată (0..1) din stare. Surse: `clip` (o intrare din `Project.clips`), `segment` (cadrele `start`, `end` ale canalelor cu chei în mod replace ale proiectului), `region` (un marker cu sfârșit), `blend` (1-D, două sau mai multe surse-frunză la praguri crescătoare). Cheile necunoscute dintr-o tranziție sau condiție sunt o eroare. CLI: `machine add|list|remove|validate`, `machine-sim <project> <script.json> <out_dir|out.json>` (script: `{"machine":"hero","duration":2,"events":[{"t":0.5,"set":{"moving":true},"trigger":["jump"],"value":{"player.health":40}}]}`).

### JSON de binding (rezumat)

```json
{"name":"hp","path":"player.health","expr":"value / 100","remap":{"in_min":0,"in_max":1,"out_min":-30,"out_max":30},
 "smooth":0.15,"target":{"kind":"prop","target":"arm_up","prop":"bone_angle"}}
```

Conductă: valoarea, apoi `expr` (variabila `value`; funcțiile de expresie ale motorului, sursă de cel mult 4 KB și imbricare de cel mult 64), apoi `remap` (limitat implicit; `curve` face easing), apoi netezirea (constantă de timp în secunde), apoi ținta: `{"kind":"prop",...}` (orice proprietate de canal, inclusiv `layer_opacity` și `layer_switch`) sau `{"kind":"input","machine","input"}` (o intrare de tip trigger se declanșează pe frontul crescător). CLI: `bind add|list|remove`, `bind-eval`.

### Layout și hărți normale per strat

Containerele de layout și regiunile de derulare sunt câmpuri de proiect `containers` / `scrolls` (omise când sunt goale): CLI `container add|list|remove|solve`, `scroll add|list|remove|eval`, `layout-preview <project> <out.svg> --size WxH`, și `--size WxH` pe `render`, `frames` și `export` (containerele se rezolvă din nou pentru acea dimensiune de canvas; fișierul rămâne neschimbat). `Layer.normal_map` (`data` PNG în base64 sau `path`, `fit` stretch / tile / exact, `strength`, `green_down`) alimentează efectul de postprocesare `relight` pentru pixelii pe care îi acoperă acel strat, sau un copil fără hartă proprie (CLI `layer-normal`, `fx ... --layer-normals`); nu face parte din fațadă.
