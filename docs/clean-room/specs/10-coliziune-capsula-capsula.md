# Funcție: coliziune_capsulă_capsulă_2d

Sursă publică: C. Ericson, *Real-Time Collision Detection*, Morgan Kaufmann, 2005, secțiunea 5.1.9 „Closest points of two line segments" (și capitolul despre capsule, 4.3 / 5.9); D. Eberly, „Distance Between Lines, Rays, and Segments" (nota tehnică Geometric Tools, publică, 1999–2000). Noțiunile (distanța dintre două segmente, capsulă = mulțimea punctelor la distanță ≤ r de un segment) sunt geometrie elementară.
Dimensiune estimată: M · Status: candidat · Data: 2026-11-09 · Limitare acoperită: L-47 („fără capsulă-capsulă și fără auto-coliziune")
Citire sursă în sesiune: NU. Sursele sunt cărți/note neaccesibile în sesiune; tot conținutul este [dedus] din geometrie și verificabil algebric, iar testele au rezultate calculabile manual. Nu se pretinde [observat] nicăieri.

## Scop
Detectează și rezolvă ciocnirea a două capsule 2D (segment cu rază), pentru lanțuri dinamice: coliziune între un lanț și obstacole de tip capsulă și auto-coliziune între segmente neînvecinate ale aceluiași lanț.

## Intrări / ieșiri
- Intrare: capsula A (capete P0, P1, raza rA > 0), capsula B (Q0, Q1, rB > 0); masele inverse wA, wB ≥ 0 (0 = fixă).
- Ieșire: indicator de coliziune; adâncimea de pătrundere δ = rA + rB − d (unde d = distanța minimă dintre segmente), normala unitară n de la A spre B, punctele cele mai apropiate de pe cele două segmente; corecția de poziție pentru fiecare capsulă.
- Erori: coordonate sau raze nefinite; rază ≤ 0.

## Comportament observabil
1. Coliziune dacă și numai dacă d < rA + rB (strict) [dedus; contactul tangent nu este coliziune].
2. Distanța minimă dintre două segmente: se parametrizează fiecare segment cu un parametru în [0,1]; se caută perechea de parametri care minimizează distanța, cu limitare la [0,1]; cazuri: segmente nedegenerate neparalele (soluția în interior apoi limitare, reevaluând celălalt parametru), paralele (mulțime de soluții; oricare pereche cu distanță minimă; rezultatul în distanță și normală este unic, punctele pot fi alese determinist, de exemplu cele mai apropiate de începutul lui A), degenerate (lungime zero = punct) [dedus, geometrie].
3. Normala n = (punctul de pe B − punctul de pe A) / d, când d > 0.
4. Dacă d = 0 (segmentele se intersectează sau se ating) normala este nedefinită: se ia normala stângă a direcției lui A, adică rotirea direcției lui A cu +90°, normalizată; dacă A este punct, se ia normala (1,0) [dedus, decizie].
5. Corecția de poziție: A se deplasează cu −n × δ × wA/(wA + wB), B cu +n × δ × wB/(wA + wB). Dacă wA + wB = 0, nicio corecție.
6. Auto-coliziune în lanț: perechile de segmente (i, j) cu |i − j| ≤ 1 se omit (învecinate) [dedus, decizie]; restul se testează ca mai sus.
7. Orice rezultat este determinist; ordinea perechilor nu modifică corecțiile individuale calculate dintr-o aceeași stare (corecțiile se acumulează apoi se aplică) [dedus, decizie].

## Formate / protocoale
Nu se introduc formate. Parametrii (raze, activarea auto-coliziunii) sunt câmpuri de schemă ale proiectului.

## Cazuri limită și erori
- Ambele segmente de lungime zero: se reduce la cercuri.
- Un segment de lungime zero: distanța punct–segment.
- Segmente coliniare care se suprapun: d = 0 (normala după pasul 4).
- Raze foarte diferite: același algoritm.
- Valori nefinite → eroare.

## Teste de acceptanță
Toleranță 1e-9. În acest fișier coordonatele se scriu între paranteze, cu „;" între componente și punct zecimal (ex. (5; 1.5)), ca să nu se confunde cu virgula zecimală. A = (0; 0)–(10; 0), rA = 1, dacă nu se spune altfel.
1. B = (5; 3)–(5; 10), rB = 1 → d = 3 > 2 → fără coliziune.
2. B = (5; 1.5)–(5; 10), rB = 1 → d = 1.5; δ = 0.5; n = (0; 1); punctele cele mai apropiate (5; 0) și (5; 1.5).
3. Paralele: rA = rB = 1.5; B = (3; 2)–(7; 2) → d = 2; δ = 1; n = (0; 1).
4. Intersecție: B = (5; −5)–(5; 5), rB = 1 → d = 0; δ = 2; n = (0; 1) (normala stângă a direcției (1; 0)).
5. Capete: B = (11.5; 0)–(20; 0), rB = 1 → d = 1.5; δ = 0.5; n = (1; 0).
6. Cercuri: A = punct (0; 0), rA = 1; B = punct (1.5; 0), rB = 1 → δ = 0.5; n = (1; 0).
7. Tangență: B = (5; 2)–(5; 10), rB = 1 → d = 2 = rA + rB → fără coliziune (δ = 0).
8. Înclinat: B = (11; 5)–(11; −5), rB = 1 → cele mai apropiate puncte (10; 0) și (11; 0); d = 1; δ = 1; n = (1; 0).
9. Corecție: la testul 2, wA = wB = 1 → A se mută cu (0; −0.25), B cu (0; +0.25); cu wB = 0 și wA = 1 → A se mută cu (0; −0.5), B rămâne pe loc; cu wA = wB = 0 → nicio mutare.
10. Auto-coliziune: lanț cu 4 segmente s0 = (0; 0)–(10; 0), s1 = (10; 0)–(10; 1), s2 = (10; 1)–(0; 1), s3 = (0; 1)–(0; 0.5). Cu raze 0.4: perechea (s0, s2) are d = 1 > 0.8 → fără coliziune; perechea (s0, s3) are d = 0.5 < 0.8 → coliziune cu δ = 0.3. Cu raze 0.6: (s0, s2) → coliziune cu δ = 0.2; (s0, s3) → δ = 0.7. Perechile (s0, s1), (s1, s2), (s2, s3) se omit chiar dacă capetele coincid; perechea (s1, s3) are d = 10 → fără coliziune.
11. Eroare: rază 0 → eroare; coordonată NaN → eroare.

## Necunoscute
- [necunoscut] Sursele nu au fost recitite; algoritmul este elementar, dar la revizuire se compară cu textul cărții.
- [necunoscut] Politica de omitere a vecinilor (distanță de 1 segment); poate fi nevoie de 2 pentru lanțuri dense.
- [necunoscut] Interacțiunea cu scara oaselor (limita L-47 menționează că toleranțele ignoră scara).
- [necunoscut] Stabilitatea la pas mare de timp (coliziune continuă) — în afara specificației.

## Checklist de filtru (verificare.md)
Spec: 10-coliziune-capsula-capsula   Data: 2026-11-09
- [x] Nicio linie de cod, pseudocod sau asamblare copiată
- [x] Niciun nume intern de funcție, variabilă sau clasă din țintă
- [x] Niciun text de UI, asset sau branding din țintă
- [x] Algoritmii sunt descriși prin intrare → ieșire, nu prin structura codului țintei
- [x] Afirmațiile sunt marcate [observat] / [dedus] / [necunoscut]
- [ ] Licența/EULA țintei verificată și notată în `1-analiza/jurnal.md` (rând adăugat; surse tipărite, nicio licență de citit; neverificat)
- [ ] Specificația nu depinde de brevete sau secrete comerciale (nu s-a făcut verificare de brevete)
- [ ] Aprobată de mine înainte de predare
