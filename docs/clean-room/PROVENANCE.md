# Proveniență și clean-room — OSATURA (chiuta/osatura)

Document de completat cu fapte reale. Nu declara clean-room pentru ce nu s-a întâmplat așa.

## Ce a existat până acum (completează onest)
- Ținta/țintele: Moho (animație 2D / rigging)
- Cum s-a documentat comportamentul țintei (documentație publică, utilizare, formate observate, decompilare?): ______
- A fost folosit vreun cod, pseudocod, decompilare sau asset al țintei în sesiunile de dezvoltare? DA / NU / NU ȘTIU: ______
- Sesiunile de dezvoltare au avut vreodată acces la binarul sau codul țintei? ______

## Risc de licență
Software comercial proprietar. Verifică EULA pentru clauze de reverse engineering. Formatele de fișier proprii pot avea restricții; preferă formate deschise.

## Dependențe terțe
Rulează `cargo license` (sau `cargo deny check licenses`) și atașează rezultatul. Verifică compatibilitatea cu licența proiectului.

## Pentru lucrul de acum înainte
Folosește fluxul din acest kit (1-analiza → 2-filtru → 3-implementare). Orice funcție nouă inspirată de o țintă trece prin specificație și filtru.

## Recomandare
Pentru lansări comerciale sau donații semnificative, cere o opinie de la un avocat de proprietate intelectuală și fă căutare de mărci (EUIPO), cum ai făcut pentru OSATURA.

## Constatări din audit adversarial (2026-11-08) — fapte verificate în repo

**Corectură la versiunea anterioară:** am spus că nu s-a putut rula `cargo metadata` și am prezentat auditul de licențe al repo-ului ca atare; rămâne valabil doar ca declarație a proiectului (nereverificată independent). Un auditor a semnalat lipsa atribuirii pentru Schneider (Graphics Gems); la verificare, atribuirea **există** în `docs/THIRD_PARTY.md:388`, deci constatarea a fost retrasă.

### Starea documentelor proprii
`CLEANROOM.md` spune deja onest că proiectul NU e clean-room strict, că nu s-au produs note de specificație înainte de implementare și că agentul AI de cercetare a consultat depozite de cod ale altor unelte (rive-runtime, spine-runtimes, Pencil2D, Glaxnimate, Friction, editoare compatibile Live2D). Acest kit completează acel document.

### Istoric
- Istoricul public începe cu un singur commit: `e067476` (2026-10-23, „initial public tree"), 225 de fișiere, +151.748 de linii. Ce s-a întâmplat înainte de 23 octombrie nu poate fi auditat. Păstrează sau arhivează istoricul real (sesiuni, jurnale, prompturi) dinaintea acestui commit.
- 150 din 214 de commit-uri sunt de la „Claude" (cod scris în mare parte cu AI); restul sunt tu, în patru identități de e-mail (12 commit-uri cu `chiuta@gmail.com`, 3 cu `alexio@trom.tf`). Nu există contribuitori externi. `COPYRIGHT` afirmă drepturi pentru autorul uman; `LICENSING.md` recunoaște că ele se atașează doar contribuției umane creative.
- `CONTRIBUTING.md` cere DCO (`Signed-off-by`), dar doar ~74 din 214 de commit-uri îl au.

### Moho și formatele lui
- Nu s-au găsit: extensia `.moho`, magic bytes, identificatori din API-ul Lua Moho, antete de copyright străine, asset-uri sau fonturi terțe. Fișierele binare (`osatura_wasm.wasm`, `face23_synth.bin`) nu conțin șiruri ale produselor terțe.
- „Moho" apare în cod doar la importul unui fișier text de foneme cu antet `MohoSwitch1` (`importers.rs`), format simplu răspândit prin Papagayo. README spune „nu citește formate proprietare... nu e compatibil cu Moho"; nuanțează formularea.
- **Termeni apropiați de cei Moho folosiți ca nume de structuri:** `LayerKind::Switch` (`project.rs:498`) și `LayerKind::Liquid` (`project.rs:501`), plus `add_switch` în API-ul de scripting. „Switch layer" e termen și în alte unelte (ex. Synfig), dar `ROADMAP.md` regula 5 cere nume proprii. Redenumește sau notează explicit că sunt termeni generici.
- `FEATURES.md` pornește de la lista de funcții a produsului de referință și a folosit pagini publice, știri și postări. `FEATURES.md:998` listează un depozit cu nume de produs comercial („Moho-Pro-14.4-...") ca „neevaluat" și `:1000` spune că depozitele de software piratat nu au fost deschise. Afirmația nu e verificabilă; elimină linkul din document.

### „Clean-room" ca etichetă
Eticheta mai apare în ~96 de fișiere în afara `CLEANROOM.md` (comentarii „Clean-room design note" în 15 module din `osatura-core/src`, `ROADMAP.md`, `FEATURES.md`, `docs/GLOSSARY.md`, documentația tradusă, pagina de site). `ROADMAP.md` cere proceduri neaplicate (două roluri, note în `docs/specs/R-xxx.md` — dosarul nu există —, jurnal cu ID/sursă/dată/implementator). Aliniază formularea la „implementare independentă".

### Cod și constante
- Constantele neobișnuite (`noise.rs`, `anim.rs`, SHA-256, FNV) sunt standard sau atribuite în comentarii; constantele de easing Penner (`7.5625`, `1.70158`) nu apar nicăieri în istoricul vizibil. Afirmația din `CLEANROOM.md` despre versiuni anterioare ale `anim.rs` nu poate fi verificată (istoric lipsă).
- `autotrace/geometry.rs` urmărește structural algoritmul Schneider (Graphics Gems I), atribuit în `THIRD_PARTY.md:388`.

### Dependențe și distribuire
- `Cargo.lock`: 364 de crate-uri, toate din crates.io, cu checksum, fără surse git/path. Șase crate-uri MPL-2.0 sunt listate în `THIRD_PARTY.md`, dar `CLEANROOM.md:11` și `deny.toml` menționează doar `smartstring` și `option-ext` ca excepții: documentele se contrazic (nu licențele). `THIRD_PARTY.md` listează `rhai 1.26.1`, iar `Cargo.lock` are `1.27.0`: regenerează.
- `try/osatura_wasm.wasm` (2,8 MB) include tiny-skia (BSD-3-Clause), png, serde_json ș.a., dar `try/` nu conține notele terților, doar un `SOURCE.txt`. Pune notificările alături sau un link din `try/index.html`; build reproductibil.
- Videoclipurile (`tutorials/v/*.mp4`) sunt comise, deși `scripts/tutorials/README.md` spune că nu sunt; licența fonturilor din subtitrările arse nu e înregistrată.
- 41 de crate-uri apar fără text de licență în `THIRD_PARTY_NOTICES.md`.

### Brevete (observații, fără evaluare)
`docs/PATENT_NOTES.md` nu acoperă: aliniere forțată pentru lip-sync (`speech.rs`), completare/interpolare automată de poze (`posesuggest`, `tinymlp`), retargeting mocap, rigging automat (`autorig`), bucle fără cusătură/potrivire de spring (`looper`, `springfit`), fizică (`physics`, `cloth`), CRDT pentru colaborare (`collab`), vectorizare automată (`autotrace`), hash perceptual (`percep`), efecte de imagine (`dof`, `relight`), merge semantic (`vcs`, `semdiff`). Ocolirea brevetelor Adobe citate în `ROADMAP.md:40` (RBF Wendland) nu a fost verificată de un specialist.

### Completat din istoricul conversațiilor Claude (2026-11-08)
Sursa: conversația claude.ai „Moho 14.5 Rust implementation" (508 de rânduri/ture; pornită cu cercetare web). Nu s-a citit integral; ce nu apare mai jos nu este confirmat.
- **Acces la Moho:** în porțiunile citite, sesiunea a pornit de la căutări web despre Moho 14.5 și de la cererea de implementare clean-room. Nu apare nicio instalare, decompilare sau citire a binarului ori a scripturilor Moho. Absența nu este dovedită pentru ture necitite. Titularul a declarat că vrea monetizare prin donații și siguranță la drepturi de autor/brevete; a rulat o căutare EUIPO („armatura" a arătat o marcă conflictuală în clasa software, „osatura" nu), după care proiectul a fost redenumit (24.10.2026). Căutarea nu acoperă brevetele. Titularul confirmă accesul la Moho: ______
- **Depozite deschise (runda 7, 2026-10-19):** agentul a citit prin WebFetch doar paginile/README-urile publice ale unor proiecte (Tahoma2D, Pencil2D, rive-runtime, spine-runtimes, dotlottie-rs, Glaxnimate, Friction, topicuri GitHub) și a consemnat în FEATURES.md doar funcții declarate. GitLab/Codeberg au fost blocate de proxy. Nu s-a citit cod sursă din depozitele GPL, ci doar README-uri. Rezultatul este un tabel de funcții („Seen in"), nu cod.
- **Intenția folosirii listei de funcții:** FEATURES.md a servit ca roadmap funcțional (idei 1–77). Funcțiile sunt idei/capabilități, nu expresie; rămâne riscul de similaritate de nume și de brevete (vezi mai sus).
- **Switch/Liquid:** decizie neconsemnată în istoric. Titularul decide (redenumire sau justificare): ______

**Limite:** nu s-a rulat `cargo`/`cargo deny` (toolchain nedescărcabil în audit); nu s-a comparat cu depozite terțe și nu s-a rulat un test de similaritate de cod; binarele nu au fost reconstruite; EULA Moho nu a fost citit; nu este opinie juridică și nici analiză de libertate de operare pentru brevete.
