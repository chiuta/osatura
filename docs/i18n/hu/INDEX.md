<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Az OSATURA dokumentációjának tartalomjegyzéke

*Itt kezdj, hogy megtaláld a megfelelő oldalt. Minden dokumentum angol nyelvű Markdown CC-BY-4.0 licenc alatt, kivéve a generált harmadik féltől származó fájlokat (lásd a fejlécüket).*

Az OSATURA egy Rustban írt 2D vektoros riggelő és animációs motor. Alfa projekt (az első buildek a `v0.1.0-alpha.3` előzetes kiadás-vázlatként készülnek; a crate-ek továbbra is 0.1.0); mielőtt egy funkcióra támaszkodnál, olvasd el a [LIMITATIONS.md](LIMITATIONS.md) fájlt.

Tartalom: [Minden dokumentum](#minden-dokumentum) - [Olvasási útvonalak célközönség szerint](#olvasási-útvonalak-célközönség-szerint) - [A tároló gyökerében lévő dokumentumok](#a-tároló-gyökerében-lévő-dokumentumok) - [Hol található válasz az egyes kérdésekre](#hol-található-válasz-az-egyes-kérdésekre) - [Konvenciók](#a-dokumentumokban-használt-konvenciók)

## Minden dokumentum

| Dokumentum | Mi ez | Célközönség |
|---|---|---|
| [GETTING_STARTED.md](GETTING_STARTED.md) | Telepítés, első projekt, első renderelés és export, első stúdió-munkamenet | mindenki, első olvasmány |
| [CLI_REFERENCE.md](CLI_REFERENCE.md) | Minden `osatura` parancs, opció, kilépési kód és példa | felhasználók, szkriptelők, CI-szerzők |
| [STUDIO.md](STUDIO.md) | A böngészős szerkesztő: panelek, gyorsbillentyűk, együttműködés, görgetési előnézetek | animátorok, riggerek |
| [RIGGING_GUIDE.md](RIGGING_GUIDE.md) | Rig építése: csontok, IK, skinning, deformerek, driverek, állapotgépek, elrendezés | riggerek, technikai művészek |
| [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) | Kulcsok, easing, klipek és take-ek, fizika, lip-sync, hang, exportválasztás | animátorok |
| [TUTORIAL_SVG.md](../../TUTORIAL_SVG.md) | Vektorrajzból sétáló GIF: importálás, rigelés szkripttel, export (a parancsok és kimenetek valódiak) | művészek |
| [COMPARISON.md](../../COMPARISON.md) | Őszinte, dátumozott összehasonlítás más 2D animációs eszközökkel | értékelők |
| [VERIFY_DOWNLOADS.md](../../VERIFY_DOWNLOADS.md) | A kiadási fájlok ellenőrzése: Sigstore-aláírások, build-eredet, SHA-256 összegek | mindenki, aki telepít |
| [UNINSTALL.md](../../UNINSTALL.md) | Az OSATURA eltávolítása és minden fájlé, amelyet létrehozhat | mindenki, aki telepít |
| [../CODE_SIGNING_POLICY.md](../../../CODE_SIGNING_POLICY.md) · [../PRIVACY.md](../../../PRIVACY.md) | Kódaláírási szabályzat (SignPath Foundation), csapatszerepek, adatvédelem | mindenki, aki telepít, karbantartók |
| [GOOD_FIRST_TASKS.md](../../GOOD_FIRST_TASKS.md) | Kis feladatok az első közreműködéshez | új közreműködők |
| [PROJECT_FORMAT.md](PROJECT_FORMAT.md) | A JSON projektformátum mezőről mezőre | eszközfejlesztők, JSON-t szerkesztő riggerek |
| [EXPORT_FORMATS.md](EXPORT_FORMATS.md) | Mit visz és mit hagy el az egyes export- és importformátum | pipeline-építők, integrátorok |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Crate-ek, modulok, adatfolyam, kiértékelési és renderelési felépítés | közreműködők, átnézők |
| [GLOSSARY.md](GLOSSARY.md) | A motorban, a dokumentációban és a CLI-ben használt fogalmak | mindenki |
| [FAQ.md](FAQ.md) | Rövid válaszok gyakori kérdésekre | mindenki |
| [API.md](API.md) | A stabil Rust homlokzat (1.4.0-s verzió): hívások, hibák, korlátok, példák | Rust-integrátorok |
| [C_ABI.md](C_ABI.md) | A C interfész (`osatura-capi`): függvények, memóriaszabályok, C- és Python-példák | C, C++, Python, Go, C# integrátorok |
| [WASM.md](WASM.md) | WebAssembly modul és JavaScript wrapper. **Állapot: a CI-ben lefordítva és Node alatt tesztelve, headless Chromiumban futtatva, Firefoxban és Safariban még nem**; tartalmazza az első böngészős futtatás ellenőrzőlistáját | webfejlesztők |
| [CI.md](CI.md) | Animációs CI (`osatura ci`): manifeszt, küszöbök, referenciaképek, jelentések, pipeline-példák | technikai művészek, build-mérnökök |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Fordítás, tesztek, konvenciók, új modul ellenőrzőlistája, kiadási ellenőrzőlista, hibakeresés | közreműködők, karbantartók |
| [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) | A tulajdonosi lépések a vázlatkiadás közzététele előtt és után | tulajdonos |
| [SECURITY.md](SECURITY.md) | Fenyegetésmodell, beépített védelmek, ismert hiányosságok, bejelentés, megerősítési ellenőrzőlista | biztonsági átnézők, telepítők |
| [LIMITATIONS.md](LIMITATIONS.md) | Az ismert korlátok számozott listája súlyossággal, kerülőmegoldással és ütemterv-azonosítóval | mindenki, aki az OSATURA-t értékeli |
| [PATENT_NOTES.md](PATENT_NOTES.md) | Szabadalmi állapotjegyzet (nem jogi tanács) | licencbirtokosok, jogászok |
| [THIRD_PARTY.md](THIRD_PARTY.md) | A harmadik féltől származó crate-ek és licencek generált jegyzéke | licencbirtokosok, csomagolók |
| [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md) | A függőségek generált licencszövegei, a binárisokkal együtt szállítandók | csomagolók |

## Olvasási útvonalak célközönség szerint

Minden útvonal egy sorrend, nem követelmény. Állj meg, ha megvan, amire szükséged van.

**Új felhasználó (látni akarja működni).** [../README.md](../../../README.hu.md) (gyors kezdés) -> [GETTING_STARTED.md](GETTING_STARTED.md) -> [STUDIO.md](STUDIO.md) -> [FAQ.md](FAQ.md) -> [LIMITATIONS.md](LIMITATIONS.md).

**Rigger.** GETTING_STARTED -> [RIGGING_GUIDE.md](RIGGING_GUIDE.md) -> [PROJECT_FORMAT.md](PROJECT_FORMAT.md) (a kézzel szerkesztett mezőkhöz) -> [GLOSSARY.md](GLOSSARY.md) -> [CLI_REFERENCE.md](CLI_REFERENCE.md) (a `lint`, `check`, `fuzz`, `autorig`, `machine` parancsok) -> [CI.md](CI.md) a rig védelméhez.

**Animátor.** GETTING_STARTED -> STUDIO -> [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) -> [EXPORT_FORMATS.md](EXPORT_FORMATS.md) (a kimeneti formátum kiválasztása) -> CLI_REFERENCE (`export`, `export-fit`).

**Pipeline- vagy eszközfejlesztő (szkriptek, más formátumok).** CLI_REFERENCE -> PROJECT_FORMAT -> EXPORT_FORMATS -> [CI.md](CI.md) -> [SECURITY.md](SECURITY.md) 4-7. szakasz (eszköz- és fájlszabályok).

**Integrátor (a motor beágyazása).** [ARCHITECTURE.md](ARCHITECTURE.md) áttekintés -> a réteg kiválasztása: Rust ([API.md](API.md)), C és más nyelvek ([C_ABI.md](C_ABI.md)), böngésző ([WASM.md](WASM.md), előbb az állapotdobozt olvasd el) -> [LIMITATIONS.md](LIMITATIONS.md) 1. és 8. szakasz -> [SECURITY.md](SECURITY.md) 8. szakasz.

**Közreműködő.** [../CONTRIBUTING.md](CONTRIBUTING.md) (DCO) -> [DEVELOPMENT.md](DEVELOPMENT.md) -> ARCHITECTURE -> [../CLEANROOM.md](CLEANROOM.md) -> [../ROADMAP.md](ROADMAP.md) 9. szakasz (mi van meg, és mi nincs) -> LIMITATIONS (válassz valamit javításra).

**Biztonsági átnéző vagy telepítő.** [SECURITY.md](SECURITY.md) -> [DEVELOPMENT.md](DEVELOPMENT.md) 3. szakasz (mi van tesztelve) -> LIMITATIONS 2. és 11. szakasz -> `osatura-cli/src/http.rs`, `studio.rs`, `collab_srv.rs`, `osatura-core/src/assets.rs`, `fsutil.rs` -> [CI.md](CI.md) korlátok.

**Licencbirtokos, csomagoló vagy jogász.** [../LICENSING.md](LICENSING.md) -> [../COPYRIGHT](../../../../../COPYRIGHT) és [../LICENSES/](../../../LICENSES/) -> [THIRD_PARTY.md](THIRD_PARTY.md) -> [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md) -> [PATENT_NOTES.md](PATENT_NOTES.md) -> [../TRADEMARK.md](TRADEMARK.md) -> [../CLEANROOM.md](CLEANROOM.md) -> kiadási ellenőrzőlista a [DEVELOPMENT.md](DEVELOPMENT.md) 7. szakaszában.

**Kiadást készítő karbantartó.** DEVELOPMENT 7. szakasz -> [../CHANGELOG.md](CHANGELOG.md) -> LIMITATIONS (frissítés) -> a README állapotdoboza.

## A tároló gyökerében lévő dokumentumok

| Fájl | Cél |
|---|---|
| [../README.md](../../../README.hu.md), [../README.ro.md](../../../README.hu.md) | Áttekintés, állapot, gyors kezdés, licencösszefoglaló (angol és román) |
| [../CHANGELOG.md](CHANGELOG.md) | Rekonstruált előzmények; Keep-a-Changelog formátum |
| [../ROADMAP.md](ROADMAP.md) | Rendezett terv; **a 9. szakasz a keltezett napló arról, hogy az egyes tételek mit tudnak és mit nem**, és ez az irányadó, ha a dokumentumok eltérnek |
| [../FEATURES.md](FEATURES.md) | Funkciómátrix más eszközökkel szemben (részei elavultak, lásd LIMITATIONS) |
| [../CLEANROOM.md](CLEANROOM.md) | Eredetnapló és szabályok a független megvalósításhoz |
| [../CONTRIBUTING.md](CONTRIBUTING.md) | Közreműködési szabályok, bejövő = kimenő licenc, DCO 1.1 |
| [../LICENSING.md](LICENSING.md), [../COPYRIGHT](../../../../../COPYRIGHT), [../LICENSES/](../../../LICENSES/) | Licencdöntés (A változat), szövegek |
| [../TRADEMARK.md](TRADEMARK.md) | A név- és logóhasználati szabályzat |
| `../REUSE.toml`, `../deny.toml` | Licenc-metaadatok a nem forrásfájlokhoz; függőségi szabályzat |

## Hol található válasz az egyes kérdésekre

| Kérdés | Oldal |
|---|---|
| Hogyan telepítsem, és hogyan készítsem el az első animációmat? | GETTING_STARTED |
| Mit csinál az X parancs? | CLI_REFERENCE |
| Mit jelent ez a mező a JSON-ban? | PROJECT_FORMAT, GLOSSARY |
| Melyik exportformátum őrzi meg a maszkokat / színátmeneteket / hangot? | EXPORT_FORMATS |
| Használhatom Pythonból / C-ből / a böngészőben? | C_ABI / WASM (állapot!) |
| Biztonságos a stúdiót hálózatra kitenni? | SECURITY 11. szakasz |
| Miért nem működik az X? Ismert ez? | LIMITATIONS, FAQ |
| Hogyan készül és hogyan tesztelik? | DEVELOPMENT |
| Használhatom kereskedelmi célra? Milyen licencek vannak? | README (licenc), LICENSING, THIRD_PARTY |
| Mi változott? | CHANGELOG, ROADMAP 9. szakasz |

## A dokumentumokban használt konvenciók

* Az **„Ellenőrizve”** azt jelenti, hogy egy parancsot vagy programot lefuttattak a dokumentumok írása közben, Linux x86-64 környezetben, GPU nélkül, webkamera nélkül és csak headless Chromiummal. A **„Nincs tesztelve”** pontosan ezt jelenti. A WebAssemblyre vonatkozó, valódi motorban érvényes állítások mindig ellenőrizetlenek.
* Az API.md és a C_ABI.md kódmintái lefordított és lefuttatott programok; a WASM.md mintái csak a JavaScript mock ellen futottak.
* A ROADMAP 9. szakasza az irányadó abban, hogy egy funkció mit tud és mit nem. Ha egy dokumentum eltér tőle vagy a kódtól, a kód az erősebb; kérjük, jelezd az eltérést.
* Az útvonalak a tároló gyökeréhez viszonyítottak, hacsak nem `docs/` előtaggal kezdődnek.
* A számok (tesztszámok, korlátok) a forrásból vagy a 2026. október 22-i futásból származnak, és a következő változás után elavulhatnak; a forrás az irányadó.
