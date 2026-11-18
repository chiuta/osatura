# Funcție: export_animație_noduri_gltf

Sursă publică: *glTF 2.0 Specification*, Khronos Group, https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html (secțiunea „Animations" și anexa de interpolare); ghidul oficial Khronos, capitolul „Animations": https://github.khronos.org/glTF-Tutorials/gltfTutorial/gltfTutorial_007_Animations.html
Dimensiune estimată: M · Status: candidat · Data: 2026-11-09 · Limitare acoperită: L-30 (exportul glTF animează doar oasele și ponderile de morfare; transformările de strat sunt cadrul 0)
Citire sursă în sesiune: doar ghidul oficial (capitolul „Animations", extras prin instrument de citire web). Specificația normativă NU a fost citită în sesiune; afirmațiile care depind de ea sunt [dedus] și trebuie confirmate de implementator.

## Scop
Exportă animația transformărilor (poziție, rotație, scară) ale straturilor/nodurilor în fișierul glTF ca animații standard, în loc să le înghețe la cadrul 0.

## Intrări / ieșiri
- Intrare: pentru fiecare strat cu cheie de transformare, serii de valori-cheie (timp în secunde, poziție 2D, unghi, scară 2D) și tipul de interpolare pe segment (menținere, liniar, cubic cu tangente).
- Ieșire: în documentul glTF, o animație cu eșantionatoare (intrare = timpii cheilor, ieșire = valorile, mod de interpolare) și canale (eșantionator, nod țintă, proprietate: poziție, rotație, scară) [observat].
- Erori: serie fără chei, timpi nestrict crescători, valori nefinite.

## Comportament observabil
1. O animație conține două liste: eșantionatoare și canale. Un eșantionator are: accesor de intrare (timpii cheilor), accesor de ieșire (valori), mod de interpolare. Un canal are: indice de eșantionator și țintă (nod + proprietate) [observat].
2. Proprietățile țintă cuprind translație, rotație, scară (și ponderi de morfare, în afara acestei specificații) [observat pentru primele trei].
3. Mai multe canale pot viza același nod (ex.: rotație și translație) [observat].
4. Moduri de interpolare: STEP = se aplică valoarea cheii anterioare; LINEAR = interpolare cu t = (timp curent − timp anterior) / (timp următor − timp anterior); vectori și scalari cu interpolare liniară, rotații (quaternioni) cu interpolare sferică (slerp), luând drumul cel mai scurt (al doilea quaternion se neagă dacă produsul scalar este negativ), cu revenire la amestec liniar normalizat pentru quaternioni aproape identici; CUBICSPLINE = fiecare cheie are trei elemente de ieșire în ordinea: tangentă de intrare, valoare, tangentă de ieșire [observat].
5. CUBICSPLINE: tangentele se înmulțesc cu Δt = timp următor − timp anterior; tangenta anterioară = Δt × tangenta de ieșire a cheii anterioare; tangenta următoare = Δt × tangenta de intrare a cheii următoare; valoarea = (2t³ − 3t² + 1)·P_anterior + (t³ − 2t² + t)·tangenta_anterioară + (−2t³ + 3t²)·P_următor + (t³ − t²)·tangenta_următoare; tangenta de intrare a primei chei și cea de ieșire a ultimei sunt ignorate [observat].
6. Pentru straturi 2D: translația are z = 0; scara are z = 1 [dedus]. Rotația în plan cu unghiul θ în jurul axei Z este quaternionul (x, y, z, w) = (0, 0, sin(θ/2), cos(θ/2)) [dedus matematic; ordinea componentelor XYZW din specificație, neconfirmată în sesiune].
7. Timpii cheilor sunt în secunde [observat în exemplul din ghid], strict crescători [dedus, specificație neconsultată]; accesorul de intrare declară minimul și maximul [dedus].
8. Mapare proiect → glTF: cheile cu interpolare „menținere" → STEP; liniar → LINEAR; Bézier cubică a proiectului nu are echivalent exact → eșantionare în chei LINEAR la fiecare cadru sau în CUBICSPLINE cu tangente calculate [dedus, decizie].
9. Unghiurile se desfășoară (fără salt la ±180°) înainte de conversia în quaternioni dacă rotația depășește jumătate de tură între chei, adăugând chei intermediare, fiindcă slerp ia drumul cel mai scurt [dedus].

## Formate / protocoale
Structura JSON glTF „animations": { samplers: [ { input, output, interpolation } ], channels: [ { sampler, target: { node, path } } ] } așa cum o descrie ghidul [observat]; accesori și vizualizări de buffer conform specificației (neconsultate).

## Cazuri limită și erori
- Un singur cheie → eșantionator cu o intrare, fără interpolare [dedus].
- Chei cu timp egal → eroare.
- Opacitate, culoare, formă: în afara acestei specificații (pe glTF de bază nu există țintă; extensii separate) [dedus].
- Quaternioni neunitari → se normalizează la scriere.

## Teste de acceptanță
Calcule din formulele de mai sus (toleranță 1e-5).
1. LINEAR, chei la 0 s și 2 s cu translație (0,0) și (10,0), t = 1 s → (5,0).
2. STEP, aceleași chei, la 1,99 s → (0,0).
3. CUBICSPLINE, două chei: P0 = 0, tangentă de ieșire 10 (cheia 0); P1 = 10, tangentă de intrare 0 (cheia 1); Δt = 2 s; la t normalizat 0,5 (timp 1 s) → 0,5·0 + 0,125·(2·10) + 0,5·10 + (−0,125)·(2·0) = 7,5.
4. CUBICSPLINE cu toate tangentele 0, același set, t = 0,5 → 5,0.
5. Rotație 90° în plan → quaternion (0, 0, 0,707107, 0,707107).
6. Slerp între identitate (0,0,0,1) și rotație 90° în jurul Z, la jumătate → rotație 45°: (0, 0, 0,382683, 0,923880).
7. Slerp cu al doilea quaternion cu semn opus (0,0,−0,707107,−0,707107) → același rezultat ca testul 6 (drum cel mai scurt).
8. Eroare: timpi 1,0 și 1,0 → eroare; serie goală → eroare.

## Necunoscute
- [necunoscut] Cerințele normative ale accesorilor (tip de componentă, min/max, aliniere) — de citit din specificație.
- [necunoscut] Ordinea componentelor quaternionului în date și cerința de normalizare.
- [necunoscut] Cum se exportă opacitatea (extensie dedicată vs. omisă).
- [necunoscut] Comportamentul playerelor pentru animații cu mai multe canale pe același nod și chei neuniforme.

## Checklist de filtru (verificare.md)
Spec: 07-gltf-animatie-transformari-strat   Data: 2026-11-09
- [x] Nicio linie de cod, pseudocod sau asamblare copiată
- [x] Niciun nume intern de funcție, variabilă sau clasă din țintă
- [x] Niciun text de UI, asset sau branding din țintă
- [x] Algoritmii sunt descriși prin intrare → ieșire, nu prin structura codului țintei
- [x] Afirmațiile sunt marcate [observat] / [dedus] / [necunoscut]
- [ ] Licența/EULA țintei verificată și notată în `1-analiza/jurnal.md` (rând adăugat; licența documentației Khronos neverificată în sesiune)
- [ ] Specificația nu depinde de brevete sau secrete comerciale (nu s-a făcut verificare de brevete)
- [ ] Aprobată de mine înainte de predare
