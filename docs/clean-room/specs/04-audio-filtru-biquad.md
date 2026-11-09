# Funcție: filtru_biquad_pe_pistă

Sursă publică: *Audio EQ Cookbook*, W3C Working Group Note, 8 iunie 2021 (adaptare după textul lui R. Bristow-Johnson): https://www.w3.org/TR/audio-eq-cookbook
Dimensiune estimată: S · Status: candidat · Data: 2026-11-09 · Limitare acoperită: L-43 (fără egalizare pe pistă)
Citire sursă în sesiune: da (formulele, extrase prin instrument de citire web și verificate numeric cu un calcul independent de control).

## Scop
Aplică unei piste audio un filtru de ordinul doi (trece-jos, trece-sus, bandă, tăietură, trece-tot, clopot, raft jos, raft sus) pentru curățarea sau egalizarea sunetului înainte de mixare.

## Intrări / ieșiri
- Intrare: eșantioane mono (sau fiecare canal separat), frecvența de eșantionare Fs (Hz, > 0), tipul de filtru, frecvența f0 (Hz, 0 < f0 < Fs/2), calitatea Q (> 0) pentru tipurile cu Q, câștigul dBgain (dB) pentru clopot și rafturi.
- Ieșire: eșantioane filtrate, aceeași lungime și număr de canale.
- Erori: f0 ≤ 0 sau ≥ Fs/2, Q ≤ 0, parametri neavând valoare finită → eroare cu mesaj.

## Comportament observabil
1. Funcția de transfer: H(z) = (b0 + b1·z⁻¹ + b2·z⁻²) / (a0 + a1·z⁻¹ + a2·z⁻²) [observat].
2. Ecuația cu diferențe, cu a0 normalizat: y[n] = (b0/a0)·x[n] + (b1/a0)·x[n−1] + (b2/a0)·x[n−2] − (a1/a0)·y[n−1] − (a2/a0)·y[n−2] [observat]. Starea inițială (eșantioane anterioare) este zero [dedus].
3. Variabile: A = 10^(dBgain/40) (doar clopot și rafturi); ω0 = 2π·f0/Fs; α = sin(ω0)/(2Q) (forma cu Q) [observat]. Formele cu lățime de bandă (BW) și cu pantă de raft (S) există în sursă; implementarea poate alege doar forma cu Q [dedus, decizie].
4. Coeficienții (c = cos ω0), toți [observat]:
   - Trece-jos: b0 = (1−c)/2, b1 = 1−c, b2 = (1−c)/2; a0 = 1+α, a1 = −2c, a2 = 1−α.
   - Trece-sus: b0 = (1+c)/2, b1 = −(1+c), b2 = (1+c)/2; a ca la trece-jos.
   - Bandă (câștig de vârf 0 dB): b0 = α, b1 = 0, b2 = −α; a ca mai sus.
   - Tăietură: b0 = 1, b1 = −2c, b2 = 1; a ca mai sus.
   - Trece-tot: b0 = 1−α, b1 = −2c, b2 = 1+α; a ca mai sus.
   - Clopot: b0 = 1+α·A, b1 = −2c, b2 = 1−α·A; a0 = 1+α/A, a1 = −2c, a2 = 1−α/A.
   - Raft jos: b0 = A·((A+1) − (A−1)c + 2√A·α), b1 = 2A·((A−1) − (A+1)c), b2 = A·((A+1) − (A−1)c − 2√A·α); a0 = (A+1) + (A−1)c + 2√A·α, a1 = −2·((A−1) + (A+1)c), a2 = (A+1) + (A−1)c − 2√A·α.
   - Raft sus: b0 = A·((A+1) + (A−1)c + 2√A·α), b1 = −2A·((A−1) + (A+1)c), b2 = A·((A+1) + (A−1)c − 2√A·α); a0 = (A+1) − (A−1)c + 2√A·α, a1 = 2·((A−1) − (A+1)c), a2 = (A+1) − (A−1)c − 2√A·α.
5. Aritmetica în virgulă mobilă dublă; rezultatul se readuce la formatul pistei cu limitare la domeniul valid [dedus].
6. Filtrele pe pistă se aplică înaintea mixării, în ordinea declarată [dedus, decizie].

## Formate / protocoale
Parametrii filtrului în proiect (tip, f0, Q, dBgain) sunt o decizie de schemă a proiectului; nu se introduce format extern.

## Cazuri limită și erori
- f0 apropiat de Fs/2: coeficienții rămân definiți, dar rezultatul poate fi instabil numeric; se acceptă cu avertisment peste 0,45·Fs [dedus].
- Q foarte mare → rezonanță îngustă; fără limită superioară în sursă [dedus: limită de siguranță 1000 aleasă de proiect].
- dBgain = 0 la clopot/rafturi → filtru identitate (A = 1) [dedus matematic: b = a]. Verifică testul 6.
- NaN/infinit în semnal → apelantul validează; la ieșire se detectează și se raportează [dedus].

## Teste de acceptanță
Fs oarecare; f0 = Fs/4 (ω0 = π/2, cos ω0 = 0, sin ω0 = 1). Toleranță 1e-5. Valorile au fost calculate independent din formulele de mai sus.
1. Trece-jos, Q = 1/√2: coeficienți normalizați b = (0,292893; 0,585786; 0,292893), a = (1; 0; 0,171573). Răspuns la impuls (x[0] = 1, apoi 0): 0,292893; 0,585786; 0,242641; −0,100505; −0,041631; 0,017244.
2. Același filtru: intrare constantă 1,0 (după regim stabil) → ieșire 1,0 (câștig în curent continuu = 1).
3. Trece-sus, Q = 1/√2: la frecvența Nyquist câștigul este 1 (suma b cu semne alternate / suma a cu semne alternate = 2/2); intrare cu alternanță +1, −1, +1, … în regim stabil → amplitudine 1.
4. Trece-sus: intrare constantă → ieșire tinde la 0.
5. Clopot, dBgain = 6,0206 (A² = 2), Q = 1: modulul lui H la ω0 = 2,0 (±1e-6); coeficienți normalizați b = (1,261204; 0; 0,216388), a = (1; 0; 0,477592).
6. Clopot cu dBgain = 0 → ieșirea este egală cu intrarea (toate eșantioanele, toleranță 1e-12).
7. Raft jos, dBgain = 6,0206, Q = 1/√2: câștig în curent continuu 2,0 și la Nyquist 1,0.
8. Erori: f0 = Fs/2 → eroare; Q = 0 → eroare; f0 = −10 → eroare.

## Necunoscute
- [necunoscut] Politica de rampă la schimbarea parametrilor în timp (clicuri).
- [necunoscut] Dacă forma cu BW/S e necesară utilizatorilor.
- Sursa este o notă W3C bazată pe un text public al autorului; statutul de licență al notei nu a fost verificat în sesiune.

## Checklist de filtru (verificare.md)
Spec: 04-audio-filtru-biquad   Data: 2026-11-09
- [x] Nicio linie de cod, pseudocod sau asamblare copiată
- [x] Niciun nume intern de funcție, variabilă sau clasă din țintă
- [x] Niciun text de UI, asset sau branding din țintă
- [x] Algoritmii sunt descriși prin intrare → ieșire, nu prin structura codului țintei
- [x] Afirmațiile sunt marcate [observat] / [dedus] / [necunoscut]
- [ ] Licența/EULA țintei verificată și notată în `1-analiza/jurnal.md` (rând adăugat; licența notei neverificată)
- [ ] Specificația nu depinde de brevete sau secrete comerciale (nu s-a făcut verificare de brevete)
- [ ] Aprobată de mine înainte de predare
