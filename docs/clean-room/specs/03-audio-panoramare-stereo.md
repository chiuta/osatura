# Funcție: panoramare_stereo_putere_egală

Sursă publică: *Web Audio API*, W3C, secțiunea „StereoPannerNode" / „Stereo Panning Algorithm"; text citit prin istoricul public al specificației: https://github.com/WebAudio/web-audio-api/pull/440/files și descrierea https://developer.mozilla.org/docs/Web/API/StereoPannerNode (MDN).
Dimensiune estimată: S · Status: candidat · Data: 2026-11-09 · Limitare acoperită: L-43 (nu există panoramare pe pistă)
Citire sursă în sesiune: da (algoritmul, în forma din cererea de modificare a specificației; versiunea finală publicată a specificației nu a fost recitită, vezi Necunoscute).

## Scop
Permite poziționarea fiecărei piste audio între stânga și dreapta la mixare (`sound mix`), cu putere totală constantă la deplasarea poziției.

## Intrări / ieșiri
- Intrare: semnal mono (1 canal) sau stereo (2 canale), valoare de panoramare p în intervalul [−1, 1] (−1 = complet stânga, 0 = centru, 1 = complet dreapta) [observat, MDN].
- Ieșire: semnal stereo (2 canale), aceeași lungime.
- p în afara intervalului: se limitează la [−1, 1] [dedus]; p nu este un număr finit → eroare.

## Comportament observabil
1. Se calculează x din p: pentru intrare mono, x = (p + 1) / 2; pentru intrare stereo, x = p + 1 dacă p ≤ 0, altfel x = p [observat].
2. Câștigurile: stânga = cos(π/2 · x), dreapta = sin(π/2 · x) [observat].
3. Intrare mono: ieșire stânga = intrare × câștig stânga; ieșire dreapta = intrare × câștig dreapta [observat].
4. Intrare stereo, p ≤ 0: ieșire stânga = intrare stânga + intrare dreapta × câștig stânga; ieșire dreapta = intrare dreapta × câștig dreapta [observat].
5. Intrare stereo, p > 0: ieșire stânga = intrare stânga × câștig stânga; ieșire dreapta = intrare dreapta + intrare stânga × câștig dreapta [observat].
6. Intrare cu alt număr de canale: reducere/extindere la mono sau stereo înainte de pași [observat că alte intrări sunt mixate; regula exactă de mixare [necunoscut]].
7. Panoramarea poate fi constantă pe pistă sau variabilă în timp (valori cheie interpolate liniar între cadre-cheie [dedus, decizie de proiectare]).

## Formate / protocoale
Eșantioane în virgulă mobilă (sau convertite din/în 16 biți de apelant). Fără format de fișier nou. Câmpul de pistă „pan" în proiect este o decizie de schemă a proiectului (valoare implicită 0; absent = 0).

## Cazuri limită și erori
- p = 0 la intrare mono: ambele canale = intrare × cos(π/4) ≈ 0,70711.
- p = 0 la intrare stereo: x = 1 → câștig stânga 0 (cos π/2), câștig dreapta 1 → ieșire identică cu intrarea.
- Eșantion NaN în intrare → se propagă NaN; apelantul trebuie să valideze [dedus].
- Pistă fără eșantioane → ieșire goală.

## Teste de acceptanță
Valori cu toleranță 1e-6.
1. Mono, intrare 1,0, p = −1 → (stânga 1,0; dreapta 0,0).
2. Mono, intrare 1,0, p = 0 → (0,707107; 0,707107); suma pătratelor = 1.
3. Mono, intrare 1,0, p = 1 → (0,0; 1,0).
4. Stereo, intrare (0,5; −0,25), p = 0 → ieșire (0,5; −0,25).
5. Stereo, intrare (0,5; 0,5), p = −1 → x = 0, câștig stânga 1, dreapta 0 → ieșire stânga = 0,5 + 0,5 × 1 = 1,0; dreapta = 0,5 × 0 = 0.
6. Stereo, intrare (0,5; 0,5), p = 1 → x = 1, câștig stânga 0, dreapta 1 → ieșire stânga = 0,5 × 0 = 0; dreapta = 0,5 + 0,5 × 1 = 1,0.
7. Stereo, intrare (1,0; 0,0), p = 0,5 → x = 0,5, câștiguri 0,707107 / 0,707107 → ieșire stânga = 0,707107; dreapta = 0 + 1,0 × 0,707107 = 0,707107.
8. p = 2,5 se tratează ca 1 (limitare); p = NaN → eroare.

## Necunoscute
- [necunoscut] Dacă versiunea finală a specificației păstrează exact aceste formule; se compară la revizuire cu textul publicat.
- [necunoscut] Mixarea canalelor pentru intrări cu mai mult de 2 canale.
- [necunoscut] Rampa de netezire la modificarea bruscă a lui p (evitarea clicurilor) — nu e în sursa citită.

## Checklist de filtru (verificare.md)
Spec: 03-audio-panoramare-stereo   Data: 2026-11-09
- [x] Nicio linie de cod, pseudocod sau asamblare copiată
- [x] Niciun nume intern de funcție, variabilă sau clasă din țintă
- [x] Niciun text de UI, asset sau branding din țintă
- [x] Algoritmii sunt descriși prin intrare → ieșire, nu prin structura codului țintei
- [x] Afirmațiile sunt marcate [observat] / [dedus] / [necunoscut]
- [ ] Licența/EULA țintei verificată și notată în `1-analiza/jurnal.md` (rând adăugat; licența documentelor W3C/MDN notată „neverificată în sesiune")
- [ ] Specificația nu depinde de brevete sau secrete comerciale (nu s-a făcut verificare de brevete)
- [ ] Aprobată de mine înainte de predare
