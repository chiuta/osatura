# Funcție: aliniere_pe_linia_de_bază

Sursă publică: *CSS Flexible Box Layout Module Level 1*, W3C, secțiunea 8.3 „Cross-axis Alignment" și 9.4 „Cross Size Determination": https://www.w3.org/TR/css-flexbox-1/ (text citit prin oglinda https://scriptagc.wasmer.app/https_drafts_csswg_org/css-flexbox-1, ciornă a aceluiași document); termeni de bază: *CSS Box Alignment Module Level 3* (W3C), neconsultat.
Dimensiune estimată: S · Status: candidat · Data: 2026-11-09 · Limitare acoperită: L-50 („no baseline align" în containerele de aranjare)
Citire sursă în sesiune: da, parțial (8.3 și o descriere a lui 9.4; definiția „prima linie de bază" aparține altei specificații și nu a fost citită).

## Scop
Într-un container de aranjare orientat pe rând, permite ca elementele (straturile membre) să se alinieze după o „linie de bază" comună (de exemplu, baza unor figuri sau etichete de dimensiuni diferite), nu doar la început, centru sau sfârșit.

## Intrări / ieșiri
- Intrare: containerul (axa principală orizontală, axa transversală verticală, cu sus = început transversal), lista elementelor cu: înălțime (mărimea transversală, ≥ 0), margine sus și jos, regimul de aliniere (linia de bază sau altul), distanța liniei de bază față de muchia superioară a cutiei de chenar (opțională), și înălțimea transversală a containerului (definită sau automată).
- Ieșire: poziția transversală (decalajul de sus) a fiecărui element în linie și mărimea transversală a liniei.
- Erori: distanță a liniei de bază negativă sau nefinită; înălțime nefinită.

## Comportament observabil
1. Elementele cu regim „linia de bază" participă la alinierea pe linia de bază: toate elementele participante din aceeași linie se aliniază astfel încât liniile lor de bază să coincidă [observat, 8.3].
2. Elementul cu cea mai mare distanță între linia sa de bază și muchia marginii transversale de început este așezat lipit de muchia de început a liniei; celelalte se deplasează în jos cu diferența [observat, 8.3].
3. Dacă un element nu are linie de bază pe axa necesară, una este sintetizată din cutia sa de chenar [observat, 8.3]. Muchia folosită: partea de jos a cutiei [dedus din convenția pentru linia de bază alfabetică, text neconsultat].
4. Distanța liniei de bază se măsoară de la muchia marginii de început (adică include marginea de sus a elementului) [dedus din formularea „cross-start margin edge"].
5. Mărimea transversală a liniei, la container cu mărime transversală automată și o singură linie: maximul dintre (a) suma celei mai mari distanțe linie-de-bază-până-la-început cu cea mai mare distanță linie-de-bază-până-la-sfârșit, calculate peste elementele participante, și (b) mărimile transversale externe ale celorlalte elemente [observat în descriere, 9.4; combinarea cu (b) [dedus]].
6. La container cu mărime transversală definită și o singură linie, mărimea liniei este mărimea interioară a containerului [observat, 9.4]; grupul de bază rămâne lipit de începutul liniei.
7. Elementele cu alt regim (început, centru, sfârșit, întindere) nu participă; se poziționează față de mărimea liniei conform regimului lor [dedus].
8. Dacă axele nu sunt paralele cu regimul (container pe coloană), „linia de bază" se comportă ca „început" [dedus din specificația de aranjare flexibilă; text neconsultat].
9. Pentru straturi fără text, distanța liniei de bază este câmp al stratului (valoare implicită: înălțimea, adică marginea de jos) [dedus, decizie de proiectare].

## Formate / protocoale
În proiect, două câmpuri noi pe strat membru: regim de aliniere „linia de bază" și distanță a liniei de bază (opțional). Decizie de schemă a proiectului.

## Cazuri limită și erori
- Un singur element în regim linie de bază: decalaj 0.
- Zero elemente participante: nicio schimbare.
- Distanțe egale: toate decalajele 0 în grup.
- Distanță a liniei de bază mai mare decât înălțimea elementului: acceptată (linia de bază în afara cutiei) [dedus].
- Valori nefinite → eroare.
- Mai multe linii (înfășurare): grupurile sunt separate pe fiecare linie [observat: „on the line"].

## Teste de acceptanță
Container pe rând, marginile 0, mărime transversală automată, o singură linie.
1. Elemente: A (înălțime 20, linie de bază la 15), B (înălțime 30, la 10), C (înălțime 10, la 10), toate în regim linie de bază → decalaje: A 0, B 5, C 5; mărimea liniei = 15 + max(5, 20, 0) = 35.
2. Același set, container cu mărime transversală definită 100 → aceleași decalaje (0, 5, 5); mărimea liniei = 100.
3. Doar A → decalaj 0; mărimea liniei = 20.
4. D (înălțime 12) fără linie de bază în regim linie de bază, împreună cu B (30, la 10): linia de bază a lui D se sintetizează la 12 (jos) [dedus] → decalaje: B 2, D 0; mărimea liniei = 12 + max(30−10, 12−12) = 12 + 20 = 32 (rezultatul depinde de ipoteza „jos"; vezi Necunoscute).
5. A (20, la 15) cu margine sus 4 → distanța măsurată de la marginea de început: 19; împreună cu B (30, la 10, margine 0): A decalaj 0 (19 > 10), B decalaj 9.
6. A, B, C ca în testul 1 plus E în regim „centru", înălțime 10 → E ocupă decalajul (35 − 10)/2 = 12,5.
7. Container pe coloană cu A, B, C în regim linie de bază → comportă ca „început": decalaje 0, 0, 0 (pe axa transversală orizontală) [dedus].
8. Eroare: distanță a liniei de bază = −1 → eroare.

## Necunoscute
- [necunoscut] Muchia pentru linia de bază sintetizată (jos presupus; testul 4 depinde de ipoteză).
- [necunoscut] Definiția exactă a „primei linii de bază" pentru elemente compuse (grupuri): propunere — linia de bază a primului membru în ordinea desenării, dacă are; altfel sintetizată.
- [necunoscut] Interacțiunea cu rotația/înclinarea stratului: se aplică pe cutia nerotită.

## Checklist de filtru (verificare.md)
Spec: 06-layout-aliniere-pe-linia-de-baza   Data: 2026-11-09
- [x] Nicio linie de cod, pseudocod sau asamblare copiată
- [x] Niciun nume intern de funcție, variabilă sau clasă din țintă
- [x] Niciun text de UI, asset sau branding din țintă
- [x] Algoritmii sunt descriși prin intrare → ieșire, nu prin structura codului țintei
- [x] Afirmațiile sunt marcate [observat] / [dedus] / [necunoscut]
- [ ] Licența/EULA țintei verificată și notată în `1-analiza/jurnal.md` (rând adăugat; licența documentului W3C neverificată în sesiune)
- [ ] Specificația nu depinde de brevete sau secrete comerciale (nu s-a făcut verificare de brevete)
- [ ] Aprobată de mine înainte de predare
