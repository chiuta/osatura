# Funcție: decodare_secvență_apng

Sursă publică: *APNG Specification*, wiki Mozilla: https://wiki.mozilla.org/APNG_Specification (secțiunile despre acTL, fcTL, fdAT, ordinea fragmentelor, dispose_op, blend_op); referință de bază pentru structura fragmentelor, CRC-32 și compunerea alfa: *Portable Network Graphics (PNG) Specification*, W3C, https://www.w3.org/TR/png-3/ (neconsultată în această sesiune, vezi mai jos).
Dimensiune estimată: M · Status: candidat · Data: 2026-11-09 · Limitare acoperită: L-31 / R-037 (decodare APNG ca strat „secvență de imagini")
Citire sursă în sesiune: specificația APNG da (extras prin instrument de citire web); specificația PNG nu. Afirmațiile care depind de PNG sunt marcate [dedus].

## Scop
Transformă un fișier PNG animat într-o listă de cadre complete RGBA cu duratele lor, pentru folosirea ca strat „secvență de imagini".

## Intrări / ieșiri
- Intrare: șir de octeți al unui fișier PNG (animat sau nu). Plafoane configurabile: cadre, pixeli per cadru, total pixeli.
- Ieșire: dimensiunile pânzei (din antetul PNG), numărul de reluări (0 = infinit), listă de cadre RGBA de dimensiunea pânzei cu durată în milisecunde. Un PNG fără animație dă un singur cadru, fără reluări definite.
- Eroare: mesaj text cu motivul.

## Comportament observabil
1. Fișierul începe cu semnătura PNG de 8 octeți, apoi fragmente (lungime, tip, date, CRC) [dedus, PNG]. CRC-ul fiecărui fragment se verifică; nepotrivire → eroare [dedus, PNG].
2. Fragmentul de control al animației (acTL) are doi întregi fără semn pe 4 octeți: număr de cadre și număr de reluări [observat]. Numărul de cadre 0 este invalid, 1 este valid; trebuie să fie egal cu numărul de fragmente de control de cadru (fcTL). Reluări 0 = infinit; altfel animația se oprește pe ultimul cadru după ultima trecere [observat]. acTL apare înaintea primului IDAT [observat].
3. Fragmentul de control de cadru (fcTL), 26 de octeți: număr de secvență (4), lățime (4), înălțime (4), decalaj x (4), decalaj y (4), numitor/numărător de întârziere (câte 2: delay_num, delay_den), dispose_op (1), blend_op (1) [observat; ordinea numitor/numărător exact: delay_num apoi delay_den].
4. Durata cadrului = delay_num / delay_den secunde; dacă delay_den este 0 se folosește 100 [observat]. Dacă delay_num este 0, cadrul următor se afișează cât mai repede; vizualizatoarele pot impune un minim [observat] — minimul proiectului: vezi Necunoscute.
5. Imaginea implicită (IDAT) face parte din animație ca prim cadru DOAR dacă un fcTL precede primul IDAT; în acest caz fcTL-ul are decalaje 0 și dimensiuni egale cu cele din antet [observat]. Dacă primul fcTL apare după toate IDAT, imaginea implicită NU se afișează în animație [observat].
6. Datele unui cadru ulterior sunt fragmente fdAT: număr de secvență de 4 octeți urmat de date, în rest ca IDAT; datele comprimate ale cadrului sunt concatenarea datelor din toate fdAT-urile sale; adâncimea de biți, tipul de culoare, compresia, filtrul, întrețeserea și paleta sunt cele ale imaginii implicite [observat].
7. Numerele de secvență sunt comune pentru fcTL și fdAT: primul fcTL este 0, apoi cresc cu 1, fără goluri sau duplicate [observat]. Fragmente APNG în ordine greșită → eroare [observat].
8. Regiunea cadrului: x ≥ 0, y ≥ 0, lățime > 0, înălțime > 0, x + lățime ≤ lățimea antetului, y + înălțime ≤ înălțimea antetului [observat].
9. Pânza (tampon de ieșire) începe complet transparentă (negru transparent) [dedus din formularea lui dispose_op = fundal; starea inițială nu e enunțată în extrasul citit].
10. Pentru fiecare cadru: (a) se decodează regiunea (ca imagine PNG obișnuită de lățime × înălțime, cu filtrele PNG) [dedus]; (b) dacă blend_op = 0 (SOURCE), pixelii cadrului, inclusiv alfa, înlocuiesc regiunea; dacă blend_op = 1 (OVER), cadrul se compune alfa peste pânză [observat]; formula OVER este cea din secțiunea „Alpha Channel Processing" a PNG, a doua variantă [observat: spec trimite acolo; formula nu a fost citită, vezi Necunoscute]; (c) se emite o instantanee a pânzei întregi drept cadru de ieșire; (d) se aplică dispose_op: 0 (NONE) = pânza rămâne; 1 (BACKGROUND) = regiunea cadrului devine transparentă (negru transparent); 2 (PREVIOUS) = regiunea revine la conținutul de dinaintea acestui cadru [observat]. La primul fcTL, dispose_op = 2 se tratează ca 1 [observat].

## Formate / protocoale
Fragmente PNG cu tipurile acTL, fcTL, fdAT așa cum sunt descrise mai sus; întregii sunt big-endian [dedus, PNG].

## Cazuri limită și erori
- acTL cu număr de cadre 0 → eroare; număr de cadre diferit de numărul de fcTL → eroare.
- Număr de secvență cu gol, duplicat sau decrescător → eroare.
- fcTL a cărui regiune iese din pânză → eroare.
- Cadru fără niciun fdAT (și care nu este imaginea implicită) → eroare [observat: „each frame needs at least one fdAT"].
- acTL după primul IDAT → eroare.
- Format de pixel diferit între cadre → imposibil prin construcție; dacă datele decompresate au altă lungime decât cea așteptată pentru regiune → eroare [dedus].
- PNG fără acTL → un singur cadru, durată 0 [dedus, decizie].
- Depășirea plafoanelor → eroare înainte de alocare.
- delay_den = 0 și delay_num = 0 → durată 0.

## Teste de acceptanță
1. acTL: num_frames = 2, num_plays = 0 → ieșire cu reluări „infinit" și 2 cadre.
2. fcTL cu delay_num = 1, delay_den = 0 → 10 ms; delay_num = 1, delay_den = 4 → 250 ms; delay_num = 3, delay_den = 100 → 30 ms.
3. Ordinea fragmentelor IHDR, acTL, IDAT, fcTL(0), fdAT(1), fcTL(2), fdAT(3) cu num_frames = 2 → 2 cadre, imaginea implicită NU este cadru. Aceeași secvență cu fcTL(0) înaintea IDAT, urmat de fcTL(1), fdAT(2) → 2 cadre, primul este imaginea implicită.
4. Primul fcTL cu dispose_op = 2 pe o pânză 1×1 → după cadrul 1, pânza este transparentă (se tratează ca BACKGROUND).
5. blend_op = OVER: pânză cu pixelul (255,0,0,255); cadru cu pixelul (0,0,255,alfa 128) → pixel rezultat (127,0,128,255). Cu blend_op = SOURCE același cadru → (0,0,255,128). (Calcul: culoare = sursă·128/255 + destinație·127/255.)
6. Eroare: numerele de secvență 0, 2 → eroare; num_frames = 0 → eroare; x_offset + lățime > lățime antet → eroare; un fdAT lipsă → eroare.
7. Eroare CRC: un octet modificat în datele unui fcTL → eroare.
8. PNG obișnuit (fără acTL) 2×2 → 1 cadru 2×2.

## Necunoscute
- [necunoscut] Formula exactă de compunere OVER pe 8 biți (rotunjire): sursa APNG trimite la PNG „varianta a doua"; testul 5 folosește valori la care rezultatul e exact. Aproximările pe 8 biți se stabilesc din specificația PNG (de citit de implementator).
- [necunoscut] Durata minimă pentru delay 0 în proiect.
- [necunoscut] Starea pânzei înainte de primul cadru (aici: transparentă, [dedus]).
- [necunoscut] Tratarea fragmentelor auxiliare între fdAT: permise [observat]; ignorate.
- Brevete: nu s-a făcut o verificare; formatul este documentat public.

## Checklist de filtru (verificare.md)
Spec: 02-apng-decodare-secventa   Data: 2026-11-09
- [x] Nicio linie de cod, pseudocod sau asamblare copiată
- [x] Niciun nume intern de funcție, variabilă sau clasă din țintă
- [x] Niciun text de UI, asset sau branding din țintă
- [x] Algoritmii sunt descriși prin intrare → ieșire, nu prin structura codului țintei
- [x] Afirmațiile sunt marcate [observat] / [dedus] / [necunoscut]
- [ ] Licența/EULA țintei verificată și notată în `1-analiza/jurnal.md` (rând adăugat; licența paginii notată „neverificată în sesiune")
- [ ] Specificația nu depinde de brevete sau secrete comerciale (nu s-a făcut verificare de brevete)
- [ ] Aprobată de mine înainte de predare
