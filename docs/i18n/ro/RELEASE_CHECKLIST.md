<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Listă de verificare pentru lansare (partea proprietarului)

*Pași pe care îi poate face doar proprietarul, în ordine. Stare la 24 oct. 2026: depozitul <https://github.com/chiuta/osatura> există, GitHub Actions este verde (teste pe Linux, macOS și Windows, clippy, wasm32, cargo-deny, REUSE), iar pre-lansarea ciornă `v0.1.0-alpha.3` a fost construită (arhive pentru Windows, macOS și Linux, modulul web și `SHA256SUMS.txt`); **nimic nu este încă publicat**. Pașii de mai jos pe care CI i-a făcut deja sunt bifați; restul au nevoie de o persoană, un avocat sau un cont. Nu constituie consultanță juridică. Elementele marcate cu **[blocker]** trebuie făcute înainte de un anunț public.*

## 1. Înainte de a publica orice

- [ ] **[blocker]** Confirmă sau înlocuiește contactul de securitate (`alexio@trom.tf` este un substituent în `docs/SECURITY.md`, în `--version` și în documentele de licență).
- [ ] **[blocker]** Caută numele „OSATURA” în registrele de mărci care te interesează (EUIPO, USPTO, OSIM) și în magazinele de aplicații / registrele de pachete. Vezi `TRADEMARK.md`. Dacă apare o suprapunere, redenumește înainte de prima lansare (numele apare în binar, crate-uri, documentație).
- [ ] **[blocker]** Cere unui avocat să verifice `LICENSING.md`, împărțirea MPL-2.0 / AGPL-3.0, `docs/THIRD_PARTY.md` și `docs/PATENT_NOTES.md`. Decide dacă vrei licențiere duală; dacă da, este nevoie de un CLA *înainte* de a accepta contribuții din exterior (în prezent: doar DCO).
- [ ] Verifică termenii referinței Graphics Gems pentru potrivirea curbelor (citată din memorie în `CLEANROOM.md`); păstrează formularea despre implementare independentă doar dacă este corectă.
- [x] `reuse lint` și `cargo deny check` trec în CI (jobul `licences`); rulează-le o dată și pe mașina ta.
- [ ] Verifică că în arbore să nu existe date personale, căi locale sau token-uri (`grep -r "/home/" .`, `grep -ri token`) și că modelul inclus și resursele demo să fie toate generate de tine.

## 2. Depozit

- [x] Depozitul GitHub există, iar arborele a fost încărcat fără `target/` și `out/` (`.gitignore`).
- [ ] Păstrează `COPYRIGHT`, `LICENSES/`, `REUSE.toml`, `.github/FUNDING.yml` (Buy Me a Coffee `echo.of.the.strings`, Patreon `alexio_tf`).
- [ ] Activează raportarea privată a vulnerabilităților (fila Security) și trimite către ea din `docs/SECURITY.md`.
- [ ] Transmite URL-ul depozitului cu `--source-url` la rularea studioului (oferta de cod sursă AGPL, ruta `/source`) și testează oferta pe o instalare nouă.

## 3. Prima rulare CI (făcută 2026-10-24)

- [x] `.github/workflows/ci.yml` este verde: testele pe Linux, Windows, macOS și `clippy -D warnings`; jobul de build wasm32; jobul cargo-deny și REUSE.
- [x] Eșecurile de la prima rulare pe Windows au fost reparate (dimensiunea stivei, teste doar-POSIX, accesul la jurnalul de colaborare) și consemnate în `docs/LIMITATIONS.md` (L-04).
- [x] Jobul wasm32 trece (build și testul Node cu modulul real). **Deschis:** rulează lista de verificare din `docs/WASM.md` într-un browser real și actualizează L-01 / L-02.

## 4. Testare cu oameni

- [ ] Dă studioul la 3–5 animatori pentru o săptămână, cu o sarcină scurtă (rig pentru un personaj, un ciclu de mers, export GIF). Notează unde se blochează; deschide câte un issue pentru fiecare.
- [ ] Testează pe browsere reale (Chrome, Firefox, Safari), un dispozitiv tactil și o sesiune doar cu tastatura.
- [ ] Pe un **Windows 11** real și pe un **Mac** real: despachetează arhiva, fă dublu clic pe `osatura.exe` (Windows) și pe `osatura-studio`, fă rig și exportă un proiect și raportează ce nu merge (L-74 până la L-76).
- [ ] Încearcă funcțiile de detector de fețe / mocap / voce cu date reale; raportează sincer că sunt limitate (L-12, L-13).

## 5. Siguranța implementării (dacă găzduiești studioul sau colaborarea)

- [ ] Pune-l în spatele unui reverse proxy care termină TLS (rețeta: `docs/SECURITY.md` secțiunea 12); nu expune niciodată portul HTTP simplu pe internet.
- [ ] Folosește token-uri pe participant (`collab-token`), setează expirarea, leagă la localhost în spatele proxy-ului.
- [ ] Comandă o verificare de securitate independentă înainte de a oferi colaborare găzduită unor necunoscuți.

## 6. Artefacte de lansare

- [x] Binarele de lansare sunt construite pe runnere găzduite de GitHub (`release.yml`), iar `SHA256SUMS.txt` este generat în ciornă. **Deschis:** verifică ciorna `v0.1.0-alpha.3`, bifează „pre-release”, publică-o și etichetează commit-ul (tag-ul este creat odată cu lansarea).
- [x] Semnarea de proveniență: semnături Sigstore fără chei (keyless) și atestări GitHub de proveniență a build-ului pentru fiecare fișier de lansare (jobul `sign` din `release.yml`, verificat în CI).
- [x] Cerințele SignPath Foundation din depozit: CODE_SIGNING_POLICY.md (legat din README, de pe site și din fiecare lansare), PRIVACY.md, docs/UNINSTALL.md, fără licențiere duală (LICENSING.md), metadatele fișierelor Windows (`build.rs`, verificate în release.yml), configurația artefactului în `.signpath/`, jobul `windows-authenticode`.
- [ ] Proprietar: activează autentificarea în doi factori pe GitHub (cerută de SignPath pentru fiecare membru al echipei).
- [ ] Proprietar: publică o lansare (SignPath cere ca proiectul să fie lansat în forma care va fi semnată), apoi aplică la https://signpath.org/apply.
- [ ] După acceptare: în SignPath creează proiectul `osatura`, politica de semnare `release-signing`, configurația de artefact `windows-zip` (lipește `.signpath/artifact-configuration.xml`) și leagă depozitul GitHub ca sistem de build de încredere; în GitHub adaugă variabila de depozit `SIGNPATH_ORGANIZATION_ID` și secretul `SIGNPATH_API_TOKEN`. Următoarea lansare este apoi semnată după aprobare manuală.
- [ ] macOS: Apple Developer ID și notarizare (încă neplanificate); până atunci Gatekeeper avertizează (L-74).
- [ ] Atașează codul sursă corespunzător pentru părțile AGPL (eticheta în sine este suficientă; păstrează-l disponibil cât timp distribui binare).
- [x] `docs/THIRD_PARTY_NOTICES.md` este regenerat (24 oct. 2026), iar fiecare arhivă îl conține; regenerează-l din nou cu `scripts/licence-audit.sh` înainte de fiecare lansare ulterioară.
- [ ] Crates.io / npm / registre de pachete: verifică dacă numele sunt libere și dacă câmpurile de licență corespund cu `LICENSING.md`.

## 7. Anunț și donații

- [ ] Primul ecran din README: ce funcționează, ce nu (leagă `docs/LIMITATIONS.md`); nu pretinde „clasa Moho” fără rezerve.
- [ ] Arată capturile de ecran din `docs/img/` și un GIF scurt făcut cu proiectul demo.
- [ ] Adaugă linkurile de donații (Buy Me a Coffee, Patreon) în README și în caseta About a studioului; nu promite funcționalități în schimbul donațiilor decât dacă poți ține promisiunea.
- [ ] Postează acolo unde sunt animatorii (comunități de rigging 2D / animație indie); cere rapoarte de bug-uri, nu laude.
- [ ] Rezervă timp pentru a răspunde rapid la primele issue-uri; primele două săptămâni decid dacă contribuitorii rămân.
