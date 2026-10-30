<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Ghid de dezvoltare

*Pentru cei care construiesc, modifică, testează și lansează OSATURA. Documentație CC-BY-4.0. Tot ce se află aici este preluat din arborele sursă, din `ROADMAP.md` secțiunea 9, [CI.md](CI.md), [CLEANROOM.md](CLEANROOM.md), [CONTRIBUTING.md](CONTRIBUTING.md) și [LICENSING.md](LICENSING.md); comenzile marcate „run” au fost rulate la scrierea acestei pagini (Linux x86-64, rustc 1.97.0, registru offline).*

Cuprins: [Mediul](#1-mediul) - [Comenzi de build și de test](#2-comenzi-de-build-și-de-test) - [Suitele de teste](#3-suitele-de-teste) - [Convenții de cod](#4-convenții-de-cod) - [Listă de verificare pentru un modul sau o comandă nouă](#5-listă-de-verificare-pentru-un-modul-sau-o-comandă-nouă) - [Reguli clean-room și DCO](#6-reguli-clean-room-și-dco) - [Listă de verificare pentru lansare](#7-listă-de-verificare-pentru-lansare) - [Integrare continuă](#8-integrare-continuă) - [Depanare](#9-depanare) - [Profilare](#10-profilare)

## 1. Mediul

| Necesar | De ce | Note |
|---|---|---|
| Rust 1.97.0 (workspace edition 2021) | totul | fixat în `rust-toolchain.toml` (rustup îl instalează la prima utilizare; `rust-version = "1.97"` în fiecare crate); `Cargo.lock` este comis, build-urile sunt compatibile cu `--locked`. Build-urile de lansare au `overflow-checks = true`: o depășire de întreg provoacă panică în loc să se încheie circular, deci testează și profilul release (`cargo test --release`) |
| Un compilator C (`cc`) | exemplul/testul C ABI (opțional) | vezi [C_ABI.md](C_ABI.md) |
| `ffmpeg` / `ffprobe` în `PATH` | export video, decodarea audio mp3/ogg, `dof-export` în .mp4/.webm, `soundtrack` | **nu este niciodată inclus și nu este necesar pentru build**; comenzile care îl cer spun acest lucru și eșuează fără mesaj de succes |
| Node.js >= 22 (cât folosește CI) și `playwright-core` + Chromium | cele cinci teste smoke ale studioului; testul wrapperului WASM | implicit: `/opt/npm-tools/node_modules/playwright-core`, `/opt/pw-browsers/chromium-1194/chrome-linux/chrome`; se pot suprascrie cu `PLAYWRIGHT_CORE`, `CHROME` |
| `cargo-deny` | politica de licențe/interdicții/surse | `scripts/licence-audit.sh` îl rulează dacă este instalat (`--no-deny` îl omite) |
| `python3` | `scripts/licence-audit.sh` generează documentația cu un script inline | rulează cu `python3 -I` |
| Un adaptor Vulkan sau GL | doar pentru cifre pe GPU real din `osatura-gpu` | testele revin la un adaptor software; vezi secțiunea 2 |
| `rustup target wasm32-unknown-unknown`, `wasm-opt` | doar pentru [WASM.md](WASM.md) | niciodată instalate în mediul de dezvoltare; CI le instalează |

Structura workspace-ului (vezi [ARCHITECTURE.md](ARCHITECTURE.md) pentru proiectare):

| Crate | Licență | Rol |
|---|---|---|
| `osatura-core` | MPL-2.0 | motorul, formatele, uneltele, fațada stabilă `api` ([API.md](API.md)); features: `scripting` (implicit, Rhai) |
| `osatura-cli` | AGPL-3.0-or-later | binarul `osatura`, serverul studio, serverul collab; features: `scripting` (implicit), `gpu` |
| `osatura-capi` | MPL-2.0 | C ABI ([C_ABI.md](C_ABI.md)) |
| `osatura-wasm` | MPL-2.0 | ABI WebAssembly + wrapper JS ([WASM.md](WASM.md)) |
| `osatura-gpu` | MPL-2.0 | renderer wgpu opțional; **nu** este membru implicit |
| `osatura-gui` | AGPL-3.0-or-later | `osatura-studio`, studioul într-o fereastră nativă (`tao` + `wry`, doar Windows și macOS; stub în rest); **nu** este membru implicit, `publish = false` |

Fișiere la nivel de repository: `ROADMAP.md` (secțiunea 9 este jurnalul datat al funcționalităților și al reviziilor), `FEATURES.md`, `CLEANROOM.md`, `LICENSING.md`, `REUSE.toml`, `deny.toml`, `LICENSES/`, `scripts/`.

## 2. Comenzi de build și de test

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

Funcționalități și părți opționale:

```sh
cargo build -p osatura-cli --no-default-features         # no Rhai (no `script` command, no smartstring)
cargo build -p osatura-core --no-default-features        # what capi, wasm and gpu use
cargo build -p osatura-cli --features gpu                # adds `render-gpu` (wgpu)
cargo test  -p osatura-gpu                               # GPU tests (31: 18 unit + 13 parity); NOT in the workspace default
OSATURA_GPU_BACKEND=vulkan osatura render-gpu ...     # restrict the wgpu backend: vulkan or gl
```

Reguli practice: folosește build-uri de depanare (debug) pentru suitele de teste (repository-ul a fost dezvoltat având în vedere spațiul pe disc: nu copia `target/`); folosește `--release` doar pentru întrebări legate de timp. Snapshot-ul fațadei se aprobă cu `OSATURA_BLESS=1 cargo test -p osatura-core api::` (secțiunea 5). `OSATURA_UI_CONFIG` selectează în teste fișierul de setări UI al studioului, `OSATURA_AUTHOR` numele autorului în commit-urile de control al versiunilor, iar `OSATURA_E2E_TIMINGS=<file>` face ca testul e2e să scrie duratele pe etape.

## 3. Suitele de teste

Numerele provin din `ROADMAP.md` secțiunea 9 și dintr-o rulare completă `cargo test --workspace --offline` din 22 oct. 2026 făcută pentru această pagină (profil debug, o mașină partajată cu alte joburi, deci duratele sunt orientative și **vor fi mai mari decât pe o mașină liberă**):

| Suită | Teste | Durată (această rulare) | Ce acoperă |
|---|---:|---:|---|
| `osatura-core` unit (`src/lib.rs`) | 1167 | 349 s | fiecare modul; include `api::tests::api_surface_snapshot` |
| `osatura-core/tests/prelude_surface.rs` | 6 | 0.7 s | fațada văzută din afara crate-ului (`#[non_exhaustive]`, maparea erorilor, un subset reprezentativ de apeluri) |
| `osatura-cli` unit (`src/usage_tests.rs`, `src/args.rs`, studio, collab, http) | 80 | 42 s | parsarea opțiunilor, USAGE față de parser, serverul HTTP, politica, jurnalul collab |
| `tests/e2e.rs` | 2 | 214 s | întregul pipeline prin binarul real (aproximativ 19 s într-un build debug pe mașină liberă și 40 s în release, conform ROADMAP; aici mașina era încărcată) |
| `tests/fixes.rs`, `tests/wave4.rs`, `tests/robust.rs` | 7, 15, 10 | 19 s, 1.2 s, 1.2 s | regresii ale valurilor de revizie și teste cu intrări ostile |
| `tests/doc_mismatch.rs` | 20 | 1.8 s | regresii documentație/comportament: ieșiri atomice, opțiuni stricte, matricea de acceptare `--json`/`--quiet`/`--allow-empty`, regulile de scară ale `golden` |
| `tests/source_offer.rs` | 3 | 0.2 s | oferta `/source` conform AGPL secțiunea 13 |
| `tests/face_cli.rs` | 1 | 0.8 s | CLI-ul de urmărire a feței |
| `osatura-core/tests/wasm_safety.rs` | 2 | 0.1 s | niciun `Instant` / `SystemTime` / `process::id` / `thread::spawn` direct în afara `clock.rs` (scanare a sursei) |
| `osatura-cli/tests/wave7.rs` | 7 | 7 s | valul 7: opțiuni pe subcomandă, `follow` / `make-loop`, `color-space`, `collab-token`, fișierul secret al studioului, intervalul de cadre pentru `dof` / `render-hdr` / `fx` |
| `tests/spdx_audit.rs` | 2 | 0.01 s | fiecare `.rs/.js/.mjs/.sh` are o linie SPDX egală cu licența crate-ului său; REUSE.toml acoperă restul |
| `osatura-capi/tests/abi.rs` | 7 | 1.1 s | C ABI prin partea Rust (test smoke în C: `osatura-capi/tests/smoke.c`, compilat manual, vezi [C_ABI.md](C_ABI.md)) |
| `osatura-wasm` unit | 8 | 1.6 s | funcțiile ABI pe gazdă |
| `osatura-gpu` unit + `tests/parity.rs` | 18 + 13 | 0.2 s + 6.5 s | paritate GPU față de CPU (adaptor software când nu există GPU) |
| Doc-tests (`osatura-core`) | 19 | 171 s (fiecare compilează un program) | exemplele fațadei (`api/`, `lib.rs`); `osatura-capi`, `osatura-wasm` nu au |

Total: 1343 de teste în binarele de unit și de integrare și 19 doc-tests, numărate pe 24 oct. 2026 cu `cargo test --workspace --exclude osatura-gpu --exclude osatura-gui` (cele 1368 din propoziția următoare includeau și testele crate-ului GPU; renumărat dintr-o rulare completă `cargo test --workspace` din 22 oct. 2026 după valul 7, duratele pe suite de mai sus provin dintr-o rulare anterioară), plus 19 doc-tests; întregul `cargo test --workspace --offline` a durat 836 s pe mașina partajată.

Suite în afara `cargo test`:

* **Teste smoke ale studioului (Chromium).** Cinci scripturi Node controlează studioul real în Chromium headless cu `playwright-core`: `studio_ui_smoke.js` (128 de verificări), `studio_collab_smoke.js` (60, două pagini pe aceeași mașină), `studio_overlays_smoke.js` (89), `studio_suggest_smoke.js` (35) și `studio_demo_smoke.js` (25: demo-ul ghidat încarcă toți cei 13 pași, nu scrie niciodată, restaurează proiectul octet cu octet și recuperează o filă închisă); toate cinci au trecut pe 24 oct. 2026 (numerele sunt cele afișate de scripturi). **Nu** sunt rulate de `cargo test` și, implicit, nu fac parte din CI. Rulare:

  ```sh
  cargo build -p osatura-cli
  node osatura-cli/tests/studio_ui_smoke.js        # same for collab / overlays / suggest / demo
  ```

  Mediu: `OSATURA_BIN` (implicit `target/debug/osatura`), `PLAYWRIGHT_CORE`, `CHROME`, iar pentru scriptul overlays `SHOTS` (directorul pentru capturi de ecran). Scripturile își șterg fișierele temporare.
* **Testul wrapperului WASM.** `node osatura-wasm/js/test.mjs` (doar mock, cu excepția cazului în care dai o cale de proiect și ai construit modulul; vezi [WASM.md](WASM.md)).
* **Auditul licențelor.** `scripts/licence-audit.sh`: reconstruiește `docs/THIRD_PARTY.md` și `docs/THIRD_PARTY_NOTICES.md` din `cargo metadata --locked --all-features` (dependențe normale și de build, dependențele de dezvoltare excluse), verifică linia SPDX a fiecărui fișier sursă față de `license` din crate-ul său, apoi rulează `cargo deny check licenses bans sources`, dacă nu se dă `--no-deny`. `deny.toml` permite licențe permisive și MPL-2.0 doar pentru crate-uri numite.
* **clippy.** `cargo clippy --workspace --all-targets -- -D warnings` se încheie cu codul 0 și fără avertismente (valul 8) și este condiție de acceptare în CI. Acolo unde un lint este greșit pentru caz, codul poartă un `#[allow(clippy::...)]` îngust, cu un comentariu (bucle cu index peste tablouri paralele, `too_many_arguments` pe semnături stabile, `manual_clamp` unde tratarea NaN diferă, nume inerente `next` / `mul` care sunt API public); nu adăuga allow-uri la nivel de crate.
* **Conectarea CI.** Repository-ul livrează `osatura ci` pentru verificări de *animație* ([CI.md](CI.md)); este altceva decât CI-ul codului. Proiectul nu are în arbore nicio configurație CI pentru cod; secțiunea 8 dă o rețetă minimă.

Adăugarea de teste: pune testele motorului lângă cod (`#[cfg(test)]`), testele de comportament ale CLI în `osatura-cli/tests/`, iar fiecare bug reparat primește un test de regresie (valurile de revizie din [CHANGELOG.md](CHANGELOG.md) enumeră ce a fost reparat și unde). Testele nu trebuie să lase fișiere temporare (folosește drop guards) și nu trebuie să depindă de ceas sau de rețea. Bugetele de timp se testează cu un ceas de test determinist.

## 4. Convenții de cod

* **Fără panici la intrare.** Tot ce citește un proiect, un fișier, un argument sau un octet din rețea returnează o eroare. Indexarea, tăierea (slicing), `unwrap` și depășirea aritmetică pe valori nesigure sunt bug-uri. Fațada (`api/`) prinde în plus panicile și returnează `ApiError::Internal` (doar unde ținta face unwinding), dar aceasta este ultima linie de apărare, nu o proiectare.
* **Politica valorilor finite.** Numerele din fișiere și din argumente sunt verificate pentru NaN și infinit și pentru ordine de mărime rezonabile (intervalele fațadei sunt în [API.md](API.md): magnitudini până la 1e7, fps în (0, 1000], cadre 1..=200000, scenă până la 32768, 4096 de oase, latura de randare până la 16384 px). Respinge, nu limita (clamp), cu excepția cazului în care funcția este documentată ca limitând (cadrele sunt limitate la timeline).
* **Verifică dimensiunile înainte de alocare.** Buffer-ele de pixeli, dimensiunile mesh-urilor și numărul de eșantioane se calculează cu aritmetică verificată și se refuză înainte de `vec![..]`.
* **Scrieri atomice.** Scrie rezultatele cu `fsutil::write_atomic` (fișier temporar în același director, `fsync`, redenumire; refuză legăturile simbolice, cu excepția cazului în care se dă `--follow-symlinks`; revine la o suprascriere pe loc cu `fsync`, documentată ca neatomică, când directorul nu poate fi scris). Nu scrie niciodată direct cu `std::fs::write` un fișier al utilizatorului. Directoarele temporare vin din `fsutil` (mod 0700, fișiere 0600). Octeții aleatori vin din `fsutil::random_bytes` (`/dev/urandom`).
* **Resursele trec prin `assets::resolve`.** Căile relative ale imaginilor, ale normal map-urilor și ale sunetelor se rezolvă față de directorul de resurse la nivel de proces (`assets::default_dir`, stabilit din folderul proiectului de intrare); căile absolute și `..` sunt refuzate, cu excepția cazului în care se dă `--allow-external-assets`; fișierele de imagine sunt limitate la 64 MB, iar cele audio la 256 MB. Comenzile care scriu un proiect derivat validează față de directorul proiectului de *intrare* și rescriu căile relative ale resurselor sau avertizează despre ele (`write_json`).
* **Aleatoriu determinist.** Orice comportament aleatoriu (fuzzing, zgomot, jitter, simulare) folosește generatorul propriu al motorului, cu seed, și o opțiune `--seed`, și este reproductibil bit cu bit. Fără crate-ul `rand`, fără timpul sistemului ca seed.
* **Println și pipe-uri închise.** CLI-ul umbrește `println!`/`print!` cu macro-uri care ies silențios la un pipe închis (`osatura ... | head` nu trebuie să intre în panică). Continuă să le folosești în modulele declarate după macro-uri în `main.rs`.
* **Erorile spun ce trebuie făcut.** Mesajele spun ce a fost greșit, ce valoare, și intervalul permis sau remedierea (testele fațadei verifică acest stil).
<!-- REUSE-IgnoreStart -->
* **Anteturi SPDX.** Fiecare fișier `.rs`, `.js`, `.mjs` și `.sh` începe cu `SPDX-License-Identifier:` egal cu licența crate-ului său (MPL-2.0 pentru core/capi/wasm/gpu/scripts, AGPL-3.0-or-later pentru CLI) și o linie de copyright; documentele Markdown poartă comentariul-antet CC-BY-4.0, ca acest fișier. Celelalte fișiere (JSON, imagini, textele de licență) sunt acoperite de `REUSE.toml`; când adaugi un tipar nou de cale, adaugă-l acolo. Altfel `cargo test -p osatura-cli --test spdx_audit` eșuează.
<!-- REUSE-IgnoreEnd -->
* **Dependențe.** Puține, Rust pur, licență permisivă. Un crate nou cere o verificare `deny.toml` (`scripts/licence-audit.sh`), o linie în `docs/THIRD_PARTY.md` (regenerat) și o justificare în intrarea din ROADMAP. `osatura-core` este construit fără features implicite de capi, wasm, gpu și CLI, așa că tot ce este greu trece în spatele unui feature.
* **Fațada mai întâi.** O capabilitate nouă a motorului, vizibilă utilizatorului, la care ar trebui să ajungă și alte limbaje, intră în `osatura-core/src/api/` cu o linie de snapshot, un doc-test și un test unitar; straturile C și wasm învelesc doar fațada.

## 5. Listă de verificare pentru un modul sau o comandă nouă

1. **Modul** în `osatura-core/src/<name>.rs`, înregistrat în `lib.rs`; antet SPDX; teste unitare (valide, la limită, intrări ostile, determinism); fără panici.
2. **Comandă CLI**: o intrare `USAGE` (scrierea exactă a fiecărei opțiuni) în `osatura-cli/src/usage.rs`, brațul de dispatch într-unul dintre `osatura-cli/src/cmds_1.rs` până la `cmds_6.rs` (sau în `main.rs` pentru `ui`) și tabelele de opțiuni în `osatura-cli/src/args.rs`: fiecare opțiune care ia o valoare trebuie listată în `VALUE_OPTS`; flag-urile care nu iau valoare, în intrarea `CMD_OPTS` a comenzii, acolo unde tabelul global ar fi greșit (de exemplu `ci --prune` este un flag, `export-oca --layers` și `autotrace --png` iau valori, `keyedit --free` este un flag). Testele unitare USAGE-față-de-parser eșuează dacă textul și parserul nu concordă. Suportă flag-urile globale (`--json`, `--quiet`, `--allow-empty`, `--allow-external-assets`, `--follow-symlinks`, `--max-input-bytes`), `<command> --help` și `--key=value`.
3. **Scrierea ieșirii** doar cu `fsutil::write_atomic` sau `write_json`; comenzile care nu produc nimic se încheie cu cod diferit de zero, cu excepția cazului în care se dă `--allow-empty`.
4. **Documentație**: comanda în [CLI_REFERENCE.md](CLI_REFERENCE.md) dacă adaugi una, o mențiune într-un ghid acolo unde e relevant și intrarea din `ROADMAP.md` secțiunea 9 (ce a fost adăugat, ce *nu* s-a făcut: lista aceea alimentează [LIMITATIONS.md](LIMITATIONS.md)).
5. **Rândul din CLEANROOM.md**: pentru fiecare algoritm nou consemnează sursa publică după care a fost implementat (manual, standard, articol) și derivarea independentă; fără rând, fără merge.
6. **Reaprobarea suprafeței fațadei** dacă `api/` s-a schimbat: rulează `OSATURA_BLESS=1 cargo test -p osatura-core api::`, revizuiește diff-ul fișierului `osatura-core/src/api_surface.txt` (o linie eliminată sau modificată este o schimbare incompatibilă și cere o versiune majoră a fațadei; o linie adăugată este una minoră), apoi rulează din nou testele fără variabilă. Actualizează [API.md](API.md), precum și wrapperele C/WASM și antetele lor, dacă ar trebui să o expună.
7. **End-to-end**: adaugă comanda în pipeline-ul e2e dacă aparține căii de producție; adaugă o verificare smoke în scripturile studioului dacă are UI.
8. **Rulare**: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/licence-audit.sh` (când s-au schimbat dependențele sau antetele).

## 6. Reguli clean-room și DCO

Proiectul este o implementare independentă. Pe scurt (textul obligatoriu este [CLEANROOM.md](CLEANROOM.md) și [CONTRIBUTING.md](CONTRIBUTING.md)):

* Implementează pornind de la standarde publice, manuale și articole, și citează-le. Nu citi, nu decompila, nu copia și nu face inginerie inversă pe cod, fișiere sau interfețe ale uneltelor proprietare de animație și nu citi și nu lipi cod sursă al uneltelor concurente în timp ce scrii o funcționalitate. Nu lipi cod din proiecte cu o licență incompatibilă.
* Consemnează fiecare metodă nouă în tabelul din `CLEANROOM.md`, cu sursa ei publică.
* Notele despre brevete sunt în [PATENT_NOTES.md](PATENT_NOTES.md); o funcționalitate care atinge o zonă sensibilă din punct de vedere al brevetelor se discută mai întâi acolo. Aceasta nu este consultanță juridică.
* Contribuțiile sunt acceptate sub licența părții pe care o modifică (inbound egal cu outbound): MPL-2.0 pentru biblioteci și scripturi, AGPL-3.0-or-later pentru `osatura-cli`, CC-BY-4.0 pentru documentație și resurse sintetice.
* **Sign-off.** Fiecare commit are nevoie de `Signed-off-by: Name <email>` (`git commit -s`), care certifică Developer Certificate of Origin 1.1 (textul în CONTRIBUTING.md). Sign-off-ul certifică doar proveniența: **nu** acordă niciun drept de relicențiere și nu este posibilă nicio licență comercială sau duală pentru codul contribuit cât timp nu există un CLA.
* Numele și logoul sunt acoperite de [TRADEMARK.md](TRADEMARK.md).

## 7. Listă de verificare pentru lansare

Ultima pre-lansare publicată este `v0.1.0-alpha.4`; următoarea este pregătită ca `v0.1.0-alpha.5` (toate crate-urile sunt încă 0.1.0; etichetele de versiune „v0.5/v0.6/v0.7” din README, FEATURES și ROADMAP sunt niveluri de funcționalități, nu versiuni de crate; vezi [CHANGELOG.md](CHANGELOG.md)). Pentru a face una:

1. **Versiune.** Stabilește versiunea fiecărui crate. Fațada are propria versiune (`API_VERSION`, azi 1.4.0: majoră la eliminarea sau schimbarea unei linii de suprafață, minoră la adăugare), iar `OSATURA_ABI_VERSION` / `arm_abi_version()` (ambele 1) se schimbă doar la o modificare incompatibilă a ABI-ului C sau wasm. Aliniază versiunile din `Cargo.toml`, ieșirea `osatura --version`, caseta de stare din README și CHANGELOG.
2. **Condiții verzi.** `cargo test --workspace`, `cargo test --doc -p osatura-core`, `cargo test -p osatura-cli --test spdx_audit`, `cargo clippy --workspace --all-targets -- -D warnings`, cele cinci teste smoke Chromium, `scripts/e2e.sh` (release), `cargo test -p osatura-gpu` dacă s-a schimbat codul GPU.
3. **Notificări terțe.** Rulează `scripts/licence-audit.sh`: regenerează `docs/THIRD_PARTY.md` (inventarul) și `docs/THIRD_PARTY_NOTICES.md` (textele de licență ale a tot ce se livrează), verifică liniile SPDX și rulează `cargo deny`. Comite fișierele regenerate. O distribuție binară trebuie să livreze pachetul de notificări alături de binar.
4. **Fișiere de licență.** `COPYRIGHT`, `LICENSES/` (textele fiecărei licențe folosite; fișierele `LICENSE-*` din rădăcină sunt copii identice ale lor) și `REUSE.toml` sunt sincronizate cu [LICENSING.md](LICENSING.md); atribuirea CC-BY-4.0 pentru documentație este intactă.
5. **Fără ffmpeg.** Nu include, nu lega și nu descărca ffmpeg în artefactele de lansare. Este detectat în `PATH` la rulare, iar documentația spune că utilizatorul îl instalează.
6. **Oferta sursei AGPL.** Binarul `osatura-cli` servește propria ofertă de sursă la `/source` (AGPL secțiunea 13) și afișează o notificare la `--version`; `release.yml` stabilește `OSATURA_SOURCE_URL` la compilare la `<server>/<repository>/tree/<commit>`, astfel încât binarele de lansare să numească sursa exactă din care au fost construite (testul de fum al workflow-ului verifică `--version` pentru asta; un simplu `cargo build` nu are adresă încorporată); operatorii serverelor studio modificate stabilesc `--source-url` la adresa unde se află sursa lor modificată. Teste: `tests/source_offer.rs`. Pentru o lansare binară, oferă sursa corespunzătoare (tag, tarball) la adresa la care utilizatorii pot ajunge.
7. **Documentație.** Actualizează caseta de stare din README, [LIMITATIONS.md](LIMITATIONS.md) și CHANGELOG (mută *Unreleased* sub noua versiune, cu dată).
8. **Construiește ciorna pe GitHub.** `.github/workflows/release.yml` rulează **doar** pentru un tag trimis `vMAJOR.MINOR.PATCH[-pre]` (sau manual de pe `main`, caz în care construiește, dar nu creează nicio lansare). Un prim job verifică scrierea tag-ului și că commit-ul lui se află pe `main`; apoi construiește arhivele pentru Windows, macOS și Linux (glibc și musl) și modulul web, semnează totul cu Sigstore, atestă proveniența, scrie SBOM-ul și `SHA256SUMS.txt` și creează o *pre-lansare ciornă* (o ciornă mai veche cu același tag este înlocuită). Verifică ciorna, compar-o cu ghidul [VERIFY_DOWNLOADS.md](../../VERIFY_DOWNLOADS.md), apoi publică-o manual.
9. **Etichetează și publică.** Trimite un tag **semnat** (`git tag -s v0.1.0-alpha.5`) de la un commit aflat deja pe `main`; trimiterea pornește pasul 8. Nu publica crate-uri sub nume care nu sunt ale tale; regulile din TRADEMARK.md se aplică fork-urilor.

## 8. Integrare continuă

Două lucruri diferite folosesc același cuvânt (un al treilea workflow, `.github/workflows/release.yml`, construiește arhivele de lansare și lansarea ciornă, vezi secțiunea 7; un al patrulea, `codeql.yml`, analizează codul):

* **CI de animație** (`osatura ci`): verificări ale *proiectelor* (lint, goldens de randare, performanță, determinism). Descris complet în [CI.md](CI.md): manifestul `osatura-ci.json`, praguri, goldens (`--update` față de `bless`), rapoarte, exemple GitHub Actions și shell simplu (marcate acolo „example, not run”).
* **CI de cod** pentru acest repository: `.github/workflows/ci.yml` are patru feluri de joburi: `test` (matrice ubuntu / windows / macos: `cargo test --workspace --exclude osatura-gpu --locked` și clippy cu `-D warnings`), `wasm32` (construiește `osatura-wasm` pentru `wasm32-unknown-unknown`, copiază modulul lângă `osatura.js` și rulează `node osatura-wasm/js/test.mjs` cu un proiect demo, inclusiv partea cu modulul real), `gpu parity` (informativ; se omite când nu există adaptor, rulează pe adaptorul software DX12 de pe Windows) și `licences` (`cargo-deny check`, care include acum advisories, interdicții și crate-uri retrase, și `reuse lint`). Toolchain-ul este fixat în `rust-toolchain.toml` (1.97.0) ca lint-urile clippy noi să nu strice build-ul; fiecare job rulează și `rustup toolchain install` ca pas de siguranță, iar workflow-urile au grupuri `concurrency`, astfel încât un push mai nou anulează rularea CI mai veche (niciodată o lansare). Nu există în mod deliberat `paths-ignore`: un workflow omis lasă o verificare obligatorie în așteptare pe vecie. Jobul `test` construiește și `osatura-gui` și rulează `osatura-studio --selftest` (un autotest real pe Windows și macOS). **Prima rulare complet verde: 2026-10-24** pe runnere găzduite de GitHub. La eșec, pașii afișează testele eșuate ca adnotări de workflow (lizibile fără acces la loguri). Windows omite testele doar-POSIX (symlink-uri, permisiuni, `/dev/zero`, separatoare de cale POSIX). Comenzile din care este alcătuit, așa cum au fost rulate la scrierea acestei documentații:

  ```sh
  cargo test --workspace --locked
  cargo test --doc -p osatura-core --locked
  cargo clippy --workspace --all-targets --locked -- -D warnings
  scripts/licence-audit.sh            # needs python3 and (optional) cargo-deny
  ```

  Pune în cache `~/.cargo/registry` și `target/` între rulări, lasă testului e2e câteva minute în build-urile debug și rulează testele smoke Chromium doar pe runner-e care au Chromium și `playwright-core`.

## 9. Depanare

* **Backtrace-uri.** `RUST_BACKTRACE=1 osatura ...` pentru orice panică (o panică la intrare este un bug; te rugăm să o raportezi împreună cu proiectul care o declanșează).
* **Ieșire citibilă de mașină.** `--json` la comenzile care îl suportă; `--quiet` pentru scripturi. Coduri de ieșire: 0 la succes, 1 la eșec (verificare eșuată, buget neîndeplinit, utilizare greșită; run: o comandă necunoscută și un argument lipsă ies amândouă cu 1); `osatura ci` separă 1 (o verificare a eșuat) de 2 (eroare).
* **Inspectarea unui proiect.** `osatura check`, `osatura lint` (exit 1 la erori) și `osatura diff a.json b.json` dau structura și problemele fără randare; `osatura render project.json N out.png` randează un cadru, iar `out.svg` dă o ieșire vectorială pe care o poți deschide într-un browser (`--bones` arată scheletul).
* **Studio.** Rulează `osatura studio project.json --port 0` (afișează portul real). Uneltele de dezvoltator ale browserului arată API-ul JSON: cererile sunt `POST /api/...` simple, cu `Content-Type: application/json`. Un 421 înseamnă un antet Host care nu este în `--allow-host`; un 403 înseamnă că o verificare Origin sau `Sec-Fetch-Site` a eșuat (folosește `--public-origin` în spatele unui proxy); 415 se referă la tipul de conținut; 429 este back-off-ul tokenului.
* **Etapele demo-ului.** Demo-ul ghidat al Studioului (și demo-ul pas cu pas de pe site-ul proiectului) sunt generate din `osatura demo-person` de `scripts/build-studio-tour.mjs` și `scripts/lib/person-stages.mjs`; build-ul eșuează dacă demo-ul finalizat nu mai corespunde cifrelor citate în texte (`EXPECT` din bibliotecă). După modificarea `osatura-core/src/demo.rs`, rulează din nou scriptul și reconstruiește CLI-ul.
* **Jurnalul de colaborare.** Jurnalul unei sesiuni partajate (`--collab-log`) este text, o linie `<hash> <json>` pe operație; un lanț rupt este raportat la încărcare.
* **Un test eșuează doar uneori.** Caută ceasul, numele directoarelor temporare și ordinea de iterare a `HashMap`; codul motorului este determinist prin construcție, deci eșecurile intermitente sunt aproape întotdeauna o problemă de igienă a testelor.
* **Snapshot-ul fațadei eșuează.** Ai schimbat o semnătură publică: vezi secțiunea 5, punctul 6.
* **GPU.** `osatura render-gpu <project> <out.png> --adapter software|auto --bench N` (doar în build-uri cu `--features gpu`) compară cu renderer-ul CPU (un adaptor software afișează că duratele lui nu înseamnă nimic); `OSATURA_GPU_BACKEND` selectează backend-ul.
* **Sanitizere pentru C ABI.** Compilează `smoke.c` sau propriul apelant cu `-fsanitize=address,undefined` față de biblioteca statică (făcut în timpul lucrului la documentație, curat; vezi [C_ABI.md](C_ABI.md)).

## 10. Profilare

* **Durate pe etape ale pipeline-ului de producție:** `scripts/e2e.sh` afișează un tabel (`OSATURA_E2E_TIMINGS` numește fișierul); cifrele de release din `ROADMAP.md` secțiunea 9 provin de acolo.
* **GPU față de CPU:** `osatura render-gpu <project> <out.png> --bench N`.
* **Buget de dimensiune și viteză:** `osatura size-report`, `osatura export-fit --max-seconds` (bugetul de timp este verificat în interiorul buclelor de candidați; un apel ffmpeg sau de cuantizare nu poate fi întrerupt).
* **Costul de randare al unui proiect:** `osatura ci` cu un prag de performanță ([CI.md](CI.md)) înregistrează timpul pe cadru pe mașina ta; duratele depind de mașină, așa că compară pe aceeași mașină.
* **Unelte generale:** construiește cu `--release` și folosește `perf record`/`perf report` sau `cargo flamegraph` (nu fac parte din proiect; se instalează separat). Build-urile debug sunt cam cu un ordin de mărime mai lente și nu spun nimic despre viteza reală.
