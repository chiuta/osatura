# Întrebări pentru titular / etapa de specificare — lot 1

Format: spec · punct · ce lipsește · ce s-a implementat între timp. Completat de sesiunea de implementare (2026-11-09).

## 08 — schema JSON
- Testul de acceptanță 7 (schema generată este validă față de meta-schema dialectului 2020-12) cere un validator extern; nu există offline. S-a verificat doar că schema folosește exclusiv cuvinte-cheie ale dialectului (test automat) și că documentele reale ale proiectului trec prin validatorul-subset propriu. Rămâne de rulat un validator extern la integrare.
- Acoperirea „întregului format" (partea M): sunt descrise câmp cu câmp doar rădăcina, `camera`, `bones`, `ik`; restul listelor (layers, channels, constraints etc.) sunt „obiect liber". Cere specificații separate de câmpuri sau o decizie de a genera descrierea din tipurile serde.
- [necunoscut din spec] politica de versionare a `$id` între versiuni de format; `$id` actual este un URI de exemplu (`https://osatura.example/schema/project-v1.json`), de înlocuit cu domeniul real.
- Nu s-au folosit `unevaluatedProperties` (secțiune necitită); modul strict folosește `additionalProperties: false`.

## 05 — curbe de ritm pe segmente și trepte
- Integrarea în tipul `Ease` (care este `Copy`): curba pe segmente are o listă de puncte, deci nu poate fi variantă `Copy` simplă. Opțiuni: (a) scoate `Copy` din `Ease` (373 de utilizări în workspace), (b) bibliotecă de curbe numite la nivel de proiect și variantă `Ease::Named(index)`, (c) număr fix de puncte. Neimplementat: modulul `segease` oferă curbele și validarea, fără legătură cu `Track`/format.
- Intrarea implicită a ultimului punct când el are intrare explicită < 100 %: spec zice „cel puțin 1" (necunoscut). S-a ales: maximul dintre 1 și cea mai mare intrare anterioară; nu e acoperit de teste de acceptanță.
- Plafon ales de implementare: 4096 de puncte/trepte.

## 03 — panoramare stereo
- Conectarea la `sound mix` și câmpul `pan` pe `SoundTrack`: mixarea curentă este un graf de filtre ffmpeg (`soundtrack::track_chain`); modulul `stereopan` este o funcție pură pe eșantioane. Nu s-a ghicit nicio echivalență cu filtrele ffmpeg. Decizie necesară: mixer intern pentru WAV, sau pan-ul se aplică în afara ffmpeg. Neimplementat: câmpul `pan` și apelul din mix.
- Rampa de netezire la schimbarea lui p și mixarea canalelor > 2: [necunoscut] în spec, neimplementate. Panoramarea cu chei (interpolare liniară în timp, p. 7 din spec) este implementată ca `pan_mono_keyed`, fără test de acceptanță al specificației.

## 04 — filtru biquad
- Același punct ca la 03: nu există încă un câmp de filtre pe `SoundTrack` și nici un mixer intern; `biquad` este funcție pură pe eșantioane (mono, sau intercalat pe canale). Decizie necesară înainte de legare.
- [necunoscut din spec] rampa la schimbarea parametrilor în timp și formele cu BW/S: neimplementate. Frecvența peste 0,45·Fs dă avertisment (`warnings`), nu eroare; `Q` > 1000 este eroare (limită a proiectului, cf. spec).

## 06 — aliniere pe linia de bază
- Conectarea la `containers::Container`: modul `Align` (start/center/end/stretch) nu are variantă „baseline", iar măsurarea membrilor din containere nu cunoaște o distanță a liniei de bază. Câmpurile propuse de spec (regim de aliniere + distanța liniei de bază, opțională, pe stratul membru) nu au fost adăugate în format; modulul `baseline` este geometrie pură pe o linie. Decizie necesară: unde se citește distanța liniei de bază (câmp pe strat) și cum se integrează cu `wrap`.
- Muchia pentru linia de bază sintetizată: implementat „jos" cum spune p. 3/9 din spec (decizie de proiectare); testul 4 depinde de ea. Dacă titularul alege altă muchie, testul 4 se schimbă.
- Mai multe linii (înfășurare): `align_line` se apelează pe fiecare linie; dimensionarea liniilor cu `align-content` nu e specificată și nu e implementată.
- Un membru `Stretch` este poziționat la început; mărimea lui nu este calculată aici.

## 01 — decodare GIF
- Lipsa L-31 din spec nu mai există în cod: `animimport.rs` (comanda `import-anim`) decodează deja GIF/APNG cu crate-urile `gif` și `png`. Modulul nou `gifseq` este un decodor propriu, după standardul GIF89a, cu plafoane verificate înainte de alocare, ieșire atomică și erori distincte. Nu a fost legat de `import-anim` (ar înlocui comportamentul existent fără nevoie dovedită). Un test diferențial compară `gifseq` cu `animimport::decode_gif` pe fișiere valide (aceiași pixeli și întârzieri, inclusiv întrețesere și eliminare 1 și 3). Decizie: păstrăm ambele, sau înlocuim implementarea din `import-anim`?
- Decizii luate unde spec zice „necunoscut": metoda de eliminare 2 → transparent; durata 0 → 0 ms (fără minim); bucla nu e citită; flux fără 0x3B dar cu cadre complete → acceptat cu avertisment. Plafoane implicite alese de implementare: 4096 de cadre, 16 Mpixeli pe ecran/regiune, 128 Mpixeli total.
- Diferență față de `animimport::decode_gif` cunoscută din cod: acela folosește `Background` = transparent la fel, dar nu are plafon total de pixeli.

## 02 — decodare APNG
- Ca la 01: `animimport::decode_apng` există deja; `apngseq` este decodorul după specificație (parsare proprie a fragmentelor, CRC-32 propriu verificat cu constantele cunoscute ale lui IEND și ale șirului "123456789", reguli de ordine/secvență, compunere și eliminare proprii; doar datele comprimate ale fiecărui cadru trec prin crate-ul `png`, ca imagine PNG obișnuită, cum cere spec). Test diferențial cu `animimport` pe fișiere valide (aceiași pixeli și întârzieri). Nelegat de `import-anim`.
- Formula OVER: [necunoscut] în spec (rotunjire pe 8 biți). Implementat în virgulă mobilă, formula obișnuită cu alfa nepremultiplicat, rotunjită la cel mai apropiat întreg; testul 5 al specificației (valori exacte) trece. Alte valori pot diferi cu ±1 față de alte decodoare.
- Alte decizii: durata 0 → 0 ms; fișier fără acTL → un cadru, durată 0; reluări: `plays` (0 = infinit).

## 10 — coliziune capsulă-capsulă (fără sursă citită în analiză)
- Spec 10 are `[dedus]` peste tot; testele de acceptanță au fost verificate aritmetic înainte de implementare (1-11, inclusiv lanțul cu 4 segmente) și sunt consistente. S-a implementat din geometrie (distanța dintre segmente), fără a consulta cărțile citate.
- Neconectat la `physics.rs` (lanțuri dinamice): nu există în spec câmpul de schemă pentru raze/auto-coliziune și nici punctul de integrare cu pasul de simulare. Modulul `capsule2d` oferă `collide`, `corrections`, `self_collisions`, `chain_corrections`.
- Politica de omitere a vecinilor: implementat doar |i-j| <= 1, cum cere spec ([necunoscut] dacă lanțurile dense au nevoie de 2).

## 09 — iluminare spot și speculară
- `postfx::Light` are deja un spot (con cu margine netedă `soft`, fără exponent, fără atenuare 1/(kc+kl·d+kq·d²), fără termen speculativ). Modelul din spec (con cu masca strictă, cos^e, atenuare cu trei factori, Blinn) este implementat separat în `lighting` ca funcție pură; nu a fost legat de efectul `relight`, ca să nu se schimbe randările existente (golden-uri). Decizie: câmpuri noi pe `Light`/material și ramură nouă în `relight`?
- Ecuația completă din sursa (1) lipsește în spec (marcată [dedus]); testele 1-7 sunt consistente aritmetic și trec. Eroare adăugată de implementare: toți factorii de atenuare zero (împărțire la zero). Normala nulă: tratată ca eroare (spec lasă alegerea).
- Termenul speculativ nu este colorat de material (doar de lumină), cum lasă spec deschis.
