# OSATURA — matricea de funcționalități + liste de verificare pentru produsul de referință și concurenți

> Formularea versiunilor: toate crate-urile sunt `0.1.0` (primele build-uri sunt pre-lansarea ciornă `v0.1.0-alpha.3`); „v0.5 / v0.6 / v0.7" din acest depozit sunt *niveluri de funcționalități* (vezi [CHANGELOG.md](CHANGELOG.md)), iar secțiunea 9 din `ROADMAP.md` este autoritatea privind ceea ce este construit.

Starea sinceră a ceea ce există. **✅ gata și testat · 🟡 parțial · ❌ neimplementat.**

> Notă privind domeniul: OSATURA este un motor implementat independent (pe baza specificațiilor și lucrărilor publice; listele de funcționalități ale concurenților au fost consultate pentru stabilirea domeniului; nu a fost copiat niciun cod sursă al terților; vezi CLEANROOM.md pentru dezvăluirea cercetării), nu o re-implementare a vreunui
> produs comercial și **nu** atinge paritatea cu o suită comercială veche de 25 de ani
> (GUI matur, instrumente de desen, intrare live, 3D, ecosistem). Primele tabele sunt organizate pe
> capabilități; **a doua jumătate a acestui fișier** este o listă de verificare, versiune cu versiune, a funcționalităților
> documentate public ale produsului de referință (sursele sunt listate la final), cu
> starea OSATURA pentru fiecare. „Inovațiile" sunt revendicate doar în raport cu tehnica generică,
> nu ca fiind dovedit absente din orice alt instrument.

---

# ★ SCARA DE CONSTRUCȚIE — de la simplu la complex, cu starea drepturilor de autor / IP (nivel de funcționalități v0.7, 20-22 oct. 2026)

Această secțiune este **singurul loc unde vezi ce poate fi construit, în ce ordine și dacă este sigur**.
Ea înlocuiește listele răzlețe de „candidați #" de mai jos; detaliile și ID-urile se află în `ROADMAP.md`
(R-xxx / I-xx), metoda clean-room în `CLEANROOM.md`, licențele în `LICENSING.md`.

**Stare:** ✅ construit și testat · 🟡 parțial construit · ⬜ planificat (neînceput)
**Stare IP (după alternativele din ROADMAP §8):**
🟢 sigur — standard deschis, matematică de manual sau design propriu ·
🟠 sigur *dacă* respecți o condiție (indicată în ultima coloană) ·
🔴 nu construi.
*Triaj de inginerie, nu consultanță juridică; un avocat trebuie să confirme fiecare 🟠 înainte de o lansare comercială.*

## Nivelul 1 — Fundamente (ore–zile fiecare; fără nicio expunere IP)
| Funcționalitate | Stare | IP | Bază sigură / condiție |
|---|---|---|---|
| Proiect JSON, validare, `evaluate(frame)` pur | ✅ | 🟢 | schemă proprie |
| Căi Bézier, umpleri, contururi, gradienți, linii întrerupte, contururi cu lățime variabilă | ✅ | 🟢 | geometrie de manual |
| Oase FK, limite, IK cu 2 oase/FABRIK, constrângeri | ✅ | 🟢 | cinematică de manual |
| Chei, easing, expresii, zgomot | ✅ | 🟢 | design propriu |
| Editor cu operații de patch și anulare/refacere exactă | ✅ | 🟢 | pointeri JSON în stil RFC 6902 |
| Export SVG, PNG, GIF, player HTML, Lottie | ✅ | 🟢 | specificații publice |
| Jurnal de proveniență, `stats` / `diff` semantic, regresie cu cadre de referință (hash exact + toleranță), plafoane de resurse, test de mutation-fuzz, poartă de versiune a formatului de fișier (R-001…R-005) | ✅ | 🟢 | design propriu |
| Vocabular standard propriu de roluri + tagger bazat pe nume (`tag`) (R-006) | ✅ | 🟢 | nume proprii, cuvinte simple de anatomie |
| Export JSON Schema (schema R-003) | ⬜ | 🟢 | schemă proprie |

## Nivelul 2 — Bazele autorării și ieșiri deschise (zile)
| Funcționalitate | Stare | IP | Bază sigură / condiție |
|---|---|---|---|
| APNG, sprite sheet PNG+JSON | ✅ | 🟢 | specificații PNG/APNG |
| WebM VP9 / AV1 cu alfa (prin ffmpeg-ul tău) | ✅ | 🟢 | codecuri specificate deschis; starea brevetelor neevaluată |
| Export OCA (Open Cel Animation) | ✅ | 🟢 | specificație publică; import ⬜ |
| Import OpenRaster (.ora) | ✅ | 🟢 | specificație deschisă, limite anti zip-bomb |
| MP4 / H.264 | ✅ | 🟠 | doar prin ffmpeg-ul *propriu al utilizatorului*; nu include niciodată un encoder |
| Deplasare/scalare/inversare/copiere/lipire de chei, easing în lot, decalaj de timp per canal, marcaje și regiuni, timecode SMPTE (R-010, R-011, R-044) | ✅ | 🟢 | editare generică |
| Instantanee de revizie cu curățare și recuperare după blocare (R-014); compoziții de straturi (R-015); evenimente de timeline cu sarcini utile (R-045); reparentare cu păstrarea poziției de repaus (parte din R-013) | ✅ | 🟢 | editare generică |
| Aliniere/distribuire, strat de referință, etichete shy/solo/lock (R-012, R-013, R-016) | ✅ | 🟢 | editare generică |
| Onion skin, urme de mișcare (suprapunerile nu se exportă niciodată) (R-020) | ✅ | 🟢 | design propriu |
| Lanț de post-efecte: blur, umbră, glow, contur, posterizare, pixelare, niveluri/gamma, fade (parte din R-022) | ✅ | 🟢 | procesare de imagine de manual |
| Trim paths, filtre per strat, măști, adâncime de câmp (R-021, R-024, R-025, R-034) | ✅ (trim paths precalculate; limitele efectelor de strat: vezi ROADMAP §9) | 🟢 | matematică standard de imagine |
| Import SVG (căi, forme, grupuri, transformări, vopsele unicolore; gradienții/textul/clip-ul sunt raportate ca avertismente) (R-032) | ✅ | 🟢 | gramatica W3C SVG 1.1 |
| Import BVH (o axă de rotație → unghiuri de os, reducere limitată a cheilor) (R-036) | ✅ | 🟢 | format text deschis |
| Export container dotLottie (R-03A) | ✅ | 🟢 | specificație publică de container (zip + manifest) |
| Export SVG animat, import de secvență de imagini, ieșire OpenEXR/16 biți, export glTF (R-032, R-037…R-039) | 🟡 (glTF: forme din cadrul 0 plus oase și ponderi morph, vezi ROADMAP R-039) | 🟢 | formate publice |
| Import Papagayo (Moho-switch) și indicii de gură Rhubarb (R-03B) | ✅ | 🟢 | se citește doar ieșirea lor text simplă |

## Nivelul 3 — Profunzime de rig și animație (săptămâni)
| Funcționalitate | Stare | IP | Bază sigură / condiție |
|---|---|---|---|
| Arcuri (precalculate), lattice-uri (FFD), deformator de coloană, straturi switch, particule, lichid, operații booleene | ✅ | 🟢 | de manual |
| Lip-sync audio → forme de gură, generator de ciclu de mers, oglindire, retime, clip, crossfade | ✅ | 🟢 | design propriu |
| **Pin warp** (înlocuiește warp-ul de tip cage / ARAP) | ✅ | 🟢 | RBF Wendland 1995 |
| **Greutăți de skin automate prin difuzie termică** | ✅ | 🟢 | ecuația Laplace |
| **Înregistrare de performanță** cu reducere limitată a cheilor | ✅ | 🟢 | Douglas–Peucker |
| **Mapare de intrare** (semnale CSV/JSON → parametri) | ✅ | 🟢 | fără tracker inclus |
| IK soft/stretch (R-051), drivere de poză: slider + joystick 2-D (R-050), schimbare de vedere condusă de unghi (R-056), auto-rig pe baza etichetelor (R-058) | ✅ | 🟢 | rigging generic; matematică de manual, design propriu |
| Constrângeri de cale, IK cu blocare a piciorului, chei de comutare a părintelui, skin-uri, magneți (R-040…R-049, R-052…R-055, R-057) | 🟡 (construite: R-040…R-043, R-044, R-047, R-048, R-052…R-055, R-057; vezi ROADMAP secțiunea 9 pentru părțile marcate NOT done) | 🟢 | rigging generic |
| Generatoare idle (clipit, respirație, legănare) și lanțuri follow-through (R-047, R-048) | ✅ | 🟢 | cu seed, deterministe |
| Vânt, gravitație, coliziuni, benzi de păr/stofă (R-070…R-072) | 🟡 (R-070, R-071 gata; benzile R-072 sunt doar oase, stofa este o foaie plană) | 🟢 | manuale mass-spring / PBD |

## Nivelul 4 — Instrumente inteligente (săptămâni–luni)
| Funcționalitate | Stare | IP | Bază sigură / condiție |
|---|---|---|---|
| **Trasare imagine → vector** (înlocuiește auto-trace-ul ML) | ✅ | 🟢 | marching squares + RDP |
| **Mașină de stări bazată pe jurnal de evenimente** + bake în canale (înlocuiește clonele de mașini de stări de runtime) | ✅ | 🟢 | semantică proprie — păstrează-ne denumirile |
| **Retargeting după atingerea efectorului** | ✅ | 🟢 | metodă proprie |
| **Solver de potrivire a pozei** (strategie evolutivă, conștient de limitele articulațiilor) | ✅ | 🟢 | Beyer & Schwefel |
| **Verificare de flash / flash roșu WCAG 2.3.1 + raport de mișcare a camerei + geamăn cu mișcare redusă** | ✅ | 🟠 | doar definiții publice; spune „ajutor de inginerie", niciodată „certificat" |
| **Rig linter**, **contracte de mișcare**, **solver de buclă fără cusătură**, **antrenor de principii** (I-02, I-04, I-05, I-14) | ✅ | 🟢 | idei proprii |
| **Potrivire de arc prin desenarea overshoot-ului** (I-08) — potrivește frecvența/amortizarea cu integratorul propriu al motorului | ✅ | 🟢 | idee proprie |
| **Explain-this-pixel** pentru oase și straturi (I-07) | ✅ | 🟢 | idee proprie |
| Înainte | după | imagine de revizuire a diferenței (primul pas din I-06; metrica perceptuală este `ci diff`, SSIM și CIEDE2000) | ✅ | 🟢 | idee proprie |
| Iluminare cu normal-map, parallax, adâncime de câmp (R-064, R-065, R-024) | ✅ | 🟢 | de manual |
| Lip-sync aliniat la transcriere (R-081) | ✅ (aliniator propriu, `speech.rs`) | 🟠 | folosește un aliniator cu licență permisivă; verifică licența modelului său |
| Urmărire facială/corporală prin webcam care alimentează input mapper-ul (R-082) | 🟡 (detector propriu antrenat doar pe fețe sintetice; webcam-uri reale și fotografii netestate) | 🟠 | utilizatorul își aduce un tracker permisiv; nu include ponderi de model decât dacă licența permite |
| Retargeting complet de mocap BVH cu potrivire automată pe ierarhie (R-083) | ✅ | 🟢 | se bazează pe importul BVH |
| Import PSD (R-035) | ⬜ | 🟠 | doar citire, specificație publicată, fără mărci Adobe; sau folosește pur și simplu ORA |

## Nivelul 5 — Platformă (luni)
| Funcționalitate | Stare | IP | Bază sigură / condiție |
|---|---|---|---|
| Studio în browser cu anulare/refacere | 🟡 | 🟢 | UI propriu; fără copierea aspectului sau denumirilor furnizorilor |
| Runtime WASM, API Rust stabil + C ABI, scripting în sandbox (R-090…R-093) | 🟡 (R-091, R-092, R-093 gata; la R-090 modulul wasm se construiește și trece testul wrapper-ului sub Node în CI, dar nu a fost niciodată rulat într-un browser) | 🟢 | API propriu |
| **Server agent / MCP** peste operații de patch: inspectare, editare (atomică, cu anulare), lint, siguranță, randare, salvare doar în propriul fișier (R-094) | ✅ | 🟢 | protocol JSON-RPC deschis |
| GUI complet: gizmo-uri, timeline, editor de grafice (R-100…R-103) | 🟡 (R-101, R-102 gata; paginile R-100 și R-103 există, dar autorul nu le-a rulat niciodată într-un browser real) | 🟢 | widget-uri generice; aspect propriu |
| Control de versiuni al animației cu merge semantic, diff perceptual CI, explain-this-pixel (I-01, I-06, I-07) | ✅ (I-01 `osatura vcs`, I-06 `osatura ci`, I-07 `explain`) | 🟢 | idei proprii |
| Pose fuzzing cu hărți termice de stres, export cu țintă de dimensiune, nivel de detaliu determinist cu o limită de eroare declarată (I-03, I-10, I-11) | 🟡 (toate trei construite; limita I-11 nu acoperă efectele, căile cu skin, valorile conduse de IK sau pozele mașinii de stări) | 🟢 | idei proprii, matematică de manual |

## Nivelul 6 — Frontieră (nivel de cercetare)
| Funcționalitate | Stare | IP | Bază sigură / condiție |
|---|---|---|---|
| Randare GPU cu teste de paritate cu CPU (R-110) | 🟡 (crate-ul `osatura-gpu` construit și testat; rulare pe hardware limitată, vezi ROADMAP) | 🟢 | wgpu (permisiv) |
| Editare multi-utilizator în timp real peste jurnalul de operații (R-111) | 🟡 (gata, cu un jurnal de sesiune persistent opțional `--collab-log`; participanții și istoricul de anulare nu sunt persistați; HTTP simplu, token partajat) | 🟢 | literatura CRDT |
| Runtime de data-binding / layout (R-091, R-095) | ✅ (design propriu; verificarea brevetelor nefăcută, docs/PATENT_NOTES.md) | 🟠 | verifică brevetele înainte de livrare |
| Instrumente asistate de ML (R-112) | 🟡 | 🟠 | doar metode clasice, fără ponderi de model: `autotrace` (k-means, potrivire Bézier Schneider, mod linie Zhang-Suen) și sugestii învățate din pozele tale (PCA, kernel-uri, MLP mic opțional într-un format propriu de ponderi); fără deep learning |

## De nu construit niciodată (🔴)
- Citirea sau scrierea fișierelor de proiect native **Live2D, Spine, Moho, Harmony, Rive, Cartoon Animator** (proprietare; expunere EULA/inginerie inversă) — folosește în schimb Lottie, OCA, ORA, APNG, SVG.
- **Solvere rapide de deformare a graficii de tip ARAP**, așa cum sunt descrise în brevetele Adobe — pin warp, FFD și skinning acoperă cazul de utilizare.
- Copierea **aspectului UI, a pictogramelor, denumirilor, conținutului de probă sau a numerelor magice/terminologiei formatelor de fișier** ale oricărui furnizor.
- Imitarea instrumentului de fotosensibilitate **Harding** sau pretinderea conformității pentru difuzare.
- Includerea de encodere **H.264/H.265/AAC**.

## Condiții valabile pentru fiecare rând 🟢
Jurnal de proveniență păstrat (`CLEANROOM.md`; implementare independentă, nu clean-room strict); doar dependențe permisive (regenerează cu `scripts/licence-audit.sh`; în prezent 363 de crate-uri terțe pentru toate platformele, toate de clasă MIT / Apache-2.0 / BSD / Zlib / Unlicense, cu excepția a șase crate-uri MPL-2.0 (smartstring prin funcționalitatea `scripting`, activă implicit, și cinci prin aplicația cu fereastră `osatura-gui`); vezi docs/THIRD_PARTY.md); nicio ilustrație sau audio terță livrată fără propriul fișier de licență; numele **OSATURA** verificat într-o căutare de mărci comerciale înainte de lansare.


# Tabele detaliate de capabilități
## Desen și forme
| Funcționalitate | Stare | Note |
|---|---|---|
| Căi Bézier cubice, deschise/închise, mânere | ✅ | |
| Umplere: unicolor, gradient liniar și radial | ✅ | gradienții urmează transformarea stratului |
| Contur: lățime, capete, îmbinări, linii întrerupte | ✅ | |
| Contur cu lățime variabilă (profil de lățime) | ✅ | contur poligonal; căi închise → două inele |
| Netezire Catmull-Rom, ajutoare pentru dreptunghi/cerc/elipsă/poligon | ✅ | |
| Operații booleene pe poligoane (reuniune/intersecție/diferență/xor) | ✅ | strat `boolean`; găurile sunt unite printr-o punte într-un singur poligon; rezultatul este poligonal (aplatizat) |
| Straturi metaball / lichid | ✅ | marching squares, oasele pot conduce blob-uri |
| Straturi de imagine (PNG), încorporate sau fișier | ✅ | |
| Pensule bitmap/vectoriale, umpleri cu textură, text/fonturi | ❌ | |
| Desen liber, instrumente de editare a punctelor, trasare de imagine | ❌ | fără GUI de desen |

## Rigging
| Funcționalitate | Stare | Note |
|---|---|---|
| Ierarhie de oase, FK, poziție de repaus | ✅ | |
| Limite de unghi / scară | ✅ | |
| IK: analitic cu 2 oase (îndoire ±) | ✅ | |
| IK: lanțuri de N oase (FABRIK) | ✅ | |
| Mix FK↔IK (animabil) | ✅ | |
| Constrângeri: aim, copy-rotation, pin (pondere animabilă) | ✅ | |
| Skinning cu amestec liniar al ancorelor + mânerelor, auto-skin | ✅ | |
| Straturi atașate la oase, părinți de straturi / grupuri | ✅ | |
| Mișcare secundară: arcuri amortizate precalculate | ✅ | determinist, acces aleatoriu (precalculat la încărcare) |
| Canale conduse de poză („driven keys") | 🟡 | o curbă mapează orice proprietate pe alta; fără UI/acțiuni dedicate pentru smart-bone |
| Fizică de corp rigid, stofă, corp moale | ❌ | |

## Deformare
| Funcționalitate | Stare | Note |
|---|---|---|
| Decalaje per punct (animabile) | ✅ | adresate prin `layer::path::index` |
| Ținte morph / blend shapes (ponderi aditive) | ✅ | |
| Lattice de formă liberă (FFD Bernstein) | ✅ | puncte de control animabile |
| Deformator de îndoire pe coloană / curbă | ✅ | rază moale opțională |
| Unduire „boil" (tremur desenat de mână) | ✅ | zgomot coerent, determinist |
| Warp de mesh de imagine cu ponderi de os | ✅ | mesh de tip grilă; fără editor de mesh |

## Animație
| Funcționalitate | Stare | Note |
|---|---|---|
| Chei + easing (linear/step/ease/cubic-bezier) | ✅ | |
| Extrapolare: hold/linear/loop/ping-pong/loop-offset | ✅ | |
| Canale de expresii | ✅ | limbaj mic și sigur de expresii; poate citi alte canale |
| Canale de zgomot, mod de amestec add/multiply/replace, ponderi | ✅ | |
| Straturi switch + expunere (cadru cu cadru, schimburi de gură) | ✅ | |
| Retime/decalaj de clip, retarget, oglindire L↔R, bake de crossfade | ✅ | funcții + CLI `mirror`; fără UI NLA |
| Ciclu de mers procedural | ✅ | generator sinusoidal simplu, nu un instrument profesional de mers |
| Lip-sync audio din WAV | 🟡 | euristică de volum/luminozitate → 5 forme de gură, **nu** recunoaștere de foneme |
| Emițător determinist de particule | ✅ | formă închisă: orice cadru este evaluat direct |
| Onion skin, urme de mișcare | ✅ | CLI `onion` și `trail`, suprapuneri în studio; nu se exportă niciodată |
| Redare audio / scrubbing pe forma de undă | 🟡 | comanda `waveform` și redare audio în studio; nu se pretinde un UI de scrubbing pe forma de undă |

## Cameră și compoziție
| Funcționalitate | Stare | Note |
|---|---|---|
| Cameră 2D pan/zoom/rotire (animabilă) | ✅ | |
| Parallax multiplan per strat de nivel superior | ✅ | |
| Opacitate, moduri de amestec, măști alfa (+inversare) | ✅ | |
| Straturi / cameră 3D reale, stereo | ❌ | |

## Ieșire și interschimb
| Funcționalitate | Stare | Note |
|---|---|---|
| Cadre SVG | ✅ | |
| Cadre PNG (rasterizare tiny-skia) | ✅ | |
| GIF animat | ✅ | |
| MP4 | ✅ | necesită `ffmpeg` în PATH |
| Player HTML autonom | ✅ | toate cadrele inline, scrubber |
| Lottie JSON | 🟡 | geometrie precalculată per cadru; gradienți → prima culoare; fără măști/moduri de amestec/imagini; **nevalidat într-un player Lottie real** |
| Import de fișiere ale altor instrumente, PSD, FBX etc. | ❌ | (deliberat: fără formate proprietare) |
| API de scripting / plugin | 🟡 | `script` Rhai în sandbox prin cereri validate, plus expresii; fără API de plugin |

## Editare
| Funcționalitate | Stare | Note |
|---|---|---|
| Model de editare JSON-patch cu anulare/refacere | ✅ | fiecare editare validată; editările invalide sunt anulate |
| Ajutoare pentru setare/ștergere de chei | ✅ | |
| Studio web (browser) | 🟡 | previzualizare, scrubber, setare de chei pe unghiul fiecărui os, editor JSON, anulare/refacere, salvare, exporturi. **Fără desen, fără editare de puncte/oase pe canvas, fără editor de grafice.** |

## Lucruri care sunt „îmbunătățiri" modeste față de linia de bază din manual
(nu se pretinde că lipsesc în altă parte)
- Fiecare editare de proiect este un patch JSON validat și inversabil → anularea nu poate corupe niciodată un proiect.
- Toată simularea (arcuri, particule) este determinist cu acces aleatoriu: derulezi sau randezi orice cadru în orice ordine și obții pixeli identici.
- Orice proprietate — inclusiv punctele de control ale deformatoarelor și ponderile morph — este adresabilă printr-o singură cale `layer::sub::id`, astfel că expresiile și curbele conduse pot lega orice de orice.
- Validarea raportează *toate* problemele într-o singură trecere, cu numele celor vinovate.
- Un singur motor → SVG, raster, GIF, MP4, HTML, Lottie; același `evaluate(frame)` alimentează toate.

## Teste
`cargo test` — 142 de teste unitare/de integrare (matematică, track-uri, expresii, solvere de rig, deformatoare,
pixeli de randare, exportatoare, anulare/refacere în editor, audio, particule, operații booleene, metaballs).

---

# Listă de verificare față de produsul de referință, versiune cu versiune

Referință: Moho (fost Anime Studio), 1999 → 14.5 „SHOWTIME" (12 oct. 2026). Listele de funcționalități provin
din paginile de lansare ale furnizorului și din presa de specialitate (vezi Surse). Lacune în evidență: pentru Anime Studio 6–8,
12.5 și 14.1 nu am găsit o listă publică detaliată, deci sunt acoperite doar prin
istoricul general. Starea este cea a OSATURA: ✅ există echivalent · 🟡 parțial / abordare diferită · ❌ lipsește.

## Versiuni timpurii (1999 – Anime Studio 8, doar istoric general)
| Funcționalitate | OSATURA |
|---|---|
| Particule | ✅ strat emițător determinist |
| Măști de strat | ✅ măști alfa + inversare |
| Onion skin-uri | ✅ (CLI `onion`, suprapunere în studio) |
| Constrângeri de os | ✅ aim / copy-rotation / pin, limite de unghi și scară |
| Scripting Lua | 🟡 (Rhai, nu Lua; plus canale de expresii) |
| Lip-sync | 🟡 euristică de amplitudine din WAV |
| Fizică | 🟡 arcuri + gravitație de particule; fără corpuri rigide |
| Creare / import 3D | ❌ |
| Import Photoshop | ❌ |
| Suport Wacom | ❌ (fără GUI de desen) |
| Smart bones, IK | 🟡 canale conduse / ✅ 2 oase + FABRIK |

## Anime Studio 9 (2013)
| Funcționalitate | OSATURA |
|---|---|
| 64 de biți, accelerare GPU | 🟡 Rust nativ pe 64 de biți; doar rasterizator CPU (fără GPU) |
| Smart Bones (cadrane de control al îndoirii articulațiilor) | 🟡 canalele conduse mapează orice valoare pe oricare alta; fără UI/acțiuni de tip cadran |
| Grafice de mișcare editabile | ❌ (curbele sunt date cu easing cubic-bezier; fără editor de grafice) |
| Timeline reproiectat | ❌ (studioul are doar un scrubber + marcaje de chei) |
| Multi-touch Wacom | ❌ |

## Anime Studio 10 (2014)
| Funcționalitate | OSATURA |
|---|---|
| Constrângeri de unghi independente (globale) | 🟡 copy-rotation către un os de control o aproximează |
| Squash & stretch | 🟡 scara osului este animabilă; fără squash/stretch automat |
| Cadrane / configurare Smart Bone | 🟡 vezi mai sus |
| Interpolare bounce / elastic / stagger | 🟡 bounce, spring (de tip elastic), back, cubic-bezier ✅; stagger ❌ |
| Găleată de vopsea, pensulă blob, radieră, reducere de puncte | ❌ |
| Personaje preîncărcate / conținut de bibliotecă | ❌ |
| Import de obiecte 3D | ❌ |

## Anime Studio 11 (2015)
| Funcționalitate | OSATURA |
|---|---|
| Animație cadru cu cadru | ✅ straturi switch + canale de expunere |
| Referențiere de straturi (duplicate legate) | ❌ |
| Ordonare animată a formelor | ❌ (ordinea de desenare este fixă per strat; straturile switch pot schimba) |
| Curbe de mișcare editabile | 🟡 datele de curbă da, fără UI de curbe |
| Import PSD (inclusiv actualizare live) | ❌ |
| Export FBX / Unity | ❌ |
| Import/înregistrare audio + lip-sync automat | 🟡 import WAV + lip-sync euristic; fără înregistrare |
| Îmbinare de contururi, instrumentul Color Points | ❌ |

## Moho 12 (2016) și 12.x
| Funcționalitate | OSATURA |
|---|---|
| Mânere Bézier | ✅ |
| Instrumente freehand îmbunătățite | ❌ |
| Smart Warp (mesh-uri personalizate) | 🟡 mesh de imagine tip grilă cu ponderi de os; deformatoare lattice / coloană pentru vectori; fără editor de mesh |
| Motion blur | ❌ |
| Oase pin | 🟡 constrângere pin, nu oase pin într-un singur punct |
| Import/export de acțiuni între rig-uri similare | 🟡 funcțiile `clip`, `retarget`, `mirror` pe seturi de canale; fără UI de bibliotecă de acțiuni |
| Animarea mai multor straturi simultan | ✅ orice număr de canale per timeline |
| UI reproiectat, ancorare de oase, auto-freeze al poziției | ❌ |

## Moho 13 (2019)
| Funcționalitate | OSATURA |
|---|---|
| Bitmap cadru cu cadru + onion skin | 🟡 straturi de imagine PNG în interiorul straturilor switch; fără onion skin, fără desen bitmap |
| Instrumente de desen bitmap / pensule personalizate | ❌ |
| Instrumente de storyboard și animatic | ❌ |
| Fereastra Actions | ❌ (doar clipuri la nivel de date) |
| Lumini 3D, materiale, toon shading | ❌ |
| Mai multe presetări de ieșire | 🟡 formate de export CLI, fără manager de presetări |
| Personaje incluse | ❌ |

## Moho 13.5 (2021) și 13.5.x
| Funcționalitate | OSATURA |
|---|---|
| Vitruvian Bones (schimbă desenele *și* seturile de oase) | 🟡 straturile switch schimbă desenele; fără schimbarea seturilor de oase |
| Quad mesh-uri (perspectivă) | 🟡 mesh-ul grilă + lattice dau warp-uri de formă liberă; fără quad proiectiv |
| Dinamica vântului (forță, direcție, turbulență) | ❌ (arcurile nu au vânt; canalele de zgomot îl pot simula) |
| Keyframing relativ | ❌ |
| Variante de visem de lip-sync alese aleatoriu | ❌ (maparea poate uni forme, fără variante aleatorii) |
| Export audio ca WAV | ❌ |
| Copiere/lipire de chei în profunzime, ajustare pivot, marcaje de timeline | ❌ (fără GUI) |
| Licențe flotante | n/a |

## Moho 14 (2023)
| Funcționalitate | OSATURA |
|---|---|
| Motor grafic nou, efecte în timp real (blur, umbră, zgomot) | ❌ fără filtre de blur/umbră/zgomot; moduri de amestec ✅ |
| Liquid Shapes (îmbinare/scădere/intersecție/amestec) | 🟡 lichid metaball ✅ + operații booleene pe poligoane ✅; nu același model de formă |
| Curvers / Compressible Curvers | 🟡 deformatorul de îndoire pe coloană este cel mai apropiat analog |
| Import SVG îmbunătățit | ❌ (doar **export** SVG) |
| Îmbunătățiri freehand, stiluri noi, îmbinarea formelor | ❌ / ❌ / 🟡 (operații booleene) |
| Multi-Stroke Fill, Connect & Create | ❌ |
| Smart Line Boil, opțiuni de pensulă | 🟡 unduire „boil" cu rată reglabilă ✅; opțiuni de pensulă ❌ |
| Culori de puncte, Add Point îmbunătățit | ❌ |
| Dynamic Bones (+ gravitație) | 🟡 oase cu arc amortizat ✅; fără termen de gravitație |
| Constrângeri cu întârziere | 🟡 arcurile dau o întârziere față de o țintă; fără constrângere explicită de întârziere |
| Opțiuni noi de timeline | ❌ |
| Îmbunătățiri cadru cu cadru | 🟡 |
| Actualizarea interfeței de scripting (Lua 5.4) | ❌ |

## Moho 14.1 – 14.2 (2023-11, 2024-06)
| Funcționalitate | OSATURA |
|---|---|
| (14.1: nu s-au găsit detalii publice) | — |
| Decalaj animat al profilului de curbă | ❌ (profilurile de lățime sunt statice) |
| Smart boil pentru zgomot vectorial | 🟡 unduirea pulsează la o rată dată; fără mod „doar în mișcare" |
| Instrument Transform Bone refăcut | ❌ (fără instrumente pe canvas) |
| Afișare pixelată | ❌ |
| Mod de amestec Colour Burn | ✅ |
| Spațiere minimă mai mică a pensulei, straturi PSD decupate | ❌ |
| Numerotarea timeline-ului pe baza ratei de cadre | ❌ |
| Opțiunea de afișare „Turn off masked layers" | ❌ |
| Lip-sync mai bun la rate de cadre mici | 🟡 |
| Nume multilingve pentru oase/straturi/marcaje | ✅ (nume UTF-8) |
| Anti-aliasing al măștii Exclude-Strokes, onion skin-uri colorate pe imagini | ❌ |
| Smart Bones care controlează dinamica oaselor | ❌ (setările arcurilor nu sunt animabile) |
| Scripturi Lua la pornire; flag de consolă Windows | ❌ / n/a |
| Export ProRes cu alfa | ❌ (doar GIF/PNG/MP4) |

## Moho 14.3 (2024-12)
| Funcționalitate | OSATURA |
|---|---|
| Opacitate de pensulă per punct (condusă de presiune) | ❌ |
| Derivă de culoare după presiunea creionului | ❌ |
| Controale ale instrumentului freehand/lățime de linie | ❌ |

## Moho 14.4 „SIDE QUEST" (2025-11)
| Funcționalitate | OSATURA |
|---|---|
| Export către Unreal / Unity / Godot | ❌ |
| Export glTF către Blender / instrumente 3D | 🟡 (`.glb`: forme din cadrul 0, oase, ponderi morph) |
| Smart Bones la 360° | ❌ |
| Create Style from Shape | ❌ |
| Opțiuni de particule (replicare de-a lungul curbei, întârziere, emisie ordonată, distribuție) | 🟡 întârziere de start ✅, emisie ordonată determinist ✅; replicare de-a lungul curbei ❌ |
| Layer Selector, cădere Magnet, import PSD, netezire Freehand, completări Lua | ❌ |
| Creare mai ușoară de cicluri | ✅ extrapolare loop + generator de ciclu de mers |

## Moho 14.5 „SHOWTIME" (2026-10-12)
| Funcționalitate | OSATURA |
|---|---|
| Live Playback (webcam / gamepad / microfon, webcam virtual, OBS) | ❌ |
| Urmărire facială → oase / Smart Bones / mesh | 🟡 (detector antrenat sintetic, `face-bake`; nevalidat pe fețe reale) |
| Înregistrare live pe timeline, control de personaje multiple | ❌ |
| Urmărire video (secvențe de imagini) | ❌ |
| Algoritmi freehand, linii drepte, tăierea cozilor | ❌ |
| Găleată de vopsea refăcută (Vector / Trace / Recolor) | ❌ |
| Mișcare inversă Magnet, selecție Select Points pe formă | ❌ |
| Categorii de pensule personalizate | ❌ |
| Culoare/opacitate onion skin, Shift & Trace, onion skin-uri magnetice | ❌ |
| Filtre / selecție multiplă în fereastra Layers | ❌ |
| Performanță: anulare/refacere mai rapidă, fișiere mai ușoare | 🟡 anularea este o inversă ieftină de JSON-patch; fișierele sunt JSON simplu |
| Export MP4/H.264 | ✅ prin ffmpeg |
| Profiluri de culoare PNG, import/export EXR | ❌ |
| Exportator în lot, flag-uri CLI `-exr` / `-pngprofile` | 🟡 CLI `frames` / `export`, fără coadă de lot |
| Rată de cadre cu 3 zecimale (de ex. 23.976) | ✅ |
| Exportul SVG gestionează contururile cu lățime variabilă și formele lichide | ✅ (contururile devin contururi închise, lichidele devin căi) |
| Liquid hide-edge, forme lichide doar cu contur, expunere de contur | ❌ |
| Linux prin Wine/Proton | ✅ Rust nativ (Linux/macOS/Windows); arhive prefabricate pentru Windows x86-64, macOS arm64 și Linux x86-64 în pre-lansarea ciornă; testat de CI pe toate trei, de o persoană doar pe Linux |

## Sinteză (doar produsul de referință; concurenții urmează mai jos)
Din funcționalitățile documentate ale produsului de referință, OSATURA acoperă nucleul de **rigging, deformare,
animație procedurală, matematică de compoziție și export** (aproximativ: schelet/IK/constrângeri,
deformare de mesh și lattice, chei și expresii, măști/amestecuri/gradienți, particule,
metaballs/operații booleene, GIF/MP4/HTML/Lottie), dar nu are **instrumente de desen, intrare live, 3D,
I/O PSD/FBX** (la momentul primului sondaj; de atunci au fost adăugate export glTF, ieșire EXR, scripting Rhai, un studio cu grafice/timeline, onion skin și efecte de blur, vezi `ROADMAP.md` secțiunea 9).
Acestea sunt blocurile mari, în mare parte legate de GUI, ale produsului și reprezintă lacuna sinceră.

## Surse
- Anunțul Moho 14.5 SHOWTIME, moho.lostmarble.com (blog, 12 oct. 2026)
- Pagina de funcționalități Moho Pro, mohoanimation.com/pages/features
- Appcast Moho 14.4 SIDE QUEST, lostmarble.com/appcasts/Moho14.4Update.html
- Moho 14.3: Digital Production, 27 dec. 2024
- Postarea de lansare Moho 14.2, moho.lostmarble.com; lista de tutoriale „Moho 14: What is new?"
- Funcționalități noi Moho Pro 13.5, moho.lostmarble.com; Lester Banks (apr. 2021); note 13.5.3
- Moho 13: Lester Banks (iun. 2019)
- Moho Pro 12: 80.lv; Digital Production (nov. 2016)
- Anime Studio 11: Architosh (mai 2015); 10: Architosh (mar. 2014); 9: Renderosity
- Datele versiunilor: mohoscripting.com/moho_versions; istoric general: Wikipedia „Moho (software)"

---

# Addendum: istoricul Moho / Anime Studio, versiunile 1 – 11 (completat)

Sursa: PDF-ul cu istoricul versiunilor găzduit de furnizor (lostmarble.com, „Anime Studio history", 2015) plus
paginile de presă pentru Anime Studio 7 și 8. Înlocuiește rândurile „doar istoric general" din partea de sus a
listei de verificare a produsului de referință. Nici aici nu s-au găsit: detalii despre 8.x, 10.1.x, 11.1, 12.5, 14.1.

| Versiune | Funcționalități documentate | OSATURA |
|---|---|---|
| Moho 1 (1999) | BeOS, port pentru Windows, ieșire Flash | ❌ Flash/SWF (învechit) |
| 2.x | I/O QuickTime, randare până la 4096², umbrire de strat, straturi switch, mostre de culoare în stiluri, legare flexibilă a oaselor, import Illustrator îmbunătățit | ✅ straturi switch, 🟡 legare pe ponderi, ❌ import Illustrator, ✅ scări raster mari (≤16×) |
| 3.x | Particule, înfășurare de imagine, măști de strat, onion skin-uri, cameră virtuală cu adâncime, warp de imagine, straturi switch îmbunătățite, suport pentru tabletă, timecode SMPTE, ascunderea graficii din afara cadrului, măști reversibile, randomizator al grosimii liniei, zone sigure, shear de strat | ✅ particule, măști (+inversare), cameră, warp de mesh de imagine, înclinare de strat; ❌ onion skin, tabletă, timecode, zone sigure, randomizator de grosime |
| 4.x | Curbe grafice de animație, constrângeri de os, randare în lot, audio, parametri animabili, ciclare de animație, scalare de chei, proiecte redimensionabile, adâncime de particule, mulțimi randomizate, legare flexibilă automată, versiune pentru Linux | ✅ constrângeri, cicluri (extrapolare loop), parametri animabili, proiecte redimensionabile, retime `clip` ≈ scalare de chei, Linux; 🟡 mulțimi (particule), 🟡 audio (doar lip-sync); ❌ UI de editor de curbe |
| 5.x | UI nou, export SWF, straturi 3D și de notă, acțiuni, stiluri, scripting Lua | ❌ straturi 3D / de notă / Lua; 🟡 acțiuni (funcții de clip); ✅ stilurile sunt vopsea per cale |
| Anime Studio 6 | Urmărire de mișcare, lip-sync integrat, pensulă scatter, Lua 5.1, auto shading, **ținte morph**, sequencer, video HD | ✅ ținte morph; 🟡 lip-sync; ❌ urmărire de mișcare, pensulă scatter, auto shading, sequencer |
| 6.1 / 6.2 | Încărcare pe Facebook, import GIF transparent | ❌ / 🟡 export GIF ✅, import ❌ |
| Anime Studio 7 | Fizică simulată (gravitație, densitate, frecare, elasticitate), design de forme 3D, înregistrare audio, bibliotecă de conținut, import de scene Poser, **instrument follow path**, gather media | 🟡 arcuri/gravitație de particule; ❌ fizică rigidă, 3D, înregistrare, bibliotecă, Poser, follow-path |
| Anime Studio 8 | Trasare automată de imagine, Character Wizard, modul pentru începători, conexiune media în timp real, stiluri globale de randare, rigging avansat de oase | ❌ toate (fără trasare, wizard, legătură live, stiluri globale) |
| Anime Studio 9 / 9.5 | Grafice de mișcare editabile, Smart Bones, **layer comps**, onion skin-uri îmbunătățite, 64 de biți + GPU, keyframing îmbunătățit, timeline colorat; 9.5: îmbunătățiri Smart Bone, depth shifting, controale de straturi imbricate, flexi-binding, îmbunătățiri ale straturilor switch | vezi tabelul AS 9; ❌ layer comps, depth shifting |
| Anime Studio 10 / 10.1 | Constrângeri noi, configurare Smart Bone, bounce/elastic/stagger, proces de randare separat, instrumente combinate de oase+puncte, previzualizare de animație, multi-document, editor de scurtături, selecție multi-formă, cadrul zero, ascunderea punctelor/oaselor, editare multi-strat, **curbe cu lățime variabilă**, transparența texturii, sursă de particule reglabilă, adâncime de câmp, blur fracționar; 10.1: auto image trace, stiluri globale, straturi patch, import PSD, media în timp real, Character Wizard | ✅ contururi cu lățime variabilă, ✅ animație multi-strat; 🟡 adâncime de câmp ❌ blur ❌; ❌ straturi patch, restul ca mai sus |
| Anime Studio 11 | Cadru cu cadru, referențiere de straturi, ordonare animată a formelor, **ținte de os animate**, **părinți de os animați**, răsturnarea oaselor, grupare de straturi de selecție, scară normalizată a stratului, export în lot, format de fișier nou | ✅ FbF, ✅ țintă IK animată (ținta este doar un os), ❌ *reparentare* animată, 🟡 răsturnarea oaselor (`mirror`), ✅ lot prin CLI, ✅ format de fișier JSON simplu |

**Lacune noi apărute aici** (de adăugat în roadmap): layer comps (stări salvate de vizibilitate a straturilor),
reparentare animată a oaselor, straturi patch, depth shifting, straturi follow-path, suprapuneri de zone sigure,
timecode SMPTE, auto shading, pensulă scatter, auto-trace de imagine.

---

# Funcționalități ale concurenților neacoperite de lista de verificare de mai sus

Produse analizate: Toon Boom Harmony 24, Spine 4.2, Live2D Cubism 5.x, Cartoon Animator 5.3, Rive,
Synfig Studio, OpenToonz, TVPaint Animation 12, Adobe Animate, Cavalry, Procreate Dreams, Duik (After
Effects), Blender Grease Pencil. Elementele de mai jos sunt cele **care nu apar deja** în lista de verificare
a produsului de referință. Calitatea surselor variază: majoritatea rândurilor provin din paginile de funcționalități/lansare
ale furnizorilor, pe care le-am preluat; rândurile marcate † provin din cunoștințe generale sau doar dintr-un titlu/cuprins și nu au fost confirmate pe o pagină.
Starea este cea a OSATURA: ✅ · 🟡 · ❌.

## Rigging, constrângeri, deformare
| Funcționalitate | Văzută în | OSATURA |
|---|---|---|
| Instrumente pentru ponderi: sudarea ponderilor între mesh-uri, copiere/lipire de ponderi între vârfuri | Spine 4.2 | ❌ (doar auto-skin, fără pictare) |
| Constrângeri de fizică a oaselor (inerție, vânt, gravitație, mix) | Spine 4.2 | 🟡 arcuri amortizate (frecvență/amortizare); fără vânt/gravitație/inerție |
| Constrângere de cale: oasele urmează o spline Bézier compusă | Spine, Rive (Follow Path) | ❌ |
| Mixuri de constrângeri, duplicare împreună cu oasele, editare multiplă, foldere | Spine 4.2 | 🟡 mixul IK și ponderea constrângerii sunt animabile; fără foldere/editare multiplă |
| Skin-uri (schimbarea seturilor de atașamente pe un schelet) | Spine, Rive solos | 🟡 straturi switch |
| „Instrument de poză" IK multi-os (tragi capătul unui membru) | Spine | ❌ (necesită GUI cu canvas) |
| Atașamente bounding-box / hit-test | Spine | ❌ (nu sunt necesare pentru ieșirea offline) |
| Free-Form Deformation, ponderi de mesh | Spine, Harmony | ✅ lattice + mesh de imagine cu ponderi |
| Lanț de deformatoare: Bone, Game Bone, Curve, Envelope, Free-Form, Mesh Warp | Harmony | 🟡 bone ✅ curve (coloană) ✅ free-form ✅ mesh warp ✅; envelope/cage ❌; game-bone ❌ |
| Shape-Aware Deformation (puncte, oase, cage-uri) | Harmony 24 | ❌ |
| Bake pentru toate tipurile de deformare | Harmony 24 | 🟡 arcuri precalculate; geometrie precalculată per cadru în exportul Lottie |
| Master controller / selector de transformare pentru grupuri de deformare | Harmony 24 | ❌ |
| Peg-uri și ierarhie de straturi | Harmony, Animate | ✅ grupuri, straturi atașate la oase, parentare |
| Auto-muscle, auto foot-roll, ieșire cinematică † | Harmony | ❌ |
| Deformatoare de rotație / warp + parametri keyform, glue | Live2D | 🟡 parametri ≈ canale conduse, warp ≈ lattice; glue ❌ |
| Blend shapes care conduc rotația, opacitatea, ordinea de desenare, culoarea multiply/screen | Live2D 5.0 | 🟡 ponderile morph conduc doar geometria |
| Auto-mesh cu control al densității uniforme pentru forme subțiri | Live2D 5.0 | ❌ (doar grilă regulată) |
| Deformatorul warp se redimensionează automat pentru a conține keyform-ul copil | Live2D 5.3 | ❌ |
| Pensulă de deformare (forme pătrate/linie/expansiune) | Live2D 5.3 | ❌ |
| Deformator facial asistat de AI / generare de mișcare facială | Live2D 5.0 | ❌ |
| Joystick-uri (control 2-D mapat pe animații) | Rive | 🟡 canalul condus mapează o intrare; fără joystick cu 2 intrări |
| Constrângeri între obiecte arbitrare (translate/rotate/scale/distance) | Rive | 🟡 doar aim, copy-rotation, pin |
| Rigging bitmap cu intercalări automate | TVPaint 12 | 🟡 mesh de imagine ✅; intercalări automate ❌ |
| Instrumentul Plastic (rigging bazat pe mesh) | OpenToonz | 🟡 mesh de imagine + ponderi |
| Auto-rigging, controlere, dinamică de arcuri | Duik † | 🟡 arcuri ✅; auto-rig ❌ |

## Animație, timeline, interactivitate
| Funcționalitate | Văzută în | OSATURA |
|---|---|---|
| Mixarea animațiilor: blend/crossfade între animații concurente la runtime | Spine | 🟡 moduri Add/Multiply + ponderi; `crossfade` precalculat; fără tranziții la runtime |
| Dopesheet + editor de grafice + previzualizare la runtime | Spine, Rive, OpenToonz | ❌ (datele o permit; fără UI) |
| Mașină de stări: intrări, listeners, tranziții cu condiții, stări de blend, straturi | Rive | ❌ |
| Evenimente, declanșatoare audio, active dinamice | Rive | ❌ |
| Data binding (elemente de design ↔ date live) | Rive | ❌ (expresiile pot citi canale, nu date externe) |
| Scripting încorporat (Luau, JavaScript, ToonzScript) | Rive, Cavalry, OpenToonz | ❌ |
| Layout-uri responsive / componente / biblioteci | Rive | ❌ |
| Trim Paths, feathering vectorial, N-slicing | Rive | ❌ |
| Temporizare independentă de rezoluție, tweening vectorial | Synfig | ✅ |
| Legarea parametrilor între straturi prin expresii | Synfig | ✅ |
| Graf de noduri procedural (generatoare, transformatoare, efectori), duplicatoare | Cavalry | ❌ (fără graf de noduri; fără instanțiere de straturi) |
| Animație condusă de date din JSON/CSV | Cavalry | ❌ |
| Performing: înregistrarea gesturilor live ca chei | Procreate Dreams | ❌ |
| Taste rapide de declanșare pe puppet-stage, motion pilot (mersuri conduse de cursor) | Cartoon Animator 5.3 | ❌ |
| Comutator de camere: camere salvate, tăieturi cu tastă rapidă, track de comutare de cameră | Cartoon Animator 5.3 | 🟡 o singură cameră; tăieturile pot fi simulate cu chei step |
| Mișcări cinematice (dolly zoom, urmărirea unei ținte) | Cartoon Animator 5.3 | 🟡 cameră animabilă; fără urmărirea țintei |
| Mocap facial (webcam / iPhone), corp condus de cap, volumul microfonului | Cartoon Animator, Live2D | ❌ (volumul microfonului doar prin WAV offline) |
| Mod livestream (OBS/Twitch, mapare Stream Deck) | Cartoon Animator 5.3 | ❌ |
| Lip-sync Motion Sync care generează chei de timeline | Live2D 5.0 | 🟡 WAV euristic → chei de expunere |
| Integrare lip-sync Rhubarb | OpenToonz | ❌ (doar euristic) |
| Extragere de root motion (compensare delta) | Spine 4.2 | 🟡 extrapolare loop-offset |
| Sincronizarea mânerelor curbelor în buclă | Spine 4.2 | ✅ extrapolare loop (fără UI de mânere) |
| Morphing cu morph hints | Harmony | 🟡 ținte morph + lerp de cale; fără hints |
| Cameră 2D bazată pe chei, cu temporizare pe timeline | TVPaint 12 | ✅ |

## Desen, pictură, instrumente bitmap
| Funcționalitate | Văzută în | OSATURA |
|---|---|---|
| Pensulă: randomizare (număr de ștampile, decalaj), hârtii duale, pensule multitexturate | Harmony 24, TVPaint 12 | ❌ |
| Opacitatea creionului după presiune/înclinare/viteză/taper/fade | Harmony 24 | ❌ |
| Ghidaje de simetrie (verticale, orizontale, duale) | Harmony 24 | ❌ |
| Instrument Cutter (lasso, trim), editor de contur, close-gap | Harmony | ❌ |
| Ghidaje de desen, ghidaje de perspectivă/riglă † | Harmony | ❌ |
| Animație fără hârtie: onion skin, flipping, light table | Harmony, Procreate Dreams, TVPaint | 🟡 (onion skin da; fără instrumente de desen sau light table) |
| Curățarea desenelor scanate, scanare, captură cu camera, stop-motion | OpenToonz | ❌ |
| Pensulă de selecție, bibliotecă de imagini | TVPaint 12 | ❌ |
| Accentuare (sharpening) la transformare | TVPaint 12 | n/a |
| Sensibilitate la presiunea creionului | Synfig | ❌ |
| Umbrire cu gradient bazată pe cale | Synfig | ❌ |
| Text cu conținut variabil/dinamic la runtime, în orice limbă | Rive, Procreate Dreams | ❌ |
| Fonturi personalizate, titluri, generice | Procreate Dreams | ❌ |
| Gestionarea paletelor și stilurilor | OpenToonz | ❌ |
| >50 de tipuri de straturi (fractali, filtre, distorsiuni) | Synfig | ❌ (≈7 tipuri de straturi) |

## Efecte și compoziție
| Funcționalitate | Văzută în | OSATURA |
|---|---|---|
| Rețea de compoziție în vedere de noduri cu porturi denumite | Harmony 24 | ❌ (arbore de straturi + măști/amestecuri) |
| Mattes, grupuri sub composite | Harmony | ✅ măști, grupuri izolate |
| Citire/scriere CryptoMatte | Harmony 24 | ❌ |
| Toon shader (contur, posterizare) | Harmony 24 | ❌ |
| Unsharp mask, schimb de canale | Harmony 24 | ❌ |
| Familia blur (radial, spin, direcțional), bokeh, zgomot fractal, tile | OpenToonz | ❌ |
| Particles FX | OpenToonz | ✅ strat emițător |
| 15 moduri de amestec RGB + 5 alfa, desenare offscreen per parte | Live2D 5.3 | 🟡 13 moduri de amestec; randare de grupuri izolate ✅ |
| Clipping (ID-uri de clip per parte) | Live2D, Rive | ✅ măști alfa |
| Imagistică HDR / HDRI | Synfig | ❌ |
| Import video, ProRes, rotoscoping peste filmare | Procreate Dreams | ❌ |
| Motor audio (voce off, muzică, SFX) | Procreate Dreams | ❌ |

## 3D, pipeline, interschimb, runtime-uri
| Funcționalitate | Văzută în | OSATURA |
|---|---|---|
| Scenă 3D, poziționare de oase FBX, graf de scenă Vulkan, desen pe plan în perspectivă | Harmony 24 | ❌ |
| Randare externă prin Maya / Blender | Harmony 24 | ❌ |
| Import vectorial: PDF / AI / SVG; EXR multi-strat; alfa DPX; OpenColorIO (ACES) | Harmony 24 | ❌ |
| Import PSD fără Photoshop | Spine 4.2, Rive | ❌ |
| Import Lottie / lipire din Figma | Rive | ❌ (doar **export** Lottie) |
| Texture packer / atlase | Spine | ❌ |
| Export JSON/binar de schelet + 19 runtime-uri oficiale / peste 40 comunitare | Spine | 🟡 JSON propriu; fără runtime-uri |
| Runtime-uri open-source (Web, Apple, Android, React, Flutter, C++, Unity, Unreal) | Rive | ❌ |
| Modul Pixels (aliniază sprite-urile la pixelii ecranului) | Spine 4.2 | ❌ |
| Export: WebM cu transparență | Live2D 5.3 | ❌ |
| Export: SVG/PDF, secvențe PNG/EXR, adâncime de 16/32 de biți, lot, ProRes | Cavalry | 🟡 SVG, PNG ✅; PDF/EXR/ProRes/adâncime mare de biți ❌ |
| Randare în cloud, linkuri de partajare, embed-uri | Rive | ❌ |
| Randare distribuită (Toonz Farm), control de versiuni | OpenToonz | ❌ |
| Export PDF / TVPaint JSON | OpenToonz | ❌ |
| Colaborare în echipă în timp real, remixare comunitară | Rive | ❌ |
| Export HTML5 Canvas / web interactiv | Animate † | 🟡 player HTML (pre-randat, neinteractiv) |
| Nativ Linux/macOS/Windows | Synfig, OpenToonz | ✅ |
| Licență open-source | Synfig, OpenToonz | ✅ MIT/Apache-2.0 |
| Vedere de parentare a straturilor, simboluri/instanțe de bibliotecă † | Animate | 🟡 parentare ✅; fără bibliotecă de instanțe |
| UI modular detașabil, spații de lucru personalizate, temă închisă/deschisă | TVPaint 12, Live2D 5.0 | ❌ |

## Roadmap candidat (ordonat după valoare ÷ efort, fără GUI la început)
1. Trim paths + urmărirea căii (oase/straturi de-a lungul unui Bézier) — Rive, Spine
2. Suprapuneri onion skin + urme de mișcare — aproape fiecare instrument 2D
3. Import SVG (căi, gradienți) — Moho 14, Harmony, Synfig
4. Termeni de vânt / gravitație pentru arcuri, dinamica oaselor condusă de intrări în stil Smart-Bone
5. Blur, umbră, glow, contur, posterizare (filtre de strat)
6. Instanțiere de straturi / duplicatoare (clonează N de-a lungul unei căi, cu decalaje per copie)
7. Mașină de stări interactivă în runtime-ul HTML/JSON (intrări, tranziții, blend)
8. Deformator envelope (cage), instrumente de copiere/sudare/netezire a ponderilor, auto-mesh cu densitate uniformă
9. Tăieturi de cameră / camere salvate, cameră cu urmărirea țintei
10. Export glTF al rig-ului; export EXR/PDF/WebM-alfa
11. Straturi de text (necesită încărcare de fonturi) cu urmărirea căii
12. GUI de studio: editare de puncte/oase pe canvas, editor de grafice, onion skin

## Surse suplimentare (concurenți)
- Note de lansare Toon Boom Harmony 24 și Premium Getting Started Guide (docs.toonboom.com)
- Spine 4.2 „The physics revolution" și „What is Spine?" (esotericsoftware.com)
- Live2D Cubism 5.0 și 5.3 „New Features" (docs.live2d.com)
- Cartoon Animator 5.3 feature highlights (discussions.reallusion.com)
- Pagina de funcționalități Rive (rive.app/features)
- Funcționalitățile Synfig Studio (zoftwarehub.com); documentația „What's New" OpenToonz
- TVPaint Animation 12 (tvpaint.com); recenzie Cavalry (superrendersfarm.com)
- Adobe Animate „Layer parenting" (adobe.com/learn); Procreate Dreams (Cartoon Brew)
- Duik: pagina de etichetă Lester Banks (doar titlu †); Blender Grease Pencil: pagina de manual nu a putut fi preluată (neacoperit)

---

# Addendum — Istoricele de versiuni ale concurenților (runda 2, 2026-10-19)

Legenda ca mai sus: ✅ gata, 🟡 parțial, ❌ încă nu. Sunt listate doar funcționalitățile confirmate pe paginile furnizorilor/proiectelor sau în presă. Nu s-au folosit site-uri de piraterie; web.archive.org nu este accesibil din acest mediu, așa că unele versiuni mai vechi rămân nedocumentate (vezi „Lacune cunoscute").

## Toon Boom Harmony 20 (note de lansare ale furnizorului)
| Funcționalitate | OSATURA |
|---|---|
| Editor centerline: ștergere de puncte, opțiune de netezime | ❌ |
| Linii de creion texturate → contururi de pensulă texturate; pipetă pentru textura creionului | ❌ |
| Instrument Line curbat / în formă de S; pensulă Auto-Fill; pensulă Repaint | ❌ |
| Ghidaje de aliniere, snapping limitat la straturile selectate | ❌ |
| Quick Ease / Ease In / Ease Out pe chei selectate | 🟡 easing per cheie ✅; comenzi de easing în lot ❌ |
| Marcaje de scenă (fără durată, ripple edit) | ❌ |
| Vedere de substituire a desenelor (redenumire/filtrare) | 🟡 straturi Switch ✅ |
| Noduri de umbrire a suprafeței: ambient occlusion, cast shadow, surface map/normal | ❌ |
| **Nod Weighted Deform** | 🟡 mesh de imagine cu ponderi ✅ |
| Nod Point Kinematic Output (eșantionare cu 1/2/3 puncte) | ❌ |
| Nod de cache pre-randare | 🟡 cache de bake al arcurilor ✅ |
| Noduri colour fade / curves / levels / gamma | ❌ |
| Formă de particule pe regiune plană folosind un desen vectorial | 🟡 emițător ✅; formă personalizată ❌ |
| Master controllers: deformator-pe-deformator, Function Wizard, Slider Wizard | 🟡 canale ✅; UI-uri de wizard ❌ |
| Scripting Python (PythonManager) | 🟡 operații JSON patch + CLI; fără scripting încorporat |
| Gestionarea spațiului de culoare, randare pe 16 biți, film cu post-procesare scriptată | ❌ (post-procesare ffmpeg manual) |
| Control Center: copiere/import/rulare de script în lot pe scene, coadă de randare | ❌ |

## OpenToonz 1.7
| Funcționalitate | OSATURA |
|---|---|
| Mod linie dreaptă pentru pensulă | ❌ |
| Suport pentru pensule GIMP + MyPaint | ❌ |
| Mod de umplere Selection+Freehand | ❌ |
| Lipire de imagine din clipboard | ❌ |
| Tablou de bord de pre-producție / storyboard | ❌ |
| Panouri personalizate, fereastră principală translucidă, rotire/răsturnare a vizualizatorului | ❌ |
| Scară Xsheet + navigare prin etichete | 🟡 date de timeline ✅; fără GUI |
| Export cu informații despre mișcarea camerei | 🟡 camera în Frame ✅ |
| **Export OCA (Open Cel Animation)** | ❌ |
| Export MOV și APNG | 🟡 MP4/GIF ✅; APNG/MOV ❌ |
| Conversie în lot TZP→TLV | ❌ |
| Floor Bump Iwa, Flow Fx; randare float + culoare liniară | ❌ |
| Suport Haiku OS | n/a |

## Synfig 1.4.2 (și 1.4.0 din runda 1)
| Funcționalitate | OSATURA |
|---|---|
| Export Lottie incl. straturi avansate de contur și blur, gamma | 🟡 export Lottie ✅ (forme/transformare); blur ❌ |
| Editor de scurtături de tastatură, scurtături de instrumente cu o singură tastă | ❌ |
| TimeLoop editabil în spațiul de lucru | 🟡 clipuri în buclă ✅ |
| Duplicarea straturilor Skeleton / Skeleton Deformation cu legături de oase | ✅ (oasele sunt date de proiect) |
| Importator SVG care gestionează atributele de stil | ❌ (fără import SVG) |

## Elemente din runda 1 (pagini ale furnizorilor, abreviat)
- **Spine 4.2/4.3**: constrângeri de fizică, slidere, mixarea constrângerilor, pixel snapping — arcuri ✅ (precalculate), slidere/mix 🟡 (IK `mix` ✅), modul pixels ❌.
- **Live2D Cubism 4.0–5.3**: ArtPath, onion skin, export WebM-alfa — ArtPath ❌; onion skin și WebM-alfa ✅.
- **Harmony 21/22**: deformatoare cu ponderi, texturi line-boil — 🟡 / ✅ (wobble).
- **Cartoon Animator 5** — vezi tabelele concurenților de mai sus.

## Noi candidați pentru roadmap din această rundă
13. Export OCA (Open Cel Animation) și APNG — OpenToonz 1.7
14. Comenzi de batch ease / „quick ease" în editor — Harmony 20
15. Filtre de corecție de culoare (fade, levels, curves, gamma) — Harmony 20
16. Umbrire de suprafață (cast shadow / AO) pentru straturi vectoriale — Harmony 20
17. Scripting încorporat (Python/Lua) peste API-ul de operații de patch — Harmony 20
18. Nod de cache pre-randare / evaluare cu cache — Harmony 20
19. Marcaje de scenă + ripple editing pe timeline — Harmony 20
20. Layer comps, reparentare animată a oaselor, straturi patch, follow-path, timecode SMPTE, zone sigure, auto-trace, pensulă scatter (lacune găsite în cercetarea Moho)

## Lacune cunoscute (nepreluabile din sursele accesibile)
Anime Studio 8.x, 10.1.x, 11.1; detalii Moho 12.5 și 14.1; Spine 3.0–4.1 (blogul 3.7 este doar o pagină video); Harmony 17, 23, 25–27 (paginile de documentație au dat eroare); Synfig 1.4.1; TVPaint 11; Cartoon Animator 4/5.1/5.2; changelog Rive.

## Surse (runda 2)
- docs.toonboom.com — note de lansare Harmony 20
- en.ubunlog.com — articol despre OpenToonz 1.7
- synfig.readthedocs.io — note de lansare Synfig 1.4.2
- esotericsoftware.com — Spine 3.7 features-rundown (pagină video, fără listă)

---

# Addendum — Surse YouTube (runda 3, 2026-10-19)

WebSearch restricționat la youtube.com returnează titluri de videoclipuri; WebFetch returnează **descrierea/lista de capitole** a fiecărui videoclip (fără transcrieri). Sunt listate doar funcționalitățile numite acolo; „nou în versiunea X" este așa cum afirmă cel care a încărcat videoclipul (adesea furnizorul).

## Moho 14.1 (descrierea videoclipului)
| Funcționalitate | OSATURA |
|---|---|
| Afișare/ascundere mânere în instrumentul Curvature | ❌ (fără canvas GUI) |
| Efectele de formă Shaded și Halo vizibile în modul de editare | 🟡 gradienți ✅; efecte halo/shade ❌ |
| Efect de pixelare în Live Project Settings | ❌ |
| Preview Animation: rezoluție mai mare, bordură de ieșire ascunsă | 🟡 previzualizare raster ✅ |
| Dialoguri modeless pentru scripturi Lua | ❌ (fără scripting) |
| Mod de amestec PSD „Linear Dodge (Add)"; moduri de amestec actualizate | 🟡 verifică lista de moduri de amestec; add/dodge ❌ |
| Indicator de document nesalvat în bara de titlu | ✅ studioul urmărește starea de anulare |
| Remedieri: evaluarea smart-bone, motion blur sub-cadru + dinamica oaselor, mânere Bézier de lungime zero | n/a |

## Moho 12.5
Paginile de titlu/descriere nu au oferit o listă de funcționalități (videoclip depășit; descărcarea gratuită 12.5 a fost retrasă). Titlurile conexe menționează tutoriale Smart Bones, rig-uri 2.5D Smart Warp și instalarea „Mynd Tools" — Smart Bones 🟡 (canale/morphs), Smart Warp 🟡 (lattice FFD ✅).

## Spine
| Versiune | Funcționalitate | OSATURA |
|---|---|---|
| 3.7 | Suport audio | ❌ (WAV analizat doar pentru lip-sync) |
| 3.7 | Exporturi refăcute | 🟡 |
| 3.7 | Type-to-search | ❌ |
| 3.7 | IK stretchy / comprimat / uniform | ❌ (IK păstrează lungimea) |
| 3.7 | Combinarea skin-urilor, duplicarea skin-urilor, placeholder-e | 🟡 straturi Switch |
| 3.7 | Copiere/lipire de vârfuri | ❌ |
| 3.7 | Randare pe grilă de pixeli | ❌ |
| 3.7 | Amestec aditiv al animațiilor | 🟡 doar morphs aditive |
| 3.7 | Runtime spine-cpp; player web | ❌ / 🟡 player HTML |
| 3.8 | Mesh tracing | 🟡 mesh grilă automat; fără trasare de imagine |
| 3.8 | Împachetare poligonală de texturi | ❌ |
| 3.8 | **Soft IK** | ❌ |
| 3.8 | Oase și constrângeri de skin | ❌ |
| 3.8 | Proiect mix-and-match | ❌ |
| 3.8 | Foldere, rigle, export PSD îmbunătățit, ecran de întâmpinare | ❌ |
| 3.8 | .json/.skel mai mici; încărcare binară în spine-ts | ❌ (doar JSON) |
| 4.0 beta | **Editor de grafice / curbe** | 🟡 easing per cheie ✅; fără editor |
| 4.1 beta | Funcția Sequence (atașament de secvență de imagini) | ❌ |

## Toon Boom Harmony 25 / 25.2
| Funcționalitate | OSATURA |
|---|---|
| Instrument Pencil Retouch | ❌ |
| Import de pensule Photoshop | ❌ |
| **Breakdown Assistant pentru animație cut-out** | ❌ (generatorul de ciclu de mers este înrudit) |
| Blocarea riglei de timeline; preferințe de șabloane de acțiuni | ❌ |
| Suport mai bun pentru fișiere de imagine, 3D și USDZ | ❌ |
| Art Layer Manager (schimbare, ștergere, îmbinare, duplicare de straturi de artă; 25.2+) | ❌ (duplicarea stratului prin operații de patch ✅) |
| Master Controller Grid Wizard | ❌ |

## Seria Cartoon Animator 5
| Funcționalitate | OSATURA |
|---|---|
| 5.1 Editare round-trip PSD și vector | ❌ |
| 5.1 Import de secvențe de imagini și GIF | ❌ |
| 5.2 **Motion Pilot** (mișcare de păpușar prin tragere) | ❌ |
| 5.2 Animație pe Motion Path / editare de cale | ❌ (roadmap #1) |
| Dinamică de arcuri, FFD, animație vectorială (existente) | ✅ / ✅ / ✅ |

## Noi candidați pentru roadmap din această rundă
21. Moduri Soft IK + stretchy IK — Spine 3.8/3.7
22. Strat de secvență de imagini (atașament cu listă de cadre) și import GIF/APNG — Spine 4.1, CA 5.1
23. Filtru de ieșire pentru pixelare / pixel-snap — Moho 14.1, Spine 4.2
24. Înregistrare în stil Motion Pilot: capturarea tragerii live a unui os/mâner în chei — CA 5.2
25. Breakdown assistant (intercalări automate cu arcuri pentru rig-uri cut-out) — Harmony 25
26. Mesh tracing din conturul alfa — Spine 3.8
27. Seturi de skin-uri (schimbarea împreună a grupurilor de atașamente) — Spine 3.7/3.8
28. Track audio în proiect + sincronizare de redare (nu doar analiză) — Spine 3.7
29. Hook-uri de scripting Lua/Python — Moho, Harmony 20

## Surse (runda 3, YouTube)
- Moho 14.1: A Complete Breakdown — youtube.com/watch?v=QT4AiV1wO1k; Moho 14.1 Update — watch?v=FCiBKBjDaNg
- Spine 3.7 features rundown — watch?v=Fi0ks2LeYSM; Spine 3.8 features rundown — watch?v=kU26B5465SQ; Spine v4-beta curve editor — watch?v=37CeFRzrjqc
- Harmony 25 changes — watch?v=kJ58C2DNIk8; Art Layer Manager 25.2+ — watch?v=AZc-8zHucOo
- Cartoon Animator 5.2 Motion Pilot — watch?v=Eu7vO2uGGN4; 5.1 sequence/GIF import — watch?v=O8EGwVwqgfs

---

# Addendum — Căutare exhaustivă a lacunelor (runda 4, 2026-10-19)

Surse: note de lansare oficiale Synfig/Rive, titluri/descrieri YouTube. Sunt listate doar funcționalitățile numite.

## Anime Studio 8 (seria YouTube „Pro Feature", titluri + descrieri)
| Funcționalitate | OSATURA |
|---|---|
| Trasare automată de imagine (bitmap → vector editabil) | ❌ (roadmap #20) |
| Character Wizard (creare ghidată de personaje) | 🟡 rig-uri demo/generator de mers; fără wizard |
| Import de fișiere Photoshop (PSD) | ❌ |
| Real-Time Media Connection (legătură live cu media externe) | ❌ |
| Instrument Scale Points | 🟡 operații de patch pe puncte |
| Presetări de proiect HD 1080p | ✅ orice dimensiune |
Anime Studio 10.1 / 11.1: nu s-a găsit nicio sursă accesibilă.

## Synfig 1.4.1 – 1.4.4
| Versiune | Funcționalitate | OSATURA |
|---|---|---|
| 1.4.1 | Straturi de sunet mixate în videoclipul exportat | ❌ (fără track audio) |
| 1.4.1 | Duplicate-layer copiază toate straturile selectate | ✅ (operații în lot) |
| 1.4.3 | Prompt de legătură cu osul: leagă transformarea în loc de origine | ✅ |
| 1.4.3 | Import Papagayo (.pgo); remedieri import SVG; framerate la secvența de imagini | ❌ / ❌ / ❌ |
| 1.4.3 | Remedieri la randarea GIF, compatibilitate FFmpeg 3.x | ✅ GIF/MP4 |
| 1.4.3 | Salvare automată (până la 15 s) | ❌ (salvare explicită) |
| 1.4.4 | Remedierea exportului ultimului cadru | ✅ acoperit de teste |

## Rive (note de lansare oficiale, sept.–oct. 2026) — cel mai activ concurent
| Funcționalitate | OSATURA |
|---|---|
| Convertor String Template (text din valori view-model, formatare de numere) | ❌ |
| Contururi întrerupte (Dash Effect), importate din SVG | ❌ |
| Ține apăsat Q/W/E + trage pentru mutare/rotire/scalare | ❌ (fără GUI) |
| Forme de pensulă animabile cu chei în modul Animate | ❌ |
| Copiere/lipire de chei de constrângere; convertoarele păstrează formulele | 🟡 constrângeri ✅, copiere ❌ |
| Data binding relativ; view models; proprietăți Blob | ❌ |
| Scripting Luau (fetch, text shaping, canvas GPU, shadere/WGSL, mesh-uri de imagine instanțiate) | ❌ |
| Stări de blend / Mix-by-view-model în timeline | 🟡 bake de crossfade |
| Listeners de tastatură, focus, gamepad; Text Input; semantică de accesibilitate | ❌ |
| Layout-uri Grid și Stack, liste virtualizate, constrângeri de scroll și fizică de scroll | ❌ |
| Set Parent (Alt+P) păstrând poziția și ordinea de desenare | 🟡 reparentare prin operații de patch; keep-world ❌ |
| Align to Key Object; decalaj de root-bone; Scroll to selection | ❌ |
| Forță aditivă pentru umpleri/contururi; canale RGBA separate animabile/legabile | 🟡 morphs aditive; RGBA cu chei ✅ |
| Contururi interioare/exterioare (acces timpuriu) | ❌ |
| Auto Trace (mai bun pentru pixel art) | ❌ |
| Conversie Flash `.fla`/XFL → Rive | ❌ |
| Presetări de randare: mașină de stări → video | 🟡 clip → MP4/GIF |
| Export `.riv` în planul gratuit; CLI (`rive push/publish/ls/docs`), extensie VS Code, simulare headless de intrare | 🟡 CLI ✅ (comenzi proprii) |
| Agent AI în editor | ❌ |
| Previzualizare Android; runtime-uri React Native / Apple / Android / Flutter | ❌ |
| Istoric de revizii și restaurare, sincronizare offline, anulare care păstrează zoom-ul | 🟡 anulare/refacere ✅ |

## TVPaint Animation 11 (descrieri YouTube)
Creare de scurtături, sub-desene removibile, netezirea liniilor, înregistrarea desenului, click-and-drag pe punctul pivot KeyFramer (11.7.1), Sprite Generator, Inlay Texture FX, linii de ghidaj, videoclipul „end of update releases" — toate ❌ (instrumente de pictură raster; OSATURA este vectorială/rig).

## Toon Boom Harmony 17 (YouTube)
Free Form Deformer 🟡 (lattice FFD ✅), Master Controllers îmbunătățite 🟡, RGB Difference Keyer ❌, Motion Blur ❌ (roadmap #5).

## Cartoon Animator 4 / Spine 3.6 / Harmony 22–23
Videoclipuri găsite (CA 4: plugin mocap Rokoko, actualizare mișcare 3D la 2D, normal-map și iluminare în 4.5; căi Spine 3.4; videoclip de funcționalități Spine 3.6; instrumente de desen Harmony 22), dar descrierile nu listează funcționalități → nu se pretinde nimic. Notabile doar după titlu: **retargeting de motion-capture** (CA 4) ❌, **normal maps/iluminare** (CA 4.5) ❌, **constrângeri de cale** (Spine 3.4) ❌.

## Noi candidați pentru roadmap din această rundă
30. Dash effect + contururi interioare/exterioare
31. Runtime de scripting încorporat (Lua/Luau) cu API fără GPU
32. Data-binding / view models + intrări de mașină de stări (tastatură/pointer/focus) pentru playerul HTML
33. Constrângeri de scroll / containere de layout (grid, stack) pentru animație în stil UI
34. Import lip-sync Papagayo (.pgo); import în stil Rhubarb
35. Set Parent păstrând transformarea world
36. Salvare automată + istoric de revizii în studio
37. Filtre Motion Blur și RGB keyer
38. Constrângeri de cale (os de-a lungul unui Bézier) — Spine 3.4, fuzionează cu roadmap #1
39. Retargeting de motion-capture către rig 2D (BVH/JSON) — CA 4
40. Iluminare cu normal-map pentru straturi de imagine — CA 4.5
41. Track de sunet mixat în exportul MP4 — Synfig 1.4.1
42. Import .fla / XFL și PSD

## Încă nepreluabile
Anime Studio 10.1 / 11.1, note detaliate Moho 12.5, pagini de detaliu Spine 3.0–3.6, Harmony 23, note detaliate Cartoon Animator 4.x, textul changelog-ului TVPaint 11.

## Surse (runda 4)
- synfig.readthedocs.io — note de lansare 1.4.1, 1.4.3, 1.4.4
- releasebot.io/updates/rive — Rive Editor 0.9–0.9.272, CLI 1.2–1.6
- YouTube: seria Anime Studio 8 Pro Feature (q_HH2zOE-Yk, CUemDIlPH6Y, 408CLy652mU, l9eXLUO9Dus); demo TVPaint 11 (MoqNrCAUy5k); Harmony 17 (1CU_3RJBcl4); playlist Cartoon Animator 4

---

# Addendum — Rețele sociale și canale comunitare (runda 5, 2026-10-19)

**Realitatea accesului:** paginile X/Twitter sunt blocate la preluare de robots.txt (se văd doar fragmentele/titlurile din rezultatele căutării); Facebook la fel; căutările Reddit cu filtru de domeniu au fost respinse de proxy-ul de căutare, iar căutările nefiltrate nu au returnat thread-uri Reddit; forumul Lost Marble servește un înveliș de redirecționare JavaScript fără conținut lizibil. Nimic nu a fost ocolit. Ce a putut fi citit:

## Canalele oficiale Moho (știri moho.lostmarble.com, fragmente X)
| Element | Note | OSATURA |
|---|---|---|
| Moho 13.5.3 (apr. 2022) | lip-sync automat îmbunătățit, interfață mai bună, oase mai puțin intruzive | ✅ lip-sync (euristică WAV) |
| Moho 13.5.5 (X, mai 2022) | **Relative Paths** pentru activele legate | ✅ directorul de active este relativ |
| Moho 14.3 „Mantis Shrimp" (dec. 2024) | actualizare axată pe pensule | ❌ |
| Moho 14.4 „Side Quest" (nov. 2025) | export către motoare de joc și software 3D | 🟡 Lottie/JSON propriu; glTF/FBX ❌ |
| Moho 14.5 „Showtime" (oct. 2026) | onion skin-uri magnetice, Shift & Trace, opțiuni Liquid Shapes, EXR/PNG cu profiluri de culoare, mod live-playback pentru streaming/controlul personajelor în timp real | vezi matricea principală; mod live ❌ |
| Ambassador Program (iun. 2026); poziție oficială: nu sunt planificate instrumente AI (mar. 2025) | politică | n/a — nici OSATURA nu are funcționalități AI |
| Webinarii (Víctor Paredes etc.) | tutoriale | n/a |

## Comparații din comunitate (forumul Spine, „Spine vs Moho 12")
- Utilizatorii apreciază în Moho: fizica încorporată, mascarea straturilor, smart bones, editarea vectorială.
- Utilizatorii critică: FFD considerat slab; **fără un runtime real**, exportul pentru jocuri limitat la FBX.
- Creature este lăudat pentru motion capture integrat (FFD criticat).
- Constrângerile de fizică din Spine 4.2 primesc reacții entuziaste pe X (postări ale utilizatorilor).
OSATURA: fizică ✅ (arcuri precalculate), mascare 🟡 (clip prin operații booleene), smart bones 🟡, editare vectorială ✅, runtime ❌, mocap ❌.

## Titluri de liste de dorințe de pe forumuri (conținut necitibil)
„Concentrate on Timeline/Workflow improvements" (2024), „Road map for Moho", „Help Shape the Future of Anime Studio", „13.5.2 performance issue" — sugerează că timeline-ul/fluxul de lucru și performanța sunt cele mai solicitate zone; nu este o listă de funcționalități verificată.

## Completări la roadmap
43. Mod live-playback / streaming în timp real (intrare de poze prin WebSocket → cadre randate) — Moho 14.5
44. Bibliotecă de runtime într-un limbaj mic și portabil (build WASM al evaluatorului) — cea mai citată slăbiciune a Moho
45. Mascarea straturilor ca tip de strat de prim rang (nu doar operații booleene)
46. Ergonomia timeline-ului/fluxului de lucru în studio (editor de grafice, selecție multiplă de chei) — liste de dorințe din comunitate

## Surse (runda 5)
- moho.lostmarble.com/blogs/news (17 postări, dec. 2021 – oct. 2026)
- Fragmente din căutarea X: postări @MohoAnimation (13.5.5 Relative Paths; prețuri Moho 14), postări ale utilizatorilor despre fizica Spine 4.2
- Thread-ul de pe forumul Spine „8033 spine vs moho 12" (de.esotericsoftware.com)
- Titluri de thread-uri de pe forumul Lost Marble (conținut necitibil)

---

# Addendum — Liste de dorințe și panouri de cereri de funcționalități (runda 6, 2026-10-19)

Sunt listate doar cererile efectiv lizibile. Coloana „OSATURA" = dacă motorul acoperă deja cererea.

## Moho (forumul Lost Marble)
Lizibil: doar indexul forumului. Există secțiunile „Feature Request Discussions" și „Unofficial Bug Reports and Feature Requests"; ultimul titlu de cerere vizibil: **„Dock Windows – Custom Interface"** (18 oct. 2026). Thread-ul de lungă durată „Help Shape the Future of Anime Studio" (828 de răspunsuri, 2011–2021) și alte thread-uri cu liste de dorințe („Road map for Moho", „My wish for Moho in 2024 – Concentrate on Timeline/Workflow improvements", „Request for top 10 Anime Studio Pro 6 features") sunt învelișuri de redirecționare JavaScript → conținut necitibil. Comparații specifice de la forum la forum (forumul Spine): fizică, mascare, smart bones dorite în Spine; Moho criticat pentru FFD slab și lipsa unui runtime.
| Cerere | OSATURA |
|---|---|
| Ferestre ancorabile / interfață personalizată | ❌ (studioul web are layout fix) |
| Îmbunătățiri de timeline / flux de lucru | 🟡 |
| Runtime real pentru jocuri | ❌ |
| FFD mai puternic | ✅ lattice Bernstein + mesh |

## Cartoon Animator (Reallusion Feedback Tracker + thread cu lista de dorințe)
| Cerere (voturi unde sunt afișate) | OSATURA |
|---|---|
| Curbă implicită de chei în preferințe | ✅ easing per cheie |
| Editare de chei multi-canal | 🟡 operații de patch în lot |
| Setări persistente ale panourilor | ❌ |
| Marcaje și canale codate prin culoare (stânga roșu / dreapta verde) | ❌ (oglindire L↔R ✅) |
| „Uber key" pentru afișarea tuturor canalelor animate | ❌ |
| Suport SVG (Affinity, Inkscape); lipire de căi vectoriale | ❌ |
| Instrument/panou Align | ❌ |
| Adâncime de câmp / filtru blur | ❌ |
| Import GIF/APNG/video prin drag-drop plasat pe timeline | ❌ |
| Lumini spot/ambientale cu umbre | ❌ |
| Auto-scroll al timeline-ului | n/a |
| Text mai bun: kerning/leading animabile | ❌ |
| Culoare SVG animabilă | ✅ |
| Ascundere globală / canale „shy" | ❌ |
| Panouri ancorate | ❌ |
| Mai multe scene per proiect | ❌ |
| **Panou Auto Animate (mișcare idle procedurală pe oase)** | 🟡 unduire cu zgomot coerent ✅ |
| Strat de desen live / proprietăți dinamice editabile (18) | ❌ |
| Hitbox și fizică (14) | 🟡 arcuri |
| Import MOV transparent (14) | ❌ |
| Panoramă 360° (5); idei de integrare AI (6) | ❌ |

## Panoul de cereri de funcționalități Live2D Cubism Editor (30 de thread-uri, 2023–2026)
| Cerere | OSATURA |
|---|---|
| Suport Linux | ✅ |
| Scurtături de tastatură în editarea manuală a mesh-ului | n/a |
| Limita blendshape coborâtă la −100% | ✅ morphs nelimitate |
| Redimensionare proporțională; instrument de gravitație; blocare mesh-la-mesh | ❌ / ❌ / ❌ |
| Glue pentru deformatoare + ponderi precise de glue | 🟡 ponderi ✅ |
| Import de fizică fără suprascrierea grupurilor existente | ❌ |
| Blocarea parametrilor în animator; loop-edit vs repeat | ❌ |
| Opțiuni „Expand Warp based on Child"; schimbarea centrului de rotație al warp-ului | ❌ |
| Instrument de tip „Set Vertex Flow", selecție de margine în buclă, mapare de mesh cu contur de densitate variabilă | ❌ (auto-mesh uniform) |
| Suport pentru pixel-art | ❌ |
| Opacitate per vârf; filtrarea părților și afișare mai curată | ❌ |
| Mutarea ușoară a cheilor | ✅ operații de patch |
| Golirea atlasului din cache din memorie | n/a |
| Fereastra de fizică: procent maxim de ieșire, editare în masă a intervalelor de pendul | 🟡 |
| RISC-V / Cubism Core încorporat | ❌ |
| Drawables opționale în specificația CDI3 | n/a |

## Toon Boom Harmony
Forumul utilizatorilor este închis și redirecționează către Discord (nu poate fi căutat); thread-urile vechi de pe „Feature Requests board" („One can only dream: A Feature List", „Software Upgrade Suggestions Part 2", „Bitmap Layer Overhaul LONG OVERDUE", „Various Quality of Life suggestions") arată doar anunțul de închidere. Nicio cerere nu poate fi recuperată.

## Synfig (lista de idei GSoC 2019–2022, numere de issue GitHub legate)
| Cerere | OSATURA |
|---|---|
| Copiere/lipire de waypoint-uri (#2139) | ✅ chei prin operații |
| Comutarea filelor cu Ctrl+Tab, scurtături de ferestre, selecția zonei de redare, căutare în proprietăți, comutarea riglelor, modificarea parametrilor cu mouse-ul | ❌ (GUI) |
| Performanță: randare/export mai rapide; cache pentru parametrii straturilor; arbore incremental de task-uri de randare | ✅ evaluate ca funcție pură; fără cache ❌ |
| Portare GTK 4, build-uri Conan/vcpkg | n/a |

## Spine
Cererile merg pe forum și sunt mutate pe un panou Trello public (nu poate fi preluat: robots.txt). De pe forum: prioritatea cea mai mare istoric este stabilitatea editorului și actualizările de runtime. Planul din era Kickstarter (mesh-uri → FFD → skinning → IK pinning) este acum livrat integral, deci mesh/FFD/skinning din OSATURA sunt la paritate cu acea listă veche; IK pinning ✅ (constrângere Pin). Titluri recente de cereri pe forum: suport pentru ultimele three.js / React Three Fiber; suport Spine 4.3 în Defold.

## Rive
Roadmap-ul public = „canalul Feature Requests" cu etichete de stare (în lucru / planificat / lansat); necesită autentificare pentru citire. Nimic recuperabil dincolo de notele de lansare (runda 4).

## OpenToonz, TVPaint, Pencil2D
Paginile GitHub (OpenToonz/Synfig) sunt blocate de robots.txt; nu s-a găsit niciun panou TVPaint lizibil; există un thread Pencil2D „Feature requests to add", dar nu a fost deschis (în afara domeniului).

## Completări la roadmap din listele de dorințe
47. Auto-animate / strat idle procedural (respirație, clipit, legănarea capului cu raritate și intensitate) — CA wish #20; se bazează pe wobble-ul existent
48. Adâncime de câmp a camerei și filtru blur — CA
49. Lumini (spot/ambientale) cu umbre proiectate pentru straturi de imagine — CA
50. Operații de aliniere / distribuire (operație de editor) — CA
51. Codare prin culoare a canalelor și vizibilitate „shy" a canalelor în studio — CA
52. Mai multe scene per proiect + tăieturi de scenă — CA
53. Auto-mesh cu densitate variabilă; netezire vertex-flow — Live2D
54. Parametru pentru centrul de rotație al deformatorului warp; redimensionare proporțională — Live2D
55. Import de fizică cu îmbinare, fără suprascrierea grupurilor — Live2D
56. Blocarea parametrilor per track în timpul editării buclelor — Live2D
57. Layout de studio ancorabil/personalizat — Moho, CA
58. Cache de parametri între cadre (evaluare incrementală) — Synfig

## Surse (runda 6)
- Reallusion Feedback Tracker: „Wish list for new features for future updates"; panoul Cartoon Animator 5 (reallusion.com/FeedBackTracker)
- community.live2d.com — cereri de funcționalități Cubism Editor (pagina 1 din 7)
- Lista de idei Synfig GSoC 2022 (synfig-docs-dev.readthedocs.io)
- Indexul forumului Lost Marble (lostmarble.net/forum); thread-urile forumului Spine 2250, 8033; thread-ul Kickstarter 1501
- Rive FAQ: „Does Rive have a public roadmap?"

---

# Addendum — Proiecte open-source pe platforme git (runda 7, 2026-10-19)

**Acces:** API-ul de căutare GitHub nu este disponibil în acest mediu („sessions are bound to their configured repositories"), deci proiectele au fost găsite prin căutare web și citite din README-urile lor publice. API-urile GitLab și Codeberg sunt blocate de proxy; paginile Codeberg/GitLab au apărut doar în rezultatele căutării (Friction a fost lizibil). Numărul de stele nu a putut fi preluat. Mai multe proiecte sunt foarte noi/în stadiu timpuriu — funcționalitățile listate sunt așa cum sunt revendicate în README-uri, nu verificate prin rulare.

## Instrumente open-source similare de rigging / animație
| Proiect | Platformă | Funcționalități notabile | Comparație cu OSATURA |
|---|---|---|---|
| **Rigby2D** (jaisurya-dev-art/rigby2d) | GitHub, MIT/Apache | Editor în stil Spine (import PSD, mesh + weight paint, mesh-uri legate, constrângeri IK/transform/path/physics/**slider**, skin-uri cu oase specifice skin-ului, evenimente, atașamente bounding-box/clipping/point/path, dopesheet + editor de grafice), export Spine JSON 3.8/4.2/4.3 + .skel, atlas, runtime-uri JS/Rust/Godot, **server MCP încorporat** pentru asistenți AI, desktop Tauri | OSATURA nu are GUI de editor, atlas, runtime-uri, skin-uri, evenimente, constrângeri slider/path; are evaluator ca funcție pură, FX de lichid/booleene/emițător, lip-sync, generator de mers |
| **SkelForm** (Retropaint/SkelForm, Rust) | GitHub | IK FABRIK, nativ + web + wrapper mobil | nucleu de rig similar ✅; fără GUI ❌ |
| **Ankhimate editor** | GitHub, pre-alpha | Moduri Setup/Animate, sloturi + skin-uri, FABRIK de orice lungime, constrângeri path/physics, trasarea siluetei mesh-ului din alfa, pensulă de ponderi cu moduri de amestec + blocarea oaselor, **timeline-uri FFD**, evenimente cu audio, marcaje, decalaje de track per os, **editor de format de export bazat pe șabloane**, runtime de referință Rust | ❌ în mare parte GUI; idei: șabloane de export, decalaje de track per os |
| **rig2d-rs** (tanhieuno41) | GitHub, Rust | Port al fizicii Spine-4.2, IK cu **mix/softness/bend/compress/stretch/uniform**, auto-bind al ponderilor prin atenuare cu distanța, weight smooth/normalise/limit-to-4, packer de atlas tip shelf, ghosting onion-skin, import/export Spine JSON | ❌ IK soft/stretch, instrumente de ponderi, atlas, interschimb Spine |
| **Boner2D**, **Skelly**, **omarvision 2d-bones-IK** | GitHub | Instrumente scheletice Unity / pentru game designeri | de bază |
| **Skeletal-Animator** (ostad-ai) | GitHub, Python | oase + FABRIK, forme dreptunghi/capsulă/cerc, 4 easing-uri, onion skin, fereastră de clip a camerei cu chei, **intervale de vizibilitate per os**, spritesheet PNG+JSON, **MP4/WebM/MOV/GIF transparent** | ❌ intervale de vizibilitate, export video/spritesheet transparent, onion skin |
| **Stretchy Studio** (MangoLion) | GitHub, MIT | import PSD/PNG, **auto-rigging AI** (detecție de poză DWPose), wizard de configurare, animație de vârfuri cu mesh per parte, skinning de vârfuri pentru membre, **auto eye clipping** (iris în interiorul ochiului), **shape keys în stil Blender** cu slidere de influență, track-uri audio pe timeline, **export Spine 4.0 JSON** | ✅ morphs ≈ shape keys; ❌ auto-rig, eye clipping, audio |
| **Iki** (zeikar/iki) | GitHub, MIT | Motor web în stil Live2D: quad-uri conduse de parametri, deformatoare warp-mesh/grid, măști stencil, fizică de arcuri/lanț, import PSD, **auto-rigger din straturi cu nume de rol** (clipit, privire, lip-sync, întoarcerea capului, aprobare din cap, înclinare, sprâncene, respirație, fizica părului), schemă JSON deschisă cu **ID-uri StandardParameter**, server MCP + plugin Claude Code | ✅ schemă JSON, lip-sync; ❌ ID-uri standard de parametri, auto-rigger, măști |
| **Umamo** (umamoorg/umamo) | GitHub, Java | Editor compatibil Live2D: citire/scriere CMO3/MOC3 (până la Cubism 5.4), import PSD/KRA/CLIP, paletă de comenzi, presetări de combinații de taste, **CLI de diagnostic** (dump/extract/convert/diff de modele, presetări de calitate auto-mesh); animația nu este încă suportată | ❌ CLI de diff de modele, paletă de comenzi |
| **Ayatsuri2D** | GitHub | model `.aom` MessagePack (superset de moc3), export moc3 v3–6, C ABI, fuzzing în CI, runtime fără panic, benchmark-uri de latență p99.9 (timpuriu, „not usable yet") | ❌ format binar, C ABI, fuzzing |
| **Ayagami** (AyagamiDev) | GitHub, Rust/wgpu | MOC3 până la SDK 5.0 incl. glue și parametri blendshape, recalculare parțială, amestec linear/sRGB, amestec avansat/off-screen SDK 5.3, componentă Godot | ❌ randor GPU, MOC3 |
| **kasane-2d**, **img2live**, **claude-live-2d**, **minivt** | GitHub | Modelare de tip Live2D / cercetare imagine-unică-la-puppet | n/a |
| **Glaxnimate** (KDE) | oglindă GitHub / KDE | animație vectorială; dependențele de build arată trasare bitmap Potrace, libav, scripting Python, libarchive; accent pe Lottie | ❌ trasare, scripting |
| **Friction** (friction/friction, Codeberg) | Codeberg, GPL-3 | animație vectorială + raster pentru web/video, motion graphics, SVG, Skia + FFmpeg + GLSL (bazat pe enve) | ❌ animație SVG, shadere |
| **Tahoma2D** | GitHub | 2D + stop-motion, bazat pe versiunea Studio Ghibli a Toonz | ❌ |
| **Pencil2D** | GitHub, GPL-2 | desen manual bitmap + vector, onion skin | ❌ |
| **q0idmation** (q0s0id) | GitHub | editor + player + **limbaj de scripting** | ❌ scripting |
| **svgMotion** (michaelsboost) | GitHub | animație vectorială în browser | ❌ |
| **kinetic** | GitHub | timeline de chei SVG → export CSS/GSAP | ❌ export CSS/GSAP |
| **dotlottie-rs** (LottieFiles) | GitHub, Rust | **mașini de stări** (guards/tranziții/acțiuni), theming/**slots** (suprascrieri de culoare/scalar/text/vector la runtime), container zip `.lottie` multi-animație, API C + WASM, randor ThorVG | 🟡 export Lottie ✅; ❌ slots/mașini de stări/container dotLottie |
| **rive-runtime** | GitHub | Runtime C++: randor vectorial GPU (Metal, Vulkan, D3D11/12, GL/WebGL), împachetat pentru Apple/Android/Flutter/Unity/Unreal/web | ❌ |
| **spine-runtimes** | GitHub | runtime-uri pentru Unity, Godot, Unreal, libGDX, MonoGame, XNA, Flutter, Haxe, C/C++/C#/TS, Android, iOS, GLFW, SDL, SFML; politică de branch per versiune de editor | ❌ |
| **Depozit de unelte Moho** (wanderjimenezRD/Moho-Pro-14.4-2D-Animation-Tools) | GitHub | doar listare; descrierea nu oferă funcționalități — neevaluat | — |

Au apărut depozite de software piratat (de ex. Adobe Animate „activate-free"): nu au fost deschise și nici folosite.

## Idei care merită adoptate (noi candidați pentru roadmap)
59. **Server MCP / API de agent** peste editorul cu operații de patch, astfel încât asistenții AI să poată face rigging și animație (Rigby2D, Iki) — se potrivește arhitecturii JSON-patch a OSATURA
60. **ID-uri standard de parametri** (ParamAngleX, EyeOpen, MouthOpen…) pentru ca orice gazdă să poată conduce un model (Iki)
61. **Auto-rigger din straturi cu nume de rol**: clipit, privire, întoarcerea/aprobarea/înclinarea capului, sprâncene, respirație, fizica părului (Iki)
62. **Auto eye clipping** (irisul mascat de forma ochiului) (Stretchy Studio) — prin strat Boolean
63. **Constrângere slider** și **constrângere de cale** (Rigby2D, Spine)
64. Moduri **IK soft / stretchy / compress / uniform** (rig2d)
65. **Instrumente de ponderi**: smooth, normalise, limit-to-N, copy, blocarea oaselor, moduri de amestec ale pensulei (rig2d, Ankhimate)
66. **Trasarea siluetei mesh-ului din alfa** (Ankhimate)
67. **Intervale de vizibilitate per os** (Skeletal-Animator)
68. **Export video cu fundal transparent (WebM/MOV cu alfa) și spritesheet PNG+JSON** (Skeletal-Animator)
69. **Motor de șabloane de export** (formate de ieșire definite de utilizator) (Ankhimate)
70. **Container dotLottie `.lottie` + slots/theming la runtime + mașini de stări** (dotlottie-rs)
71. **CLI de diff / dump / convert pentru modele** (`osatura diff a.json b.json`) (Umamo)
72. **Paletă de comenzi + scurtături redefinibile** în studio (Umamo)
73. **Loader cu fuzzing, cale de runtime fără panic, benchmark-uri de latență** (Ayatsuri2D) — întărirea robusteții pentru `Engine::new`
74. **Import/export Spine JSON** pentru interschimb (Rigby2D, rig2d, Stretchy Studio)
75. **Packer de atlas tip shelf** pentru straturi de imagine (rig2d, Rigby2D)
76. **Evenimente cu valori cu chei și indicii audio** pe timeline (Rigby2D, Ankhimate)
77. **Decalaje de track / time-shift per os** (Ankhimate)

## Surse (runda 7)
- github.com/jaisurya-dev-art/rigby2d; Retropaint/SkelForm; Ankhimate/editor; tanhieuno41/rig2d-rs (release v0.1.0)
- github.com/zeikar/iki; MangoLion/stretchystudio; umamoorg/umamo; Yoru-KenomoVT/ayatsuri_2d; AyagamiDev/ayagami
- github.com/ostad-ai/Skeletal-Animator; KDE/glaxnimate; tahoma2d/tahoma2d; pencil2d/pencil
- github.com/rive-app/rive-runtime; EsotericSoftware/spine-runtimes; LottieFiles/dotlottie-rs
- github.com/topics/animation-editor, /topics/vector-animation
- codeberg.org/friction/friction

---

# Addendum — Alte surse publice (runda 8, 2026-10-19)

Surse: itch.io, Steam, AlternativeTo, Wikipedia, rezultate de căutare academice/de brevete, recenzii ale instrumentelor After Effects. Afirmațiile provin din paginile de magazin/proiect, nu din teste practice.

## Instrumente indie / de nișă pentru rigging (itch.io, Steam)
| Instrument | Funcționalități notabile | OSATURA |
|---|---|---|
| **Inochi Creator / Inochi2D** (Kitsunebi Games, open source, beta) | rigging 2D de puppet deschis, orientat spre VTuberi + runtime în timp real; format propriu (nu este compatibil Live2D/Spine); export GIF/secvență de imagini; UI multilingv | 🟡 aceeași nișă (format de rig deschis) |
| **Keyframe.it** (browser, gratuit) | oase + IK + mesh-uri cu ponderi + mișcare cu arcuri; **clipuri pentru simboluri de joc** (idle, land, anticipation, win); editor de imagini încorporat + import PSD; export GIF, sprite sheet, video, player HTML autonom, **pachete Spine 3.8/4.2**; salvare automată în browser; conector de agent AI **keyframe-mcp**; asistent AI încorporat opțional | ✅ player HTML/GIF; ❌ sprite sheet, pachet Spine, MCP, editor în browser |
| **RigBone 2D** (Godot) | poziționare de oase, poză-ca-cadru, onion skin, decupare + fundal transparent, secvență de cadre PNG (fără runtime necesar) | ❌ onion skin, secvență PNG cu decupare alfa |
| **SpineBones** | sloturi + atașamente de imagine, grupuri, IK, **previzualizare audio pe timeline**, imagine de referință de fundal, export JSON / sprite sheet / WebM | ❌ |
| **PySpine, Tangent (sprite animator)** | listate pe itch.io; textul de funcționalități nu a fost citit | — |
| **Mocap2D / FBX2SPINE** (Steam) | import mocap FBX/BVH; **Auto Match** al scheletului după nume + ierarhie; **Smart Retargeting** (proiecție, bun pentru vedere laterală); **Direct Retargeting** pe axe; **conversie automată in-place** pentru bucle de locomoție; **IK cu contact cu solul** (foot lock); previzualizare live; import/export FBX, BVH, schelete Spine, Unity 2D Animation | 🟡 retarget după nume ✅; ❌ import mocap, IK foot-lock, conversie in-place |
| **VTube Studio** (Steam) | urmărire prin webcam (OpenSeeFace), **gură condusă de voce**, mouse/creion ca intrare de urmărire, mapare a oricărei intrări → orice parametru de model, taste rapide pentru expresii/animații, „obiecte" PNG fixate care urmăresc modelul; doar modele Live2D | 🟡 canale de parametri ✅; ❌ mapare de intrare live, taste rapide, obiecte fixate |
| VBridger, Vitamins | Instrumente Steam pentru VTubing; neevaluate | — |

## Ecosistemul de rigging pentru After Effects (recenzia School of Motion)
| Funcționalitate | Sursă | OSATURA |
|---|---|---|
| **Controler joystick** (5 poze-cheie: origine/dreapta/stânga/sus/jos) interpolat în 2D | Joysticks 'n Sliders | ❌ — candidat puternic |
| **Slidere** care mixează un număr nelimitat de poze | JnS, Duik | 🟡 canale morph |
| Structuri (oase independente de grafică, reutilizabile), autorig | Duik Bassel | ✅ |
| Controlere: slider, slider 2D, unghi | Duik | ❌ |
| **Connector** (slider/joystick/rotație care leagă orice proprietate de alta), control de proprietate master | Duik | 🟡 expresii? (motor de expresii ✅ în canale) |
| Chei de poză intermediară între stări de poză | Duik | ✅ |
| Animarea căilor de mască | JnS | ✅ morph-uri de cale |

## Aplicații concurente / alternative (AlternativeTo, lista Wikipedia la 2026)
Tahoma2D (fork OpenToonz, stop-motion), TupiTube, enve (întrerupt; a devenit Friction), Expressive Animator (SVG interactiv), Cavalry, Jitter (motion design web), Blazy (Figma → animație, export CSS/React), Animation Desk, Pivot Animator, FlipaClip, animația Krita, Blender (mar. 2026), Clip Studio Paint (animație), Adobe Character Animator (puppeteering cu webcam; pagina „what's new" blocată de robots.txt), Adobe Animate (ultima versiune listată oct. 2023), CrazyTalk, DigiCel FlipBook, Apple Motion, RETAS. Niciuna nu adaugă funcționalități de rigging dincolo de ce acoperă rundele anterioare; idei noi: **presetări de la instrument de design → animație cu export CSS/React** (Blazy) și **puppeteering cu webcam** (Character Animator, VTube Studio).

## Literatură academică și de brevete (doar titluri/rezumate)
- Jacobson et al. 2011, **Bounded Biharmonic Weights** pentru deformare în timp real — ponderi de skinning netede automate dintr-un cage/schelet ✅ idee: îmbunătățirea `auto_skin` (în prezent bazat pe distanță).
- Brevetele Adobe US9865073 / US10078910, **„Computer animation of artwork using fast 2-dimensional deformation solving"** (warp-ul de mesh din Character Animator). *Notă clean-room:* OSATURA folosește FFD Bernstein și skinning cu ponderi, nu acel solver brevetat; orice viitor solver de tip ARAP ar trebui revizuit cu un avocat. Aceasta nu este consultanță juridică.
- Auto-rigging arXiv 2025–26: **ASMR** (schelet de mesh 3D + skin prin priori generativi 2D), UniRig, „How to Train Your Dragon" (rigging prin difuzie), HumanRig, SPRig, UniMate — toate 3D/ML; relevante doar ca viitoare funcționalitate de „sugestie de auto-rig".
- Stony Brook TVC 2017 **retargeting automat al ponderilor de skinning** — copierea ponderilor între mesh-uri (✅ roadmap: copierea ponderilor).

## Noi candidați pentru roadmap din această rundă
78. **Controler joystick**: punct de control 2D care amestecă 5 poze-cheie (și variante cu N poze) → conduce orice canale — JnS / Duik / parametri Live2D
79. Import mocap (BVH) + potrivire automată a scheletului + conversie in-place a buclelor + IK foot-lock — Mocap2D
80. Mapare de intrare live: orice intrare (mouse, MIDI, nivel audio, repere de webcam) → parametru, plus taste rapide pentru expresii — VTube Studio
81. Obiecte/recuzită fixate care urmăresc un os/ancoră cu propriul decalaj — VTube Studio
82. Presetări de clipuri pentru simboluri de joc (idle / land / anticipation / win) — Keyframe.it
83. Export de spritesheet (PNG + JSON) și secvență PNG transparentă cu decupare — Keyframe.it, RigBone
84. Previzualizare audio pe timeline cu formă de undă — SpineBones
85. Strat cu imagine de referință (fundal nerandat) — SpineBones
86. Ponderi de auto-skin biharmonice mărginite sau prin difuzie termică (înlocuiesc atenuarea cu distanța) — lucrarea BBW
87. Presetări de la instrumente de design → export CSS/React — Blazy
88. Puppeteering cu urmărire facială prin webcam în studio (WebRTC + model de repere) — Character Animator, VTube Studio

## Surse (runda 8)
- itch.io: kitsunebi-games/inochi-creator, keyframeit.itch.io, itch.io/blog/1586091 (RigBone 2.0), itch.io/t/6177139 (SpineBones)
- Steam: FBX2SPINE / Mocap2D (app 2467960), VTube Studio (app 1325860)
- schoolofmotion.com — Joysticks 'n Sliders vs Duik Bassel
- alternativeto.net (alternative la Moho, Expressive Animator); en.wikipedia.org/wiki/List_of_2D_animation_software
- Jacobson 2011 BBW (homes.cs.washington.edu/~jovan/papers/jacobson-2011-bbw.pdf); brevete US9865073B2 / US10078910; arXiv 2503.13579, 2503.15586, 2504.12451; retargeting de ponderi Stony Brook TVC 2017

---

# Addendum — Adobe Character Animator, setul complet de funcționalități (runda 9, 2026-10-19)

Surse: tabelul de versiuni Wikipedia (incl. 26.0 din ian. 2026), pagina de produs Adobe, pagina Adobe Help „Behaviors" și notele de lansare (oct. 2018, oct. 2020/feb. 2021, oct./dec. 2021, 2023). Indexul „what's new" al Adobe returnează doar 2023 și 2026; 24.x și 25.x sunt listate de Wikipedia ca versiuni doar de stabilitate.

## Concept
Grafica pe straturi din Photoshop/Illustrator (și EPS/SVG din oct. 2018) devine un **puppet** cu rig automat pe baza **numelor/etichetelor straturilor**; performanța este captată live (față/corp prin webcam, microfon, tastatură, mouse, MIDI) ca **take-uri** pe un timeline; ieșirile merg către secvențe PNG, WAV, Media Encoder (incl. **video cu alfa**), Dynamic Link către After Effects / Premiere și Syphon / Mercury Transmit live. Modul Starter este gratuit; Pro necesită Creative Cloud.

## Comportamente (lista completă din Adobe Help)
| Comportament | Ce face | OSATURA |
|---|---|---|
| **Face** | cap, ochi, sprâncene, nas, gură din webcam; parallax și forța poziției capului | ❌ (fără urmărire live) — roadmap #88 |
| **Body** (Body Tracker, Sensei AI, 22.0) | brațe, trunchi, picioare din webcam | ❌ |
| **Lip Sync** | vizeme ale gurii din voce; acuratețe asistată de transcriere (22.1); motor 3.4 mai bun la F, W-Oo, mai puține L/M | 🟡 WAV → 5 vizeme ✅; aliniere la transcriere ❌ |
| **Nutcracker Jaw** | maxilarul inferior din față/voce | ❌ |
| **Eye Gaze** | pupilele se mișcă independent | ✅ constrângeri aim/canale |
| **Head & Body Turner** | schimbă grupuri de straturi (față, trei sferturi, profil) pe măsură ce capul/corpul se rotește | 🟡 straturi Switch ✅; fără schimbare automată condusă de unghi |
| **Walk** | ciclu de mers, alergare, accent pe poze, legănarea umerilor/șoldurilor, **Pin Feet When Standing** (3.4/3.5) | ✅ generator de mers; ❌ pin-feet |
| **Limb IK** (fost Arm IK) | direcție de îndoire + stretch pentru brațe și picioare | 🟡 IK cu 2 oase ✅; stretch ❌ |
| **Leader/Follower** (22.5) | modelează modul în care membrele se îndoaie/se întind | ❌ |
| **Dragger** | mouse-ul/atingerea mută o regiune | ❌ (studioul nu este interactiv pe canvas) |
| **Physics / Dangle** | balansare, coliziune, gravitație, vânt; squash & stretch, amortizare | 🟡 arcuri ✅; coliziune/vânt ❌ |
| **Magnets** | mânerele se lipesc între ele prin apropiere | ❌ |
| **Particles** | emite obiecte fizice (zăpadă, foc de tun) | ✅ emițător |
| **Motion Lines** | linii de urmă în spatele obiectelor rapide | ❌ |
| **Wiggler** | unduire pe grafică vectorială | ✅ wobble/boil |
| **Breath** | mișcare de respirație | 🟡 canal manual; roadmap #61 |
| **Auto Blink** | clipiri automate / pâlpâire de strat | ❌ (roadmap #61) |
| **Motion Library** (23.0, peste 350 de mișcări, parametru de forță 23.1) | aplică animații pentru tot corpul | 🟡 doar mers; ❌ bibliotecă |
| **Cycle Layers** | parcurge pe rând straturile dintr-un grup | 🟡 straturi Switch ✅; ciclare de cadre ❌ |
| **Layer Picker / Motion Trigger / Fader** | afișează un strat; comută după direcția mișcării; fade la declanșare | 🟡 |
| **Transform** | mutare/scalare/rotire/opacitate a mânerelor | ✅ |
| **Handle Fixer** | blochează părți dintr-un mesh | ✅ pin-uri |
| **Triggers** | apăsarea unei taste afișează straturi/gesturi; **panoul Controls** cu slidere, cadrane de unghi, butoane, mapare MIDI | ❌ — roadmap #80 |
| **Collide, Dynamic, Cycle Pause, Auto-swap (22.5)** | etichete/comportamente pentru fizică și schimburi de tip cadru | ❌ |

## Funcționalități de timeline, înregistrare și flux de lucru
| Funcționalitate | Versiune | OSATURA |
|---|---|---|
| Take-uri, replay-uri (înregistrări reutilizabile declanșabile), replay-uri inteligente, replay-uri editabile | 2.0 / 22.0 / 23.6 | 🟡 clipuri ✅; replay-uri ❌ |
| Chei, întinderea în timp a cheilor, camere de scenă, audio declanșabil | 3.0 / 3.2 | ✅ chei/cameră; ❌ audio declanșabil |
| Speech-Aware Animation (mișcare a capului/sprâncenelor din voce, Sensei) | 3.4 | ❌ — roadmap #61 |
| Îmbinare/secvențiere de take-uri; rânduri de take colorate; filtre de căutare (`kind:` `shared:` `armed:`); rânduri shy; restrângerea take-urilor | 3.4 | ❌ |
| Snap la cadre / obiecte; snapping pe timeline | 3.5 | ❌ |
| Operații pe zona de lucru: ripple delete, insert time, loop, trim, export | 2.0 | 🟡 operații de clip ✅ |
| Panou de probleme de rigging; căutare de etichete și triggere | 3.0 | ✅ validarea colectează toate erorile |
| Marcaje de istoric; recuperare automată a proiectelor deteriorate (4.2); format de fișier nou, compact și rezistent la corupere (4.0) | | 🟡 anulare ✅; recuperare ❌ |
| Puppet Maker (șabloane: păr, piele, accesorii) | 22.1.1 | ❌ |
| Characterizer (transformă o fotografie/pictură într-un puppet) | 2.0 | ❌ |
| Quick Export pentru partajare pe rețele sociale | 22.5 | 🟡 GIF/MP4 ✅ |
| Instantanee de scenă, numărătoare inversă la înregistrare, grafică personalizată pentru butoanele de control | 1.5 | ❌ |
| Selecția camerei, multi-monitor, HiDPI | 2.0 | n/a |
| Ieșire live (Syphon, Mercury Transmit), extensie Twitch Character Trigger | 1.1 / 2.1 | ❌ — roadmap #43 |
| MIDI, scripturi prin V8 (JavaScript) | 1.1 / 2.0 | ❌ — roadmap #31 |
| Partajare/publicare de puppet-uri, fișiere de puppet partajabile | Beta 5 | 🟡 JSON portabil |
| Dynamic Link către AE / Premiere | Beta 5 | ❌ |
| Slăbiciuni raportate (recenzie): curbă de învățare abruptă, dependență de hardware (webcam/microfon), timp lung de pregătire a personajului, Pro cu plată, complexitatea configurării triggerelor | | oportunitate: pipeline-ul offline determinist al OSATURA, CLI/JSON |

## Ce iese în evidență pentru OSATURA (noi candidați pentru roadmap)
89. **Auto-rig bazat pe etichete din numele straturilor** (cap, ochi, sprâncene, vizeme ale gurii, etichete de braț/picior) — ideea centrală din Character Animator; extinde roadmap #61
90. **Take-uri și replay-uri**: înregistrarea performanței parametrilor ca clipuri reutilizabile, declanșabile, cu easing de blend
91. **Pin Feet When Standing** și blocare la contactul cu solul pentru generatorul de mers
92. Comportamente de stretch ale membrelor **Leader/Follower**; **Magnets**; **Motion Lines**
93. Format **Motion Library** (clipuri de mișcare denumite cu forță 0-100%)
94. **Head & Body Turner**: schimbarea automată a grupurilor de vederi dintr-un canal de unghi yaw
95. **Lip-sync asistat de transcriere** (aliniere forțată a textului la fonemele WAV)
96. **Mișcare a capului/sprâncenelor sensibilă la vorbire**, din energia audio și pauze
97. **Export video cu alfa** (ProRes 4444 / WebM VP9 alfa) — prezent și în Skeletal-Animator (roadmap #68)
98. **Audio declanșabil** și parametri reactivi la audio

## Surse (runda 9)
- en.wikipedia.org/wiki/Adobe_Character_Animator (lista de funcționalități și tabelul de versiuni până la 26.0, ian. 2026)
- adobe.com/products/character-animator.html
- helpx.adobe.com — Behaviors in Character Animator; Feature summaries oct./dec. 2021, oct. 2020/feb. 2021; What's new (2023–2026)
- community.adobe.com — „Character Animator October 2018 detailed changes"
- manytools.com/review/adobe-character-animator (puncte forte/slabe)
