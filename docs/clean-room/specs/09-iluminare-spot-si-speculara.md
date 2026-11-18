# Funcție: iluminare_spot_și_speculară

Sursă publică: (1) pagina de manual a funcției de lumină din biblioteca grafică OpenGL (specificație publică Khronos/SGI), text citit la https://manpath.be/c7/3g/glLighti — atenuare, con de proiector, exponent, unghi limită, intervale valide; (2) J. F. Blinn, „Models of Light Reflection for Computer Synthesized Pictures", SIGGRAPH 1977 (model cu vector la jumătate); (3) B. T. Phong, „Illumination for Computer Generated Pictures", Communications of the ACM 18(6), 1975; (4) J. H. Lambert, *Photometria* (1760), difuzia cu cosinus.
Dimensiune estimată: M · Status: candidat · Data: 2026-11-09 · Limitare acoperită: L-24 / R-064 („doar lumini punctiforme, fără proiectoare, fără termen speculativ")
Citire sursă în sesiune: doar (1), parțial (pagina nu conține ecuația completă ambient/difuz/speculativ). Sursele (2)–(4) NU au fost recitite; afirmațiile din ele sunt [dedus] (cunoștințe de manual), de confirmat la revizuire.

## Scop
Extinde iluminarea pe hărți de normale cu: proiectoare (lumini cu con și exponent), atenuare cu distanța și strălucire speculativă, pentru volum mai credibil la personaje 2D.

## Intrări / ieșiri
- Pentru fiecare pixel: poziția p (x, y; înălțimea z a suprafeței 0 în lipsă de hartă de înălțime), normala unitară n = (nx, ny, nz) din harta de normale, culoarea de bază.
- Pentru fiecare lumină: poziția (x, y, z, z > 0), culoarea și intensitatea, factori de atenuare (constant kc, liniar kl, pătratic kq, toți ≥ 0), opțional direcție de proiector d (vector nenul), unghi limită θc (în grade), exponent e.
- Pentru material: strălucire s (exponent speculativ), coeficient speculativ ks ≥ 0.
- Vizualizatorul este ortografic, deasupra planului, vector spre observator v = (0, 0, 1) [dedus, decizie de proiectare].
- Ieșire: contribuția luminii la pixel: difuz (RGB), speculativ (RGB).
- Erori: θc în afara [0, 90] și diferit de 180; e în afara [0, 128]; factor de atenuare negativ; d de lungime zero; s negativ [observat pentru primele patru, în intervalele din pagina de manual].

## Comportament observabil
1. Vectorul spre lumină l = direcția unitară de la punct la poziția luminii; distanța d_p = lungimea vectorului punct → lumină.
2. Atenuarea = 1 / (kc + kl·d_p + kq·d_p²) [observat: intensitatea se împarte la suma celor trei termeni]; implicit (1, 0, 0) = fără atenuare [observat]; lumina direcțională nu are atenuare [observat; proiectul nu are lumini direcționale — [dedus] nu se aplică].
3. Efectul de proiector: se calculează unghiul dintre direcția proiectorului d și direcția de la lumină la punct. Dacă acest unghi este mai mare decât θc, lumina este complet mascată (contribuția 0) [observat]. În interiorul conului, intensitatea se înmulțește cu cosinusul acelui unghi ridicat la puterea e [observat]. Unghi limită 180 înseamnă că lumina nu este proiector și se distribuie uniform; e = 0 înseamnă distribuție uniformă în con [observat]. Valoarea implicită a limitei este 180 [observat].
4. Difuz: max(0, n·l) × culoare lumină × atenuare × efect de proiector [dedus, Lambert].
5. Speculativ: dacă n·l > 0, ks × (max(0, n·h))^s × culoare lumină × atenuare × efect de proiector, unde h este vectorul unitar al sumei l + v; altfel 0 [dedus, Blinn 1977].
6. Contribuțiile mai multor lumini se însumează; rezultatul se limitează la domeniul valid al culorii în etapa finală [dedus].
7. Ieșirea este deterministă și independentă de ordinea luminilor.
8. Umbrele proiectate nu fac parte din această specificație (rămân cele existente).

## Formate / protocoale
Câmpuri noi pe lumină (direcție de proiector, unghi limită, exponent, factori de atenuare) și pe material (coeficient speculativ, strălucire) — decizie de schemă a proiectului; valorile implicite păstrează comportamentul actual (limită 180, atenuare (1, 0, 0), ks = 0).

## Cazuri limită și erori
- n·l ≤ 0: difuz și speculativ 0.
- Punct exact sub proiector (unghi 0): efectul de proiector = 1.
- Unghi exact egal cu limita: în interior (nu „mai mare") [observat: masca se aplică doar dacă unghiul este mai mare decât limita].
- Normală neunitară → se normalizează; normală nulă → eroare sau tratare ca (0,0,1) [dedus, decizie].
- Lumină la z ≤ 0 → eroare (în acest model lumina e deasupra planului) [dedus].

## Teste de acceptanță
Toleranță 1e-4. Punct la origine, n = (0,0,1), v = (0,0,1).
1. Lumină la (0,0,10), fără atenuare, fără proiector: n·l = 1; difuz 1,0; speculativ cu ks = 1, s = 32 → 1,0.
2. Lumină la (10,0,10): n·l = 0,707107; n·h = 0,923880; speculativ cu s = 2 → 0,853553; cu s = 32 → 0,079376; cu s = 8 → 0,530790.
3. Atenuare (1; 0,1; 0) cu lumina la (0,0,10) (d_p = 10) → factor 0,5; difuz 0,5.
4. Proiector la (0,0,10), direcție (0,0,−1), limită 30°, exponent 2. Punct (0,0,0): unghi 0 → efect 1,0. Punct (5,7735; 0; 0): unghi 30° (exact pe limită; „mai mare decât limita" nu e satisfăcut, deci în interior, cu toleranță de comparare 1e-6) → efect cos²(30°) = 0,75. Punct (10;0;0): unghi 45° > 30° → contribuție 0.
5. Limita 180 la același proiector: punctul (10,0,0) primește lumină (nu este proiector); efect de proiector 1.
6. Fața întoarsă: normala n = (1,0,0), lumina la (−10,0,10): n·l = −0,707107 → difuz 0 și speculativ 0.
7. Erori: limita 100 → eroare; exponent 200 → eroare; kl = −0,1 → eroare; direcție (0,0,0) → eroare.

## Necunoscute
- [necunoscut] Ecuația completă din sursa (1) nu e în pagina citită; termenii difuz/speculativ sunt din manuale clasice [dedus].
- [necunoscut] Modelul de observator (ortografic, față) este alegerea proiectului; nu e din sursă.
- [necunoscut] Dacă termenul speculativ trebuie colorat de material sau doar de lumină.
- Brevete: modele clasice din 1760–1977; nu s-a făcut verificare de brevete pentru implementări specifice.

## Checklist de filtru (verificare.md)
Spec: 09-iluminare-spot-si-speculara   Data: 2026-11-09
- [x] Nicio linie de cod, pseudocod sau asamblare copiată
- [x] Niciun nume intern de funcție, variabilă sau clasă din țintă
- [x] Niciun text de UI, asset sau branding din țintă
- [x] Algoritmii sunt descriși prin intrare → ieșire, nu prin structura codului țintei
- [x] Afirmațiile sunt marcate [observat] / [dedus] / [necunoscut]
- [ ] Licența/EULA țintei verificată și notată în `1-analiza/jurnal.md` (rând adăugat; licența paginii de manual neverificată în sesiune)
- [ ] Specificația nu depinde de brevete sau secrete comerciale (nu s-a făcut verificare de brevete)
- [ ] Aprobată de mine înainte de predare
