# OSATURA — licențiere (DECIS: Varianta A, aplicată la 22 oct. 2026; vezi LICENSE și LICENSES/)

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/README.md) · [Română](https://github.com/chiuta/osatura/blob/main/README.ro.md) · [Français](https://github.com/chiuta/osatura/blob/main/README.fr.md) · [Italiano](https://github.com/chiuta/osatura/blob/main/README.it.md) · [Español](https://github.com/chiuta/osatura/blob/main/README.es.md) · [Português](https://github.com/chiuta/osatura/blob/main/README.pt.md) · [Deutsch](https://github.com/chiuta/osatura/blob/main/README.de.md) · [Magyar](https://github.com/chiuta/osatura/blob/main/README.hu.md)

**Stare:** Varianta A de mai jos a fost aleasă și aplicată: MPL-2.0 pentru biblioteci (core, capi, wasm, gpu), AGPL-3.0-or-later pentru `osatura-cli` (CLI + studio) și `osatura-gui` (studioul într-o fereastră nativă), CC-BY-4.0 pentru documentație și pentru resursele și modelele sintetice, politica de mărci în TRADEMARK.md, DCO în CONTRIBUTING.md, anteturi SPDX în toate fișierele .rs/.js/.mjs/.sh și REUSE.toml pentru restul, oferta de cod sursă conform secțiunii 13 din AGPL (`/source`, `--source-url`), câmpurile `license` din Cargo completate. Acțiuni rămase proprietarului: verificarea de către un avocat, căutarea mărcii (EUIPO/OSIM), verificarea brevetelor (docs/PATENT_NOTES.md), un CLA dacă se dorește licențiere duală, includerea notificărilor despre terți în lansările binare.


*Nu constituie consultanță juridică. Licența este DECISĂ (Varianta A, vezi LICENSE); secțiunile de mai jos sunt păstrate ca justificare și ca alternativele care au fost luate în considerare. Cere unui avocat (de preferat unul care cunoaște dreptul de autor românesc/european) să confirme înainte de prima lansare publică. Contribuții: nu există niciun drept de relicențiere sau de licențiere duală pentru contribuțiile terților până când nu există un CLA (CONTRIBUTING.md).*

## 1. Ce trebuie să facă licența pentru tine
Venitul tău provine din **donații (Buy Me a Coffee)**, nu din taxe de licență. Așadar licența nu creează banii — o fac **bunăvoința, vizibilitatea și încrederea**. Ea are trei roluri:
1. **Să maximizeze numărul de utilizatori** (mai mulți utilizatori → mai mulți susținători): trebuie să fie liberă de folosit, inclusiv în lucrări comerciale.
2. **Să împiedice o companie să ia codul, să îl închidă și să îl vândă** (sau să îl găzduiască drept serviciu plătit) fără a oferi nimic în schimb.
3. **Să îți păstreze dreptul de a vinde excepții** (licență comercială, suport) și de a apăra numele.

Trebuie să se potrivească și cu convingerile tale **trade-free / TROM**: software-ul nu trebuie să fie ciuntit, finanțat prin reclame sau legat de colectarea de date. (Licențele care interzic utilizarea comercială, de exemplu CC-NC sau Polyform-NC, *nu* sunt open-source și nu înseamnă același lucru cu „trade-free”; verifică formularea proprie a TROM înainte de a adopta oricare dintre ele.)

## 2. Constatări care limitează alegerea
- **Dependențele nu sunt un obstacol.** Auditul tuturor crate-urilor terțe (363 la regenerarea din 24 oct. 2026 cu `scripts/licence-audit.sh`, numărate pentru toate platformele împreună după adăugarea aplicației cu fereastră; erau 160 pe 22 oct.; numărul se schimbă, vezi docs/THIRD_PARTY.md): toate sunt MIT, Apache-2.0, BSD, Zlib, ISC, Unlicense, CC0, Unicode sau licențe duale ale acestora, cu excepția a șase crate-uri MPL-2.0 cu copyleft slab, la nivel de fișier: smartstring (MPL-2.0+, atins doar prin funcționalitatea `scripting`, care este ACTIVATĂ implicit în `osatura-core` și `osatura-cli`) și `cssparser`, `cssparser-macros`, `dtoa-short`, `selectors` și `option-ext` (atinse doar prin aplicația cu fereastră `osatura-gui`); o compilare a bibliotecilor cu `--no-default-features` nu conține niciun crate terț MPL. Ele pot fi combinate cu AGPL, MPL sau o licență proprietară; crate-ul MPL cere păstrarea codului sursă/notificării.
- **Titularul dreptului de autor.** O mare parte din cod a fost scrisă cu Claude. Dreptul de autor se leagă, în general, de contribuția creativă *umană* (ghidurile US Copyright Office; legislația UE/română cere, la fel, creația intelectuală proprie a unui autor uman). Consecințe practice: (a) poți și ar trebui să licențiezi proiectul în calitate de autor/compilator uman al său, dar (b) nu promite exclusivitatea fiecărei linii, (c) păstrează în depozit notele tale de proiectare, selecțiile și modificările ca dovadă a paternității, (d) un DCO atestă doar proveniența; pentru a relicenția ulterior contribuțiile (de exemplu licențiere duală) este nevoie de un CLA sau de o cesiune explicită în aval, iar acestea NU există.
- **Nicio licență nu poate forța donațiile.** Nu condiționa niciodată accesul la cod de o donație; un fork ar elimina pur și simplu condiția. Vinde *comoditate și extra*, nu permisiune.

## 3. Pachet recomandat — „bibliotecă open core, aplicație protejată” (toate variantele de mai jos sunt gata de aplicat)

### ★ Varianta A (recomandată): bibliotecă MPL-2.0 + aplicații AGPL-3.0 + marcă
| Parte | Licență | De ce |
|---|---|---|
| `osatura-core` (motor, evaluator, formate, exportatoare) și orice viitor runtime/WASM/C-ABI | **MPL-2.0** | Copyleft la nivel de fișier: oricine îl poate integra în jocuri/aplicații (închise sau deschise), dar modificările aduse *fișierelor noastre* trebuie împărtășite. Include o **acordare explicită de brevete** și încetare în caz de proces pentru brevete. Maximizează adopția, de care au nevoie donațiile. |
| `osatura-cli`, `osatura-gui`, studio (browser/fereastră) | **AGPL-3.0-or-later** | Oprește forkurile închise și clonele „OSATURA ca serviciu web plătit”; *proiectele și randările* utilizatorilor rămân în întregime ale lor. |
| Numele „OSATURA”, logo | **Politică de mărci** (fișier separat) | Forkurile trebuie să se redenumească; îi protejează pe susținători de imitații. Verifică disponibilitatea (EUIPO/OSIM/WIPO) înainte de lansare. |
| Rigurile de probă, grafica, sunetul create de tine | **CC BY 4.0** (sau CC BY-SA) | Libere de reutilizat cu atribuire; grafica terților își păstrează propria licență. |
| Documentație | **CC BY 4.0** | |
<!-- REUSE-IgnoreStart -->
Cargo: `license = "MPL-2.0"` (core), `license = "AGPL-3.0-or-later"` (cli). SPDX header per file: `// SPDX-License-Identifier: MPL-2.0`.
<!-- REUSE-IgnoreEnd -->
**Licențiere duală:** posibilă doar pentru codul scris exclusiv de proprietar sau după un CLA/o cesiune în aval din partea fiecărui contribuitor. NU este posibilă astăzi pentru codul contribuit (CONTRIBUTING.md).

### Varianta B: AGPL-3.0-or-later peste tot (+ licență comercială la cerere)
Cea mai puternică poziție împotriva deturnării și cea mai apropiată de etosul trade-free. Cost: companiile și motoarele de joc vor evita integrarea runtime-ului, deci mai puțini utilizatori și mai puțini susținători indirecți. Alege-o dacă protecția contează mai mult decât acoperirea.

### Varianta C: Apache-2.0 peste tot (nealeasă; workspace-ul aplică Varianta A: biblioteci MPL-2.0 și `license = "AGPL-3.0-or-later"` pentru `osatura-cli`)
Acoperire maximă și zero fricțiune; include acordarea de brevete. Cost: oricine — inclusiv o companie mare — îl poate închide și vinde fără nicio obligație. Donațiile depind atunci exclusiv de bunăvoință. Alege-o doar dacă vrei să fii o utilitate publică.

### Varianta D: Cod sursă disponibil (BUSL-1.1 sau Polyform Shield/NC cu dată de deschidere amânată)
Împiedică concurența comercială timp de N ani. Cost: **nu este open source**; intră în conflict cu povestea clean-room/formate deschise, cu contribuțiile comunității și cu valorile trade-free; mulți utilizatori și distribuții o vor refuza. **Nu este recomandată.**

| | Acoperire | Împotriva deturnării | Acordare de brevete | Venit din licență duală | Se potrivește cu trade-free |
|---|---|---|---|---|---|
| **A core MPL + aplicații AGPL** | mare | bună | da | da | da |
| B AGPL peste tot | medie | cea mai puternică | da (v3) | da | da |
| C Apache peste tot | maximă | niciuna | da | nu | parțial |
| D cod sursă disponibil | mică | puternică | n/a | da | nu |

## 4. Cum să câștigi efectiv din donații (toate compatibile cu Varianta A)
1. **Pagina Buy Me a Coffee** (`buymeacoffee.com/echo.of.the.strings`) legată din README, din textul `--version` al CLI, din subsolul studioului și din linia de credit a playerului HTML (un credit *ce poate fi eliminat* — niciodată obligatoriu).
2. **Abonamente, nu bariere de plată:** niveluri lunare pentru versiuni timpurii, votarea următorului element din roadmap, numele în `SUPPORTERS.md`, chat privat. Toți primesc același cod; membrii primesc *timp și voce*.
3. **Extra plătite care nu sunt codul:** binare/programe de instalare semnate, gata de rulat, pachete de resurse (riguri, fundaluri, biblioteci de cicluri de mers) sub o licență care permite folosirea în filme/jocuri, dar nu revânzarea pachetului în sine, cursuri video, ateliere.
4. **Comenzi și suport:** servicii de rigging, exportatoare la comandă, consultanță de integrare; licența comercială duală pentru cei care integrează.
5. **Niveluri de sponsorizare pentru organizații** (logo în README) și cereri de finanțare către fonduri open-source/creative (NLnet, programe de tip Sovereign Tech, fonduri culturale) — ele plătesc exact pentru acest fel de infrastructură deschisă.
6. **Pagină de transparență:** publică veniturile și ce a finanțat fiecare; crește măsurabil sprijinul recurent.
7. Păstrează **Patreon** (`patreon.com/c/alexio_tf`) și BMC vizibile ambele, dar dă-le scopuri distincte (BMC pentru mulțumiri unice; Patreon pentru sprijin recurent), ca susținătorii să nu fie confuzi.

## 5. Listă de verificare pentru aplicarea Variantei A (≈ 1 oră de lucru după alegere)
- [x] `LICENSES/MPL-2.0.txt`, `LICENSES/AGPL-3.0-or-later.txt`, `LICENSES/CC-BY-4.0.txt`, `LICENSES/LicenseRef-third-party.txt` (structura REUSE) plus rezumatul `LICENSE`; `REUSE.toml` acoperă fișierele care nu conțin cod sursă.
- [x] `license =` setat în toate cele șase fișiere `Cargo.toml`; anteturi SPDX în toate fișierele .rs/.js/.mjs/.sh, verificate de `osatura-cli/tests/spdx_audit.rs`.
- [x] `docs/THIRD_PARTY.md` și `docs/THIRD_PARTY_NOTICES.md`, ambele regenerate de `scripts/licence-audit.sh`.
- [x] `CONTRIBUTING.md` cu textul DCO 1.1. Un CLA este în continuare necesar înainte de orice licență duală.
- [x] `TRADEMARK.md` (permis: „compatibil cu OSATURA”, redistribuirea nemodificată; nepermis: versiuni modificate sub același nume).
- [x] `.github/FUNDING.yml` (`buy_me_a_coffee: echo.of.the.strings`) pentru ca platforma să afișeze butonul Sponsor.
- [~] Căutare de marcă: primul nume, ARMATURA, intra în conflict cu o marcă din clasele 9/42 (EUIPO, 24 oct. 2026), așa că proiectul a fost redenumit OSATURA; căutarea EUIPO pentru „osatura” nu a găsit niciun conflict (un rezultat expirat fără legătură, în clasa 10). Numele de pachete `osatura` pe crates.io, npm și PyPI sunt libere (verificat 24 oct. 2026), dar există o marcă apropiată, OSATRA (Open-Xchange AG, înreg. SUA 7459236, clasele 9/38/42). Căutările OSIM / TMview / WIPO / de domenii și avizul unui avocat rămân deschise, vezi [docs/LIMITATIONS.md](LIMITATIONS.md) L-78.
