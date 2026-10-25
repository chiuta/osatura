# OSATURA — licencelés (ELDÖNTVE: A változat, alkalmazva 2026. okt. 22-én; lásd COPYRIGHT és LICENSES/)

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/LICENSING.md) · [Română](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/ro/LICENSING.md) · [Français](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/fr/LICENSING.md) · [Italiano](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/it/LICENSING.md) · [Español](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/es/LICENSING.md) · [Português](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/pt/LICENSING.md) · [Deutsch](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/de/LICENSING.md) · [Magyar](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/hu/LICENSING.md)

**Állapot:** Az alábbi A változatot választottuk és alkalmaztuk: MPL-2.0 a könyvtárakra (core, capi, wasm, gpu), AGPL-3.0-or-later az `osatura-cli`-re (CLI + studio) és az `osatura-gui`-ra (a studio natív ablakban), CC-BY-4.0 a dokumentációra és a szintetikus asset-ekre/modellekre, védjegyszabályzat a TRADEMARK.md-ben, DCO a CONTRIBUTING.md-ben, SPDX-fejlécek minden .rs/.js/.mjs/.sh fájlban és REUSE.toml a többihez, AGPL 13. szakasz szerinti forrásajánlat (`/source`, `--source-url`), a Cargo `license` mezők beállítva. Hátralévő tulajdonosi teendők: jogi felülvizsgálat, védjegykeresés (EUIPO/OSIM), szabadalmi átvilágítás (docs/PATENT_NOTES.md), CLA, ha kettős licencelést szeretnénk, harmadik felek értesítéseinek mellékelése a bináris kiadásokhoz.


*Nem jogi tanács. A licenc ELDÖNTÖTT (A változat, lásd COPYRIGHT); az alábbi szakaszok indoklásként és a mérlegelt alternatívák bemutatásaként maradtak meg. Az első nyilvános kiadás előtt erősíttesd meg egy jogásszal (lehetőleg olyannal, aki ismeri a román/uniós szerzői jogot). Hozzájárulások: amíg nincs CLA, harmadik felek hozzájárulásaira nem áll fenn újralicencelési vagy kettős licencelési jog (CONTRIBUTING.md).*

## 1. Mit kell tudnia a licencnek a te érdekedben
A bevételed **adományokból (Buy Me a Coffee)** származik, nem licencdíjakból. A licenc tehát nem teremt pénzt — **a jóindulat, a láthatóság és a bizalom teremt**. Három feladata van:
1. **A felhasználók számának maximalizálása** (több felhasználó → több támogató): ingyen használhatónak kell lennie, kereskedelmi munkában is.
2. **Megakadályozni, hogy egy cég elvigye a kódot, zárttá tegye és eladja** (vagy fizetős szolgáltatásként üzemeltesse), anélkül hogy bármit visszaadna.
3. **Megőrizni a jogodat kivételek eladására** (kereskedelmi licenc, támogatás) és a név védelmére.

Összhangban kell lennie a **trade-free / TROM** meggyőződéseiddel is: a szoftver nem lehet megcsonkított, hirdetésekből finanszírozott vagy adatgyűjtéshez kötött. (A kereskedelmi felhasználást tiltó licencek, például a CC-NC vagy a Polyform-NC, *nem* nyílt forráskódúak, és nem azonosak a „trade-free” fogalommal; bármelyik elfogadása előtt nézd meg a TROM saját megfogalmazását.)

## 2. A választást korlátozó megállapítások
- **A függőségek nem akadályok.** Az összes külső crate auditja (363 a `scripts/licence-audit.sh` 2026. okt. 24-i újragenerálásakor, az ablakos alkalmazás hozzáadása után az összes platformon együtt számolva; 2026. okt. 22-én 160 volt; a szám változik, lásd docs/THIRD_PARTY.md): mind MIT, Apache-2.0, BSD, Zlib, ISC, Unlicense, CC0, Unicode vagy ezek kettős licencei, hat gyenge (fájlszintű copyleftes) MPL-2.0 crate kivételével: smartstring (MPL-2.0+, csak a `scripting` feature-ön keresztül érhető el, amely alapértelmezés szerint BE van kapcsolva az `osatura-core`-ban és az `osatura-cli`-ben), valamint a `cssparser`, `cssparser-macros`, `dtoa-short`, `selectors` és `option-ext` (csak az `osatura-gui` ablakos alkalmazáson keresztül érhetők el); a könyvtárak `--no-default-features` kapcsolóval épített változatában nincs MPL-licencű külső crate. Kombinálhatók AGPL-lel, MPL-lel vagy zárt (proprietary) licenccel; az MPL crate esetében meg kell őrizni a forrást/értesítést.
- **Szerzői jogi tulajdon.** A kód nagy részét Claude-dal írtuk. A szerzői jog általában az *emberi* alkotói hozzájáruláshoz kötődik (az amerikai Szerzői Jogi Hivatal iránymutatása; az uniós/román jog is az emberi szerző saját szellemi alkotását követeli meg). Gyakorlati következmények: (a) a projektet jogosan és érdemes az emberi szerzőjeként/összeállítójaként licencelned, de (b) ne ígérj kizárólagosságot minden sorra, (c) tartsd a repóban a saját tervezési jegyzeteidet, válogatásaidat és szerkesztéseidet a szerzőség bizonyítékaként, (d) a DCO csak az eredetet igazolja; a hozzájárulások későbbi újralicenceléséhez (pl. kettős licencelés) CLA vagy kifejezett kimenő jogátruházás kell, és ez NINCS meg.
- **Semmilyen licenc nem kényszeríthet adományra.** Soha ne zárj kódot adomány mögé; egy fork egyszerűen eltávolítaná a zárat. *Kényelmet és extrákat* adj el, ne engedélyt.

## 3. Ajánlott csomag — „Nyitott core könyvtár, védett alkalmazás” (az alábbi összes változat alkalmazásra kész)

### ★ A változat (ajánlott): MPL-2.0 könyvtár + AGPL-3.0 alkalmazások + védjegy
| Rész | Licenc | Miért |
|---|---|---|
| `osatura-core` (motor, kiértékelő, formátumok, exportálók) és minden jövőbeli runtime/WASM/C-ABI | **MPL-2.0** | Fájlszintű copyleft: bárki beágyazhatja játékokba/alkalmazásokba (zártakba és nyitottakba is), de a *mi fájljainkon* végzett módosításokat meg kell osztani. Kifejezett **szabadalmi felhatalmazást** tartalmaz, és szabadalmi per esetén megszűnik. Maximalizálja az elterjedést, és az adományokhoz erre van szükség. |
| `osatura-cli`, `osatura-gui`, studio (böngésző/ablak) | **AGPL-3.0-or-later** | Megakadályozza a zárt forkokat és az „OSATURA mint fizetős webszolgáltatás” klónokat; a felhasználók saját *projektjei és renderei* teljes egészében az övék maradnak. |
| Az „OSATURA” név, logó | **Védjegyszabályzat** (külön fájl) | A forkoknak nevet kell váltaniuk; megvédi a támogatókat az utánzatoktól. Az indulás előtt ellenőrizd az elérhetőséget (EUIPO/OSIM/WIPO). |
| Általad készített minta-rigek, grafikák, hanganyagok | **CC BY 4.0** (vagy CC BY-SA) | Szabadon újrahasznosítható, forrásmegjelöléssel; a harmadik fél grafikái megtartják a saját licencüket. |
| Dokumentáció | **CC BY 4.0** | |
<!-- REUSE-IgnoreStart -->
Cargo: `license = "MPL-2.0"` (core), `license = "AGPL-3.0-or-later"` (cli). SPDX header per file: `// SPDX-License-Identifier: MPL-2.0`.
<!-- REUSE-IgnoreEnd -->
**Kettős licencelés:** csak olyan kódra lehetséges, amelyet kizárólag a tulajdonos írt, vagy minden közreműködő CLA-ja/kimenő jogátruházása után. Ma a hozzájárulásként érkezett kódra NEM lehetséges (CONTRIBUTING.md).

### B változat: AGPL-3.0-or-later mindenhol (+ kereskedelmi licenc kérésre)
A legerősebb eltérítés elleni védelem, és legközelebb áll a trade-free szellemiséghez. Ára: a cégek és a játékmotorok kerülni fogják a runtime beágyazását, így kevesebb felhasználó és kevesebb közvetett támogató lesz. Akkor válaszd, ha a védelem fontosabb az elérésnél.

### C változat: Apache-2.0 mindenhol (nem ezt választottuk; a workspace az A változatot alkalmazza: MPL-2.0 könyvtárak és `license = "AGPL-3.0-or-later"` az `osatura-cli`-re)
Maximális elérés és nulla súrlódás; szabadalmi felhatalmazással. Ára: bárki — egy nagy cég is — zárttá teheti és eladhatja, kötelezettség nélkül. Az adományok ekkor kizárólag a jóindulaton múlnak. Csak akkor válaszd, ha közműszerű szolgáltatás akarsz lenni.

### D változat: Source-available (BUSL-1.1 vagy Polyform Shield/NC, késleltetett nyitási dátummal)
N évig megakadályozza a kereskedelmi versenyt. Ára: **nem nyílt forráskódú**; ütközik a clean-room/nyílt formátum narratívával, a közösségi hozzájárulásokkal és a trade-free értékekkel; sok felhasználó és disztribúció elutasítja. **Nem ajánlott.**

| | Elérés | Eltérítés elleni védelem | Szabadalmi felhatalmazás | Kettős licencelésből származó bevétel | Illik a trade-free-hez |
|---|---|---|---|---|---|
| **A MPL core + AGPL alkalmazások** | magas | jó | igen | igen | igen |
| B AGPL mindenhol | közepes | legerősebb | igen (v3) | igen | igen |
| C Apache mindenhol | legmagasabb | nincs | igen | nem | részben |
| D source-available | alacsony | erős | n/a | igen | nem |

## 4. Hogyan keress ténylegesen adományokból (mind összeegyeztethető az A változattal)
1. **Buy Me a Coffee oldal** (`buymeacoffee.com/echo.of.the.strings`), linkelve a README-ből, a CLI `--version` szövegéből, a studio láblécéből és a HTML lejátszó kreditsorából (*eltávolítható* kredit — soha nem kötelező).
2. **Tagságok, nem paywallok:** havi szintek korai buildekhez, szavazás a következő ütemterv-tételről, név a `SUPPORTERS.md`-ben, privát chat. Mindenki ugyanazt a kódot kapja; a tagok *időt és beleszólást* kapnak.
3. **Fizetős extrák, amelyek nem a kód:** aláírt, azonnal futtatható binárisok/telepítők, asset-csomagok (rigek, hátterek, járásciklus-könyvtárak) olyan licenc alatt, amely engedi a filmekben/játékokban való használatot, de a csomag viszonteladását nem, videókurzusok, workshopok.
4. **Megrendelések és támogatás:** rigelési szolgáltatások, egyedi exportálók, integrációs tanácsadás; a beágyazóknak szóló kettős kereskedelmi licenc.
5. **Szponzori szintek szervezeteknek** (logó a README-ben) és pályázatok nyílt forráskódú/kreatív alapokhoz (NLnet, Sovereign Tech-típusú programok, kulturális alapok) — pontosan az ilyen nyílt infrastruktúrát fizetik.
6. **Átláthatósági oldal:** tedd közzé a bevételeket és azt, hogy mit finanszíroztak belőlük; mérhetően növeli az ismétlődő támogatást.
7. Tartsd láthatóan a **Patreont** (`patreon.com/c/alexio_tf`) és a BMC-t is, de adj mindkettőnek külön célt (BMC egyszeri köszönet; Patreon ismétlődő), hogy a támogatók ne zavarodjanak össze.

## 5. Ellenőrzőlista az A változat alkalmazásához (≈ 1 óra munka a választás után)
- [x] `LICENSES/MPL-2.0.txt`, `LICENSES/AGPL-3.0-or-later.txt`, `LICENSES/CC-BY-4.0.txt`, `LICENSES/LicenseRef-third-party.txt` (REUSE-elrendezés) plusz az összefoglaló `COPYRIGHT`; a `REUSE.toml` lefedi a nem forrásfájlokat.
- [x] `license =` beállítva mind a hat `Cargo.toml` fájlban; SPDX-fejlécek minden .rs/.js/.mjs/.sh fájlban, az `osatura-cli/tests/spdx_audit.rs` ellenőrzi.
- [x] `docs/THIRD_PARTY.md` és `docs/THIRD_PARTY_NOTICES.md`, mindkettőt a `scripts/licence-audit.sh` generálja újra.
- [x] `CONTRIBUTING.md` a DCO 1.1 szövegével. Bármilyen kettős licenc előtt még szükség van CLA-ra.
- [x] `TRADEMARK.md` (megengedett: „compatible with OSATURA”, módosítatlan újraterjesztés; nem megengedett: módosított buildek ugyanezen a néven).
- [x] `.github/FUNDING.yml` (`buy_me_a_coffee: echo.of.the.strings`), hogy a platform megjelenítse a Sponsor gombot.
- [~] Védjegykeresés: az első név, az ARMATURA, ütközött egy 9/42. osztályú védjeggyel (EUIPO, 2026. okt. 24.), ezért a projektet OSATURA-ra neveztük át; az „osatura” EUIPO-keresése nem talált ütközést (egy nem kapcsolódó, lejárt találat a 10. osztályban). Az `osatura` csomagnevek a crates.io-n, az npm-en és a PyPI-n szabadok (ellenőrizve 2026. okt. 24-én), de létezik egy közeli védjegy, az OSATRA (Open-Xchange AG, US lajstromszám 7459236, 9/38/42. osztály). Az OSIM- / TMview- / WIPO- / domainkeresések és egy ügyvédi vélemény még nyitottak, lásd [docs/LIMITATIONS.md](LIMITATIONS.md) L-78.
