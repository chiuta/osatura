<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->

# Glosarul OSATURA

Termeni de domeniu folosiți în cod și în documentație, în ordine alfabetică (a termenilor englezi). „Vezi” trimite la modulul (în `osatura-core/src`, dacă nu se precizează altfel) sau la documentul în care termenul este implementat sau specificat. Documente: [PROJECT_FORMAT.md](PROJECT_FORMAT.md), [ARCHITECTURE.md](ARCHITECTURE.md). Sursele tehnicilor sunt enumerate în `CLEANROOM.md`.

**Constrângere aim.** O constrângere care rotește un os astfel încât axa lui +x să indice spre capul altui os, plus un offset. Vezi `rig`; PROJECT_FORMAT secțiunea 4.3.

**Ancoră.** Un punct de pe o cale Bezier, cu o poziție și mânere opționale de intrare și de ieșire; poate purta și greutăți de os. Vezi `path`; PROJECT_FORMAT secțiunea 6.

**Limite de unghi.** Minimul și maximul pe os ale rotației animate, în grade, impuse după canale și după fiecare lanț IK. Vezi `rig`, `engine`.

**Versiunea API.** Versiunea suprafeței Rust stabile (`api` și `prelude`), în prezent 1.4.0, verificată printr-un test de tip snapshot. Nu este același lucru cu `format` al fișierului sau cu `ABI_VERSION` din C. Vezi `api`.

**Resursă (asset).** Un fișier la care face referire un proiect (imagine, normal map, sunet). Căile trebuie să fie relative și să rămână în directorul proiectului, cu excepția cazului în care se dă `--allow-external-assets`. Vezi `assets`; PROJECT_FORMAT secțiunea 18.

**Bake.** A calcula valori în timp și a le stoca drept canale sau chei obișnuite, astfel încât să nu mai fie nevoie de instrumentul care le-a calculat. Folosit de fizică, clipuri, magnet, footlock, mașini de stări și altele. Vezi ARCHITECTURE secțiunea 4.

**Bend (îndoire).** Semnul (+1 sau -1) care alege în ce parte se pliază un lanț IK cu două oase, de exemplu cotul spre interior sau spre exterior. Vezi `rig`.

**Curbă Bezier.** O curbă cubică parametrică definită de două ancore și mânerele lor. Aplatizată în segmente de dreaptă (subdiviziune de Casteljau) pentru umplere și măsurare. Vezi `path`.

**Binding.** O regulă de runtime care mapează o valoare externă (o variabilă de joc, un slider) printr-o expresie, un remap și o netezire pe o proprietate sau pe o intrare a unei mașini de stări. Vezi `binding`.

**Mod de amestecare (blend mode).** Modul în care culoarea unui strat se combină cu ce se află dedesubt (multiply, screen, overlay etc.). Formulele urmează W3C Compositing and Blending Level 1. Vezi `raster`, `fraster`.

**Blend tree.** O amestecare 1-D a mai multor clipuri controlată de o intrare numerică, folosită în interiorul unei mașini de stări. Vezi `machine`.

**Blob.** Un cerc al unui strat de lichid (metaball). Vezi `fx`.

**Os.** Un segment denumit al scheletului, cu o poziție a capului, lungime, unghi de repaus și părinte opțional. Oasele controlează geometria cu skin și straturile atașate. Vezi `rig`; PROJECT_FORMAT secțiunea 4.

**Strat boolean.** Un strat care combină căi vectoriale prin reuniune, intersecție, diferență sau xor. Prima cale este subiectul, iar celelalte sunt reunite drept clip. Vezi `fx`.

**Bounce (easing).** O curbă de easing a unei mingi care cade și revine cu restituția 0,6 și trei reveniri. O derivare originală din căderea liberă. Vezi `anim`.

**Casetă de încadrare (bounding box).** Cel mai mic dreptunghi aliniat la axe din jurul geometriei; folosit pentru aliniere, selecție și potrivirea normal map-urilor. Vezi `layout`, `pick`.

**Deformator cage.** Un deformator în care conținutul din interiorul unui poligon închis urmează vârfurile poligonului folosind coordonate cu valoare medie. Vezi `cage`; vezi și MVC.

**Spline Catmull-Rom.** Un spline cubic care trece prin punctele sale de control. Easing-ul `smooth` folosește o variantă neuniformă prin cheile vecine. Vezi `anim`, netezirea `liquid` din `fx`.

**Canal.** O valoare animată: o țintă, o proprietate, o sursă (chei, expresie, zgomot sau condus) și un mod de amestecare și o pondere. Canalele rulează în ordinea listei. Vezi PROJECT_FORMAT secțiunea 8.

**CIE76.** Diferența de culoare originală, distanța euclidiană în CIE L*a*b*. Ieftină, mai puțin precisă decât CIEDE2000. Vezi `percep`.

**CIEDE2000.** Formula de diferență de culoare CIE 2000 (CIE TR 142-2001), implementată așa cum este tipărită în Sharma, Wu și Dalal (2005) și verificată cu perechile lor de test publicate. Folosită de CI-ul de animație pentru a judeca schimbarea vizibilă. Vezi `percep`, `ci`; `docs/CI.md`.

**Clean room.** Regula de dezvoltare conform căreia codul este scris pe baza standardelor publice și a metodelor din manuale, fără cod copiat și fără reproducerea formatelor sau a numelor altor instrumente. Fiecare tehnică este consemnată în `CLEANROOM.md`.

**Clip.** Un take reutilizabil de piste de animație care poate fi plasat pe timeline cu viteză, bucle, pondere, fade-uri și un mod de amestecare. Vezi `clipstack`; PROJECT_FORMAT secțiunea 11.1.

**Cloth.** Un deformator de tip grilă ale cărui vârfuri se mișcă exact cu propriile deplasări. Este și simularea de dinamică bazată pe poziții care face bake acelor deplasări. Vezi `cloth`.

**Constrângere.** O regulă aplicată după IK care limitează un os: aim, copy rotation sau pin. Vezi `rig`.

**Container (layout).** Un grup de layout unidimensional în stil flexbox, cu spațiere, padding, justificare, aliniere și wrapping, care poziționează straturi și alte containere. Vezi `containers`.

**Constrângere copy-rotation.** O constrângere care face ca un os să corespundă rotației în lume a altui os. Vezi `rig`.

**CRDT.** Tip de date replicat fără conflicte: o structură pe care replicile o pot edita independent și o pot îmbina la același rezultat fără coordonare. OSATURA folosește un CRDT bazat pe operații. Vezi `collab`; ARCHITECTURE secțiunea 10. Termeni înrudiți: registru LWW, RGA, ceas Lamport, ceas vectorial, tombstone.

**Easing Bezier cubic.** O curbă de easing în stil CSS definită de două puncte de control între (0,0) și (1,1). Vezi `anim`.

**Deformator.** Un modificator de formă aplicat unui strat vectorial sau de imagine în spațiul de repaus: lattice, spine, pin, cage sau cloth. Vezi PROJECT_FORMAT secțiunea 7.1.

**Triangulare Delaunay.** O triangulare a punctelor în care niciun punct nu se află în interiorul cercului circumscris al vreunui triunghi, ceea ce evită triunghiurile subțiri. Construită cu algoritmul Bowyer-Watson pentru mesh-urile de imagine. Vezi `meshtrace`.

**Adâncime de câmp (DoF).** Estompare care crește odată cu distanța dintre `parallax` al unui strat și `focus` al camerei. Aplicată doar de comenzile `dof` și `dof-export`. Vezi `dof`.

**Determinism.** Proprietatea că aceeași intrare dă aceeași ieșire. Evaluarea unui cadru este pură; randarea este identică octet cu octet pe o singură mașină și un singur build, neverificată între platforme. Vezi ARCHITECTURE secțiunea 6.

**Dope sheet.** O vedere de timeline care arată cheile ca marcaje pe canal și le editează în timp. Vezi `timeline`, `keyops`.

**Cheie condusă (driven key).** Un canal a cărui intrare este valoarea altei proprietăți, nu timpul, de exemplu unghiul cotului la dimensiunea unei umflături. Vezi `drivers`; PROJECT_FORMAT secțiunea 8.1.

**Easing.** Funcția de modelare aplicată segmentului care începe la o cheie. Vezi `anim`; PROJECT_FORMAT secțiunea 8.4.

**Editor.** Obiectul care deține un proiect și istoricul lui de anulare și aplică operațiile. Vezi `edit`; ARCHITECTURE secțiunea 5.

**Emițător.** Un strat care produce particule ca funcție în formă închisă de timp. Vezi `fx`.

**Motor (engine).** Forma validată, cu nume rezolvate, a unui proiect, care evaluează cadre. Vezi `engine`.

**Plic (envelope, collab).** Mesajul care poartă operații între replici: id-ul replicii, număr de secvență, timp Lamport, dependențe și operații. Vezi `collab`.

**Evaluare.** Calcularea poziției, geometriei și camerei pentru un cadru dintr-un proiect. O funcție pură de cadru. Vezi PROJECT_FORMAT secțiunea 16.

**FABRIK.** Forward And Backward Reaching Inverse Kinematics: un solver IK iterativ care trage alternativ lanțul spre țintă dinspre vârf și apoi îl reancorează la rădăcină. Folosit pentru lanțuri de mai mult de două oase. Vezi `rig`.

**FK.** Cinematică directă: calcularea transformărilor în lume ale oaselor din transformările părinților lor și din propriile unghiuri și deplasări. Vezi `rig`, `engine`.

**Versiunea formatului.** Câmpul întreg `format` al unui proiect, în prezent 1. Fișierele mai noi sunt respinse. Vezi PROJECT_FORMAT secțiunea 19.

**Cadru (frame).** (1) Un pas de timp al timeline-ului, `0..frames-1`. (2) Structura `Frame`: rezultatul evaluării unui pas (oase, geometrie, cameră, flag-uri). Vezi `engine`.

**Glue.** O legătură care face ca vârfurile unui mesh să urmeze vârfurile altuia (snap, rigid sau offset). Vezi `links`.

**Mâner de gradient.** Un punct de control al unui gradient liniar sau radial care poate fi animat cu `grad_dx` și `grad_dy`. Vezi PROJECT_FORMAT secțiunea 8.2.

**Strat grup.** Un strat care conține alte straturi și le transmite transformarea sa. Vezi `project`.

**Mâner (handle).** Un punct de control Bezier atașat unei ancore. Coordonate absolute în fișier. Vezi `path`.

**HOG.** Histogramă de gradienți orientați, o trăsătură de formă locală (Dalal și Triggs 2005) folosită de detectorul de fețe. Vezi `facedet`.

**IK.** Cinematică inversă: calcularea unghiurilor oaselor astfel încât vârful unui lanț să atingă o țintă. Vezi `rig`. Termeni înrudiți: FABRIK, soft IK, stretch IK, bend.

**IK mix.** Amestecul dintre poziția FK (0) și rezultatul IK (1) pentru un lanț. Animat cu `ik_mix`. Vezi PROJECT_FORMAT secțiunea 4.2.

**Mesh de imagine.** O grilă sau un mesh liber de triunghiuri peste un strat de imagine, folosit pentru greutățile oaselor și deformatori. Vezi PROJECT_FORMAT secțiunea 5.8.

**Cheie (keyframe).** Un triplet (cadru, valoare, easing) pe o pistă. Easing-ul modelează segmentul care începe la cheie. Vezi `anim`.

**L*a*b*.** Spațiul de culoare CIE 1976 conceput astfel încât distanțele să aproximeze diferențele percepute. Vezi `percep`.

**Ceas Lamport.** Un contor logic care ordonează evenimentele într-un sistem distribuit fără un ceas comun (Lamport 1978). Vezi `collab`.

**Lattice.** Un deformator cu o grilă de puncte de control peste un dreptunghi, ale cărui deplasări îndoaie conținutul prin amestecare Bezier-Bernstein. Vezi PROJECT_FORMAT secțiunea 7.1.

**Strat (layer).** Un nod din arborele scenei: vector, grup, switch, liquid, boolean, emitter sau imagine. Vezi PROJECT_FORMAT secțiunea 5.

**Efect de strat.** Un filtru pe strat (blur, drop shadow, tint, fill) randat pe un canvas în afara ecranului. Vezi `layerfx`.

**Nivel de detaliu (LOD).** O versiune simplificată a unui proiect, produsă sub o limită de eroare declarată în pixeli, pentru previzualizare rapidă sau rezultat mic. Vezi `lod`, `osatura lod`.

**Linter.** Verificatorul consultativ (`osatura lint`) care raportează probleme de calitate a animației cu coduri precum L011 și L012. Separat de validare. Vezi `lint`.

**Strat liquid.** Un strat de metaball-uri al cărui contur este izocontur al unui câmp de sumă de influențe. Vezi `fx`.

**Registru LWW.** Registru last-writer-wins: o celulă CRDT în care supraviețuiește scrierea cu cea mai mare amprentă de timp logică. Folosit pentru cheile obiectelor. Vezi `collab`.

**Magnet.** Un instrument de bake care trage pozițiile spre un punct cu atenuare smoothstep. Vezi `magnet`.

**Marching squares.** Un algoritm care extrage linii de contur dintr-un câmp scalar eșantionat pe o grilă. Folosit pentru contururile liquid și trasarea bitmap-urilor. Vezi `fx`, `trace`, `meshtrace`.

**Marker.** Un cadru sau o regiune de cadre denumită pe timeline. Mașinile de stări pot folosi o regiune de marker ca sursă. Vezi `keyops`; PROJECT_FORMAT secțiunea 10.2.

**Mască.** Un strat al cărui alfa limitează vizibilitatea altui strat; poate fi inversat. Un strat ascuns poate fi totuși mască. Vezi `raster`.

**Merge (CRDT).** Combinarea stărilor a două replici astfel încât ambele să convergă la același document. Vezi `collab`.

**Mesh lock.** O legătură care face ca un mesh urmăritor să urmeze un mesh gazdă cu o intensitate 0..1. Vezi `links`.

**Metaball.** Un blob a cărui influență de câmp se amestecă cu vecinii astfel încât blob-urile apropiate fuzionează. Vezi stratul liquid.

**Morph (blend shape).** Un set alternativ de ancore pentru o cale, aplicat aditiv cu o pondere. Vezi PROJECT_FORMAT secțiunea 6.1.

**MVC.** Coordonate cu valoare medie (Floater 2003): coordonate barocentrice generalizate pentru un punct din interiorul unui poligon, folosite de deformatorul cage. Vezi `cage`.

**Non-distructiv.** Lasă intactă munca existentă. Bake-rii adaugă canale în loc să rescrie chei, iar editorul înregistrează operații inverse. Vezi ARCHITECTURE secțiunile 4 și 5.

**Normal map.** O imagine care stochează direcția suprafeței ca RGB, folosită de efectul `relight` și de iluminarea pe strat. Vezi `postfx`; PROJECT_FORMAT secțiunea 7.3.

**Op.** O editare exprimată ca `set`, `insert`, `remove` sau `batch` pe un pointer JSON RFC 6901; are o inversă pentru anulare. Vezi `edit`.

**OT.** Transformare operațională: o alternativă la CRDT care rescrie operațiile concurente una față de alta, de obicei cu un server central. OSATURA nu o folosește; folosește un CRDT cu un secvențiator central. Vezi ARCHITECTURE secțiunea 10.

**Parallax.** Un factor pe strat care scalează mișcarea camerei, astfel încât straturile îndepărtate se mișcă mai puțin. Este și valoarea de adâncime folosită pentru adâncimea de câmp. Vezi PROJECT_FORMAT secțiunea 10.1.

**Particulă.** Un punct cu viață scurtă emis de un emițător; poziția lui este o funcție în formă închisă de timp. Vezi `fx`.

**Cale (țintă).** Numele separat prin slash al unui strat imbricat, precum `parent/child`; sub-țintele folosesc `::` ca în `layer::path::morph`. Vezi PROJECT_FORMAT secțiunea 5.

**Cale (vectorială).** O secvență de ancore, deschisă sau închisă, cu umplere, contur și morph-uri opționale. Vezi `path`.

**Constrângere pin.** O constrângere care alunecă capul unui os la capul altui os. Vezi `rig`.

**Pin warp.** Un deformator în care pinii trași mută grafica lin într-o rază, folosind o funcție de bază radială cu suport compact (Wendland 1995). Vezi `pinwarp`.

**Player.** Învelișul de runtime din fațadă care adaugă mașini de stări și binding-uri unei animații. Vezi `api`.

**Preorder.** Ordinea de vizitare a arborelui de straturi: un nod, apoi copiii lui. Evaluarea o folosește pentru ca părinții să fie calculați primii.

**Proxy.** O imagine cu detaliu redus folosită în timpul scrub-ului, construită dintr-un nivel de detaliu. Vezi `scrub`, `lod`.

**QEM.** Metrica erorii cuadratice (Garland și Heckbert): o măsură a erorii geometrice cauzate de colapsarea unui vârf, folosită pentru simplificarea mesh-urilor. Vezi `lod`.

**Raster.** O imagine de pixeli sau modulul care randează un cadru în pixeli cu tiny-skia. Vezi `raster`.

**RDP.** Ramer-Douglas-Peucker: simplificare recursivă a liniilor care păstrează punctele mai îndepărtate de coardă decât o toleranță. Vezi `trace`, `sizefit`.

**Strat de referință.** Un strat marcat `reference`; ascuns, cu excepția cazului în care editorul arată referințele. Vezi PROJECT_FORMAT secțiunea 5.1.

**Remap.** O mapare afină a unui interval, cu o curbă și un clamp opționale, aplicată unei valori de binding.
Vezi `binding`.

**Replică (collab).** Copia documentului a unui participant în CRDT. Vezi `collab`.

**Poziție de repaus (rest pose).** Transformările oaselor fără animație aplicată; matricile de skin sunt `world * inverse(rest
world)`. Vezi `rig`.

**RGA.** Array replicat extensibil (Roh et al. 2011): un CRDT de secvență în care fiecare element are un id unic, iar ștergerile lasă tombstone-uri. Folosit pentru array-urile JSON. Vezi `collab`.

**Rig.** Scheletul și ajutoarele lui: oase, lanțuri IK, constrângeri și arcuri. Vezi PROJECT_FORMAT secțiunea 4.

**Scrub.** Tragerea capului de redare; studio memorează cadrele în cache după revizie și folosește proxy-uri. Vezi `scrub`.

**Skew.** Un unghi de forfecare în transformarea unui strat. Vezi PROJECT_FORMAT secțiunea 5.1.

**Skin (animație).** O variantă denumită de straturi, oase, constrângeri sau IK care poate fi activată într-un grup, cum ar fi costumele. Vezi `skins`.

**Skinning.** Deformarea geometriei prin amestecarea transformărilor oaselor, cu greutăți pe ancoră sau pe vârf. OSATURA normalizează greutățile și tratează greutățile goale ca statice. Vezi `engine`, `weights`.

**Soft IK.** O opțiune IK care atenuează extensia aproape de întinderea completă pentru a evita redresarea bruscă. Interval de la 0 la 0,9. Vezi `rig`.

**SPDX.** Software Package Data Exchange: un mod standard de a declara licența unui fișier, ca în
<!-- REUSE-IgnoreStart -->
`SPDX-License-Identifier: MPL-2.0`. Fiecare fișier sursă poartă una. Vezi `LICENSING.md`.
<!-- REUSE-IgnoreEnd -->

**Deformator spine.** Un deformator care îndoaie geometria de-a lungul unei curbe, proiectând ancorele pe o curbă de repaus și reatașându-le la curba deformată. Vezi PROJECT_FORMAT secțiunea 7.1.

**Arc (os).** Un arc amortizat de ordinul doi care face ca rotația unui os să întârzie și să depășească ținta; se face bake la încărcare, astfel încât orice cadru poate fi evaluat. Vezi PROJECT_FORMAT secțiunea 4.4.

**SSIM.** Indice de similaritate structurală (Wang et al. 2004): o măsură a calității imaginii din medii locale, variante și covarianță. OSATURA folosește o versiune pe blocuri. Vezi `percep`.

**Scenă (stage).** Spațiul de coordonate al imaginii de ieșire: pixeli, +x spre dreapta, +y în jos.

**Mașină de stări.** Un graf de stări de animație cu tranziții conduse de intrări. Există o versiune cu bake, cu jurnal de evenimente, și o versiune de runtime. Vezi `statemachine`, `machine`.

**Easing step.** Un easing care păstrează valoarea cheii până la următoarea cheie. Folosit pentru straturile switch. Vezi `anim`.

**Stretch IK.** O opțiune IK care scalează lanțul până la o limită (1..4) pentru a atinge o țintă îndepărtată. Vezi `rig`.

**Contur (stroke).** Marginea unei căi: culoare, lățime, cap, join, dash și un profil de lățime opțional. Vezi PROJECT_FORMAT secțiunea 6.3.

**Strat switch.** Un strat care arată exact unul dintre copiii săi, ales de un canal de index. Folosit pentru animația cadru cu cadru și pentru formele gurii. Vezi PROJECT_FORMAT secțiunea 5.4.

**Take.** Un set denumit de piste care formează conținutul unui clip. Vezi `clipstack`.

**Tombstone.** Un marcaj lăsat într-un array CRDT în locul unui element șters, pentru ca operațiile concurente să poată fi totuși plasate. Compactarea elimină tombstone-urile pe care le-a văzut fiecare replică. Vezi `collab`.

**Pistă (track).** Lista de chei a unui canal, cu un mod de extrapolare. Vezi `anim`.

**Transformare.** Poziția, rotația, scala, skew-ul și ancora unui strat; matricea locală este `T(position) R Shear S T(-anchor)`. Vezi PROJECT_FORMAT secțiunea 5.1.

**Ceas vectorial.** Un vector de contoare pe replică care spune dacă un eveniment s-a petrecut înaintea altuia sau dacă sunt concurente (Fidge, Mattern). Vezi `collab`.

**Greutăți (os).** Influența fiecărui os asupra unei ancore sau a unui vârf. Pot fi pictate sau calculate prin difuzie. Vezi `weights`.

**Wobble.** O deplasare deterministă prin zgomot adăugată geometriei unui strat, cu amplitudine, scală, rată și seed. Vezi PROJECT_FORMAT secțiunea 5.1.

**Subțiere Zhang-Suen.** Un algoritm paralel care reduce o formă binară la un schelet de un pixel. Folosit de autotrace pentru trasarea liniei centrale. Vezi `autotrace`.
