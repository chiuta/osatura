<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->

# OSATURA szószedet

A kódban és a dokumentációban használt szakkifejezések, az angol ábécésorrendben. A „Lásd” arra a
modulra (az `osatura-core/src` mappában, ha nincs másként jelölve) vagy dokumentumra utal, ahol a kifejezést
megvalósítják vagy meghatározzák. Dokumentumok: [PROJECT_FORMAT.md](PROJECT_FORMAT.md), [ARCHITECTURE.md](ARCHITECTURE.md).
A technikák forrásai a `CLEANROOM.md` fájlban vannak felsorolva.

**Aim kényszer (Aim constraint).** Olyan kényszer, amely úgy forgat egy csontot, hogy a +x tengelye egy másik csont
fejére mutasson, egy eltolással kiegészítve. Lásd `rig`; PROJECT_FORMAT 4.3. szakasz.

**Horgony (Anchor).** Egy Bézier-útvonal pontja pozícióval és opcionális bejövő és kimenő fogantyúkkal; csontsúlyokat is
hordozhat. Lásd `path`; PROJECT_FORMAT 6. szakasz.

**Szögkorlátok (Angle limits).** Csontonkénti minimum és maximum az animált forgásra, fokban, a csatornák után és minden
IK-lánc után érvényesítve. Lásd `rig`, `engine`.

**API-verzió (API version).** A stabil Rust felület (`api` és `prelude`) verziója, jelenleg 1.4.0,
pillanatkép-teszttel ellenőrizve. Nem azonos a fájl `format` mezőjével vagy a C `ABI_VERSION` értékével. Lásd `api`.

**Eszköz (Asset).** A projekt által hivatkozott fájl (kép, normáltérkép, hang). Az útvonalaknak relatívnak kell lenniük, és
a projektkönyvtárban kell maradniuk, hacsak nincs megadva az `--allow-external-assets`. Lásd `assets`;
PROJECT_FORMAT 18. szakasz.

**Bake.** Értékek időbeli kiszámítása és közönséges csatornákként vagy kulcsokként való eltárolása, hogy többé ne
legyen szükség a számoló eszközre. Használja a fizika, a klipek, a magnet, a footlock, az állapotgépek és más. Lásd
ARCHITECTURE 4. szakasz.

**Hajlítás (Bend).** Az előjel (+1 vagy -1), amely kiválasztja, merre hajlik egy kétcsontos IK-lánc, például a könyök befelé vagy kifelé.
Lásd `rig`.

**Bézier-görbe (Bezier curve).** Két horgonnyal és azok fogantyúival meghatározott paraméteres harmadfokú görbe. Kitöltéshez és mérésekhez
szakaszokra bontják (de Casteljau-felosztás). Lásd `path`.

**Kötés (Binding).** Futásidejű szabály, amely egy külső értéket (játékváltozót, csúszkát) kifejezésen, leképezésen és
simításon keresztül egy tulajdonságra vagy állapotgép-bemenetre képez le. Lásd `binding`.

**Keverési mód (Blend mode).** Az, ahogy egy réteg színe összeadódik az alatta lévővel (multiply, screen, overlay és
így tovább). A képletek a W3C Compositing and Blending Level 1 specifikációt követik. Lásd `raster`, `fraster`.

**Blend tree.** Több klip egydimenziós keverése számbemenettel vezérelve, állapotgépen belül használva. Lásd
`machine`.

**Blob.** Egy folyékony (metaball) réteg egy köre. Lásd `fx`.

**Csont (Bone).** A csontváz egy megnevezett szakasza fejpozícióval, hosszal, nyugalmi szöggel és opcionális
szülővel. A csontok skinnelt geometriát és csatolt rétegeket hajtanak. Lásd `rig`; PROJECT_FORMAT 4. szakasz.

**Boole-réteg (Boolean layer).** Olyan réteg, amely vektoros útvonalakat egyesítéssel, metszettel, különbséggel vagy xor-ral kombinál. Az
első útvonal a tárgy, a többit klipként egyesítik. Lásd `fx`.

**Pattogás (easing) (Bounce).** Leeső és visszapattanó labda easing-görbéje 0,6-os restitúcióval és három
visszapattanással. Szabadesésből származtatott eredeti levezetés. Lásd `anim`.

**Befoglaló doboz (Bounding box).** A geometria körüli legkisebb tengelyirányú téglalap; igazításra, kiválasztásra és a
normáltérképek illesztésére használják. Lásd `layout`, `pick`.

**Cage deformer.** Olyan deformer, amelyben egy zárt sokszögön belüli tartalom a sokszög csúcsait követi
mean-value koordinátákkal. Lásd `cage`; lásd még MVC.

**Catmull-Rom spline.** Harmadfokú spline, amely áthalad a kontrollpontjain. A `smooth` easing ennek nem
egyenletes változatát használja a szomszédos kulcsokon át. Lásd `anim`, a `liquid` simítását az `fx`-ben.

**Csatorna (Channel).** Egy animált érték: cél, tulajdonság, forrás (kulcsok, kifejezés, zaj vagy vezérelt), valamint
keverési mód és súly. A csatornák a lista sorrendjében futnak. Lásd PROJECT_FORMAT 8. szakasz.

**CIE76.** Az eredeti színkülönbség, az euklideszi távolság a CIE L*a*b* térben. Olcsó, kevésbé pontos,
mint a CIEDE2000. Lásd `percep`.

**CIEDE2000.** A CIE 2000 színkülönbség-képlet (CIE TR 142-2001), Sharma, Wu és Dalal (2005) közlése szerint megvalósítva, és
az általuk közölt tesztpárokkal ellenőrizve. Az animációs CI használja a látható változás megítélésére. Lásd `percep`, `ci`; `docs/CI.md`.

**Clean room.** Az a fejlesztési szabály, hogy a kód nyilvános szabványokból és tankönyvi módszerekből íródik,
másolt kód nélkül, és más eszközök formátumainak vagy neveinek reprodukálása nélkül. Minden technikát a
`CLEANROOM.md` naplóz.

**Klip (Clip).** Animációs sávok újrafelhasználható take-je, amely sebességgel, hurkokkal, súllyal, ki- és beúsztatással és
keverési móddal helyezhető az idővonalra. Lásd `clipstack`; PROJECT_FORMAT 11.1. szakasz.

**Cloth.** Rácsos deformer, amelynek csúcsai pontosan a saját eltolásaikkal mozognak. Az ezeket az eltolásokat bake-elő,
pozícióalapú dinamikai szimuláció is. Lásd `cloth`.

**Kényszer (Constraint).** Az IK után alkalmazott szabály, amely korlátoz egy csontot: aim, copy rotation vagy pin. Lásd `rig`.

**Konténer (elrendezés) (Container).** Flexbox-stílusú egydimenziós elrendezési csoport térközzel, kitöltéssel, igazítással és
sortöréssel, amely rétegeket és más konténereket pozicionál. Lásd `containers`.

**Copy-rotation kényszer.** Olyan kényszer, amely egy csontot egy másik csont világforgásához igazít. Lásd
`rig`.

**CRDT.** Conflict-free replicated data type: olyan adatszerkezet, amelyet a replikák függetlenül szerkeszthetnek, és
koordináció nélkül ugyanarra az eredményre egyesíthetők. Az OSATURA műveletalapú CRDT-t használ. Lásd `collab`;
ARCHITECTURE 10. szakasz. Kapcsolódó: LWW register, RGA, Lamport-óra, vektoróra, tombstone.

**Köbös Bézier easing (Cubic Bezier easing).** CSS-stílusú easing-görbe, amelyet a (0,0) és (1,1) közötti két kontrollpont határoz meg. Lásd `anim`.

**Deformer.** Vektoros vagy képrétegre nyugalmi térben alkalmazott alakmódosító: lattice, spine, pin,
cage vagy cloth. Lásd PROJECT_FORMAT 7.1. szakasz.

**Delaunay-háromszögelés (Delaunay triangulation).** Pontok háromszögelése, amelyben egyetlen pont sem esik egyetlen háromszög
köré írt körébe sem, ami elkerüli a vékony háromszögeket. Képmeshekhez a Bowyer–Watson-algoritmussal készül.
Lásd `meshtrace`.

**Mélységélesség (DoF) (Depth of field).** Elmosás, amely a réteg `parallax` értékének a kamera `focus` értékétől való távolságával nő.
Csak a `dof` és a `dof-export` parancs alkalmazza. Lásd `dof`.

**Determinizmus (Determinism).** Az a tulajdonság, hogy ugyanaz a bemenet ugyanazt a kimenetet adja. A képkockák kiértékelése tiszta;
a renderelés egy gépen és buildben bájtra azonos, platformok között nem ellenőrizték. Lásd
ARCHITECTURE 6. szakasz.

**Dope sheet.** Idővonalnézet, amely a kulcsokat csatornánként jelekként mutatja, és időben szerkeszti őket. Lásd
`timeline`, `keyops`.

**Vezérelt kulcs (Driven key).** Olyan csatorna, amelynek bemenete egy másik tulajdonság értéke az idő helyett, például a könyök
szögétől a dudor méretéig. Lásd `drivers`; PROJECT_FORMAT 8.1. szakasz.

**Easing.** A kulccsal induló szakaszra alkalmazott alakító függvény. Lásd `anim`; PROJECT_FORMAT
8.4. szakasz.

**Szerkesztő (Editor).** Az az objektum, amely a projektet és annak visszavonási előzményeit birtokolja, és a műveleteket alkalmazza. Lásd `edit`;
ARCHITECTURE 5. szakasz.

**Emitter.** Olyan réteg, amely az idő zárt alakú függvényeként részecskéket állít elő. Lásd `fx`.

**Motor (Engine).** A projekt ellenőrzött, névfeloldott alakja, amely a képkockákat kiértékeli. Lásd `engine`.

**Boríték (collab) (Envelope).** A replikák között műveleteket szállító üzenet: replikaazonosító, sorszám,
Lamport-idő, függőségek és műveletek. Lásd `collab`.

**Kiértékelés (Evaluation).** Egy képkocka pózának, geometriájának és kamerájának kiszámítása egy projektből. A képkocka
tiszta függvénye. Lásd PROJECT_FORMAT 16. szakasz.

**FABRIK.** Forward And Backward Reaching Inverse Kinematics: iteratív IK-solver, amely felváltva húzza a láncot a célpont felé a
csúcstól, majd újra a gyökérhez rögzíti. Kettőnél több csontú láncokhoz használják. Lásd `rig`.

**FK.** Előre kinematika (Forward kinematics): a csontok világtranszformációinak kiszámítása a szüleik transzformációiból és a
saját szögeikből és eltolásaikból. Lásd `rig`, `engine`.

**Formátumverzió (Format version).** A projekt egész értékű `format` mezője, jelenleg 1. Az újabb fájlokat elutasítja.
Lásd PROJECT_FORMAT 19. szakasz.

**Képkocka (Frame).** (1) Az idővonal egy időlépése, `0..frames-1`. (2) A `Frame` struct: egy lépés
kiértékelésének eredménye (csontok, geometria, kamera, jelzők). Lásd `engine`.

**Glue.** Olyan kapcsolat, amely az egyik mesh csúcsait egy másik mesh csúcsait követi (snap, rigid vagy offset). Lásd
`links`.

**Színátmenet-fogantyú (Gradient handle).** Lineáris vagy radiális színátmenet kontrollpontja, amely a
`grad_dx` és `grad_dy` csatornákkal animálható. Lásd PROJECT_FORMAT 8.2. szakasz.

**Csoportréteg (Group layer).** Más rétegeket tartalmazó réteg, amely továbbadja nekik a transzformációját. Lásd `project`.

**Fogantyú (Handle).** Egy horgonyhoz tartozó Bézier-kontrollpont. A fájlban abszolút koordináták. Lásd `path`.

**HOG.** Orientált gradiensek hisztogramja (histogram of oriented gradients), helyi alakjellemző (Dalal és Triggs 2005), amelyet az
arcdetektor használ. Lásd `facedet`.

**IK.** Inverz kinematika (Inverse kinematics): a csontszögek kiszámítása úgy, hogy a lánc csúcsa elérjen egy célpontot. Lásd `rig`.
Kapcsolódó: FABRIK, soft IK, stretch IK, bend.

**IK-keverés (IK mix).** Az FK-póz (0) és az IK-eredmény (1) közötti keverés egy láncnál. Az `ik_mix` csatornával animálható.
Lásd PROJECT_FORMAT 4.2. szakasz.

**Képmesh (Image mesh).** Egy képréteg fölötti rácsos vagy szabad háromszögháló, csontsúlyokhoz és deformerekhez használva.
Lásd PROJECT_FORMAT 5.8. szakasz.

**Kulcskocka (kulcs) (Keyframe).** Egy sávon lévő (képkocka, érték, easing) hármas. Az easing a kulcsnál kezdődő szakaszt
alakítja. Lásd `anim`.

**L*a*b*.** CIE 1976 színtér, amelyet úgy terveztek, hogy a távolságok közelítsék az érzékelt különbségeket. Lásd
`percep`.

**Lamport-óra (Lamport clock).** Logikai számláló, amely közös óra nélkül rendezi az eseményeket egy elosztott rendszerben
(Lamport 1978). Lásd `collab`.

**Lattice.** Deformer, amelynek egy téglalap fölötti kontrollpontrácsa az eltolásain keresztül, Bézier–Bernstein-keveréssel
hajlítja a tartalmat. Lásd PROJECT_FORMAT 7.1. szakasz.

**Réteg (Layer).** A jelenetfa egy csomópontja: vektor, csoport, kapcsoló, folyadék, boole, emitter vagy kép. Lásd
PROJECT_FORMAT 5. szakasz.

**Rétegeffekt (Layer effect).** Rétegenkénti szűrő (elmosás, vetett árnyék, színezés, kitöltés), amelyet képernyőn kívüli vásznon renderelnek.
Lásd `layerfx`.

**Részletességi szint (LOD) (Level of detail).** Egy projekt egyszerűsített változata, amelyet megadott, pixelben mért hibahatár alatt állítanak elő, gyors előnézethez vagy kis kimenethez.
Lásd `lod`, `osatura lod`.

**Linter.** A tanácsadó ellenőrző (`osatura lint`), amely az animáció minőségi problémáit jelzi olyan kódokkal, mint
az L011 és az L012. Különbözik az érvényesítéstől. Lásd `lint`.

**Folyadékréteg (Liquid layer).** Metaballokból álló réteg, amelynek körvonala egy hatásösszeg-mező izokontúrja. Lásd
`fx`.

**LWW register.** Last-writer-wins register: CRDT-cella, ahol a legmagasabb logikai időbélyegű írás marad meg. Objektumkulcsokhoz
használják. Lásd `collab`.

**Magnet.** Bake-elő eszköz, amely smoothstep lecsengéssel húzza a pozíciókat egy pont felé. Lásd `magnet`.

**Marching squares.** Algoritmus, amely rácson mintavételezett skalármezőből kontúrvonalakat nyer ki.
Folyadékkörvonalakhoz és bitkép-követéshez használják. Lásd `fx`, `trace`, `meshtrace`.

**Jelölő (Marker).** Az idővonal egy megnevezett képkockája vagy képkockatartománya. Az állapotgépek jelölőtartományt használhatnak
forrásként. Lásd `keyops`; PROJECT_FORMAT 10.2. szakasz.

**Maszk (Mask).** Olyan réteg, amelynek alfája korlátozza egy másik réteg láthatóságát; invertálható. Egy rejtett réteg
is lehet maszk. Lásd `raster`.

**Merge (CRDT).** Két replika állapotának egyesítése úgy, hogy mindkettő ugyanahhoz a dokumentumhoz konvergáljon. Lásd
`collab`.

**Mesh-zár (Mesh lock).** Olyan kapcsolat, amely egy követő meshet 0..1 erősséggel egy gazdamesh-hez köt. Lásd `links`.

**Metaball.** Olyan blob, amelynek mezőhatása összekeveredik a szomszédaival, így a közeli blobok összeolvadnak. Lásd folyadékréteg.

**Morph (blend shape).** Egy útvonal alternatív horgonykészlete, súllyal additívan alkalmazva. Lásd
PROJECT_FORMAT 6.1. szakasz.

**MVC.** Mean-value koordináták (Floater 2003): általánosított baricentrikus koordináták egy sokszögön belüli pontra,
a cage deformer használja. Lásd `cage`.

**Roncsolásmentes (Non-destructive).** A meglévő munka sértetlenül hagyása. A bake-elők csatornákat fűznek hozzá a kulcsok átírása helyett, és
a szerkesztő inverz műveleteket rögzít. Lásd ARCHITECTURE 4. és 5. szakasz.

**Normáltérkép (Normal map).** A felületi irányt RGB-ben tároló kép, a `relight` effekt és a rétegenkénti
megvilágítás használja. Lásd `postfx`; PROJECT_FORMAT 7.3. szakasz.

**Op.** `set`, `insert`, `remove` vagy `batch` műveletként, egy RFC 6901 JSON-pointeren kifejezett szerkesztés; visszavonáshoz
van inverze. Lásd `edit`.

**OT.** Operational transformation: a CRDT-k alternatívája, amely az egyidejű műveleteket egymáshoz képest írja át,
általában központi szerverrel. Az OSATURA nem használja; CRDT-t használ központi szekvenszerrel. Lásd ARCHITECTURE 10. szakasz.

**Parallax.** Rétegenkénti tényező, amely a kamera mozgását skálázza, így a távoli rétegek kevesebbet mozognak. Egyben a
mélységélességhez használt mélységérték. Lásd PROJECT_FORMAT 10.1. szakasz.

**Részecske (Particle).** Az emitter által kibocsátott rövid életű pont; a pozíciója az idő zárt alakú függvénye. Lásd
`fx`.

**Útvonal (cél) (Path, target).** Egy beágyazott réteg perjellel tagolt neve, például `parent/child`; az altárgyak `::`-t használnak,
mint a `layer::path::morph`. Lásd PROJECT_FORMAT 5. szakasz.

**Útvonal (vektor) (Path, vector).** Horgonyok nyílt vagy zárt sorozata opcionális kitöltéssel, körvonallal és morphokkal. Lásd
`path`.

**Pin kényszer (Pin constraint).** Olyan kényszer, amely egy csont fejét egy másik csont fejére csúsztatja. Lásd `rig`.

**Pin warp.** Deformer, amelyben a húzott pinek egy sugáron belül simán mozgatják a grafikát, kompakt tartójú
radiális bázisfüggvénnyel (Wendland 1995). Lásd `pinwarp`.

**Lejátszó (Player).** A homlokzat futásidejű burkolója, amely állapotgépeket és kötéseket ad egy animációhoz.
Lásd `api`.

**Preorder.** A rétegfa bejárási sorrendje: egy csomópont, majd a gyerekei. A kiértékelés ezt használja, hogy a
szülők legyenek előbb kiszámítva.

**Proxy.** A görgetés közben használt, csökkentett részletességű kép, részletességi szintből építve. Lásd `scrub`, `lod`.

**QEM.** Quadric error metric (Garland és Heckbert): egy csúcs összevonása okozta geometriai hiba mértéke,
meshegyszerűsítéshez használják. Lásd `lod`.

**Raszter (Raster).** Pixelkép, vagy az a modul, amely egy képkockát pixelekké renderel a tiny-skiával. Lásd `raster`.

**RDP.** Ramer–Douglas–Peucker: rekurzív vonalegyszerűsítés, amely megtartja azokat a pontokat, amelyek a húrtól
egy tűrésnél messzebb vannak. Lásd `trace`, `sizefit`.

**Referenciaréteg (Reference layer).** `reference` jelölésű réteg; rejtett, hacsak a szerkesztő nem mutatja a referenciákat. Lásd
PROJECT_FORMAT 5.1. szakasz.

**Leképezés (Remap).** Egy tartomány affin leképezése opcionális görbével és korlátozással, egy kötés értékére alkalmazva.
Lásd `binding`.

**Replika (collab) (Replica).** Egy résztvevő példánya a dokumentumról a CRDT-ben. Lásd `collab`.

**Nyugalmi póz (Rest pose).** A csonttranszformációk animáció nélkül; a skin-mátrixok `world * inverse(rest
world)`. Lásd `rig`.

**RGA.** Replicated growable array (Roh et al. 2011): szekvencia-CRDT, ahol minden elemnek egyedi
azonosítója van, és a törlések tombstone-okat hagynak. JSON-tömbökhöz használják. Lásd `collab`.

**Rig.** A csontváz és segédei: csontok, IK-láncok, kényszerek és rugók. Lásd
PROJECT_FORMAT 4. szakasz.

**Görgetés (Scrub).** A lejátszófej húzása; a stúdió revízió szerint gyorsítótárazza a képkockákat, és proxykat használ. Lásd `scrub`.

**Dőlés (Skew).** Egy réteg transzformációjában a nyírási szög. Lásd PROJECT_FORMAT 5.1. szakasz.

**Skin (animáció) (Skin, animation).** Rétegek, csontok, kényszerek vagy IK megnevezett változata, amely csoportban bekapcsolható,
például jelmezek. Lásd `skins`.

**Skinning.** A geometria deformálása a csontok transzformációinak keverésével, horgonyonként vagy csúcsonként súlyozva.
Az OSATURA normalizálja a súlyokat, és az üres súlyokat statikusnak tekinti. Lásd `engine`, `weights`.

**Soft IK.** IK-opció, amely a teljes kinyúlás közelében simítja a nyúlást, hogy elkerülje az egyenesre pattanást. Tartomány: 0-tól
0,9-ig. Lásd `rig`.

**SPDX.** Software Package Data Exchange: szabványos módja a fájl licencének megadására, például
<!-- REUSE-IgnoreStart -->
`SPDX-License-Identifier: MPL-2.0`. Minden forrásfájl hordoz ilyet. Lásd `LICENSING.md`.
<!-- REUSE-IgnoreEnd -->

**Spine deformer.** Deformer, amely a geometriát egy görbe mentén hajlítja úgy, hogy a horgonyokat egy nyugalmi
görbére vetíti, majd a deformált görbéhez köti újra. Lásd PROJECT_FORMAT 7.1. szakasz.

**Rugó (csont) (Spring, bone).** Csillapított másodrendű rugó, amely egy csont forgását késlelteti és túllendíti; betöltéskor bake-elik, így
bármely képkocka kiértékelhető. Lásd PROJECT_FORMAT 4.4. szakasz.

**SSIM.** Structural similarity index (Wang et al. 2004): képminőségi mérték helyi középértékekből, szórásokból és kovarianciából.
Az OSATURA blokkos változatot használ. Lásd `percep`.

**Színpad (Stage).** A kimeneti kép koordinátatere: pixelek, +x jobbra, +y lefelé.

**Állapotgép (State machine).** Animációs állapotok gráfja, bemenetek által vezérelt átmenetekkel. Van bake-elt,
eseménynaplós változata és futásidejű változata. Lásd `statemachine`, `machine`.

**Lépcsős easing (Step easing).** Olyan easing, amely a kulcs értékét a következő kulcsig tartja. Kapcsolórétegekhez használják. Lásd
`anim`.

**Stretch IK.** IK-opció, amely a láncot egy határig (1..4) skálázza, hogy elérjen egy távoli célpontot. Lásd
`rig`.

**Körvonal (Stroke).** Egy útvonal körvonala: festék, vastagság, végződés, csatlakozás, szaggatás és opcionális vastagságprofil. Lásd
PROJECT_FORMAT 6.3. szakasz.

**Kapcsolóréteg (Switch layer).** Olyan réteg, amely a gyerekei közül pontosan egyet mutat, egy indexcsatorna által kiválasztva. Képkockánkénti animációhoz
és szájalakokhoz használják. Lásd PROJECT_FORMAT 5.4. szakasz.

**Take.** Sávok megnevezett halmaza, amely egy klip tartalmát alkotja. Lásd `clipstack`.

**Tombstone.** CRDT-tömbben egy törölt elem helyén hagyott jelölő, hogy az egyidejű műveletek még elhelyezhetők legyenek. A tömörítés eltávolítja azokat a tombstone-okat, amelyeket minden replika látott. Lásd `collab`.

**Sáv (Track).** Egy csatorna kulcsainak listája extrapolációs móddal. Lásd `anim`.

**Transzformáció (Transform).** Egy réteg pozíciója, forgása, skálája, dőlése és horgonya; a lokális mátrix
`T(position) R Shear S T(-anchor)`. Lásd PROJECT_FORMAT 5.1. szakasz.

**Vektoróra (Vector clock).** Replikánkénti számlálóvektor, amely megmondja, hogy egy esemény megelőzött-e egy másikat, vagy
egyidejűek-e (Fidge, Mattern). Lásd `collab`.

**Súlyok (csont) (Weights, bone).** Az egyes csontok hatása egy horgonyra vagy csúcsra. Festhetők vagy diffúzióval
számíthatók. Lásd `weights`.

**Wobble.** A réteg geometriájához adott determinisztikus zajeltolás amplitúdóval, skálával, sebességgel és seeddel. Lásd
PROJECT_FORMAT 5.1. szakasz.

**Zhang–Suen-vékonyítás (Zhang-Suen thinning).** Párhuzamos algoritmus, amely egy bináris alakzatot egypixeles csontvázra redukál. Az
autotrace használja középvonal-követéshez. Lásd `autotrace`.
