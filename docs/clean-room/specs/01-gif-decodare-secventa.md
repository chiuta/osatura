# Funcție: decodare_secvență_gif

Sursă publică: *Graphics Interchange Format, Version 89a* (CompuServe, 31 iulie 1990), copie W3C: https://www.w3.org/Graphics/GIF/spec-gif89a.txt (secțiunile 17, 18, 20, 22, 23, 26, 27 și apendicele B, E, F).
Dimensiune estimată: M · Status: candidat · Data: 2026-11-09 · Limitare acoperită: L-31 / R-037 (decodare GIF ca strat „secvență de imagini")
Citire sursă în sesiune: da, secțiunile enumerate (extras prin instrument de citire web; textul integral nu a fost recitit linie cu linie). Tot ce nu apare în sursă este marcat [dedus] sau [necunoscut].

## Scop
Transformă un fișier GIF (statică sau animată) într-o listă de cadre complete (pânză întreagă, RGBA) cu durata fiecărui cadru, ca să poată fi folosită ca strat „secvență de imagini". Rezolvă lipsa raportată în L-31 (astăzi trebuie convertit manual la secvență PNG).

## Intrări / ieșiri
- Intrare: șir de octeți al unui fișier GIF. Plafoane configurabile (valori implicite alese de proiect, nu de standard): număr maxim de cadre, număr maxim de pixeli per cadru, total maxim de pixeli decodați.
- Ieșire: lățimea și înălțimea ecranului logic (întregi fără semn, 1–65535 [observat: 2 octeți]); numărul de bucle (opțional, vezi Necunoscute); listă ordonată de cadre, fiecare cu: imagine RGBA (lățime × înălțime ecran) cu canal alfa 0 sau 255, durată în milisecunde.
- Eroare: mesaj text care indică motivul (semnătură invalidă, date trunchiate, cod LZW invalid, depășire plafon, versiune necunoscută).

## Comportament observabil
1. Primii 6 octeți: „GIF" urmat de „87a" sau „89a" [observat, secț. 17]. Orice altceva este eroare.
2. Descriptorul ecranului logic: lățime și înălțime pe 2 octeți fiecare, octetul cel mai puțin semnificativ primul [observat, secț. 18]; apoi octetul împachetat (bit 7 = există tabel global de culori; biții 6–4 = rezoluția culorii; bit 3 = sortare; biții 2–0 = dimensiunea tabelului), indexul culorii de fundal, raportul de aspect.
3. Dacă steagul tabelului global este setat, urmează un tabel de 3 × 2^(N+1) octeți (R, G, B pe intrare), unde N este valoarea pe 3 biți [observat, secț. 18].
4. Urmează o succesiune de blocuri, identificate prin primul octet: 0x21 = extensie, 0x2C = descriptor de imagine, 0x3B = terminator al fluxului [observat, secț. 20, 23, 26, 27].
5. Extensia de control grafic (0x21, 0xF9): dimensiune de bloc fixă 4; octet împachetat cu biții 4–2 = metoda de eliminare, bit 1 = intrare utilizator, bit 0 = culoare transparentă; durata pe 2 octeți în sutimi de secundă; indexul culorii transparente (prezent doar dacă bitul 0 este setat); terminator 0x00 [observat, secț. 23]. Se aplică DOAR următorului cadru de imagine [dedus din gramatica din apendicele B: extensia precede blocul grafic].
6. Extensiile de aplicație (0x21, 0xFF), de comentariu (0x21, 0xFE) și de text simplu (0x21, 0x01) sunt parcurse prin sub-blocuri (octet de dimensiune 0–255, apoi atâția octeți; dimensiune 0 încheie secvența) [observat, secț. 15–16] și ignorate, cu excepția cazului descris la Necunoscute.
7. Descriptorul de imagine (0x2C): stânga, sus, lățime, înălțime pe câte 2 octeți; octet împachetat: bit 7 = tabel local, bit 6 = întrețesut, bit 5 = sortare, biții 4–3 rezervați (zero), biții 2–0 = dimensiunea tabelului local [observat, secț. 20]. Dacă există tabel local, el înlocuiește tabelul global pentru acest cadru [observat, secț. 20 / 21; formulare exactă [dedus]].
8. Date imagine: un octet „dimensiune minimă de cod LZW", apoi sub-blocuri de date. Dimensiunea inițială a codului = minim + 1 biți; cod de golire (Clear) = 2^minim; cod de sfârșit de informație (EOI) = Clear + 1; prima intrare liberă în tabel = Clear + 2 [observat, secț. 22, apendice F]. Codurile se citesc începând cu bitul cel mai puțin semnificativ al fiecărui octet [dedus; confirmat prin vectorul de test 1 de mai jos, verificat cu un decodor independent]. Lungimea maximă a unui cod este 12 biți (valoare maximă 4095); lungimea crește cu un bit când următoarea intrare nu mai încape pe lungimea curentă [observat, secț. 22]. Codul Clear poate apărea oriunde; EOI este ultimul cod, înaintea terminatorului de bloc [observat].
9. Dacă tabelul s-a umplut (4096 intrări), decodorul nu mai adaugă intrări până la următorul Clear [observat, pagina de gardă a specificației, „decoder must not change its full table until a Clear code arrives"].
10. Caz LZW clasic „cod care nu este încă în tabel" (șirul precedent + primul său caracter): acceptat [dedus, manual LZW]. Un cod mai mare decât prima intrare liberă este eroare.
11. Întrețesere: când bitul 6 este setat, rândurile stocate sunt în patru treceri: rândurile 0, 8, 16… ; apoi 4, 12, 20… ; apoi 2, 6, 10… ; apoi 1, 3, 5… [observat, apendice E]. Indicii stocați se aranjează în ordinea naturală a rândurilor.
12. Compunerea pe pânză: pânza începe complet transparentă [dedus; standardul nu definește starea de dinaintea primului cadru]. Cadrul se desenează la poziția (stânga, sus) peste pânză; pixelii al căror index este culoarea transparentă nu modifică pânza; ceilalți o înlocuiesc (alfa 255).
13. Metoda de eliminare se aplică DUPĂ afișarea cadrului, înaintea celui următor [observat, secț. 23 formulează „what to do with the graphic after it's displayed"; momentul exact [dedus]]: 0 sau 1 = pânza rămâne; 2 = zona cadrului revine la fundal; 3 = zona cadrului revine la conținutul de dinaintea cadrului; 4–7 nedefinite [observat]. Pentru valorile 4–7 se tratează ca 0 [dedus, decizie de implementare].
14. Durata: valoarea × 10 ms [observat: unitate = sutime de secundă]. Durata 0: vezi Necunoscute.
15. Fiecare cadru din ieșire este o instantanee a întregii pânze după desenarea cadrului (nu doar a regiunii).
16. Terminatorul 0x3B încheie decodarea; octeți după el sunt ignorați [dedus].

## Formate / protocoale
Format binar, ordinea octeților și câmpurile exact așa cum apar în sursă (vezi mai sus). Valorile multi-octet sunt little-endian [observat, secț. 18].

## Cazuri limită și erori
- Semnătură diferită de GIF87a/GIF89a → eroare.
- Flux încheiat înainte de terminator în mijlocul datelor LZW → eroare „date trunchiate"; cadrele complet decodate înainte NU se returnează [dedus, decizie: ieșire atomică; vezi Necunoscute].
- Flux fără octet 0x3B dar cu cadre complete → acceptat, cu avertisment [dedus].
- Cadru cu regiune care depășește ecranul logic → decupat la pânză [dedus; standardul presupune încadrarea].
- Mai puțini indici decodați decât lățime × înălțime → eroare; mai mulți → surplusul se ignoră [dedus].
- Index de culoare mai mare decât tabelul activ → eroare.
- Niciun tabel de culori (nici global, nici local) → eroare [dedus].
- Dimensiune zero (lățime sau înălțime) → eroare.
- Depășirea plafoanelor de cadre/pixeli → eroare înainte de alocare (protecție la „bombe de decompresie") [dedus].
- GIF87a fără extensii: același traseu, durată implicită (vezi Necunoscute).

## Teste de acceptanță
Vectorii hexazecimali 1 și 2 au fost verificați cu un decodor GIF independent (nu face parte din produs) înainte de a fi scriși aici.
1. Intrare (35 octeți, hex): `474946383961 0100 0100 90 00 00 000000 ffffff 2c 0000 0000 0100 0100 00 02 02 4c01 00 3b` (spațiile sunt doar de lizibilitate) → ecran 1×1, un cadru, pixelul (0,0) = (255,255,255,255).
2. Intrare: același flux cu, înaintea descriptorului de imagine, extensia `21f904 01 0700 01 00` (transparentă setată, durată 7, index transparent 1) → un cadru, durată 70 ms, pixelul (0,0) cu alfa 0.
3. Întrețesere: imagine 1 coloană × 8 rânduri, bit întrețesut setat, rândul stocat k (k = 0..7) conține indexul k → ordinea stocată corespunde rândurilor de ieșire 0,4,2,6,1,3,5,7, deci coloana de ieșire de sus în jos are valorile 0,4,2,5,1,6,3,7.
4. Eliminare: ecran 2×1; cadrul A acoperă ambii pixeli (roșu, roșu) cu metoda 1; cadrul B acoperă doar pixelul 1 (albastru) → cadrul 2 de ieșire = (roșu, albastru). Aceeași secvență cu metoda 3 pe A → cadrul 2 = (transparent, albastru) (pânza dinaintea lui A era transparentă).
5. Eroare semnătură: primii 6 octeți „GIF90a" → eroare, nicio ieșire.
6. Eroare LZW: la datele din testul 1, înlocuiește al doilea cod (1) cu 7 (mai mare decât prima intrare liberă 6) → eroare „cod invalid".
7. Plafon: GIF 65535 × 65535 cu plafon de 16 Mpixeli → eroare, fără alocare de pânză.
8. Trunchiere: testul 1 tăiat după primii 32 de octeți (sub-blocul de date anunță 2 octeți, dar rămâne unul) → eroare „date trunchiate".

## Necunoscute
- [necunoscut] Numărul de repetări (buclă) NU este definit în specificația citită; extensia de aplicație cu identificator de 8 octeți și cod de autentificare de 3 octeți există [observat, secț. 26], dar conținutul uzual pentru bucle nu este descris în sursa citită. Implementarea tratează bucla ca „necunoscută = o singură trecere" până când se citește o sursă publică separată.
- [necunoscut] Durata 0 (sau lipsa extensiei de control): standardul o definește doar ca valoare; ce durată minimă folosește proiectul este decizia întreținătorului.
- [necunoscut] Metoda 2 spune „culoarea de fundal"; practica uzuală este transparent, dar sursa citită nu o spune. Decizia (transparent vs. culoarea indexului de fundal) aparține întreținătorului; testul 4 evită cazul.
- [necunoscut] Comportament la flux trunchiat: ieșire atomică (eroare) sau cadre parțiale.
- Brevete: brevetele LZW asociate formatului sunt expirate [dedus din cunoștințe generale, neverificat în sesiune].

## Checklist de filtru (verificare.md)
Spec: 01-gif-decodare-secventa   Data: 2026-11-09
- [x] Nicio linie de cod, pseudocod sau asamblare copiată
- [x] Niciun nume intern de funcție, variabilă sau clasă din țintă
- [x] Niciun text de UI, asset sau branding din țintă
- [x] Algoritmii sunt descriși prin intrare → ieșire, nu prin structura codului țintei
- [x] Afirmațiile sunt marcate [observat] / [dedus] / [necunoscut]
- [ ] Licența/EULA țintei verificată și notată în `1-analiza/jurnal.md` (rând adăugat; licența documentului notată ca „neverificată în sesiune")
- [ ] Specificația nu depinde de brevete sau secrete comerciale (nu s-a făcut verificare de brevete; vezi nota de mai sus)
- [ ] Aprobată de mine înainte de predare
