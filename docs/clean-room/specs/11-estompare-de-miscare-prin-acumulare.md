# Funcție: estompare_mișcare_prin_eșantionare_temporală

Sursă publică: M. Potmesil, I. Chakravarty, „Modeling Motion Blur in Computer-Generated Images", SIGGRAPH 1983; J. Korein, N. Badler, „Temporal Anti-Aliasing in Computer Generated Animation", SIGGRAPH 1983; R. L. Cook, T. Porter, L. Carpenter, „Distributed Ray Tracing", SIGGRAPH 1984 (eșantionare temporală stratificată în intervalul de expunere). Definiția „unghi de obturator" (timp de expunere = unghi/360 × durata cadrului) este terminologie de cinematografie.
Dimensiune estimată: M · Status: candidat · Data: 2026-11-09 · Limitare acoperită: FEATURES „Motion blur ❌" (nepornit)
Citire sursă în sesiune: NU. Sursele sunt articole/convenții neaccesibile în sesiune; conținutul este [dedus] din metoda clasică și are teste aritmetice. Nicio afirmație nu este marcată [observat].

## Scop
Adaugă la randare estomparea de mișcare: imaginea unui cadru este media mai multor randări ale scenei în momente din intervalul de expunere, ca obiectele rapide să pară continue, nu sacadate.

## Intrări / ieșiri
- Intrare: proiect, momentul cadrului t (secunde), rata de cadre fps (> 0), unghiul obturatorului φ în grade (0 ≤ φ ≤ 360), numărul de eșantioane N (întreg ≥ 1, cu plafon), faza obturatorului (momentul de început al expunerii față de t; implicit 0, adică expunerea începe la t).
- Ieșire: imagine RGBA de aceeași dimensiune ca randarea normală.
- Erori: φ în afara [0, 360] sau nefinit; N < 1 sau peste plafon; fps ≤ 0.

## Comportament observabil
1. Durata expunerii E = (φ / 360) / fps secunde [dedus, terminologie].
2. Intervalul de expunere este [t + faza, t + faza + E].
3. Momentele eșantioanelor: intervalul se împarte în N subintervale egale; eșantionul i (i = 0..N−1) este la mijlocul subintervalului i: t + faza + (i + 0,5)/N × E [dedus; eșantionare stratificată deterministă, fără aleatoriu].
4. Pentru φ = 0 se face o singură randare la t + faza, indiferent de N [dedus].
5. Pentru fiecare eșantion se randă scena completă la momentul respectiv, cu toate efectele ei (deformări, fizică precalculată, filtre, mascare).
6. Mediarea se face pe valori cu alfa premultiplicat, în spațiul de culoare în care lucrează compunerea proiectului (liniar sau codat, după setarea proiectului), cu medie aritmetică simplă a celor N imagini; la final se demultiplică alfa [dedus].
7. Acumularea se face în virgulă mobilă; rotunjirea la 8 biți se face o singură dată, la final, la cel mai apropiat întreg [dedus; vezi Necunoscute pentru jumătăți].
8. Sunetul, evenimentele și numărul de cadre nu sunt afectate.
9. Costul este proporțional cu N randări per cadru; apelantul poate cere un număr automat de eșantioane în funcție de deplasarea maximă în pixeli [necunoscut: regulă].
10. Rezultatul este reproductibil: aceleași intrări → aceiași octeți.

## Formate / protocoale
Câmpuri noi de proiect/setări de export: unghi de obturator, număr de eșantioane, fază (decizie de schemă). Valori implicite: φ = 0 (dezactivat).

## Cazuri limită și erori
- Scenă statică: media N imagini identice = imaginea.
- N = 1: o singură randare la mijlocul intervalului de expunere: t + faza + E/2.
- φ = 360 cu faza 0: expunerea acoperă tot cadrul, de la t la t + 1/fps; ultimul eșantion nu atinge t + 1/fps (mijloc de subinterval).
- Timp înainte de începutul sau după sfârșitul proiectului: se limitează la [0, durata] [dedus].
- Eșantioane cu NaN → eroare.

## Teste de acceptanță
1. φ = 0, orice N → rezultat identic cu randarea simplă la t (octet cu octet).
2. Scenă statică, φ = 180, N = 8 → identică cu randarea simplă.
3. Momentele: fps = 24, φ = 180, N = 4, t = 10/24 s, faza 0 → E = 0,0208333 s; eșantioanele la t + 0,0026042; t + 0,0078125; t + 0,0130208; t + 0,0182292 secunde.
4. N = 1, φ = 360, fps = 25, faza 0 → un eșantion la t + 0,02 s.
5. Mediere: un pixel cu valorile celor patru eșantioane 0, 254, 0, 254 (opac, gri) → 127.
6. Alfa premultiplicat: patru eșantioane pentru un pixel: primul roșu opac (255; 0; 0; alfa 255), celelalte trei complet transparente (0; 0; 0; alfa 0) → medie premultiplicată (0.25; 0; 0; alfa 0.25) în scara 0–1 → după demultiplicare culoarea este (255; 0; 0) cu alfa 0.25, adică 64 pe 8 biți (63.75 rotunjit). O mediere directă a culorilor fără premultiplicare ar da roșu 64, adică greșit.
7. Fază: faza = −E/2 centrează expunerea pe t: pentru φ = 360, fps = 25, N = 2 → E = 0,04 s, subintervale de 0,02 s, eșantioane la t − 0,01 s și t + 0,01 s.
8. Erori: φ = 400 → eroare; N = 0 → eroare; fps = 0 → eroare; φ = NaN → eroare.

## Necunoscute
- [necunoscut] Sursele nu au fost recitite; metoda este clasică, dar convențiile (faza implicită, centrarea) sunt alegeri ale proiectului.
- [necunoscut] Rotunjirea mediei la jumătăți (127,5): „în sus" sau „la par"; testele evită cazul.
- [necunoscut] Regula pentru numărul automat de eșantioane.
- [necunoscut] Dacă φ > 360 trebuie permis (expunere mai lungă decât cadrul; efect artistic).
- Brevete: metoda de bază datează din 1983–1984; nu s-a făcut verificare de brevete pentru variante moderne (de ex. estompare în spațiul ecranului din vectori de viteză), care NU sunt acoperite aici.

## Checklist de filtru (verificare.md)
Spec: 11-estompare-de-miscare-prin-acumulare   Data: 2026-11-09
- [x] Nicio linie de cod, pseudocod sau asamblare copiată
- [x] Niciun nume intern de funcție, variabilă sau clasă din țintă
- [x] Niciun text de UI, asset sau branding din țintă
- [x] Algoritmii sunt descriși prin intrare → ieșire, nu prin structura codului țintei
- [x] Afirmațiile sunt marcate [observat] / [dedus] / [necunoscut]
- [ ] Licența/EULA țintei verificată și notată în `1-analiza/jurnal.md` (rând adăugat; articole tipărite, neverificat)
- [ ] Specificația nu depinde de brevete sau secrete comerciale (nu s-a făcut verificare de brevete)
- [ ] Aprobată de mine înainte de predare
