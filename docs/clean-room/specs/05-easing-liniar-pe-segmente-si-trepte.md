# Funcție: curbă_de_ritm_pe_segmente_și_trepte

Sursă publică: *CSS Easing Functions Level 2*, W3C First Public Working Draft, 29 august 2024: https://www.w3.org/TR/css-easing-2 (secțiunile 2.1 linear(), 2.3 Bézier cubic, 2.4 steps() și 2.4.1 algoritmul treptelor).
Dimensiune estimată: S · Status: candidat · Data: 2026-11-09 · Limitare acoperită: L-16 (tipuri de interpolare limitate), FEATURES „bounce / elastic / stagger"
Citire sursă în sesiune: da (extras prin instrument de citire web). Documentul este un proiect de lucru; textul final poate diferi (vezi Necunoscute).

## Scop
Adaugă două familii de curbe de ritm pentru valori cheie, definite public: (a) curbă liniară pe segmente, cu puncte de control date de autor (permite să se „deseneze" orice ritm, inclusiv rebound/bounce aproximat); (b) trepte uniforme cu patru poziții ale salturilor. Proiectul are acum doar menținere până la cheia următoare (treaptă unică) și Bézier cubică.

## Intrări / ieșiri
- Intrare: progresul p (număr real; în mod normal în [0,1], dar și în afara lui) și descrierea curbei.
- Ieșire: valoarea de ritm f(p), număr real (poate depăși [0,1]).
- Descriere a), „puncte": listă de ≥ 2 puncte; fiecare punct are o ieșire (număr) și opțional una sau două intrări (procente sau fracții).
- Descriere b), „trepte": număr întreg de trepte n și poziția salturilor, una din: salt-la-început, salt-la-sfârșit, fără-salt-la-capete, salt-la-ambele-capete (cu sinonime „început" = salt-la-început, „sfârșit" = salt-la-sfârșit) [observat].
- Erori: mai puțin de 2 puncte; n ≤ 0; n = 1 cu poziția fără-salt-la-capete (trebuie n > 1) [observat].

## Comportament observabil
**Curba pe segmente**
1. Fiecare punct are ieșire; un punct cu două intrări produce două puncte cu aceeași ieșire (un salt vertical) [observat].
2. Intrarea primului punct lipsă = 0; intrarea ultimului punct lipsă = 1 [observat: „first stop's input defaults to 0; last … at least 1"; formularea exactă pentru ultimul punct, vezi Necunoscute].
3. Intrările lipsă se completează prin interpolare liniară între cele mai apropiate puncte vecine care au intrare [observat].
4. O intrare mai mică decât cea mai mare intrare dinaintea ei se ridică la aceasta (nu se elimină) [observat].
5. Evaluare: se caută ultimul punct A cu intrare ≤ p (sau punctul 0 dacă niciunul); dacă A este ultimul punct, se ia cel dinaintea lui; B este punctul următor. Rezultat: interpolare liniară între A și B; dacă intrările lui A și B sunt egale, rezultatul este ieșirea lui B [observat].
6. În afara intervalului definit, traiectoria dreptei dintre cele mai apropiate două puncte continuă (extrapolare) [observat].
7. Cuvântul „liniar" fără puncte = identitate, echivalent cu două puncte (0 și 1) [observat].

**Trepte**
8. Pași: pasul curent = parte întreagă inferioară a lui (p × n). Pentru salt-la-început și salt-la-ambele-capete se adaugă 1. Dacă „steagul dinainte" e setat și p × n este întreg, se scade 1 (steagul este fals în evaluarea obișnuită de la începutul unui interval [dedus]). Dacă p ≥ 0 și pasul < 0, pasul = 0. Numărul de salturi = n (început/sfârșit), n − 1 (fără-salt-la-capete), n + 1 (ambele). Dacă p ≤ 1 și pasul > salturi, pasul = salturi. Rezultat = pas / salturi [observat, 2.4.1].
9. Cuvântul-cheie „treaptă-început" este echivalent cu o singură treaptă cu salt-la-început; „treaptă-sfârșit" cu o singură treaptă cu salt-la-sfârșit [observat].
10. Poziția implicită, când lipsește, este salt-la-sfârșit [observat].

**Bézier cubică (completare de validare)**
11. Punctele de control P1, P2 date prin patru numere; capetele fixe (0,0) și (1,1); abscisele lui P1 și P2 trebuie să fie în [0,1], altfel definiția este invalidă [observat]. Cuvinte-cheie: „ease" = (0,25; 0,1; 0,25; 1), „ease-in" = (0,42; 0; 1; 1), „ease-out" = (0; 0; 0,58; 1), „ease-in-out" = (0,42; 0; 0,58; 1) [observat].

## Formate / protocoale
În proiect, câmp nou de tip de interpolare (decizie de schemă a proiectului). Nu se introduce format extern. Puncte: perechi (intrare, ieșire) cu intrări crescătoare după normalizare.

## Cazuri limită și erori
- Mai puțin de 2 puncte → eroare [observat].
- Valori NaN/infinit → eroare.
- p negativ sau > 1: curba pe segmente se extrapolează; treptele se limitează conform pasului 8.
- Plafon pe numărul de puncte/trepte (decizie de siguranță a proiectului).

## Teste de acceptanță
Curba A = puncte (ieșire/intrare): 0; 0,25 la 75%; 1.
1. A la p = 0,5 → 0,166667 (interpolare între (0;0) și (0,75;0,25)).
2. A la p = 0,75 → 0,25; la p = 1 → 1; la p = 1,2 → 1,6 (extrapolare cu panta 3); la p = −0,1 → −0,033333.
3. Curba B = puncte: 0; 0,8 la 60%; 0,4 la 30%; 1. Intrarea 30% se ridică la 60%. La p = 0,59 → 0,786667; la p = 0,6 → 0,4 (ultimul punct cu intrare ≤ 0,6 este al treilea; se interpolează spre ultimul).
4. Un singur punct → eroare.
5. Trepte(4, salt-la-sfârșit): p = 0,3 → 0,25; p = 1 → 1,0; p = 0 → 0.
6. Trepte(4, salt-la-început): p = 0,3 → 0,5; p = 0 → 0,25.
7. Trepte(4, fără-salt-la-capete): p = 0,5 → 0,666667 (pas 2 din 3 salturi); p = 0 → 0; p = 1 → 1.
8. Trepte(4, salt-la-ambele-capete): p = 0 → 0,2; p = 1 → 1,0.
9. Trepte(1, fără-salt-la-capete) → eroare; trepte(0, orice) → eroare.
10. Bézier cubică (1,5; 0; 0,5; 1) → eroare (abscisă în afara [0,1]); cuvântul „ease" → același rezultat ca (0,25; 0,1; 0,25; 1).

## Necunoscute
- [necunoscut] Textul este un proiect de lucru (FPWD); detaliile pot fi schimbate în versiuni ulterioare. Verifică la revizuire cu ultima versiune publicată.
- [necunoscut] Regula exactă pentru intrarea implicită a ultimului punct când el are o intrare explicită < 100%: sursa citită spune „cel puțin 1" pentru valoarea implicită; testele o evită.
- [necunoscut] Extrapolarea Bézier în afara [0,1] (secțiunea 2.3 menționează tangenta capetelor, cu cazuri speciale) nu e detaliată aici; proiectul poate limita la [0,1].
- [necunoscut] Semnificația „steagului dinainte" la animații cu direcție inversă (proiectul nu are direcție inversă pe segment).

## Checklist de filtru (verificare.md)
Spec: 05-easing-liniar-pe-segmente-si-trepte   Data: 2026-11-09
- [x] Nicio linie de cod, pseudocod sau asamblare copiată
- [x] Niciun nume intern de funcție, variabilă sau clasă din țintă
- [x] Niciun text de UI, asset sau branding din țintă
- [x] Algoritmii sunt descriși prin intrare → ieșire, nu prin structura codului țintei
- [x] Afirmațiile sunt marcate [observat] / [dedus] / [necunoscut]
- [ ] Licența/EULA țintei verificată și notată în `1-analiza/jurnal.md` (rând adăugat; licența documentului W3C neverificată în sesiune)
- [ ] Specificația nu depinde de brevete sau secrete comerciale (nu s-a făcut verificare de brevete)
- [ ] Aprobată de mine înainte de predare
