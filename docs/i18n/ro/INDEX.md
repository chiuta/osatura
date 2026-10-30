<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Indexul documentației OSATURA

*Începe de aici pentru a găsi pagina potrivită. Toate documentele sunt în Markdown englez sub CC-BY-4.0, cu excepția fișierelor generate ale terților (vezi antetele lor).*

OSATURA este un motor de rigging și animație vectorială 2D scris în Rust. Este un proiect alpha (primele build-uri sunt pregătite ca pre-lansare ciornă `v0.1.0-alpha.3`; crate-urile sunt încă 0.1.0); citește [LIMITATIONS.md](LIMITATIONS.md) înainte de a te baza pe o funcționalitate.

Cuprins: [Toate documentele](#toate-documentele) - [Trasee de lectură în funcție de public](#trasee-de-lectură-în-funcție-de-public) - [Documente din rădăcina repository-ului](#documente-din-rădăcina-repository-ului) - [Unde își găsește fiecare întrebare răspunsul](#unde-își-găsește-fiecare-întrebare-răspunsul) - [Convenții](#convenții-folosite-în-documente)

## Toate documentele

| Document | Ce este | Public |
|---|---|---|
| [GETTING_STARTED.md](GETTING_STARTED.md) | Instalare, primul proiect, prima randare și primul export, prima sesiune în studio | toată lumea, prima lectură |
| [CLI_REFERENCE.md](CLI_REFERENCE.md) | Fiecare comandă `osatura`, opțiune, cod de ieșire și exemplu | utilizatori, autori de scripturi, autori de CI |
| [STUDIO.md](STUDIO.md) | Editorul din browser: panouri, scurtături, colaborare, previzualizări scrub | animatori, riggeri |
| [RIGGING_GUIDE.md](RIGGING_GUIDE.md) | Construirea unui rig: oase, IK, skinning, deformatori, drivere, mașini de stări, layout | riggeri, artiști tehnici |
| [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) | Chei, easing, clipuri și take-uri, fizică, lip-sync, audio, alegeri de export | animatori |
| [TUTORIAL_SVG.md](../../TUTORIAL_SVG.md) | De la un desen vectorial la un GIF cu mers: import, rig prin script, export (comenzile și ieșirile sunt reale) | artiști |
| [COMPARISON.md](../../COMPARISON.md) | O comparație sinceră, datată, cu alte instrumente de animație 2D | evaluatori |
| [VERIFY_DOWNLOADS.md](../../VERIFY_DOWNLOADS.md) | Verificarea fișierelor de release: semnături Sigstore, proveniența build-ului, sume SHA-256 | oricine instalează |
| [UNINSTALL.md](../../UNINSTALL.md) | Dezinstalarea OSATURA și a tuturor fișierelor pe care le poate crea | oricine instalează |
| [../CODE_SIGNING_POLICY.md](../../../CODE_SIGNING_POLICY.md) · [../PRIVACY.md](../../../PRIVACY.md) | Politica de semnare a codului (SignPath Foundation), rolurile echipei, confidențialitate | oricine instalează, mentenanți |
| [GOOD_FIRST_TASKS.md](../../GOOD_FIRST_TASKS.md) | Sarcini mici pentru o primă contribuție | contributori noi |
| [PROJECT_FORMAT.md](PROJECT_FORMAT.md) | Formatul JSON al proiectului, câmp cu câmp | constructori de instrumente, riggeri care editează JSON |
| [EXPORT_FORMATS.md](EXPORT_FORMATS.md) | Ce transportă și ce pierde fiecare format de export și de import | constructori de pipeline-uri, integratori |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Crate-uri, module, flux de date, designul evaluării și al randării | contribuitori, recenzenți |
| [GLOSSARY.md](GLOSSARY.md) | Termeni folosiți în motor, în documente și în CLI | toată lumea |
| [FAQ.md](FAQ.md) | Răspunsuri scurte la întrebări frecvente | toată lumea |
| [API.md](API.md) | Fațada Rust stabilă (versiunea 1.4.0): apeluri, erori, limite, exemple | integratori Rust |
| [C_ABI.md](C_ABI.md) | Interfața C (`osatura-capi`): funcții, reguli de memorie, exemple în C și Python | integratori C, C++, Python, Go, C# |
| [WASM.md](WASM.md) | Modulul WebAssembly și wrapper-ul JavaScript. **Stare: construit și testat sub Node în CI; rulat în Chromium headless, nu încă în Firefox sau Safari**; include lista de verificare pentru prima rulare în browser | dezvoltatori web |
| [CI.md](CI.md) | CI de animație (`osatura ci`): manifest, praguri, goldens, rapoarte, exemple de pipeline | artiști tehnici, ingineri de build |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Compilare, teste, convenții, lista de verificare pentru module noi, lista de verificare pentru lansare, depanare | contribuitori, maintaineri |
| [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) | Pașii din partea proprietarului înainte și după publicarea lansării ciornă | proprietar |
| [SECURITY.md](SECURITY.md) | Model de amenințări, protecții încorporate, lacune cunoscute, raportare, lista de verificare pentru hardening | recenzori de securitate, cei care fac deploy |
| [LIMITATIONS.md](LIMITATIONS.md) | Listă numerotată a limitelor cunoscute cu severitate, soluție ocolitoare și id din roadmap | oricine evaluează OSATURA |
| [PATENT_NOTES.md](PATENT_NOTES.md) | Notă despre starea brevetelor (nu este sfat juridic) | licențiați, avocați |
| [THIRD_PARTY.md](THIRD_PARTY.md) | Inventar generat al crate-urilor și licențelor terților | licențiați, împachetători |
| [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md) | Texte de licență generate ale dependențelor, de livrat împreună cu binarele | împachetători |

## Trasee de lectură în funcție de public

Fiecare traseu este o ordine, nu o cerință. Oprește-te când ai găsit ce îți trebuie.

**Utilizator nou (vrea să vadă că funcționează).** [../README.md](../../../README.ro.md) (pornire rapidă) -> [GETTING_STARTED.md](GETTING_STARTED.md) -> [STUDIO.md](STUDIO.md) -> [FAQ.md](FAQ.md) -> [LIMITATIONS.md](LIMITATIONS.md).

**Rigger.** GETTING_STARTED -> [RIGGING_GUIDE.md](RIGGING_GUIDE.md) -> [PROJECT_FORMAT.md](PROJECT_FORMAT.md) (pentru câmpurile pe care le editezi manual) -> [GLOSSARY.md](GLOSSARY.md) -> [CLI_REFERENCE.md](CLI_REFERENCE.md) (comenzile `lint`, `check`, `fuzz`, `autorig`, `machine`) -> [CI.md](CI.md) pentru a proteja rig-ul.

**Animator.** GETTING_STARTED -> STUDIO -> [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) -> [EXPORT_FORMATS.md](EXPORT_FORMATS.md) (alege formatul de ieșire) -> CLI_REFERENCE (`export`, `export-fit`).

**Dezvoltator de pipeline-uri sau instrumente (scripturi, alte formate).** CLI_REFERENCE -> PROJECT_FORMAT -> EXPORT_FORMATS -> [CI.md](CI.md) -> [SECURITY.md](SECURITY.md) secțiunile 4 până la 7 (reguli pentru resurse și fișiere).

**Integrator (încorporează motorul).** Prezentarea generală din [ARCHITECTURE.md](ARCHITECTURE.md) -> alege nivelul: Rust ([API.md](API.md)), C și alte limbaje ([C_ABI.md](C_ABI.md)), browser ([WASM.md](WASM.md), citește mai întâi caseta de stare) -> [LIMITATIONS.md](LIMITATIONS.md) secțiunile 1 și 8 -> [SECURITY.md](SECURITY.md) secțiunea 8.

**Contribuitor.** [../CONTRIBUTING.md](CONTRIBUTING.md) (DCO) -> [DEVELOPMENT.md](DEVELOPMENT.md) -> ARCHITECTURE -> [../CLEANROOM.md](CLEANROOM.md) -> [../ROADMAP.md](ROADMAP.md) secțiunea 9 (ce există și ce nu) -> LIMITATIONS (alege ceva de reparat).

**Recenzor de securitate sau cel care face deploy.** [SECURITY.md](SECURITY.md) -> [DEVELOPMENT.md](DEVELOPMENT.md) secțiunea 3 (ce este testat) -> LIMITATIONS secțiunile 2 și 11 -> `osatura-cli/src/http.rs`, `studio.rs`, `collab_srv.rs`, `osatura-core/src/assets.rs`, `fsutil.rs` -> limitele din [CI.md](CI.md).

**Licențiat, împachetător sau avocat.** [../LICENSING.md](LICENSING.md) -> [../COPYRIGHT](../../../COPYRIGHT) și [../LICENSES/](../../../LICENSES/) -> [THIRD_PARTY.md](THIRD_PARTY.md) -> [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md) -> [PATENT_NOTES.md](PATENT_NOTES.md) -> [../TRADEMARK.md](TRADEMARK.md) -> [../CLEANROOM.md](CLEANROOM.md) -> lista de verificare pentru lansare din [DEVELOPMENT.md](DEVELOPMENT.md) secțiunea 7.

**Maintainer care face o lansare.** DEVELOPMENT secțiunea 7 -> [../CHANGELOG.md](CHANGELOG.md) -> LIMITATIONS (actualizare) -> caseta de stare din README.

## Documente din rădăcina repository-ului

| Fișier | Scop |
|---|---|
| [../README.md](../../../README.ro.md), [../README.ro.md](../../../README.ro.md) | Prezentare generală, stare, pornire rapidă, rezumatul licenței (engleză și română) |
| [../CHANGELOG.md](CHANGELOG.md) | Istoric reconstituit; format Keep-a-Changelog |
| [../ROADMAP.md](ROADMAP.md) | Plan ordonat; **secțiunea 9 este jurnalul datat al ce face și ce nu face fiecare element** și autoritatea atunci când documentele nu concordă |
| [../FEATURES.md](FEATURES.md) | Matricea de funcționalități în comparație cu alte instrumente (părți sunt depășite, vezi LIMITATIONS) |
| [../CLEANROOM.md](CLEANROOM.md) | Jurnal de proveniență și reguli pentru implementarea independentă |
| [../CONTRIBUTING.md](CONTRIBUTING.md) | Reguli de contribuție, licență de intrare = licență de ieșire, DCO 1.1 |
| [../LICENSING.md](LICENSING.md), [../COPYRIGHT](../../../COPYRIGHT), [../LICENSES/](../../../LICENSES/) | Decizia privind licența (Varianta A), texte |
| [../TRADEMARK.md](TRADEMARK.md) | Politica privind numele și logoul |
| `../REUSE.toml`, `../deny.toml` | Metadate de licență pentru fișierele care nu sunt sursă; politica dependențelor |

## Unde își găsește fiecare întrebare răspunsul

| Întrebare | Pagină |
|---|---|
| Cum instalez și cum fac prima mea animație? | GETTING_STARTED |
| Ce face comanda X? | CLI_REFERENCE |
| Ce înseamnă acest câmp din JSON? | PROJECT_FORMAT, GLOSSARY |
| Ce format de export păstrează măștile / gradienții / audio? | EXPORT_FORMATS |
| Îl pot folosi din Python / C / browser? | C_ABI / WASM (stare!) |
| Este sigur să expun studio într-o rețea? | SECURITY secțiunea 11 |
| De ce nu funcționează X? Este cunoscut? | LIMITATIONS, FAQ |
| Cum este construit și testat? | DEVELOPMENT |
| Îl pot folosi comercial? Care sunt licențele? | README (licență), LICENSING, THIRD_PARTY |
| Ce s-a schimbat? | CHANGELOG, ROADMAP secțiunea 9 |

## Convenții folosite în documente

* **„Verificat”** înseamnă că o comandă sau un program a fost rulat în timp ce se scriau documentele, într-un mediu cu Linux x86-64, fără GPU, fără webcam și doar cu Chromium headless. **„Netestat”** înseamnă exact asta. Afirmațiile despre WebAssembly într-un motor real sunt întotdeauna neverificate.
* Exemplele de cod din API.md și C_ABI.md sunt programele care au fost compilate și rulate; exemplele din WASM.md au rulat doar împotriva mock-ului JavaScript.
* Secțiunea 9 din ROADMAP este autoritatea pentru ce face și ce nu face o funcționalitate. Dacă un document nu concordă cu ea sau cu codul, codul câștigă; te rugăm să raportezi diferența.
* Căile sunt relative la rădăcina repository-ului, cu excepția celor care încep cu `docs/`.
* Numerele (numărul de teste, limite) provin din sursă sau dintr-o rulare din 22 oct. 2026 și pot deveni depășite după următoarea modificare; sursa este autoritatea.
