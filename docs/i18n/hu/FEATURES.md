# OSATURA — funkciómátrix + referencia- és versenytárs-ellenőrzőlisták

> Verziójelölés: az összes crate `0.1.0` (az első buildek a `v0.1.0-alpha.3` előzetes kiadás-vázlatként készülnek); a „v0.5 / v0.6 / v0.7” ebben a repositoryban *funkciószintet* jelent (lásd [CHANGELOG.md](CHANGELOG.md)), és hogy mi készült el, azt a `ROADMAP.md` 9. szakasza dönti el.

Őszinte állapotkép arról, ami létezik. **✅ kész és tesztelt · 🟡 részleges · ❌ nincs megvalósítva.**

> Hatókör: az OSATURA önállóan megvalósított motor (nyilvános specifikációkból és tanulmányokból; a versenytársak funkciólistáit a hatókör felméréséhez használtuk; harmadik féltől származó forráskódot nem másoltunk; a kutatás ismertetését lásd a CLEANROOM.md fájlban), nem valamely
> kereskedelmi termék újraimplementálása, és **nem** éri el egy 25 éves kereskedelmi csomag szintjét
> (kiforrott GUI, rajzeszközök, élő bemenet, 3D, ökoszisztéma). Az első táblázatok képesség szerint
> szerveződnek; a **fájl második fele** a referenciatermék nyilvánosan dokumentált funkcióinak verziónkénti ellenőrzőlistája (a források a végén vannak felsorolva),
> az OSATURA mindegyikhez tartozó állapotával. Az „újítások” kizárólag az általános technikához képest értendők,
> nem azt állítják, hogy bármely más eszközben bizonyítottan hiányoznának.

---

# ★ ÉPÍTÉSI LÉTRA — az egyszerűtől a bonyolultig, szerzői jogi / IP-státusszal (funkciószint v0.7, 2026. okt. 20–22.)

Ez a szakasz az **egyetlen hely**, ahol látható, mi építhető, milyen sorrendben, és biztonságos-e.
Felváltja a lejjebb szétszórt „jelölt #” listákat; a részletek és az azonosítók a `ROADMAP.md` fájlban
(R-xxx / I-xx), a clean-room módszer a `CLEANROOM.md` fájlban, a licencek a `LICENSING.md` fájlban találhatók.

**Állapot:** ✅ megépítve és tesztelve · 🟡 részben megépítve · ⬜ tervezett (nem kezdődött el)
**IP-státusz (a ROADMAP 8. szakaszának alternatívái után):**
🟢 biztonságos — nyílt szabvány, tankönyvi matematika vagy saját tervezés ·
🟠 biztonságos, *ha* betartasz egy feltételt (az utolsó oszlopban szerepel) ·
🔴 ne építsd meg.
*Mérnöki szűrés, nem jogi tanács; minden 🟠 tételt jogásznak kell megerősítenie a kereskedelmi kiadás előtt.*

## 1. szint — Alapok (egyenként órák–napok; semmilyen IP-kitettség)
| Funkció | Állapot | IP | Biztonságos alap / feltétel |
|---|---|---|---|
| Projekt-JSON, validáció, tiszta `evaluate(frame)` | ✅ | 🟢 | saját séma |
| Bézier-útvonalak, kitöltések, körvonalak, színátmenetek, szaggatott vonalak, változó vastagságú körvonalak | ✅ | 🟢 | tankönyvi geometria |
| FK csontok, korlátok, 2 csontos/FABRIK IK, kényszerek | ✅ | 🟢 | tankönyvi kinematika |
| Kulcsok, easing, kifejezések, zaj | ✅ | 🟢 | saját tervezés |
| Patch-op szerkesztő pontos visszavonással/újra | ✅ | 🟢 | RFC 6902-stílusú JSON-mutatók |
| SVG, PNG, GIF, HTML-lejátszó, Lottie-export | ✅ | 🟢 | nyilvános specifikációk |
| Származási napló (provenance log), `stats` / szemantikus `diff`, referenciakocka-regresszió (pontos hash + tűréshatár), erőforrás-korlátok, mutációs fuzz-teszt, fájlformátum-verziókapu (R-001…R-005) | ✅ | 🟢 | saját tervezés |
| Saját szabványos szerepkör-szókincs + névalapú címkéző (`tag`) (R-006) | ✅ | 🟢 | saját nevek, egyszerű anatómiai szavak |
| JSON Schema export (R-003 séma) | ⬜ | 🟢 | saját séma |

## 2. szint — Szerzői alapok és nyílt kimenetek (napok)
| Funkció | Állapot | IP | Biztonságos alap / feltétel |
|---|---|---|---|
| APNG, sprite sheet PNG+JSON | ✅ | 🟢 | PNG/APNG specifikációk |
| WebM VP9 / AV1 alfával (a te ffmpeg-edeken keresztül) | ✅ | 🟢 | nyíltan specifikált kodekek; a szabadalmi helyzetet nem vizsgáltuk |
| OCA-export (Open Cel Animation) | ✅ | 🟢 | nyilvános specifikáció; import ⬜ |
| OpenRaster (.ora) import | ✅ | 🟢 | nyílt specifikáció, zip-bomba elleni korlátok |
| MP4 / H.264 | ✅ | 🟠 | csak a *felhasználó saját* ffmpeg-jén keresztül; soha ne csomagolj be kódolót |
| Kulcsok eltolása/skálázása/megfordítása/másolása/beillesztése, kötegelt easing, csatornánkénti időeltolás, jelölők és régiók, SMPTE időkód (R-010, R-011, R-044) | ✅ | 🟢 | általános szerkesztés |
| Revíziós pillanatképek metszéssel és összeomlás utáni helyreállítással (R-014); réteg-kompok (R-015); időtengely-események hasznos adattal (R-045); szülőváltás a nyugalmi póz megtartásával (az R-013 része) | ✅ | 🟢 | általános szerkesztés |
| Igazítás/elosztás, referenciaréteg, shy/solo/lock címkék (R-012, R-013, R-016) | ✅ | 🟢 | általános szerkesztés |
| Onion skin, mozgásnyomok (a rávetítések soha nem exportálódnak) (R-020) | ✅ | 🟢 | saját tervezés |
| Utóeffekt-lánc: blur, vetett árnyék, glow, kontúr, posterize, pixelate, levels/gamma, fade (az R-022 része) | ✅ | 🟢 | tankönyvi képfeldolgozás |
| Trim path-ok, rétegenkénti szűrők, maszkok, mélységélesség (R-021, R-024, R-025, R-034) | ✅ (beégetett trim path-ok; a rétegeffektek korlátai: lásd ROADMAP 9. szakasz) | 🟢 | szabványos képmatematika |
| SVG-import (útvonalak, alakzatok, csoportok, transzformációk, egyszínű kitöltések; a színátmenetek/szöveg/vágás figyelmeztetésként jelenik meg) (R-032) | ✅ | 🟢 | W3C SVG 1.1 nyelvtan |
| BVH-import (egy forgástengely → csontszögek, korlátozott kulcsredukció) (R-036) | ✅ | 🟢 | nyílt szöveges formátum |
| dotLottie konténer-export (R-03A) | ✅ | 🟢 | nyilvános konténerspecifikáció (zip + manifest) |
| Animált SVG-export, képsorozat-import, OpenEXR/16 bites kimenet, glTF-export (R-032, R-037…R-039) | 🟡 (glTF: a 0. képkocka alakzatai, plusz csontok és morph-súlyok, lásd ROADMAP R-039) | 🟢 | nyilvános formátumok |
| Papagayo (Moho-switch) és Rhubarb szájjel-jelzés import (R-03B) | ✅ | 🟢 | csak a sima szöveges kimenetüket olvassuk |

## 3. szint — Rig- és animációs mélység (hetek)
| Funkció | Állapot | IP | Biztonságos alap / feltétel |
|---|---|---|---|
| Rugók (beégetett), rácsok (FFD), gerinc-deformer, kapcsolórétegek, részecskék, folyadék, booleanok | ✅ | 🟢 | tankönyvi |
| Hang → szájforma lip-sync, járásciklus-generátor, tükrözés, retime, klip, keresztáttűnés | ✅ | 🟢 | saját tervezés |
| **Pin warp** (a cage / ARAP-stílusú warp helyett) | ✅ | 🟢 | Wendland 1995 RBF |
| **Hődiffúziós automatikus skin-súlyok** | ✅ | 🟢 | Laplace-egyenlet |
| **Teljesítmény-rögzítés** korlátozott kulcsredukcióval | ✅ | 🟢 | Douglas–Peucker |
| **Bemenet-leképezés** (CSV/JSON jelek → paraméterek) | ✅ | 🟢 | nincs bundle-ölt követő |
| Lágy/nyújtó IK (R-051), póz-meghajtók: csúszka + 2D joystick (R-050), szögvezérelt nézetváltás (R-056), címke-alapú automatikus rig (R-058) | ✅ | 🟢 | általános rigging; tankönyvi matematika, saját tervezés |
| Útvonal-kényszerek, talprögzítő IK, szülőváltó kulcsok, skinek, mágnesek (R-040…R-049, R-052…R-055, R-057) | 🟡 (megépült: R-040…R-043, R-044, R-047, R-048, R-052…R-055, R-057; a NEM kész részeket lásd a ROADMAP 9. szakaszában) | 🟢 | általános rigging |
| Üresjárati generátorok (pislogás, lélegzés, ringás) és követő mozgás láncok (R-047, R-048) | ✅ | 🟢 | seedelt, determinisztikus |
| Szél, gravitáció, ütközések, haj-/ruhacsíkok (R-070…R-072) | 🟡 (az R-070, R-071 kész; az R-072 csíkjai csak csontok, a ruha sík lap) | 🟢 | tömeg-rugó / PBD tankönyvek |

## 4. szint — Okos eszközök (hetek–hónapok)
| Funkció | Állapot | IP | Biztonságos alap / feltétel |
|---|---|---|---|
| **Kép → vektor trace** (az ML-es automatikus trace helyett) | ✅ | 🟢 | marching squares + RDP |
| **Eseménynapló-alapú állapotgép** + beégetés csatornákba (a futtatókörnyezeti állapotgép-klónok helyett) | ✅ | 🟢 | saját szemantika — tartsd meg a neveinket |
| **Effektor-elérésű retargeting** | ✅ | 🟢 | saját módszer |
| **Pose-match solver** (evolúciós stratégia, ízületi korlátokat figyelembe vevő) | ✅ | 🟢 | Beyer & Schwefel |
| **WCAG 2.3.1 villogás / vörös villogás ellenőrzés + kameramozgás-jelentés + csökkentett mozgású iker** | ✅ | 🟠 | csak nyilvános definíciók; „mérnöki segédeszköz”-nek nevezd, soha ne „tanúsítottnak” |
| **Rig linter**, **mozgásszerződések**, **zökkenőmentes hurok solver**, **elvek edzője** (I-02, I-04, I-05, I-14) | ✅ | 🟢 | saját ötletek |
| **Rajzold-meg-a-túllendülést rugóillesztés** (I-08) — frekvenciát/csillapítást illeszt a motor saját integrátorával | ✅ | 🟢 | saját ötlet |
| **Explain-this-pixel** csontokra és rétegekre (I-07) | ✅ | 🟢 | saját ötlet |
| Before | after | különbség-ellenőrző kép (az I-06 első lépése; az észlelési mérőszám a `ci diff`, SSIM és CIEDE2000) | ✅ | 🟢 | saját ötlet |
| Normálmap-megvilágítás, parallaxis, mélységélesség (R-064, R-065, R-024) | ✅ | 🟢 | tankönyvi |
| Átirat-igazított lip-sync (R-081) | ✅ (saját igazító, `speech.rs`) | 🟠 | használj megengedő licencű igazítót; ellenőrizd a modell licencét |
| Webkamerás arc-/testkövetés, amely a bemenet-leképezőt táplálja (R-082) | 🟡 (saját detektor, kizárólag szintetikus arcokon tanítva; valódi webkamerákon és fényképeken nem tesztelt) | 🟠 | a felhasználó hoz megengedő licencű követőt; ne csomagolj modellsúlyokat, hacsak a licenc nem engedi |
| Teljes BVH mocap retargeting hierarchia szerinti automatikus illesztéssel (R-083) | ✅ | 🟢 | a BVH-importra épül |
| PSD-import (R-035) | ⬜ | 🟠 | csak olvasás, publikált specifikáció, Adobe-védjegyek nélkül; vagy egyszerűen használj ORA-t |

## 5. szint — Platform (hónapok)
| Funkció | Állapot | IP | Biztonságos alap / feltétel |
|---|---|---|---|
| Böngészős stúdió visszavonással/újra | 🟡 | 🟢 | saját UI; gyártói elrendezések vagy nevek másolása nélkül |
| WASM futtatókörnyezet, stabil Rust API + C ABI, sandboxolt szkriptelés (R-090…R-093) | 🟡 (az R-091, R-092, R-093 kész; az R-090 wasm-modul a CI-ben lefordul és átmegy a wrapper tesztjén Node alatt, és csak headless Chromiumban futtatták) | 🟢 | saját API |
| **Ágens / MCP-szerver** patch-opok fölött: vizsgálat, szerkesztés (atomi, visszavonható), lint, biztonság, renderelés, mentés kizárólag a saját fájljába (R-094) | ✅ | 🟢 | nyílt JSON-RPC protokoll |
| Teljes GUI: gizmók, idővonal, görbeszerkesztő (R-100…R-103) | 🟡 (az R-101, R-102 kész; az R-100 és R-103 oldalai léteznek, de a szerző soha nem futtatta őket valódi böngészőben) | 🟢 | általános widgetek; saját megjelenés |
| Animáció-verziókezelés szemantikus merge-dzsel, CI észlelési diff, explain-this-pixel (I-01, I-06, I-07) | ✅ (I-01 `osatura vcs`, I-06 `osatura ci`, I-07 `explain`) | 🟢 | saját ötletek |
| Póz-fuzzing terhelési hőtérképekkel, méretcélzott export, determinisztikus részletességi szint (LOD) deklarált hibahatárral (I-03, I-10, I-11) | 🟡 (mindhárom megépült; az I-11 határ nem terjed ki az effektekre, a skinelt útvonalakra, az IK-vezérelt értékekre és az állapotgép-pózokra) | 🟢 | saját ötletek, tankönyvi matematika |

## 6. szint — Határterület (kutatási szint)
| Funkció | Állapot | IP | Biztonságos alap / feltétel |
|---|---|---|---|
| GPU-renderelő CPU-paritásteszttel (R-110) | 🟡 (az `osatura-gpu` crate megépült és tesztelt; a hardveres futtatás korlátozott, lásd ROADMAP) | 🟢 | wgpu (megengedő) |
| Valós idejű többfelhasználós szerkesztés az op-napló fölött (R-111) | 🟡 (kész, opcionális tartós munkamenet-naplóval `--collab-log`; a résztvevők és a visszavonási előzmények nem tartósak; sima HTTP, közös token) | 🟢 | CRDT-irodalom |
| Adatkötési / elrendezési futtatókörnyezet (R-091, R-095) | ✅ (saját tervezés; szabadalmi szűrés nem történt, docs/PATENT_NOTES.md) | 🟠 | szállítás előtt szabadalmi szűrés |
| ML-támogatott eszközök (R-112) | 🟡 | 🟠 | kizárólag klasszikus módszerek, modellsúlyok nélkül: `autotrace` (k-means, Schneider-féle Bézier-illesztés, Zhang-Suen vonalmód) és a saját pózaidból tanult javaslatok (PCA, kernelek, opcionális kis MLP saját súlyformátumban); nincs mélytanulás |

## Soha ne építsd meg (🔴)
- A natív **Live2D, Spine, Moho, Harmony, Rive, Cartoon Animator** projektfájlok olvasása vagy írása (zárt formátumok; EULA/visszafejtési kitettség) — használj helyette Lottie-t, OCA-t, ORA-t, APNG-t, SVG-t.
- Az Adobe szabadalmaiban leírt **ARAP-stílusú gyors grafikadeformációs solverek** — a pin warp, az FFD és a skinning lefedi a felhasználási esetet.
- Bármely gyártó **UI-elrendezésének, ikonjainak, neveinek, mintatartalmának vagy fájlformátum-mágikus számainak/terminológiájának** másolása.
- A **Harding** fényérzékenység-ellenőrző eszköz utánzása, illetve sugárzási (broadcast) megfelelőség állítása.
- **H.264/H.265/AAC kódolók** bundle-ölése.

## Feltételek, amelyek minden 🟢 sorra érvényesek
Vezetett származási napló (`CLEANROOM.md`; független megvalósítás, nem szigorú clean-room); csak megengedő függőségek (a `scripts/licence-audit.sh` újragenerálja; jelenleg 363 harmadik féltől származó crate az összes platformon együtt, mind MIT / Apache-2.0 / BSD / Zlib / Unlicense jellegű, hat MPL-2.0 crate kivételével (a smartstring a `scripting` feature-ön keresztül, amely alapértelmezetten be van kapcsolva, öt pedig az `osatura-gui` ablakos alkalmazáson keresztül); lásd docs/THIRD_PARTY.md); harmadik féltől származó grafika vagy hang nem szállítható saját licencfájl nélkül; az **OSATURA** nevet az indulás előtt védjegykeresésben tisztázni kell.


# Részletes képességtáblázatok
## Rajzolás és alakzatok
| Funkció | Állapot | Megjegyzések |
|---|---|---|
| Köbös Bézier-útvonalak, nyitott/zárt, fogantyúk | ✅ | |
| Kitöltés: egyszínű, lineáris és radiális színátmenet | ✅ | a színátmenetek követik a réteg transzformációját |
| Körvonal: vastagság, végződések, csatlakozások, szaggatás | ✅ | |
| Változó vastagságú körvonal (vastagságprofil) | ✅ | sokszög-kontúr; zárt útvonalak → két gyűrű |
| Catmull-Rom simítás, téglalap/kör/ellipszis/sokszög segédek | ✅ | |
| Sokszög-booleanok (egyesítés/metszet/különbség/xor) | ✅ | `boolean` réteg; a lyukak egyetlen sokszögbe áthidalva; az eredmény sokszögű (lapított) |
| Metaball / folyadékrétegek | ✅ | marching squares, a csontok vezérelhetik a foltokat |
| Képrétegek (PNG), beágyazott vagy fájl | ✅ | |
| Bitmap-/vektorecsetek, textúrakitöltések, szöveg/betűtípusok | ❌ | |
| Szabadkézi rajzolás, pontszerkesztő eszközök, kép trace | ❌ | nincs rajzoló GUI |

## Rigging
| Funkció | Állapot | Megjegyzések |
|---|---|---|
| Csonthierarchia, FK, nyugalmi póz | ✅ | |
| Szög- / skálakorlátok | ✅ | |
| IK: analitikus 2 csontos (hajlítás ±) | ✅ | |
| IK: N csontos láncok (FABRIK) | ✅ | |
| FK↔IK keverés (animálható) | ✅ | |
| Kényszerek: aim, copy-rotation, pin (a súly animálható) | ✅ | |
| Horgonyok + fogantyúk lineáris-keverésű skinningje, automatikus skin | ✅ | |
| Csonthoz rögzített rétegek, rétegszülőség / csoportok | ✅ | |
| Másodlagos mozgás: beégetett csillapított rugók | ✅ | determinisztikus, véletlen hozzáférésű (betöltéskor beégetve) |
| Póz által vezérelt csatornák („driven keys”) | 🟡 | egy görbe bármely tulajdonságot leképez egy másikra; nincs dedikált smart-bone UI/akció |
| Merevtest-fizika, ruha, lágytest | ❌ | |

## Deformáció
| Funkció | Állapot | Megjegyzések |
|---|---|---|
| Pontonkénti eltolások (animálható) | ✅ | címzés: `layer::path::index` |
| Morph targetek / blend shape-ek (additív súlyok) | ✅ | |
| Szabad formájú rács (Bernstein FFD) | ✅ | animálható vezérlőpontok |
| Gerinc- / görbehajlító deformer | ✅ | opcionális lágy sugár |
| „Boil” remegés (kézzel rajzolt jitter) | ✅ | koherens zaj, determinisztikus |
| Képháló-warp csontsúlyokkal | ✅ | rácsháló; nincs hálószerkesztő |

## Animáció
| Funkció | Állapot | Megjegyzések |
|---|---|---|
| Kulcsok + easing (lineáris/step/ease/köbös Bézier) | ✅ | |
| Extrapoláció: hold/lineáris/loop/ping-pong/loop-offset | ✅ | |
| Kifejezés-csatornák | ✅ | kis, biztonságos kifejezésnyelv; más csatornákat is olvashat |
| Zajcsatornák, add/multiply/replace keverés, súlyok | ✅ | |
| Kapcsolórétegek + expozíció (képkockáról képkockára, szájcserék) | ✅ | |
| Klip retime/eltolás, retarget, tükrözés B↔J, keresztáttűnés beégetése | ✅ | függvények + CLI `mirror`; nincs NLA UI |
| Procedurális járásciklus | ✅ | egyszerű szinuszos generátor, nem profi járáseszköz |
| Hang lip-sync WAV-ból | 🟡 | hangosság/fényesség heurisztika → 5 szájforma, **nem** fonémafelismerés |
| Determinisztikus részecskeemitter | ✅ | zárt alakú: bármely képkocka közvetlenül kiértékelhető |
| Onion skin, mozgásnyomok | ✅ | CLI `onion` és `trail`, stúdió-rávetítések; soha nem exportálódnak |
| Hanglejátszás / hullámforma-scrubbing | 🟡 | `waveform` parancs és stúdióbeli hanglejátszás; hullámforma-scrubbing UI-t nem állítunk |

## Kamera és kompozitálás
| Funkció | Állapot | Megjegyzések |
|---|---|---|
| 2D kamera pásztázás/zoom/forgatás (animálható) | ✅ | |
| Többsíkú parallaxis legfelső szintű rétegenként | ✅ | |
| Átlátszatlanság, keverési módok, alfamaszkok (+invertálás) | ✅ | |
| Valódi 3D rétegek / kamera, sztereó | ❌ | |

## Kimenet és adatcsere
| Funkció | Állapot | Megjegyzések |
|---|---|---|
| SVG képkockák | ✅ | |
| PNG képkockák (tiny-skia raszter) | ✅ | |
| Animált GIF | ✅ | |
| MP4 | ✅ | `ffmpeg` szükséges a PATH-on |
| Önálló HTML-lejátszó | ✅ | az összes képkocka beágyazva, scrubber |
| Lottie JSON | 🟡 | képkockánként beégetett geometria; a színátmenetek → első szín; nincsenek maszkok/keverési módok/képek; **valódi Lottie-lejátszóban nem validált** |
| Más eszközök fájljainak, PSD, FBX stb. importja | ❌ | (szándékosan: nincsenek zárt formátumok) |
| Szkriptelés / plugin API | 🟡 | sandboxolt Rhai `script` validált kéréseken keresztül, plusz kifejezések; nincs plugin API |

## Szerkesztés
| Funkció | Állapot | Megjegyzések |
|---|---|---|
| JSON-patch szerkesztési modell visszavonással/újra | ✅ | minden szerkesztés validált; az érvénytelen szerkesztések visszagörgetődnek |
| Kulcs beállító/törlő segédek | ✅ | |
| Webes stúdió (böngésző) | 🟡 | előnézet, scrubber, csontonkénti szögkulcsolás, JSON-szerkesztő, visszavonás/újra, mentés, exportok. **Nincs rajzolás, nincs pont-/csontszerkesztés a vásznon, nincs görbeszerkesztő.** |

## Dolgok, amelyek szerény „javítások” a tankönyvi alaphoz képest
(nem állítjuk, hogy máshol hiányoznának)
- Minden projektszerkesztés validált, invertálható JSON-patch → a visszavonás soha nem tudja megrongálni a projektet.
- Minden szimuláció (rugók, részecskék) véletlen hozzáférésű determinisztikus: bármely képkockát bármilyen sorrendben görgethetsz vagy renderelhetsz, azonos pixeleket kapsz.
- Minden tulajdonság — a deformer vezérlőpontjait és a morph-súlyokat is beleértve — egyetlen `layer::sub::id` útvonallal címezhető, így a kifejezések és a vezérelt görbék bármit bármivel összekötnek.
- A validáció az *összes* problémát egy menetben jelenti, az érintett nevekkel.
- Egy motor → SVG, raszter, GIF, MP4, HTML, Lottie; ugyanaz az `evaluate(frame)` táplálja mindet.

## Tesztek
`cargo test` — 142 unit/integrációs teszt (matematika, sávok, kifejezések, rig solverek, deformerek,
pixelrenderelés, exportálók, szerkesztő visszavonás/újra, hang, részecskék, booleanok, metaballok).

---

# Ellenőrzőlista a referenciatermékhez képest, verziónként

Referencia: Moho (korábban Anime Studio), 1999 → 14.5 „SHOWTIME” (2026. okt. 12.). A funkciólisták
a gyártói kiadási oldalakról és a szakmai sajtóból származnak (lásd Források). Hiányok az adatokban: az Anime Studio 6–8,
12.5 és 14.1 verziókról nem találtam részletes nyilvános listát, ezért ezeket csak az
általános történet fedi. Az állapot az OSATURA-é: ✅ létezik egyenértékű · 🟡 részleges / eltérő megközelítés · ❌ hiányzik.

## Korai verziók (1999 – Anime Studio 8, csak általános történet)
| Funkció | OSATURA |
|---|---|
| Részecskék | ✅ determinisztikus emitterréteg |
| Rétegmaszkok | ✅ alfamaszkok + invertálás |
| Onion skinek | ✅ (CLI `onion`, stúdió-rávetítés) |
| Csontkényszerek | ✅ aim / copy-rotation / pin, szög- és skálakorlátok |
| Lua szkriptelés | 🟡 (Rhai, nem Lua; kifejezés-csatornák is) |
| Lip-sync | 🟡 amplitúdó-heurisztika WAV-ból |
| Fizika | 🟡 rugók + részecskegravitáció; nincsenek merevtestek |
| 3D létrehozás / import | ❌ |
| Photoshop-import | ❌ |
| Wacom-támogatás | ❌ (nincs rajzoló GUI) |
| Smart bone-ok, IK | 🟡 vezérelt csatornák / ✅ 2 csontos + FABRIK |

## Anime Studio 9 (2013)
| Funkció | OSATURA |
|---|---|
| 64 bit, GPU-gyorsítás | 🟡 natív 64 bites Rust; csak CPU-rasztermotor (nincs GPU) |
| Smart Bones (ízületi hajlítást vezérlő tárcsák) | 🟡 a vezérelt csatornák bármely értéket bármely másikra leképeznek; nincs tárcsa-UI/akció |
| Szerkeszthető mozgásgráfok | ❌ (a görbék adatok köbös Bézier easinggel; nincs görbeszerkesztő) |
| Újratervezett idővonal | ❌ (a stúdióban csak scrubber és kulcsjelek vannak) |
| Wacom multi-touch | ❌ |

## Anime Studio 10 (2014)
| Funkció | OSATURA |
|---|---|
| Független (globális) szögkorlátok | 🟡 a vezérlőcsontra mutató copy-rotation megközelíti |
| Squash & stretch | 🟡 a csont skálája animálható; nincs automatikus squash/stretch |
| Smart Bone tárcsák / beállítás | 🟡 lásd fent |
| Bounce / elastic / stagger interpoláció | 🟡 bounce, spring (elasztikusszerű), back, köbös Bézier ✅; stagger ❌ |
| Festékesvödör, folt-ecset, radír, pontok csökkentése | ❌ |
| Előre betöltött karakterek / könyvtártartalom | ❌ |
| 3D objektum import | ❌ |

## Anime Studio 11 (2015)
| Funkció | OSATURA |
|---|---|
| Képkockáról képkockára animáció | ✅ kapcsolórétegek + expozíciós csatornák |
| Réteg-hivatkozás (kapcsolt duplikátumok) | ❌ |
| Animált alakzat-sorrend | ❌ (a rajzolási sorrend rétegenként rögzített; a kapcsolórétegek cserélhetnek) |
| Szerkeszthető mozgásgörbék | 🟡 görbeadat igen, görbe-UI nincs |
| PSD-import (élő frissítéssel) | ❌ |
| FBX / Unity export | ❌ |
| Hang import/felvétel + automatikus lip-sync | 🟡 WAV import + heurisztikus lip-sync; felvétel nincs |
| Körvonalak egyesítése, Color Points eszköz | ❌ |

## Moho 12 (2016) és 12.x
| Funkció | OSATURA |
|---|---|
| Bézier-fogantyúk | ✅ |
| Továbbfejlesztett szabadkézi eszközök | ❌ |
| Smart Warp (egyedi hálók) | 🟡 képrácsháló csontsúlyokkal; rács / gerinc deformerek vektorokhoz; nincs hálószerkesztő |
| Mozgási elmosás (motion blur) | ❌ |
| Pin bone-ok | 🟡 pin kényszer, nem egypontos pin bone-ok |
| Import/export akciók hasonló rigek között | 🟡 `clip`, `retarget`, `mirror` függvények csatornakészleteken; nincs akciókönyvtár-UI |
| Több réteg egyidejű animálása | ✅ időtengelyenként tetszőleges számú csatorna |
| Újratervezett UI, csont-rögzítés, pozíció automatikus befagyasztása | ❌ |

## Moho 13 (2019)
| Funkció | OSATURA |
|---|---|
| Bitmap képkockáról képkockára + onion skin | 🟡 PNG képrétegek kapcsolórétegeken belül; nincs onion skin, nincs bitmap-rajzolás |
| Bitmap-rajzeszközök / egyedi ecsetek | ❌ |
| Storyboard- és animatic-eszközök | ❌ |
| Actions ablak | ❌ (csak adatszintű klipek) |
| 3D fények, anyagok, toon shading | ❌ |
| További kimeneti előbeállítások | 🟡 CLI exportformátumok, nincs előbeállítás-kezelő |
| Mellékelt karakterek | ❌ |

## Moho 13.5 (2021) és 13.5.x
| Funkció | OSATURA |
|---|---|
| Vitruvian Bones (rajzok *és* csontkészletek cseréje) | 🟡 a kapcsolórétegek rajzokat cserélnek; csontkészlet-csere nincs |
| Quad hálók (perspektíva) | 🟡 a rácsháló + rács szabad formájú warpokat ad; nincs projektív quad |
| Szél-dinamika (erő, irány, turbulencia) | ❌ (a rugóknak nincs szele; a zajcsatornák utánozhatják) |
| Relatív kulcsolás | ❌ |
| Véletlenszerűen választott lip-sync viseme-változatok | ❌ (a leképezés egyesíthet alakokat, véletlen változatok nincsenek) |
| Hang exportálása WAV-ként | ❌ |
| Mély kulcsmásolás/beillesztés, pivot-igazítás, idővonal-jelölők | ❌ (nincs GUI) |
| Lebegő licencek | n/a |

## Moho 14 (2023)
| Funkció | OSATURA |
|---|---|
| Új grafikus motor, valós idejű effektek (blur, árnyék, zaj) | ❌ nincsenek blur/árnyék/zaj szűrők; keverési módok ✅ |
| Liquid Shapes (egyesítés/kivonás/metszet/keverés) | 🟡 metaball folyadék ✅ + sokszög-booleanok ✅; nem ugyanaz az alakzatmodell |
| Curvers / Compressible Curvers | 🟡 a gerinchajlító deformer a legközelebbi analógia |
| Továbbfejlesztett SVG-import | ❌ (csak SVG-**export**) |
| Szabadkézi fejlesztések, új stílusok, alakzat-egyesítés | ❌ / ❌ / 🟡 (booleanok) |
| Multi-Stroke Fill, Connect & Create | ❌ |
| Smart Line Boil, ecsetbeállítások | 🟡 „boil” remegés állítható sebességgel ✅; ecsetbeállítások ❌ |
| Pontszínek, továbbfejlesztett Add Point | ❌ |
| Dynamic Bones (+ gravitáció) | 🟡 csillapított rugós csontok ✅; gravitációs tag nincs |
| Késleltetett kényszerek | 🟡 a rugók késést adnak a cél mögött; explicit késleltetési kényszer nincs |
| Új idővonal-opciók | ❌ |
| Képkockáról képkockára fejlesztések | 🟡 |
| Szkriptfelület frissítése (Lua 5.4) | ❌ |

## Moho 14.1 – 14.2 (2023-11, 2024-06)
| Funkció | OSATURA |
|---|---|
| (14.1: nem találtam nyilvános részletet) | — |
| Animált görbeprofil-eltolás | ❌ (a vastagságprofilok statikusak) |
| Smart boil vektorzajhoz | 🟡 a remegés megadott sebességgel ketyeg; nincs „csak mozgás közben” mód |
| Átdolgozott Transform Bone eszköz | ❌ (nincsenek vászoneszközök) |
| Pixelesítés-megjelenítés | ❌ |
| Colour Burn keverési mód | ✅ |
| Kisebb minimális ecsettávolság, PSD vágott rétegek | ❌ |
| Képkockasebesség-alapú idővonal-számozás | ❌ |
| „Maszkolt rétegek kikapcsolása” megjelenítési opció | ❌ |
| Jobb lip-sync alacsony képkockasebességnél | 🟡 |
| Többnyelvű nevek csontokhoz/rétegekhez/jelölőkhöz | ✅ (UTF-8 nevek) |
| Exclude-Strokes maszk élsimítása, színes onion skinek képeken | ❌ |
| Csontdinamikát vezérlő Smart Bones | ❌ (a rugóbeállítások nem animálhatók) |
| Lua szkriptek indításkor; Windows konzol kapcsoló | ❌ / n/a |
| ProRes alfa export | ❌ (csak GIF/PNG/MP4) |

## Moho 14.3 (2024-12)
| Funkció | OSATURA |
|---|---|
| Pontonkénti ecsetátlátszatlanság (nyomásvezérelt) | ❌ |
| Színsodródás tollnyomás szerint | ❌ |
| Szabadkézi/vonalvastagság-eszköz vezérlők | ❌ |

## Moho 14.4 „SIDE QUEST” (2025-11)
| Funkció | OSATURA |
|---|---|
| Export Unrealba / Unitybe / Godotba | ❌ |
| glTF-export Blenderbe / 3D-eszközökbe | 🟡 (`.glb`: a 0. képkocka alakzatai, csontok, morph-súlyok) |
| 360°-os Smart Bones | ❌ |
| Stílus létrehozása alakzatból | ❌ |
| Részecske-opciók (replikálás görbe mentén, késleltetés, rendezett emisszió, eloszlás) | 🟡 indítási késleltetés ✅, determinisztikus rendezett emisszió ✅; replikálás görbe mentén ❌ |
| Layer Selector, Magnet lecsengés, PSD-import, szabadkézi simítás, Lua kiegészítések | ❌ |
| Könnyebb ciklusalkotás | ✅ loop extrapoláció + járásciklus-generátor |

## Moho 14.5 „SHOWTIME” (2026-10-12)
| Funkció | OSATURA |
|---|---|
| Live Playback (webkamera / gamepad / mikrofon, virtuális webkamera, OBS) | ❌ |
| Arckövetés → csontok / Smart Bones / háló | 🟡 (szintetikusan tanított detektor, `face-bake`; valódi arcokon nem validált) |
| Élő rögzítés az idővonalra, többkarakteres vezérlés | ❌ |
| Videókövetés (képsorozatok) | ❌ |
| Szabadkézi algoritmusok, egyenes vonalak, farkak levágása | ❌ |
| Újraírt Paint Bucket (Vector / Trace / Recolor) | ❌ |
| Magnet fordított mozgatás, Select Points alakzat-kijelölés | ❌ |
| Egyedi ecsetkategóriák | ❌ |
| Onion skin szín/átlátszatlanság, Shift & Trace, mágneses onion skinek | ❌ |
| Rétegek ablak szűrői / többszörös kijelölés | ❌ |
| Teljesítmény: gyorsabb visszavonás/újra, könnyebb fájlok | 🟡 a visszavonás olcsó JSON-patch inverz; a fájlok sima JSON-ok |
| MP4/H.264 export | ✅ ffmpeg-en keresztül |
| PNG színprofilok, EXR import/export | ❌ |
| Kötegelt exportáló, `-exr` / `-pngprofile` CLI kapcsolók | 🟡 `frames` / `export` CLI, nincs kötegsor |
| Képkockasebesség 3 tizedesjeggyel (pl. 23.976) | ✅ |
| Az SVG-export kezeli a változó vastagságú körvonalakat és a folyadékalakzatokat | ✅ (a körvonalak kontúrokká, a folyadékok útvonalakká válnak) |
| Liquid hide-edge, csak körvonalas folyadékalakzatok, körvonal-expozíció | ❌ |
| Linux Wine/Proton segítségével | ✅ natív Rust (Linux/macOS/Windows); előre elkészített Windows x86-64, macOS arm64 és Linux x86-64 archívumok az előzetes kiadás-vázlatban; a CI mindhárom rendszeren teszteli, ember csak Linuxon |

## Összegzés (csak a referenciatermék; a versenytársak lejjebb következnek)
A referenciatermék dokumentált funkciói közül az OSATURA a **rigging, a deformáció,
a procedurális animáció, a kompozitálás-matematika és az export** magját fedi le (nagyjából: csontváz/IK/kényszerek,
háló- és rácsdeformáció, kulcsok és kifejezések, maszkok/keverések/színátmenetek, részecskék,
metaballok/booleanok, GIF/MP4/HTML/Lottie), de **nincsenek rajzeszközei, nincs élő bemenete, nincs 3D-je,
nincs PSD/FBX I/O-ja** (az első felmérés idején; azóta bekerült a glTF-export, az EXR-kimenet, a Rhai szkriptelés, a görbe-/idővonal-stúdió, az onion skin és a blur effektek, lásd `ROADMAP.md` 9. szakasz).
Ezek a termék nagy, többnyire GUI-hoz kötött blokkjai, és ők jelentik az őszinte hiányt.

## Források
- Moho 14.5 SHOWTIME bejelentés, moho.lostmarble.com (blog, 2026. okt. 12.)
- Moho Pro funkcióoldal, mohoanimation.com/pages/features
- Moho 14.4 SIDE QUEST appcast, lostmarble.com/appcasts/Moho14.4Update.html
- Moho 14.3: Digital Production, 2024. dec. 27.
- Moho 14.2 kiadási bejegyzés, moho.lostmarble.com; „Moho 14: What is new?” oktatóanyag-lista
- Moho Pro 13.5 újdonságai, moho.lostmarble.com; Lester Banks (2021. ápr.); 13.5.3 jegyzetek
- Moho 13: Lester Banks (2019. jún.)
- Moho Pro 12: 80.lv; Digital Production (2016. nov.)
- Anime Studio 11: Architosh (2015. máj.); 10: Architosh (2014. márc.); 9: Renderosity
- Verziódátumok: mohoscripting.com/moho_versions; általános történet: Wikipedia „Moho (software)”

---

# Kiegészítés: a Moho / Anime Studio története, 1 – 11. verzió (kitöltve)

Forrás: a gyártó által üzemeltetett verziótörténeti PDF (lostmarble.com, „Anime Studio history”, 2015), valamint az
Anime Studio 7 és 8 sajtóoldalai. Felváltja a hivatkozási ellenőrzőlista elején található „csak általános történet” sorokat.
Itt sem találtam meg: a 8.x, 10.1.x, 11.1, 12.5, 14.1 részleteit.

| Verzió | Dokumentált funkciók | OSATURA |
|---|---|---|
| Moho 1 (1999) | BeOS, Windows-portolás, Flash-kimenet | ❌ Flash/SWF (elavult) |
| 2.x | QuickTime I/O, renderelés 4096²-ig, rétegárnyékolás, kapcsolórétegek, színmintázatok a stílusokban, rugalmas csontkötés, továbbfejlesztett Illustrator-import | ✅ kapcsolórétegek, 🟡 súlykötés, ❌ Illustrator-import, ✅ nagy raszterskálák (≤16×) |
| 3.x | Részecskék, képburkolás, rétegmaszkok, onion skinek, virtuális kamera mélységgel, képwarp, továbbfejlesztett kapcsolórétegek, táblagép-támogatás, SMPTE időkód, képen kívüli grafika elrejtése, megfordítható maszkok, vonalvastagság-randomizáló, biztonsági zónák, rétegdöntés (shear) | ✅ részecskék, maszkok (+invertálás), kamera, képháló-warp, réteg-skew; ❌ onion skin, táblagép, időkód, biztonsági zónák, vastagság-randomizáló |
| 4.x | Grafikus animációs görbék, csontkényszerek, kötegelt renderelés, hang, animálható paraméterek, animációs ciklusok, kulcsskálázás, átméretezhető projektek, részecskemélység, randomizált tömegek, automatikus rugalmas kötés, Linux-kiadás | ✅ kényszerek, ciklusok (loop extrapoláció), animálható paraméterek, átméretezhető projektek, `clip` retime ≈ kulcsskálázás, Linux; 🟡 tömegek (részecskék), 🟡 hang (csak lip-sync); ❌ görbeszerkesztő UI |
| 5.x | Új UI, SWF-export, 3D- és jegyzetrétegek, akciók, stílusok, Lua szkriptelés | ❌ 3D- / jegyzetrétegek / Lua; 🟡 akciók (klipfüggvények); ✅ a stílusok útvonalankénti festékek |
| Anime Studio 6 | Mozgáskövetés, integrált lip-sync, scatter ecset, Lua 5.1, automatikus árnyékolás, **morph targetek**, szekvenszer, HD videó | ✅ morph targetek; 🟡 lip-sync; ❌ mozgáskövetés, scatter ecset, automatikus árnyékolás, szekvenszer |
| 6.1 / 6.2 | Facebook-feltöltés, átlátszó GIF-import | ❌ / 🟡 GIF-export ✅, import ❌ |
| Anime Studio 7 | Szimulált fizika (gravitáció, sűrűség, súrlódás, rugalmasság), 3D alaktervezés, hangfelvétel, tartalomkönyvtár, Poser-jelenet import, **follow path eszköz**, média összegyűjtése | 🟡 rugók/részecskegravitáció; ❌ merevtest-fizika, 3D, felvétel, könyvtár, Poser, follow-path |
| Anime Studio 8 | Automatikus képkövetés (trace), Character Wizard, kezdő mód, valós idejű médiakapcsolat, globális renderelési stílusok, fejlett csontrigging | ❌ mind (nincs trace, varázsló, élő kapcsolat, globális stílusok) |
| Anime Studio 9 / 9.5 | Szerkeszthető mozgásgráfok, Smart Bones, **layer comps**, továbbfejlesztett onion skinek, 64 bit + GPU, fejlesztett kulcsolás, színes idővonal; 9.5: Smart Bone fejlesztések, mélységeltolás, beágyazott rétegvezérlők, flexi-binding, kapcsolóréteg-fejlesztések | lásd az AS 9 táblázatot; ❌ layer comps, mélységeltolás |
| Anime Studio 10 / 10.1 | Új kényszerek, Smart Bone beállítás, bounce/elastic/stagger, különálló renderfolyamat, kombinált csont+pont eszközök, animáció-előnézet, több dokumentum, gyorsbillentyű-szerkesztő, többalakzatos kijelölés, nulladik képkocka, pontok/csontok elrejtése, többrétegű szerkesztés, **változó vastagságú görbék**, textúra-átlátszóság, állítható részecskeforrás, mélységélesség, törtértékű blur; 10.1: automatikus képtrace, globális stílusok, patch rétegek, PSD-import, valós idejű média, Character Wizard | ✅ változó vastagságú körvonalak, ✅ többrétegű animáció; 🟡 mélységélesség ❌ blur ❌; ❌ patch rétegek, a többi a fentiek szerint |
| Anime Studio 11 | Képkockáról képkockára, rétegek hivatkozása, animált alakzatsorrend, **animált csontcélok**, **animált csontszülőség**, csontfordítás, kijelölési rétegcsoportosítás, normalizált rétegskála, kötegelt export, új fájlformátum | ✅ FbF, ✅ animált IK-cél (a cél csak egy csont), ❌ animált *újraszülőzés*, 🟡 csontfordítás (`mirror`), ✅ köteg a CLI-n keresztül, ✅ sima JSON fájlformátum |

**Itt felszínre került új hiányok** (vedd fel az ütemtervbe): layer comps (mentett rétegláthatósági állapotok),
animált csont-újraszülőzés, patch rétegek, mélységeltolás, follow-path rétegek, biztonságizóna-rávetítések,
SMPTE időkód, automatikus árnyékolás, scatter ecset, kép automatikus trace-e.

---

# Versenytárs-funkciók, amelyeket a fenti ellenőrzőlista nem fed le

Felmért termékek: Toon Boom Harmony 24, Spine 4.2, Live2D Cubism 5.x, Cartoon Animator 5.3, Rive,
Synfig Studio, OpenToonz, TVPaint Animation 12, Adobe Animate, Cavalry, Procreate Dreams, Duik (After
Effects), Blender Grease Pencil. Az alábbiak azok a tételek, amelyek **még nem szerepelnek** a referencia-
ellenőrzőlistán. A források minősége változó: a legtöbb sor a gyártók általam lekért funkció-/kiadási oldalairól származik;
a †-gal jelölt sorok általános tudásból vagy csak címből/tartalomjegyzékből származnak, és oldalon nem lettek megerősítve.
Az állapot az OSATURA-é: ✅ · 🟡 · ❌.

## Rigging, kényszerek, deformáció
| Funkció | Hol látható | OSATURA |
|---|---|---|
| Súlyeszközök: súlyok összeforrasztása hálók között, súlyok másolása/beillesztése csúcspontok között | Spine 4.2 | ❌ (csak automatikus skin, festés nincs) |
| Csont-fizikai kényszerek (tehetetlenség, szél, gravitáció, keverés) | Spine 4.2 | 🟡 csillapított rugók (frekvencia/csillapítás); nincs szél/gravitáció/tehetetlenség |
| Útvonal-kényszer: a csontok összetett Bézier-spline-t követnek | Spine, Rive (Follow Path) | ❌ |
| Kényszerkeverések, duplikálás a csontokkal, többes szerkesztés, mappák | Spine 4.2 | 🟡 az IK-keverés és a kényszersúly animálható; nincs mappa/többes szerkesztés |
| Skinek (csatolmánykészletek cseréje egy csontvázon) | Spine, Rive solo-k | 🟡 kapcsolórétegek |
| Többcsontos IK „póz-eszköz” (egy végtag végének húzása) | Spine | ❌ (vászon-GUI kell hozzá) |
| Befoglaló doboz / találat-teszt csatolmányok | Spine | ❌ (offline kimenethez nem kell) |
| Szabad formájú deformáció, hálósúlyok | Spine, Harmony | ✅ rács + súlyozott képháló |
| Deformer-lánc: Bone, Game Bone, Curve, Envelope, Free-Form, Mesh Warp | Harmony | 🟡 bone ✅ curve (gerinc) ✅ free-form ✅ mesh warp ✅; envelope/cage ❌; game-bone ❌ |
| Alakzattudatos deformáció (pontok, csontok, cage-ek) | Harmony 24 | ❌ |
| Minden deformációtípus beégetése | Harmony 24 | 🟡 a rugók beégetve; a geometria képkockánként beégetve a Lottie-exportban |
| Fő vezérlő / transzformációválasztó deformcsoportokhoz | Harmony 24 | ❌ |
| Pegek és rétegek hierarchiája | Harmony, Animate | ✅ csoportok, csonthoz rögzített rétegek, szülőség |
| Automatikus izom, automatikus talpgörgetés, kinematikai kimenet † | Harmony | ❌ |
| Forgató / warp deformerek + keyform paraméterek, glue | Live2D | 🟡 a paraméterek ≈ vezérelt csatornák, a warp ≈ rács; glue ❌ |
| Blend shape-ek, amelyek forgatást, átlátszatlanságot, rajzolási sorrendet, multiply/screen színt vezérelnek | Live2D 5.0 | 🟡 a morph-súlyok csak geometriát vezérelnek |
| Automatikus háló egyenletes sűrűségvezérléssel vékony alakzatokhoz | Live2D 5.0 | ❌ (csak szabályos rács) |
| A warp deformer automatikusan átméreteződik, hogy magába foglalja a gyermek keyformot | Live2D 5.3 | ❌ |
| Deform ecset (négyzet/vonal/kiterjesztés alakzatok) | Live2D 5.3 | ❌ |
| MI-támogatott arcdeformer / arcmozgás-generálás | Live2D 5.0 | ❌ |
| Joystickok (2D vezérlés animációkra leképezve) | Rive | 🟡 a vezérelt csatorna egy bemenetet képez le; nincs 2 bemenetes joystick |
| Kényszerek tetszőleges objektumok között (eltolás/forgatás/skála/távolság) | Rive | 🟡 csak aim, copy-rotation, pin |
| Bitmap-rigging automatikus közbülső képkockákkal | TVPaint 12 | 🟡 képháló ✅; automatikus közbülső képkockák ❌ |
| Plastic eszköz (hálóalapú rigging) | OpenToonz | 🟡 képháló + súlyok |
| Automatikus rigging, vezérlők, rugódinamika | Duik † | 🟡 rugók ✅; automatikus rig ❌ |

## Animáció, idővonal, interaktivitás
| Funkció | Hol látható | OSATURA |
|---|---|---|
| Animációkeverés: egyidejű animációk keverése/keresztáttűnése futásidőben | Spine | 🟡 Add/Multiply módok + súlyok; beégetett `crossfade`; futásidejű átmenetek nincsenek |
| Dopesheet + görbeszerkesztő + futásidejű előnézet | Spine, Rive, OpenToonz | ❌ (az adatok támogatják; UI nincs) |
| Állapotgép: bemenetek, figyelők, feltételes átmenetek, blend állapotok, rétegek | Rive | ❌ |
| Események, hangtriggerek, dinamikus erőforrások | Rive | ❌ |
| Adatkötés (tervezőelemek ↔ élő adatok) | Rive | ❌ (a kifejezések csatornákat olvashatnak, külső adatot nem) |
| Beágyazott szkriptelés (Luau, JavaScript, ToonzScript) | Rive, Cavalry, OpenToonz | ❌ |
| Reszponzív elrendezések / komponensek / könyvtárak | Rive | ❌ |
| Trim Paths, vektoros feathering, N-slicing | Rive | ❌ |
| Felbontásfüggetlen időzítés, vektoros tweening | Synfig | ✅ |
| Rétegek közötti paraméter-összekapcsolás kifejezésekkel | Synfig | ✅ |
| Procedurális csomópontgráf (generátorok, transzformálók, effektorok), duplikátorok | Cavalry | ❌ (nincs csomópontgráf; nincs rétegpéldányosítás) |
| Adatvezérelt animáció JSON/CSV-ből | Cavalry | ❌ |
| Performing: élő gesztusok rögzítése kulcsokként | Procreate Dreams | ❌ |
| Bábszínpadi trigger-gyorsbillentyűk, motion pilot (kurzorvezérelt járások) | Cartoon Animator 5.3 | ❌ |
| Kameraváltó: mentett kamerák, gyorsbillentyűs vágások, kameraváltó sáv | Cartoon Animator 5.3 | 🟡 egy kamera; a vágások step kulcsokkal utánozhatók |
| Filmes mozgások (dolly zoom, célpont követése) | Cartoon Animator 5.3 | 🟡 animálható kamera; célkövetés nincs |
| Arc-mocap (webkamera / iPhone), fejvezérelt test, mikrofonhangerő | Cartoon Animator, Live2D | ❌ (a mikrofonhangerő csak offline WAV-on keresztül) |
| Élő közvetítési mód (OBS/Twitch, Stream Deck-leképezés) | Cartoon Animator 5.3 | ❌ |
| Motion Sync lip-sync, amely idővonal-kulcsokat generál | Live2D 5.0 | 🟡 heurisztikus WAV → expozíciós kulcsok |
| Rhubarb lip-sync integráció | OpenToonz | ❌ (csak heurisztika) |
| Root motion kinyerése (delta-kompenzáció) | Spine 4.2 | 🟡 loop-offset extrapoláció |
| Hurkoló görbe fogantyú-szinkronja | Spine 4.2 | ✅ loop extrapoláció (fogantyú-UI nincs) |
| Morphing morph-tippekkel | Harmony | 🟡 morph targetek + útvonal-lerp; tippek nincsenek |
| Kulcsalapú 2D kamera idővonal-időzítéssel | TVPaint 12 | ✅ |

## Rajzolás, festés, bitmap-eszközök
| Funkció | Hol látható | OSATURA |
|---|---|---|
| Ecset: randomizálás (bélyegszám, eltolás), kettős papírok, többtextúrás ecsetek | Harmony 24, TVPaint 12 | ❌ |
| Ceruza átlátszatlansága nyomás/dőlés/sebesség/elvékonyodás/halványodás szerint | Harmony 24 | ❌ |
| Szimmetria-segédvonalak (függőleges, vízszintes, kettős) | Harmony 24 | ❌ |
| Cutter eszköz (lasszó, vágás), kontúrszerkesztő, rés zárása | Harmony | ❌ |
| Rajzsegédvonalak, perspektíva-/vonalzó-segédvonalak † | Harmony | ❌ |
| Papír nélküli animáció: onion skin, lapozás, fényasztal | Harmony, Procreate Dreams, TVPaint | 🟡 (onion skin van; rajzeszközök és fényasztal nincs) |
| Beolvasott rajzok tisztítása, szkennelés, kamerás rögzítés, stop-motion | OpenToonz | ❌ |
| Kijelölő ecset, képkönyvtár | TVPaint 12 | ❌ |
| Élesítés transzformáláskor | TVPaint 12 | n/a |
| Tollnyomás-érzékenység | Synfig | ❌ |
| Útvonalalapú színátmenetes árnyékolás | Synfig | ❌ |
| Szöveg változó/dinamikus futásidejű tartalommal, bármely nyelven | Rive, Procreate Dreams | ❌ |
| Egyedi betűtípusok, főcímek, stáblisták | Procreate Dreams | ❌ |
| Palaták és stílusok kezelése | OpenToonz | ❌ |
| >50 rétegtípus (fraktálok, szűrők, torzítások) | Synfig | ❌ (≈7 rétegfajta) |

## Effektek és kompozitálás
| Funkció | Hol látható | OSATURA |
|---|---|---|
| Csomópontnézetes kompozitáló hálózat névvel ellátott portokkal | Harmony 24 | ❌ (rétegfa + maszkok/keverések) |
| Matte-ok, csoportok a kompozit alatt | Harmony | ✅ maszkok, izolált csoportok |
| CryptoMatte olvasás/írás | Harmony 24 | ❌ |
| Toon shader (kontúr, posterize) | Harmony 24 | ❌ |
| Unsharp mask, csatornacsere | Harmony 24 | ❌ |
| Blur-család (radiális, spin, irányított), bokeh, fraktálzaj, csempézés | OpenToonz | ❌ |
| Részecske FX | OpenToonz | ✅ emitterréteg |
| 15 RGB + 5 alfa keverési mód, alkatrészenkénti offscreen rajzolás | Live2D 5.3 | 🟡 13 keverési mód; izolált csoport renderelés ✅ |
| Clipping (alkatrészenkénti clip ID-k) | Live2D, Rive | ✅ alfamaszkok |
| HDR / HDRI képalkotás | Synfig | ❌ |
| Videóimport, ProRes, rotoszkóp felvétel fölött | Procreate Dreams | ❌ |
| Hangmotor (narráció, zene, SFX) | Procreate Dreams | ❌ |

## 3D, pipeline, adatcsere, futtatókörnyezetek
| Funkció | Hol látható | OSATURA |
|---|---|---|
| 3D jelenet, FBX csontpózolás, Vulkan jelenetgráf, perspektíva-síkú rajzolás | Harmony 24 | ❌ |
| Külső renderelés Maya / Blender segítségével | Harmony 24 | ❌ |
| Vektorimport: PDF / AI / SVG; többrétegű EXR; DPX alfa; OpenColorIO (ACES) | Harmony 24 | ❌ |
| PSD-import Photoshop nélkül | Spine 4.2, Rive | ❌ |
| Lottie / Figma beillesztés-import | Rive | ❌ (csak Lottie-**export**) |
| Textúracsomagoló / atlaszok | Spine | ❌ |
| JSON/bináris csontváz-export + 19 hivatalos / 40+ közösségi futtatókörnyezet | Spine | 🟡 saját JSON; futtatókörnyezetek nincsenek |
| Nyílt forráskódú futtatókörnyezetek (Web, Apple, Android, React, Flutter, C++, Unity, Unreal) | Rive | ❌ |
| Pixels mód (sprite-ok igazítása képernyőpixelekhez) | Spine 4.2 | ❌ |
| Export: WebM átlátszósággal | Live2D 5.3 | ❌ |
| Export: SVG/PDF, PNG/EXR sorozatok, 16/32 bites mélység, köteg, ProRes | Cavalry | 🟡 SVG, PNG ✅; PDF/EXR/ProRes/nagy bitmélység ❌ |
| Felhőalapú renderelés, megosztási linkek, beágyazások | Rive | ❌ |
| Elosztott renderelés (Toonz Farm), verziókezelés | OpenToonz | ❌ |
| PDF / TVPaint JSON export | OpenToonz | ❌ |
| Valós idejű csapatmunka, közösségi remixelés | Rive | ❌ |
| HTML5 Canvas / interaktív webes export | Animate † | 🟡 HTML-lejátszó (előrenderelt, nem interaktív) |
| Natív Linux/macOS/Windows | Synfig, OpenToonz | ✅ |
| Nyílt forráskódú licenc | Synfig, OpenToonz | ✅ MIT/Apache-2.0 |
| Rétegszülőség-nézet, szimbólumok/könyvtárpéldányok † | Animate | 🟡 szülőség ✅; példánykönyvtár nincs |
| Moduláris leválasztható UI, egyedi munkaterületek, sötét/világos téma | TVPaint 12, Live2D 5.0 | ❌ |

## Ütemterv-jelöltek (érték ÷ ráfordítás szerint rendezve, először GUI nélkül)
1. Trim path-ok + útvonalkövetés (csontok/rétegek Bézier mentén) — Rive, Spine
2. Onion skin + mozgásnyom-rávetítések — szinte minden 2D eszközben
3. SVG-import (útvonalak, színátmenetek) — Moho 14, Harmony, Synfig
4. Szél / gravitáció tagok a rugókhoz, Smart-Bone-stílusú bemenetekkel vezérelt csontdinamika
5. Blur, vetett árnyék, glow, kontúr, posterize (rétegszűrők)
6. Rétegpéldányosítás / duplikátorok (N klón egy útvonal mentén, példányonkénti eltolásokkal)
7. Interaktív állapotgép a HTML/JSON futtatókörnyezetben (bemenetek, átmenetek, blend)
8. Envelope (cage) deformer, súlymásoló/-összeforrasztó/-simító eszközök, egyenletes sűrűségű automatikus háló
9. Kameravágások / mentett kamerák, célkövető kamera
10. A rig glTF-exportja; EXR/PDF/WebM-alfa export
11. Szövegrétegek (betűtípus-betöltés kell hozzá) útvonalkövetéssel
12. Stúdió GUI: vászonon pont-/csontszerkesztés, görbeszerkesztő, onion skin

## További források (versenytársak)
- Toon Boom Harmony 24 kiadási jegyzetek és a Premium Getting Started Guide (docs.toonboom.com)
- Spine 4.2 „The physics revolution” és „What is Spine?” (esotericsoftware.com)
- Live2D Cubism 5.0 és 5.3 „New Features” (docs.live2d.com)
- Cartoon Animator 5.3 funkció-kiemelések (discussions.reallusion.com)
- Rive funkcióoldal (rive.app/features)
- Synfig Studio funkciók (zoftwarehub.com); OpenToonz „What's New” dokumentáció
- TVPaint Animation 12 (tvpaint.com); Cavalry-értékelés (superrendersfarm.com)
- Adobe Animate „Layer parenting” (adobe.com/learn); Procreate Dreams (Cartoon Brew)
- Duik: Lester Banks címkeoldal (csak cím †); Blender Grease Pencil: a kézikönyvoldal nem érhető el (nem lefedett)

---

# Kiegészítés — Versenytársak verziótörténete (2. kör, 2026-10-19)

A jelmagyarázat a fentiekkel azonos: ✅ kész, 🟡 részleges, ❌ még nincs. Csak a gyártói/projektoldalakon vagy a sajtóban megerősített funkciók szerepelnek. Kalózoldalakat nem használtunk; a web.archive.org nem érhető el ebből a környezetből, ezért néhány régebbi kiadás dokumentálatlan marad (lásd „Ismert hiányok”).

## Toon Boom Harmony 20 (gyártói kiadási jegyzetek)
| Funkció | OSATURA |
|---|---|
| Centerline-szerkesztő: pont törlése, simasági opció | ❌ |
| Textúrázott ceruzavonalak → textúrázott ecsetvonások; ceruzatextúra-pipetta | ❌ |
| Görbült / S alakú vonaleszköz; Auto-Fill ecset; Repaint ecset | ❌ |
| Igazító segédvonalak, a kijelölt rétegekre korlátozott illesztés | ❌ |
| Quick Ease / Ease In / Ease Out a kijelölt kulcsokon | 🟡 kulcsonkénti easing ✅; kötegelt easing-parancsok ❌ |
| Jelenetjelölők (időtartam nélküli, ripple edit) | ❌ |
| Rajzhelyettesítő nézet (átnevezés/szűrés) | 🟡 kapcsolórétegek ✅ |
| Felületárnyékoló csomópontok: ambient occlusion, vetett árnyék, felületi térkép/normál | ❌ |
| **Weighted Deform csomópont** | 🟡 súlyozott képháló ✅ |
| Point Kinematic Output csomópont (1/2/3 pontos mintavétel) | ❌ |
| Pre-render cache csomópont | 🟡 rugó-beégetési gyorsítótár ✅ |
| Színfade / görbék / levels / gamma csomópontok | ❌ |
| Síkrégió-alakú részecske vektoros rajz használatával | 🟡 emitter ✅; egyedi alakzat ❌ |
| Fő vezérlők: deformer-a-deformeren, Function Wizard, Slider Wizard | 🟡 csatornák ✅; varázsló-UI-k ❌ |
| Python szkriptelés (PythonManager) | 🟡 JSON patch opok + CLI; beágyazott szkriptelés nincs |
| Színtérkezelés, 16 bites renderelés, szkriptelt utófeldolgozású film | ❌ (ffmpeg utófeldolgozás kézzel) |
| Control Center: kötegelt jelenetmásolás/-import/szkriptfuttatás, rendersor | ❌ |

## OpenToonz 1.7
| Funkció | OSATURA |
|---|---|
| Egyenes vonal mód az ecsethez | ❌ |
| GIMP + MyPaint ecsettámogatás | ❌ |
| Kijelölés+szabadkézi kitöltési mód | ❌ |
| Vágólapról képbeillesztés | ❌ |
| Előgyártási irányítópult / storyboard | ❌ |
| Egyedi panelek, áttetsző főablak, nézet forgatása/tükrözése | ❌ |
| Xsheet skála + címkenavigáció | 🟡 idővonal-adatok ✅; GUI nincs |
| Export kameramozgás-információval | 🟡 kamera a Frame-ben ✅ |
| **OCA (Open Cel Animation) export** | ❌ |
| MOV és APNG export | 🟡 MP4/GIF ✅; APNG/MOV ❌ |
| Kötegelt TZP→TLV konverzió | ❌ |
| Floor Bump Iwa, Flow Fx; lebegőpontos + lineáris színű renderelés | ❌ |
| Haiku OS támogatás | n/a |

## Synfig 1.4.2 (és az 1.4.0 az 1. körből)
| Funkció | OSATURA |
|---|---|
| Lottie-export fejlett körvonal- és blur-rétegekkel, gammával | 🟡 Lottie-export ✅ (alakzatok/transzformáció); blur ❌ |
| Gyorsbillentyű-szerkesztő, egybillentyűs eszköz-gyorsbillentyűk | ❌ |
| Szerkeszthető TimeLoop a munkaterületen | 🟡 hurkoló klipek ✅ |
| Skeleton / Skeleton Deformation réteg duplikálása csontkapcsolatokkal | ✅ (a csontok projektadatok) |
| Stílusattribútumokat kezelő SVG-importáló | ❌ (nincs SVG-import) |

## Az 1. kör tételei (gyártói oldalak, rövidítve)
- **Spine 4.2/4.3**: fizikai kényszerek, csúszkák, kényszerkeverés, pixel snapping — rugók ✅ (beégetett), csúszkák/keverés 🟡 (IK `mix` ✅), pixels mód ❌.
- **Live2D Cubism 4.0–5.3**: ArtPath, onion skin, WebM-alfa export — ArtPath ❌; onion skin és WebM-alfa ✅.
- **Harmony 21/22**: súlyozott deformerek, vonal-boil textúrák — 🟡 / ✅ (remegés).
- **Cartoon Animator 5** — lásd a fenti versenytárs-táblázatokat.

## Új ütemterv-jelöltek ebből a körből
13. OCA (Open Cel Animation) és APNG export — OpenToonz 1.7
14. Kötegelt easing / „quick ease” parancsok a szerkesztőben — Harmony 20
15. Színkorrekciós szűrők (fade, levels, görbék, gamma) — Harmony 20
16. Felületárnyékolás (vetett árnyék / AO) vektorrétegekhez — Harmony 20
17. Beágyazott szkriptelés (Python/Lua) a patch-op API fölött — Harmony 20
18. Pre-render cache csomópont / gyorsítótárazott kiértékelés — Harmony 20
19. Jelenetjelölők + ripple szerkesztés az idővonalon — Harmony 20
20. Layer comps, animált csont-újraszülőzés, patch rétegek, follow-path, SMPTE időkód, biztonsági zónák, auto-trace, scatter ecset (a Moho-kutatásban talált hiányok)

## Ismert hiányok (elérhető forrásokból nem lekérhető)
Anime Studio 8.x, 10.1.x, 11.1; Moho 12.5 és 14.1 részletek; Spine 3.0–4.1 (a 3.7-es blog csak videóoldal); Harmony 17, 23, 25–27 (a dokumentációs oldalak hibát adtak); Synfig 1.4.1; TVPaint 11; Cartoon Animator 4/5.1/5.2; Rive changelog.

## Források (2. kör)
- docs.toonboom.com — Harmony 20 kiadási jegyzetek
- en.ubunlog.com — OpenToonz 1.7 cikk
- synfig.readthedocs.io — Synfig 1.4.2 kiadási jegyzetek
- esotericsoftware.com — Spine 3.7 funkcióáttekintés (videóoldal, lista nélkül)

---

# Kiegészítés — YouTube-források (3. kör, 2026-10-19)

A kizárólag a youtube.com-ra korlátozott WebSearch videócímeket ad vissza; a WebFetch az egyes videók **leírását/fejezetlistáját** adja vissza (átiratok nélkül). Csak az ott megnevezett funkciók szerepelnek; az „új az X verzióban” a feltöltő (gyakran a gyártó) állítása szerint értendő.

## Moho 14.1 (videóleírás)
| Funkció | OSATURA |
|---|---|
| Fogantyúk megjelenítése/elrejtése a Curvature eszközben | ❌ (nincs GUI-vászon) |
| Szerkesztő módban látható Shaded és Halo alakzateffektek | 🟡 színátmenetek ✅; halo/shade effektek ❌ |
| Live Project Settings pixelesítő effekt | ❌ |
| Preview Animation: nagyobb felbontás, rejtett kimeneti szegély | 🟡 raszteres előnézet ✅ |
| Módusmentes párbeszédablakok Lua szkriptekhez | ❌ (nincs szkriptelés) |
| PSD „Linear Dodge (Add)” keverési mód; frissített keverési módok | 🟡 ellenőrizd a keverési módok listáját; add/dodge ❌ |
| Mentetlen dokumentum jelzője a címsorban | ✅ a stúdió követi a visszavonási állapotot |
| Javítások: smart-bone kiértékelés, képkocka alatti motion blur + csontdinamika, nulla hosszúságú Bézier-fogantyúk | n/a |

## Moho 12.5
A cím-/leíróoldalak nem adtak funkciólistát (a videó elavult; az ingyenes 12.5 letöltést visszavonták). A kapcsolódó címek Smart Bones oktatóanyagokat, Smart Warp 2.5D rigeket és a „Mynd Tools” telepítését említik — Smart Bones 🟡 (csatornák/morphok), Smart Warp 🟡 (FFD rács ✅).

## Spine
| Verzió | Funkció | OSATURA |
|---|---|---|
| 3.7 | Hangtámogatás | ❌ (a WAV-ot csak lip-synchez elemezzük) |
| 3.7 | Újragondolt exportok | 🟡 |
| 3.7 | Gépeléses keresés | ❌ |
| 3.7 | Nyújtható / összenyomott / egységes IK | ❌ (az IK hosszmegőrző) |
| 3.7 | Skinek kombinálása, skinek duplikálása, helyőrzők | 🟡 kapcsolórétegek |
| 3.7 | Csúcspont másolása/beillesztése | ❌ |
| 3.7 | Pixelrácsos renderelés | ❌ |
| 3.7 | Additív animációkeverés | 🟡 csak additív morphok |
| 3.7 | spine-cpp futtatókörnyezet; webes lejátszó | ❌ / 🟡 HTML-lejátszó |
| 3.8 | Hálókövetés (mesh tracing) | 🟡 automatikus rácsháló; képkövetés nincs |
| 3.8 | Sokszögű textúracsomagolás | ❌ |
| 3.8 | **Soft IK** | ❌ |
| 3.8 | Skin-csontok és -kényszerek | ❌ |
| 3.8 | Mix-and-match projekt | ❌ |
| 3.8 | Mappák, vonalzók, továbbfejlesztett PSD-export, üdvözlőképernyő | ❌ |
| 3.8 | Kisebb .json/.skel; bináris betöltés a spine-ts-ben | ❌ (csak JSON) |
| 4.0 béta | **Görbeszerkesztő** | 🟡 kulcsonkénti easing ✅; szerkesztő nincs |
| 4.1 béta | Sequence (képsorozat-csatolmány) függvény | ❌ |

## Toon Boom Harmony 25 / 25.2
| Funkció | OSATURA |
|---|---|
| Pencil Retouch eszköz | ❌ |
| Photoshop-ecsetek importja | ❌ |
| **Breakdown Assistant cut-out animációhoz** | ❌ (a járásciklus-generátor rokon) |
| Idővonal-vonalzó zárolása; akciósablon-beállítások | ❌ |
| Jobb kép-, 3D- és USDZ-fájltámogatás | ❌ |
| Art Layer Manager (rajzrétegek cseréje, törlése, egyesítése, duplikálása; 25.2+) | ❌ (rétegduplikálás patch opokkal ✅) |
| Master Controller Grid Wizard | ❌ |

## Cartoon Animator 5 sorozat
| Funkció | OSATURA |
|---|---|
| 5.1 PSD és vektor round-trip szerkesztés | ❌ |
| 5.1 Képsorozat- és GIF-import | ❌ |
| 5.2 **Motion Pilot** (mozgás bábozása húzással) | ❌ |
| 5.2 Motion Path animáció / útvonalszerkesztés | ❌ (ütemterv #1) |
| Rugódinamika, FFD, vektoranimáció (meglévő) | ✅ / ✅ / ✅ |

## Új ütemterv-jelöltek ebből a körből
21. Soft IK + nyújtó IK módok — Spine 3.8/3.7
22. Képsorozat-réteg (képkockalista-csatolmány) és GIF/APNG-import — Spine 4.1, CA 5.1
23. Pixelesítő / pixel-snap kimeneti szűrő — Moho 14.1, Spine 4.2
24. Motion Pilot-stílusú rögzítés: egy csont/fogantyú élő húzásának rögzítése kulcsokba — CA 5.2
25. Breakdown-asszisztens (automatikus közbülső képkockák ívekkel cut-out rigekhez) — Harmony 25
26. Hálókövetés az alfa-kontúrból — Spine 3.8
27. Skin-készletek (csatolmánycsoportok együttes cseréje) — Spine 3.7/3.8
28. Hangsáv a projektben + lejátszás-szinkron (nem csak elemzés) — Spine 3.7
29. Lua/Python szkriptelési kampók — Moho, Harmony 20

## Források (3. kör, YouTube)
- Moho 14.1: A Complete Breakdown — youtube.com/watch?v=QT4AiV1wO1k; Moho 14.1 Update — watch?v=FCiBKBjDaNg
- Spine 3.7 features rundown — watch?v=Fi0ks2LeYSM; Spine 3.8 features rundown — watch?v=kU26B5465SQ; Spine v4-beta curve editor — watch?v=37CeFRzrjqc
- Harmony 25 changes — watch?v=kJ58C2DNIk8; Art Layer Manager 25.2+ — watch?v=AZc-8zHucOo
- Cartoon Animator 5.2 Motion Pilot — watch?v=Eu7vO2uGGN4; 5.1 sequence/GIF import — watch?v=O8EGwVwqgfs

---

# Kiegészítés — Kimerítő hiánykeresés (4. kör, 2026-10-19)

Források: a Synfig/Rive hivatalos kiadási jegyzetei, YouTube-címek/leírások. Csak a megnevezett funkciók szerepelnek.

## Anime Studio 8 (YouTube „Pro Feature” sorozat, címek + leírások)
| Funkció | OSATURA |
|---|---|
| Automatikus képkövetés (bitmap → szerkeszthető vektor) | ❌ (ütemterv #20) |
| Character Wizard (vezetett karakterkészítés) | 🟡 demo rigek/járásgenerátor; varázsló nincs |
| Photoshop (PSD) fájlok importja | ❌ |
| Real-Time Media Connection (élő kapcsolat külső médiához) | ❌ |
| Scale Points eszköz | 🟡 patch opok a pontokon |
| 1080p HD projekt-előbeállítások | ✅ tetszőleges méret |
Anime Studio 10.1 / 11.1: nem találtam elérhető forrást.

## Synfig 1.4.1 – 1.4.4
| Verzió | Funkció | OSATURA |
|---|---|---|
| 1.4.1 | Hangrétegek belekeverése az exportált videóba | ❌ (nincs hangsáv) |
| 1.4.1 | A réteg duplikálása az összes kijelölt réteget másolja | ✅ (kötegelt opok) |
| 1.4.3 | Csontkapcsolat-felugró: a transzformáció összekapcsolása az origó helyett | ✅ |
| 1.4.3 | Papagayo (.pgo) import; SVG-import javítások; képsorozat képkockasebessége | ❌ / ❌ / ❌ |
| 1.4.3 | GIF-renderelési javítások, FFmpeg 3.x kompatibilitás | ✅ GIF/MP4 |
| 1.4.3 | Automatikus mentés (akár 15 s) | ❌ (explicit mentés) |
| 1.4.4 | Utolsó képkocka exportjának javítása | ✅ tesztek fedik |

## Rive (hivatalos kiadási jegyzetek, 2026. szept.–okt.) — a legaktívabb versenytárs
| Funkció | OSATURA |
|---|---|
| String Template konverter (szöveg view-model értékekből, számformázás) | ❌ |
| Szaggatott körvonalak (Dash Effect), SVG-ből importálva | ❌ |
| Q/W/E nyomva tartása + húzás mozgatáshoz/forgatáshoz/skálázáshoz | ❌ (nincs GUI) |
| Brush alakzatok kulcsolhatók Animate módban | ❌ |
| Kényszer-kulcsok másolása/beillesztése; a konverterek megtartják a képleteket | 🟡 kényszerek ✅, másolás ❌ |
| Relatív adatkötés; view modellek; Blob tulajdonságok | ❌ |
| Luau szkriptelés (fetch, szövegformázás, GPU-vászon, shaderek/WGSL, példányosított képhálók) | ❌ |
| Blend állapotok / Mix-by-view-model idővonal | 🟡 beégetett crossfade |
| Billentyűzet-, fókusz-, gamepad-figyelők; Text Input; akadálymentességi szemantika | ❌ |
| Rács- és Stack elrendezések, virtualizált listák, görgetési kényszerek és görgetési fizika | ❌ |
| Set Parent (Alt+P) a pozíció és a rajzolási sorrend megtartásával | 🟡 újraszülőzés patch opokkal; világkoordináta-megtartás ❌ |
| Align to Key Object; gyökércsont-eltolás; Scroll to selection | ❌ |
| Additív erősség kitöltésekhez/körvonalakhoz; külön RGBA-csatornák kulcsolhatók/köthetők | 🟡 additív morphok; RGBA kulcsolva ✅ |
| Belső/külső körvonalak (korai hozzáférés) | ❌ |
| Auto Trace (jobb pixel arthoz) | ❌ |
| Flash `.fla`/XFL → Rive konverzió | ❌ |
| Render-előbeállítások: állapotgép → videó | 🟡 klip → MP4/GIF |
| Ingyenes csomagban `.riv` export; CLI (`rive push/publish/ls/docs`), VS Code kiterjesztés, headless bemenet-szimuláció | 🟡 CLI ✅ (saját parancsok) |
| MI-ágens a szerkesztőben | ❌ |
| Android-előnézet; React Native / Apple / Android / Flutter futtatókörnyezetek | ❌ |
| Revíziótörténet és visszaállítás, offline szinkron, nagyítást megtartó visszavonás | 🟡 visszavonás/újra ✅ |

## TVPaint Animation 11 (YouTube-leírások)
Gyorsbillentyű-létrehozás, eltávolítható alrajzok, vonalsimítás, rajzrögzítés, KeyFramer pivot-pont kattintás-és-húzás (11.7.1), Sprite Generator, Inlay Texture FX, segédvonalak, „end of update releases” videó — mind ❌ (raszteres festőeszközök; az OSATURA vektoros/rig).

## Toon Boom Harmony 17 (YouTube)
Free Form Deformer 🟡 (FFD rács ✅), feltuningolt Master Controllerek 🟡, RGB Difference Keyer ❌, Motion Blur ❌ (ütemterv #5).

## Cartoon Animator 4 / Spine 3.6 / Harmony 22–23
Talált videók (CA 4: Rokoko mocap bővítmény, 3D-mozgás-2D-be frissítés, normálmap és megvilágítás a 4.5-ben; Spine 3.4 útvonalak; Spine 3.6 funkcióvideó; Harmony 22 rajzeszközök), de a leírások nem sorolnak fel funkciókat → nem állítunk semmit. Csak cím alapján említésre méltó: **motion-capture retargeting** (CA 4) ❌, **normálmapok/megvilágítás** (CA 4.5) ❌, **útvonal-kényszerek** (Spine 3.4) ❌.

## Új ütemterv-jelöltek ebből a körből
30. Dash effekt + belső/külső körvonalak
31. Beágyazott szkript-futtatókörnyezet (Lua/Luau) GPU-mentes API-val
32. Adatkötés / view modellek + állapotgép-bemenetek (billentyűzet/mutató/fókusz) a HTML-lejátszóhoz
33. Görgetési kényszerek / elrendezési konténerek (rács, stack) UI-stílusú animációhoz
34. Papagayo (.pgo) lip-sync import; Rhubarb-stílusú import
35. Set Parent a világtranszformáció megtartásával
36. Automatikus mentés + revíziótörténet a stúdióban
37. Motion Blur és RGB keyer szűrők
38. Útvonal-kényszerek (csont Bézier mentén) — Spine 3.4, összeolvad az ütemterv #1-gyel
39. Motion-capture retargeting 2D rigre (BVH/JSON) — CA 4
40. Normálmap-megvilágítás képrétegekhez — CA 4.5
41. Hangsáv belekeverése az MP4-exportba — Synfig 1.4.1
42. .fla / XFL és PSD-import

## Továbbra sem lekérhető
Anime Studio 10.1 / 11.1, a Moho 12.5 részletes jegyzetei, Spine 3.0–3.6 részletoldalak, Harmony 23, Cartoon Animator 4.x részletes jegyzetek, a TVPaint 11 changelog-szövege.

## Források (4. kör)
- synfig.readthedocs.io — kiadási jegyzetek 1.4.1, 1.4.3, 1.4.4
- releasebot.io/updates/rive — Rive Editor 0.9–0.9.272, CLI 1.2–1.6
- YouTube: Anime Studio 8 Pro Feature sorozat (q_HH2zOE-Yk, CUemDIlPH6Y, 408CLy652mU, l9eXLUO9Dus); TVPaint 11 demo (MoqNrCAUy5k); Harmony 17 (1CU_3RJBcl4); Cartoon Animator 4 lejátszási lista

---

# Kiegészítés — Közösségi hálózatok és közösségi csatornák (5. kör, 2026-10-19)

**Hozzáférési valóság:** az X/Twitter oldalait a robots.txt letiltja a lekérés elől (csak a keresési találatok kivonatai/címei láthatók); a Facebookkal ugyanez a helyzet; a tartományszűrős Reddit-keresést a keresési proxy elutasította, a szűrő nélküli keresések pedig nem adtak vissza Reddit-szálakat; a Lost Marble fórum JavaScript-átirányító héjat szolgál ki olvasható tartalom nélkül. Semmit nem kerültünk meg. Ami olvasható volt:

## A Moho hivatalos csatornái (moho.lostmarble.com hírek, X-kivonatok)
| Tétel | Megjegyzések | OSATURA |
|---|---|---|
| Moho 13.5.3 (2022. ápr.) | továbbfejlesztett automatikus lip-sync, jobb felület, kevésbé tolakodó csontok | ✅ lip-sync (WAV-heurisztika) |
| Moho 13.5.5 (X, 2022. máj.) | **Relative Paths** a csatolt erőforrásokhoz | ✅ az erőforrás-könyvtár relatív |
| Moho 14.3 „Mantis Shrimp” (2024. dec.) | ecsetközpontú frissítés | ❌ |
| Moho 14.4 „Side Quest” (2025. nov.) | export játékmotorokba és 3D-szoftverekbe | 🟡 Lottie/saját JSON; glTF/FBX ❌ |
| Moho 14.5 „Showtime” (2026. okt.) | mágneses onion skinek, Shift & Trace, Liquid Shapes opciók, EXR/PNG színprofilokkal, élő lejátszási mód karakterek valós idejű streameléséhez/vezérléséhez | lásd a fő mátrixot; élő mód ❌ |
| Ambassador Program (2026. jún.); hivatalos álláspont: nem terveznek MI-eszközöket (2025. márc.) | irányelv | n/a — az OSATURA-ban sincsenek MI-funkciók |
| Webinárok (Víctor Paredes stb.) | oktatóanyagok | n/a |

## Közösségi összehasonlítások (Spine fórum, „Spine vs Moho 12”)
- A felhasználók a Mohóban dicsérik: a beépített fizikát, a rétegmaszkolást, a smart bone-okat, a vektorszerkesztést.
- A felhasználók kritizálják: az FFD-t gyengének tartják; **nincs valódi futtatókörnyezet (runtime)**, a játékexport FBX-re korlátozódik.
- A Creature-t az integrált motion capture miatt dicsérik (az FFD-t kritizálják).
- A Spine 4.2 fizikai kényszerei lelkes reakciókat kapnak az X-en (felhasználói posztok).
OSATURA: fizika ✅ (beégetett rugók), maszkolás 🟡 (clip booleanokon keresztül), smart bone-ok 🟡, vektorszerkesztés ✅, futtatókörnyezet ❌, mocap ❌.

## Látott fórumos kívánságlista-címek (a tartalom nem olvasható)
„Concentrate on Timeline/Workflow improvements” (2024), „Road map for Moho”, „Help Shape the Future of Anime Studio”, „13.5.2 performance issue” — arra utal, hogy az idővonal/munkafolyamat és a teljesítmény a legtöbbet kért terület; nem ellenőrzött funkciólista.

## Ütemterv-kiegészítések
43. Valós idejű élő lejátszási / streaming mód (WebSocket pózbemenet → renderelt képkockák) — Moho 14.5
44. Futtatókörnyezeti könyvtár egy kis, hordozható nyelven (az értékelő WASM-buildje) — a leggyakrabban említett Moho-gyengeség
45. Rétegmaszkolás elsőrangú rétegtípusként (nem csak boolean opok)
46. Idővonal-/munkafolyamat-ergonómia a stúdióban (görbeszerkesztő, kulcsok többes kijelölése) — közösségi kívánságlisták

## Források (5. kör)
- moho.lostmarble.com/blogs/news (17 bejegyzés, 2021. dec. – 2026. okt.)
- X keresési kivonatok: @MohoAnimation posztok (13.5.5 Relative Paths; Moho 14 árazás), felhasználói posztok a Spine 4.2 fizikájáról
- Spine fórum, „8033 spine vs moho 12” szál (de.esotericsoftware.com)
- Lost Marble fórum szálcímei (a tartalom nem olvasható)

---

# Kiegészítés — Kívánságlisták és funkciókérő táblák (6. kör, 2026-10-19)

Csak ténylegesen olvasható kérések szerepelnek. Az „OSATURA” oszlop azt mutatja, hogy a kérést a motor már lefedi-e.

## Moho (Lost Marble fórum)
Olvasható: csak a fórum főoldala. Léteznek a „Feature Request Discussions” és az „Unofficial Bug Reports and Feature Requests” szakaszok; a legutóbbi látható kérés címe: **„Dock Windows – Custom Interface”** (2026. okt. 18.). A régóta futó „Help Shape the Future of Anime Studio” szál (828 válasz, 2011–2021) és más kívánságlista-szálak („Road map for Moho”, „My wish for Moho in 2024 – Concentrate on Timeline/Workflow improvements”, „Request for top 10 Anime Studio Pro 6 features”) JavaScript-átirányító héjak → a tartalom nem olvasható. Konkrét fórumok közötti összehasonlítások (Spine fórum): a Spine-ban fizikát, maszkolást, smart bone-okat szeretnének; a Mohót gyenge FFD-ért és a futtatókörnyezet hiányáért kritizálják.
| Kérés | OSATURA |
|---|---|
| Dokkolható ablakok / egyedi felület | ❌ (a webes stúdió elrendezése rögzített) |
| Idővonal- / munkafolyamat-fejlesztések | 🟡 |
| Valódi futtatókörnyezet játékokhoz | ❌ |
| Erősebb FFD | ✅ Bernstein-rács + háló |

## Cartoon Animator (Reallusion Feedback Tracker + kívánságlista-szál)
| Kérés (szavazatok, ahol látszanak) | OSATURA |
|---|---|
| Alapértelmezett kulcsgörbe a beállításokban | ✅ kulcsonkénti easing |
| Többcsatornás kulcsszerkesztés | 🟡 patch kötegelt opok |
| Ragadós panelbeállítások | ❌ |
| Színkódolt jelölők és csatornák (bal piros / jobb zöld) | ❌ (tükrözés B↔J ✅) |
| „Uber key” az összes animált csatorna megjelenítéséhez | ❌ |
| SVG-támogatás (Affinity, Inkscape); vektorútvonalak beillesztése | ❌ |
| Igazító eszköz/panel | ❌ |
| Mélységélesség / blur szűrő | ❌ |
| Drag-and-drop GIF/APNG/videó import az idővonalra helyezve | ❌ |
| Spot/ambient fények árnyékokkal | ❌ |
| Idővonal automatikus görgetése | n/a |
| Jobb szöveg: animálható kerning/leading | ❌ |
| Animálható SVG-szín | ✅ |
| Globális elrejtés / „shy” csatornák | ❌ |
| Dokkolt panelek | ❌ |
| Több jelenet projektenként | ❌ |
| **Auto Animate panel (procedurális üresjárati mozgás a csontokon)** | 🟡 koherens zajos remegés ✅ |
| Élő rajzréteg / dinamikus szerkeszthető kellékek (18) | ❌ |
| Hitbox és fizika (14) | 🟡 rugók |
| Átlátszó MOV import (14) | ❌ |
| 360°-os panoráma (5); MI-integrációs ötletek (6) | ❌ |

## Live2D Cubism Editor funkciókérő tábla (30 szál, 2023–2026)
| Kérés | OSATURA |
|---|---|
| Linux-támogatás | ✅ |
| Gyorsbillentyűk a kézi hálószerkesztésben | n/a |
| Blendshape-korlát lecsökkentése −100%-ig | ✅ a morphok korlátlanok |
| Arányos átméretezés; gravitációs eszköz; háló-háló zárolás | ❌ / ❌ / ❌ |
| Glue a deformerekhez + pontos glue-súlyok | 🟡 súlyok ✅ |
| Fizika importja a meglévő csoportok felülírása nélkül | ❌ |
| Paraméterek zárolása az animátorban; loop-szerkesztés vs ismétlés | ❌ |
| „Expand Warp based on Child” opciók; a warp forgásközéppontjának módosítása | ❌ |
| „Set Vertex Flow”-szerű eszköz, hurokél-kijelölés, változó sűrűségű körvonal-háló leképezés | ❌ (az automatikus háló egyenletes) |
| Pixel art támogatás | ❌ |
| Csúcspont-átlátszatlanság; alkatrészek szűrése és tisztább megjelenítés | ❌ |
| Kulcsok könnyű mozgatása | ✅ patch opok |
| A gyorsítótárazott atlasz kiürítése a memóriából | n/a |
| Fizika ablak: kimeneti max %, ingatartományok tömeges szerkesztése | 🟡 |
| RISC-V / beágyazott Cubism Core | ❌ |
| Opcionális drawable-ök a CDI3 specifikációban | n/a |

## Toon Boom Harmony
A felhasználói fórum bezárt, és Discordra irányít át (nem kereshető); a régi „Feature Requests board” szálak („One can only dream: A Feature List”, „Software Upgrade Suggestions Part 2”, „Bitmap Layer Overhaul LONG OVERDUE”, „Various Quality of Life suggestions”) csak a bezárási értesítést mutatják. Kérések nem nyerhetők vissza.

## Synfig (GSoC ötletlista 2019–2022, hivatkozott GitHub-issue számok)
| Kérés | OSATURA |
|---|---|
| Waypointok másolása/beillesztése (#2139) | ✅ kulcsok opokkal |
| Ctrl+Tab lapváltás, ablak-gyorsbillentyűk, lejátszási terület kijelölése, tulajdonságkeresés, vonalzók ki/be, egérvezérelt paramétermódosítás | ❌ (GUI) |
| Teljesítmény: gyorsabb renderelés/export; rétegparaméterek gyorsítótárazása; inkrementális rendertask-fa | ✅ tiszta függvényű evaluate; gyorsítótár ❌ |
| GTK 4 portolás, Conan/vcpkg buildek | n/a |

## Spine
A kérések a fórumra mennek, és egy nyilvános Trello-táblára kerülnek át (nem lekérhető: robots.txt). A fórumról: a legnagyobb prioritás történelmileg a szerkesztő stabilitása és a futtatókörnyezetek frissítése. A Kickstarter-korszak terve (hálók → FFD → skinning → IK pinning) mára teljesen elkészült, így az OSATURA háló/FFD/skinning képességei paritásban vannak azzal a régi listával; az IK pinning ✅ (Pin kényszer). Legutóbbi fórumos kérések címei: a legújabb three.js / React Three Fiber támogatása; Spine 4.3 támogatás a Defoldban.

## Rive
A nyilvános ütemterv = „Feature Requests” csatorna állapotcímkékkel (folyamatban / tervezett / megjelent); olvasásához bejelentkezés kell. A kiadási jegyzeteken (4. kör) túl semmi nem nyerhető vissza.

## OpenToonz, TVPaint, Pencil2D
A GitHub (OpenToonz/Synfig) oldalait a robots.txt blokkolja; olvasható TVPaint-táblát nem találtunk; létezik egy Pencil2D „Feature requests to add” szál, de nem nyitottuk meg (hatókörön kívül).

## Ütemterv-kiegészítések a kívánságlistákból
47. Auto-animate / procedurális üresjárati réteg (légzés, pislogás, fejringás ritkasággal és intenzitással) — CA kívánság #20; a meglévő remegésre épül
48. Kamera mélységélessége és blur szűrő — CA
49. Fények (spot/ambient) vetett árnyékkal képrétegekhez — CA
50. Igazítás / elosztás műveletek (szerkesztői op) — CA
51. Csatornák színkódolása és „shy” csatornalátszóság a stúdióban — CA
52. Több jelenet projektenként + jelenetvágások — CA
53. Változó sűrűségű automatikus háló; csúcspont-folyás simítása — Live2D
54. Warp-deformer forgásközéppont-paraméter; arányos átméretezés — Live2D
55. Fizika importja összefésüléssel, a csoportok felülírása nélkül — Live2D
56. Paraméterzárolás sávonként loop-szerkesztés közben — Live2D
57. Dokkolható/egyedi stúdióelrendezés — Moho, CA
58. Paraméter-gyorsítótár képkockák között (inkrementális kiértékelés) — Synfig

## Források (6. kör)
- Reallusion Feedback Tracker: „Wish list for new features for future updates”; Cartoon Animator 5 tábla (reallusion.com/FeedBackTracker)
- community.live2d.com — Cubism Editor funkciókérések (1. oldal a 7-ből)
- Synfig GSoC 2022 ötletlista (synfig-docs-dev.readthedocs.io)
- Lost Marble fórum főoldala (lostmarble.net/forum); Spine fórum 2250., 8033. szál; Kickstarter-szál 1501
- Rive GyIK: „Does Rive have a public roadmap?”

---

# Kiegészítés — Nyílt forráskódú projektek git-platformokon (7. kör, 2026-10-19)

**Hozzáférés:** a GitHub keresési API nem érhető el ebben a környezetben („sessions are bound to their configured repositories”), ezért a projekteket webes kereséssel találtuk meg, és nyilvános README-jeikből olvastuk. A GitLab és a Codeberg API-it a proxy blokkolja; a Codeberg/GitLab oldalak csak a keresési találatokban jelentek meg (a Friction olvasható volt). A csillagszámokat nem lehetett lekérni. Több projekt nagyon új/korai fázisú — a felsorolt funkciók a README-k állításai szerint szerepelnek, futtatással nem ellenőriztük őket.

## Hasonló nyílt forráskódú rigging- / animációs eszközök
| Projekt | Platform | Említésre méltó funkciók | Összehasonlítás az OSATURA-val |
|---|---|---|---|
| **Rigby2D** (jaisurya-dev-art/rigby2d) | GitHub, MIT/Apache | Spine-stílusú szerkesztő (PSD-import, háló + súlyfestés, összekapcsolt hálók, IK/transzformáció/útvonal/fizika/**slider** kényszerek, skinek skin-specifikus csontokkal, események, befoglaló doboz/clipping/pont/útvonal csatolmányok, dopesheet + görbeszerkesztő), Spine JSON 3.8/4.2/4.3 + .skel export, atlasz, JS/Rust/Godot futtatókörnyezetek, **beépített MCP-szerver** MI-asszisztensekhez, Tauri asztali alkalmazás | Az OSATURA-ból hiányzik a szerkesztő-GUI, az atlasz, a futtatókörnyezetek, a skinek, az események, a slider/útvonal kényszerek; megvan a tiszta függvényű kiértékelő, a folyadék/boolean/emitter FX, a lip-sync, a járásgenerátor |
| **SkelForm** (Retropaint/SkelForm, Rust) | GitHub | FABRIK IK, natív + web + mobil burkoló | hasonló rig-mag ✅; GUI nincs ❌ |
| **Ankhimate editor** | GitHub, pre-alfa | Setup/Animate módok, slotok + skinek, tetszőleges hosszúságú FABRIK, útvonal/fizika kényszerek, hálósziluett-követés alfából, súlyecset keverési módokkal + csontzárolással, **FFD idővonalak**, események hanggal, jelölők, csontonkénti sáv-eltolások, **sablonalapú exportformátum-szerkesztő**, Rust referencia-futtatókörnyezet | ❌ többnyire GUI; ötletek: exportsablonok, csontonkénti sáv-eltolások |
| **rig2d-rs** (tanhieuno41) | GitHub, Rust | Spine-4.2 fizika-port, IK **mix/softness/bend/compress/stretch/uniform** módokkal, automatikus súlykötés távolság-lecsengéssel, súly simítás/normalizálás/4-re korlátozás, polcos atlaszcsomagoló, onion skin szellemképek, Spine JSON import/export | ❌ soft/stretch IK, súlyeszközök, atlasz, Spine-csere |
| **Boner2D**, **Skelly**, **omarvision 2d-bones-IK** | GitHub | Unity / játéktervezői csontváz-eszközök | alap |
| **Skeletal-Animator** (ostad-ai) | GitHub, Python | csontok + FABRIK, téglalap/kapszula/kör alakzatok, 4 easing, onion skin, kulcsolt kamera-vágóablak, **csontonkénti láthatósági tartományok**, spritesheet PNG+JSON, **átlátszó MP4/WebM/MOV/GIF** | ❌ láthatósági tartományok, átlátszó videó-/spritesheet-export, onion skin |
| **Stretchy Studio** (MangoLion) | GitHub, MIT | PSD/PNG import, **MI automatikus rigging** (DWPose pózfelismerés), beállító varázsló, alkatrészenkénti háló csúcspont-animációval, csúcspont-skinning végtagokhoz, **automatikus szemvágás** (írisz a szemen belül), **Blender-stílusú shape key-ek** befolyás-csúszkákkal, hangsávok az idővonalon, **Spine 4.0 JSON export** | ✅ a morphok ≈ shape key-ek; ❌ auto-rig, szemvágás, hang |
| **Iki** (zeikar/iki) | GitHub, MIT | Live2D-stílusú webes motor: paramétervezérelt quadok, warp-háló/rács deformerek, stencil maszkok, rugó/lánc fizika, PSD-import, **automatikus rigger szerepnevű rétegekből** (pislogás, tekintet, ajaksinkron, fejfordítás, bólintás, döntés, szemöldök, légzés, hajfizika), nyílt JSON-séma **StandardParameter ID-kkal**, MCP-szerver + Claude Code plugin | ✅ JSON-séma, lip-sync; ❌ szabványos paraméter-ID-k, automatikus rigger, maszkok |
| **Umamo** (umamoorg/umamo) | GitHub, Java | Live2D-kompatibilis szerkesztő: CMO3/MOC3 olvasás/írás (Cubism 5.4-ig), PSD/KRA/CLIP import, parancspaletta, billentyűkötés-előbeállítások, **diagnosztikai CLI** (modellek dump/extract/convert/diff, automatikus háló minőségi előbeállítások); animáció még nem támogatott | ❌ modell-diff CLI, parancspaletta |
| **Ayatsuri2D** | GitHub | `.aom` MessagePack modell (a moc3 felülhalmaza), moc3 v3–6 export, C ABI, fuzzing a CI-ban, pánikmentes futtatókörnyezet, p99.9 késleltetési benchmarkok (korai, „not usable yet”) | ❌ bináris formátum, C ABI, fuzzing |
| **Ayagami** (AyagamiDev) | GitHub, Rust/wgpu | MOC3 az SDK 5.0-ig, glue-val és blendshape paraméterekkel, részleges újraszámítás, lineáris/sRGB keverés, SDK 5.3 fejlett blend/off-screen, Godot-komponens | ❌ GPU-renderelő, MOC3 |
| **kasane-2d**, **img2live**, **claude-live-2d**, **minivt** | GitHub | Live2D-szerű modellezés / egyképes bábgenerálás kutatás | n/a |
| **Glaxnimate** (KDE) | GitHub tükör / KDE | vektoranimáció; a build-függőségek Potrace bitmap-követést, libav-ot, Python szkriptelést, libarchive-ot mutatnak; Lottie-fókusz | ❌ követés, szkriptelés |
| **Friction** (friction/friction, Codeberg) | Codeberg, GPL-3 | vektoros + raszteres animáció webre/videóhoz, motion graphics, SVG, Skia + FFmpeg + GLSL (az enve alapján) | ❌ SVG-animáció, shaderek |
| **Tahoma2D** | GitHub | 2D + stop-motion, a Toonz Studio Ghibli verzióján alapul | ❌ |
| **Pencil2D** | GitHub, GPL-2 | bitmap + vektor kézzel rajzolt, onion skin | ❌ |
| **q0idmation** (q0s0id) | GitHub | szerkesztő + lejátszó + **szkriptnyelv** | ❌ szkriptelés |
| **svgMotion** (michaelsboost) | GitHub | vektoranimáció böngészőben | ❌ |
| **kinetic** | GitHub | SVG kulcs-idővonal → CSS/GSAP export | ❌ CSS/GSAP export |
| **dotlottie-rs** (LottieFiles) | GitHub, Rust | **állapotgépek** (guardok/átmenetek/akciók), témázás/**slotok** (szín/skalár/szöveg/vektor futásidejű felülírások), többanimációs `.lottie` zip-konténer, C API + WASM, ThorVG renderelő | 🟡 Lottie-export ✅; ❌ slotok/állapotgépek/dotLottie konténer |
| **rive-runtime** | GitHub | C++ futtatókörnyezet: GPU vektorrenderelő (Metal, Vulkan, D3D11/12, GL/WebGL), Apple/Android/Flutter/Unity/Unreal/web burkolókkal | ❌ |
| **spine-runtimes** | GitHub | futtatókörnyezetek Unityhez, Godothoz, Unrealhez, libGDX-hez, MonoGame-hez, XNA-hoz, Flutterhez, Haxe-hoz, C/C++/C#/TS-hez, Androidhoz, iOS-hez, GLFW-hez, SDL-hez, SFML-hez; szerkesztőverziónként egy ág irányelv | ❌ |
| **Moho tools repo** (wanderjimenezRD/Moho-Pro-14.4-2D-Animation-Tools) | GitHub | csak listázás; a leírás nem említ funkciókat — nem értékeltük | — |

Kalózszoftver-repók is felbukkantak (pl. „activate-free” Adobe Animate): nem nyitottuk meg és nem használtuk őket.

## Átvételre érdemes ötletek (új ütemterv-jelöltek)
59. **MCP-szerver / ágens-API** a patch-op szerkesztő fölött, hogy MI-asszisztensek rigelhessenek és animálhassanak (Rigby2D, Iki) — illik az OSATURA JSON-patch architektúrájához
60. **Szabványos paraméter-ID-k** (ParamAngleX, EyeOpen, MouthOpen…), hogy bármely hoszt vezérelhessen egy modellt (Iki)
61. **Automatikus rigger szerepnevű rétegekből**: pislogás, tekintet, fejfordítás/bólintás/döntés, szemöldök, légzés, hajfizika (Iki)
62. **Automatikus szemvágás** (az írisz a szem alakjával maszkolva) (Stretchy Studio) — Boolean rétegen keresztül
63. **Slider kényszer** és **útvonal-kényszer** (Rigby2D, Spine)
64. **Soft / stretchy / compress / uniform IK** módok (rig2d)
65. **Súlyeszközök**: simítás, normalizálás, N-re korlátozás, másolás, csontzárolás, ecset-keverési módok (rig2d, Ankhimate)
66. **Hálósziluett-követés alfából** (Ankhimate)
67. **Csontonkénti láthatósági tartományok** (Skeletal-Animator)
68. **Átlátszó hátterű videóexport (WebM/MOV alfával) és spritesheet PNG+JSON** (Skeletal-Animator)
69. **Exportsablon-motor** (felhasználó által definiált kimeneti formátumok) (Ankhimate)
70. **dotLottie `.lottie` konténer + futásidejű slotok/témázás + állapotgépek** (dotlottie-rs)
71. **Modell diff / dump / convert CLI** (`osatura diff a.json b.json`) (Umamo)
72. **Parancspaletta + újrakötött gyorsbillentyűk** a stúdióban (Umamo)
73. **Fuzzolt betöltő, pánikmentes futtatókörnyezeti útvonal, késleltetési benchmarkok** (Ayatsuri2D) — az `Engine::new` robusztussági megerősítése
74. **Spine JSON import/export** az adatcseréhez (Rigby2D, rig2d, Stretchy Studio)
75. **Polcos atlaszcsomagoló** képrétegekhez (rig2d, Rigby2D)
76. **Események kulcsolt értékekkel és hangjelzésekkel** az idővonalon (Rigby2D, Ankhimate)
77. **Csontonkénti sáv-eltolások / időeltolás** (Ankhimate)

## Források (7. kör)
- github.com/jaisurya-dev-art/rigby2d; Retropaint/SkelForm; Ankhimate/editor; tanhieuno41/rig2d-rs (v0.1.0 kiadás)
- github.com/zeikar/iki; MangoLion/stretchystudio; umamoorg/umamo; Yoru-KenomoVT/ayatsuri_2d; AyagamiDev/ayagami
- github.com/ostad-ai/Skeletal-Animator; KDE/glaxnimate; tahoma2d/tahoma2d; pencil2d/pencil
- github.com/rive-app/rive-runtime; EsotericSoftware/spine-runtimes; LottieFiles/dotlottie-rs
- github.com/topics/animation-editor, /topics/vector-animation
- codeberg.org/friction/friction

---

# Kiegészítés — Más nyilvános források (8. kör, 2026-10-19)

Források: itch.io, Steam, AlternativeTo, Wikipedia, tudományos/szabadalmi keresési találatok, After Effects eszközértékelések. Az állítások bolt-/projektoldalakról származnak, nem gyakorlati tesztekből.

## Indie / niche rigging eszközök (itch.io, Steam)
| Eszköz | Említésre méltó funkciók | OSATURA |
|---|---|---|
| **Inochi Creator / Inochi2D** (Kitsunebi Games, nyílt forráskódú, béta) | nyílt, VTuber-orientált 2D bábrigging + valós idejű futtatókörnyezet; saját formátum (nem Live2D/Spine-kompatibilis); GIF/képsorozat-export; többnyelvű UI | 🟡 ugyanaz a niche (nyílt rigformátum) |
| **Keyframe.it** (böngésző, ingyenes) | csontok + IK + súlyozott hálók + rugós mozgás; **játékszimbólum-klipek** (idle, land, anticipation, win); beépített képszerkesztő + PSD-import; export GIF, sprite sheet, videó, önálló HTML-lejátszó, **Spine 3.8/4.2 csomagok**; automatikus mentés a böngészőben; **keyframe-mcp** MI-ágens összekötő; opcionális beépített MI-asszisztens | ✅ HTML-lejátszó/GIF; ❌ sprite sheet, Spine-csomag, MCP, böngészős szerkesztő |
| **RigBone 2D** (Godot) | csontpózolás, póz-mint-képkocka, onion skin, vágás + átlátszó háttér, PNG képkocka-sorozat (futtatókörnyezet nélkül) | ❌ onion skin, PNG-sorozat alfával és vágással |
| **SpineBones** | slotok + képcsatolmányok, csoportok, IK, **hang-előnézet az idővonalon**, háttér-referenciakép, JSON / sprite sheet / WebM export | ❌ |
| **PySpine, Tangent (sprite animator)** | az itch.io-n listázva; funkciószöveget nem olvastunk | — |
| **Mocap2D / FBX2SPINE** (Steam) | FBX/BVH mocap import; **Auto Match** csontváz név + hierarchia szerint; **Smart Retargeting** (vetítés, oldalnézethez jó); **Direct Retargeting** tengelyenként; **automatikus helyben-konverzió** mozgáshurkokhoz; **talajérintkezési IK** (talprögzítés); élő előnézet; import/export FBX, BVH, Spine csontvázak, Unity 2D Animation | 🟡 név szerinti retarget ✅; ❌ mocap-import, talprögzítő IK, helyben-konverzió |
| **VTube Studio** (Steam) | webkamerás követés (OpenSeeFace), **hangvezérelt száj**, egér/toll mint követési bemenet, tetszőleges bemenet → tetszőleges modellparaméter leképezés, gyorsbillentyűk kifejezésekhez/animációkhoz, a modellt követő kitűzött PNG „itemek”; csak Live2D modellek | 🟡 paramétercsatornák ✅; ❌ élő bemenet-leképezés, gyorsbillentyűk, kitűzött itemek |
| VBridger, Vitamins | Steam-eszközök a VTubing körül; nem értékeltük | — |

## After Effects rigging ökoszisztéma (School of Motion értékelés)
| Funkció | Forrás | OSATURA |
|---|---|---|
| **Joystick vezérlő** (5 kulcspóz: origó/jobb/bal/felső/alsó) 2D-ben interpolálva | Joysticks 'n Sliders | ❌ — erős jelölt |
| **Csúszkák**, amelyek korlátlan számú pózt kevernek | JnS, Duik | 🟡 morph-csatornák |
| Struktúrák (a grafikától független, újrahasznosítható csontok), autorig | Duik Bassel | ✅ |
| Vezérlők: csúszka, 2D csúszka, szög | Duik | ❌ |
| **Connector** (csúszka/joystick/forgatás, amely bármely tulajdonságot egy másikhoz köt), fő tulajdonságvezérlés | Duik | 🟡 kifejezések? (a kifejezésmotor ✅ a csatornákban) |
| Köztes pózkulcsok a póz-állapotok között | Duik | ✅ |
| Maszkútvonalak animálása | JnS | ✅ útvonal-morphok |

## Versengő / alternatív alkalmazások (AlternativeTo, Wikipedia-lista 2026-os állapotban)
Tahoma2D (OpenToonz fork, stop-motion), TupiTube, enve (megszűnt; Friction lett belőle), Expressive Animator (interaktív SVG), Cavalry, Jitter (webes motion design), Blazy (Figma → animáció, CSS/React export), Animation Desk, Pivot Animator, FlipaClip, Krita animáció, Blender (2026. márc.), Clip Studio Paint (animáció), Adobe Character Animator (webkamerás bábozás; a „what's new” oldalt a robots.txt blokkolja), Adobe Animate (a legutóbbi listázott kiadás 2023. okt.), CrazyTalk, DigiCel FlipBook, Apple Motion, RETAS. Egyik sem ad hozzá olyan riggingfunkciót, amelyet a korábbi körök ne fednének le; új ötletek: **Figma/tervezőeszköz → animáció előbeállítások CSS/React exporttal** (Blazy) és **webkamerás bábozás** (Character Animator, VTube Studio).

## Tudományos és szabadalmi irodalom (csak címek/absztraktok)
- Jacobson et al. 2011, **Bounded Biharmonic Weights** valós idejű deformációhoz — automatikus sima skinning-súlyok cage-ből/csontvázból ✅ ötlet: az `auto_skin` javítása (jelenleg távolságalapú).
- Adobe szabadalmak US9865073 / US10078910, **„Computer animation of artwork using fast 2-dimensional deformation solving”** (a Character Animator háló-warpja). *Clean-room megjegyzés:* az OSATURA Bernstein FFD-t és súlyozott skinninget használ, nem a szabadalmaztatott solvert; bármely jövőbeli ARAP-stílusú solvert jogásszal kell átnézetni. Ez nem jogi tanács.
- arXiv 2025–26 automatikus rigging: **ASMR** (3D háló csontváz + skin 2D generatív priorokból), UniRig, „How to Train Your Dragon” (diffúziós rigging), HumanRig, SPRig, UniMate — mind 3D/ML; csak egy jövőbeli „auto-rig javaslat” funkció szempontjából releváns.
- Stony Brook TVC 2017 **automatikus skinning-súly retargeting** — súlyok másolása hálók között (✅ ütemterv: súlymásolás).

## Új ütemterv-jelöltek ebből a körből
78. **Joystick vezérlő**: 5 kulcspózt (és N-pózos változatokat) keverő 2D vezérlőpont → tetszőleges csatornákat vezérel — JnS / Duik / Live2D paraméterek
79. Mocap-import (BVH) + automatikus csontváz-illesztés + helyben-hurokkonverzió + talprögzítő IK — Mocap2D
80. Élő bemenet-leképezés: bármely bemenet (egér, MIDI, hangszint, webkamera-landmarkok) → paraméter, plusz gyorsbillentyűk kifejezésekhez — VTube Studio
81. Kitűzött itemek/kellékek, amelyek egy csontot/horgonyt követnek saját eltolással — VTube Studio
82. Játékszimbólum-klip előbeállítások (idle / land / anticipation / win) — Keyframe.it
83. Spritesheet (PNG + JSON) és átlátszó PNG-sorozat export vágással — Keyframe.it, RigBone
84. Idővonali hang-előnézet hullámformával — SpineBones
85. Referenciakép-réteg (nem renderelt háttér) — SpineBones
86. Bounded-biharmonic vagy hődiffúziós automatikus skin-súlyok (a távolsági lecsengés helyett) — BBW tanulmány
87. Tervezőeszköz-előbeállítások → CSS/React export — Blazy
88. Webkamerás arckövetéses bábozás a stúdión keresztül (WebRTC + landmark modell) — Character Animator, VTube Studio

## Források (8. kör)
- itch.io: kitsunebi-games/inochi-creator, keyframeit.itch.io, itch.io/blog/1586091 (RigBone 2.0), itch.io/t/6177139 (SpineBones)
- Steam: FBX2SPINE / Mocap2D (app 2467960), VTube Studio (app 1325860)
- schoolofmotion.com — Joysticks 'n Sliders vs Duik Bassel
- alternativeto.net (Moho, Expressive Animator alternatívák); en.wikipedia.org/wiki/List_of_2D_animation_software
- Jacobson 2011 BBW (homes.cs.washington.edu/~jovan/papers/jacobson-2011-bbw.pdf); US9865073B2 / US10078910 szabadalmak; arXiv 2503.13579, 2503.15586, 2504.12451; Stony Brook TVC 2017 súly-retargeting

---

# Kiegészítés — Adobe Character Animator, teljes funkciókészlet (9. kör, 2026-10-19)

Források: Wikipedia verziótáblázat (a 2026. jan.-i 26.0-val együtt), Adobe termékoldal, az Adobe Help „Behaviors” oldala és kiadási jegyzetek (2018. okt., 2020. okt./2021. febr., 2021. okt./dec., 2023). Az Adobe saját „what's new” indexe csak 2023-at és 2026-ot adja vissza; a 24.x és 25.x verziókat a Wikipedia csak stabilitási kiadásokként sorolja fel.

## Koncepció
A rétegezett Photoshop/Illustrator grafika (2018. okt. óta EPS/SVG is) **réteg-nevekből/címkékből** automatikusan rigelt **bábbá** válik; az előadást élőben rögzíti (webkamerás arc/test, mikrofon, billentyűzet, egér, MIDI) **take-ekként** egy idővonalon; a kimenetek: PNG-sorozatok, WAV, Media Encoder (beleértve a **videót alfával**), Dynamic Link After Effectshez / Premiere-hez, valamint élő Syphon / Mercury Transmit. A Starter mód ingyenes; a Pro-hoz Creative Cloud kell.

## Behaviorök (teljes lista az Adobe Helpből)
| Behavior | Mit csinál | OSATURA |
|---|---|---|
| **Face** | fej, szemek, szemöldök, orr, száj webkamerából; parallaxis és fejpozíció-erősség | ❌ (nincs élő követés) — ütemterv #88 |
| **Body** (Body Tracker, Sensei AI, 22.0) | karok, törzs, lábak webkamerából | ❌ |
| **Lip Sync** | szájformák (visemek) hangból; átirat-támogatott pontosság (22.1); a 3.4-es motor jobb F, W-Oo, kevesebb L/M hibával | 🟡 WAV → 5 viseme ✅; átirat-igazítás ❌ |
| **Nutcracker Jaw** | alsó állkapocs arcból/hangból | ❌ |
| **Eye Gaze** | a pupillák egymástól függetlenül mozognak | ✅ aim kényszerek/csatornák |
| **Head & Body Turner** | rétegcsoportokat cserél (elöl, negyed, oldal) a fej/test forgásakor | 🟡 kapcsolórétegek ✅; szögvezérelt automatikus csere nincs |
| **Walk** | járásciklus, futás, pózkiemelés, váll/csípő ringás, **Pin Feet When Standing** (3.4/3.5) | ✅ járásgenerátor; ❌ pin-feet |
| **Limb IK** (korábban Arm IK) | hajlítási irány + nyújtás karokhoz és lábakhoz | 🟡 2 csontos IK ✅; nyújtás ❌ |
| **Leader/Follower** (22.5) | alakítja, hogyan hajlanak/nyúlnak a végtagok | ❌ |
| **Dragger** | egér/érintés mozgat egy régiót | ❌ (a stúdió nem interaktív a vásznon) |
| **Physics / Dangle** | lengés, ütközés, gravitáció, szél; squash & stretch, csillapítás | 🟡 rugók ✅; ütközés/szél ❌ |
| **Magnets** | a fogantyúk közelség alapján összetapadnak | ❌ |
| **Particles** | fizikai objektumokat bocsát ki (hó, ágyútűz) | ✅ emitter |
| **Motion Lines** | nyomvonalak a gyors objektumok mögött | ❌ |
| **Wiggler** | remegés vektorgrafikán | ✅ remegés/boil |
| **Breath** | légzőmozgás | 🟡 kézi csatorna; ütemterv #61 |
| **Auto Blink** | automatikus pislogás / rétegvillogás | ❌ (ütemterv #61) |
| **Motion Library** (23.0, 350+ mozgás, erősség-paraméter 23.1) | teljes testes animációk alkalmazása | 🟡 csak járás; ❌ könyvtár |
| **Cycle Layers** | léptetés egy csoport rétegein | 🟡 kapcsolórétegek ✅; képkockaléptetés ❌ |
| **Layer Picker / Motion Trigger / Fader** | réteg megjelenítése; váltás mozgásirány szerint; áttűnés triggerre | 🟡 |
| **Transform** | fogantyúk mozgatása/skálázása/forgatása/átlátszatlansága | ✅ |
| **Handle Fixer** | a háló részeinek zárolása | ✅ pinek |
| **Triggers** | billentyűleütés rétegeket/gesztusokat jelenít meg; **Controls panel** csúszkákkal, szögtárcsákkal, gombokkal, MIDI-leképezéssel | ❌ — ütemterv #80 |
| **Collide, Dynamic, Cycle Pause, Auto-swap (22.5)** | címkék/behaviorök fizikához és képkockaszerű cserékhez | ❌ |

## Idővonal-, rögzítési és munkafolyamat-funkciók
| Funkció | Verzió | OSATURA |
|---|---|---|
| Take-ek, replayek (újrahasznosítható, triggerelhető felvételek), smart replayek, szerkeszthető replayek | 2.0 / 22.0 / 23.6 | 🟡 klipek ✅; replayek ❌ |
| Kulcsok, kulcsok időnyújtása, jelenetkamerák, triggerelhető hang | 3.0 / 3.2 | ✅ kulcsok/kamera; ❌ triggerelhető hang |
| Speech-Aware Animation (fej-/szemöldökmozgás hangból, Sensei) | 3.4 | ❌ — ütemterv #61 |
| Take-ek összefésülése/sorrendezése; színes take-sorok; keresőszűrők (`kind:` `shared:` `armed:`); shy sorok; take-ek összecsukása | 3.4 | ❌ |
| Illesztés képkockákhoz / objektumokhoz; idővonal-snapping | 3.5 | ❌ |
| Munkaterület-műveletek: ripple törlés, idő beszúrása, loop, vágás, export | 2.0 | 🟡 klipműveletek ✅ |
| Rigging-problémák panel; címke- és triggerkeresés | 3.0 | ✅ a validáció minden hibát összegyűjt |
| Előzmény-könyvjelzők; sérült projekt automatikus helyreállítása (4.2); új kompakt/sérülésálló fájlformátum (4.0) | | 🟡 visszavonás ✅; helyreállítás ❌ |
| Puppet Maker (sablonok: haj, bőr, kiegészítők) | 22.1.1 | ❌ |
| Characterizer (fotó/festmény átalakítása bábbá) | 2.0 | ❌ |
| Quick Export közösségi megosztáshoz | 22.5 | 🟡 GIF/MP4 ✅ |
| Jelenet-pillanatképek, felvételi visszaszámlálás, egyedi vezérlőgomb-grafika | 1.5 | ❌ |
| Kamera kiválasztása, több monitor, HiDPI | 2.0 | n/a |
| Élő kimenet (Syphon, Mercury Transmit), Twitch Character Trigger kiterjesztés | 1.1 / 2.1 | ❌ — ütemterv #43 |
| MIDI, szkriptek V8-cal (JavaScript) | 1.1 / 2.0 | ❌ — ütemterv #31 |
| Bábok megosztása/közzététele, megosztható bábfájlok | Beta 5 | 🟡 hordozható JSON |
| Dynamic Link AE-hez / Premiere-hez | Beta 5 | ❌ |
| Jelentett gyengeségek (értékelés): meredek tanulási görbe, hardverfüggés (webkamera/mikrofon), hosszú karakter-előkészítési idő, fizetős Pro, a triggerek beállításának bonyolultsága | | lehetőség: az OSATURA determinisztikus offline pipeline-ja, CLI/JSON |

## Ami kiemelkedik az OSATURA szempontjából (új ütemterv-jelöltek)
89. **Címkealapú automatikus rig réteg-nevekből** (fej, szemek, szemöldök, szájforma-visemek, kar/láb címkék) — a Character Animator alapötlete; kiterjeszti az ütemterv #61-et
90. **Take-ek és replayek**: az előadás paramétereinek rögzítése újrahasznosítható, triggerelhető klipekként, blend easinggel
91. **Pin Feet When Standing** és talajérintkezési zárolás a járásgenerátorhoz
92. **Leader/Follower** végtagnyújtó behaviorök; **Magnets**; **Motion Lines**
93. **Motion Library** formátum (elnevezett mozgásklipek 0–100% erősséggel)
94. **Head & Body Turner**: nézetcsoportok automatikus cseréje egy yaw-szög csatornából
95. **Átirat-támogatott lip-sync** (a szöveg kényszerített igazítása a WAV fonémáihoz)
96. **Beszédtudatos fej-/szemöldökmozgás** a hang energiájából és szüneteiből
97. **Videóexport alfával** (ProRes 4444 / WebM VP9 alfa) — a Skeletal-Animatorban is megvan (ütemterv #68)
98. **Triggerelhető hang** és hangreaktív paraméterek

## Források (9. kör)
- en.wikipedia.org/wiki/Adobe_Character_Animator (funkciólista és verziótáblázat a 26.0-ig, 2026. jan.)
- adobe.com/products/character-animator.html
- helpx.adobe.com — Behaviors in Character Animator; funkcióösszefoglalók 2021. okt./dec., 2020. okt./2021. febr.; What's new (2023–2026)
- community.adobe.com — „Character Animator October 2018 detailed changes”
- manytools.com/review/adobe-character-animator (erősségek/gyengeségek)
