<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Checklist di rilascio (a cura del titolare)

*Passaggi che solo il titolare può compiere, in ordine. Stato al 24 ott 2026: il repository <https://github.com/chiuta/osatura> esiste, GitHub Actions è verde (test su Linux, macOS e Windows, clippy, wasm32, cargo-deny, REUSE) e la pre-release in bozza `v0.1.0-alpha.3` è stata compilata (archivi Windows, macOS e Linux, il modulo web e `SHA256SUMS.txt`); **nulla è ancora pubblicato**. I passaggi qui sotto che la CI ha già svolto sono spuntati; gli altri richiedono una persona, un avvocato o un account. Non è consulenza legale. Le voci contrassegnate **[blocker]** vanno completate prima di un annuncio pubblico.*

## 1. Prima di pubblicare qualsiasi cosa

- [ ] **[blocker]** Conferma o sostituisci il contatto per la sicurezza (`alexio@trom.tf` è un segnaposto in `docs/SECURITY.md`, in `--version` e nei documenti di licenza).
- [ ] **[blocker]** Cerca il nome "OSATURA" nei registri dei marchi che ti interessano (EUIPO, USPTO, OSIM) e negli app store / registri dei pacchetti. Vedi `TRADEMARK.md`. Se c'è un conflitto, cambia nome prima della prima release (il nome compare nel binario, nei crate, nella documentazione).
- [ ] **[blocker]** Chiedi a un avvocato di rivedere `LICENSING.md`, la suddivisione MPL-2.0 / AGPL-3.0, `docs/THIRD_PARTY.md` e `docs/PATENT_NOTES.md`. Decidi se vuoi la doppia licenza; in tal caso serve un CLA *prima* di accettare contributi esterni (oggi: solo DCO).
- [ ] Verifica i termini del riferimento sull'approssimazione di curve di Graphics Gems (citato a memoria in `CLEANROOM.md`); mantieni la formulazione sull'implementazione indipendente solo se è accurata.
- [x] `reuse lint` e `cargo deny check` passano in CI (il job `licences`); eseguili una volta anche sulla tua macchina.
- [ ] Verifica che nell'albero non ci siano dati personali, percorsi locali o token (`grep -r "/home/" .`, `grep -ri token`), e che il modello incluso e gli asset della demo siano tutti autogenerati.

## 2. Repository

- [x] Il repository GitHub esiste e l'albero è stato inviato senza `target/` e `out/` (`.gitignore`).
- [ ] Conserva `COPYRIGHT`, `LICENSES/`, `REUSE.toml`, `.github/FUNDING.yml` (Buy Me a Coffee `echo.of.the.strings`, Patreon `alexio_tf`).
- [ ] Abilita la segnalazione privata delle vulnerabilità (scheda Security) e fai puntare `docs/SECURITY.md` ad essa.
- [ ] Passa l'URL del repository con `--source-url` quando avvii lo studio (offerta del sorgente AGPL, route `/source`) e prova l'offerta da un'installazione pulita.

## 3. Primo giro di CI (eseguito il 2026-10-24)

- [x] `.github/workflows/ci.yml` è verde: test su Linux, Windows, macOS e `clippy -D warnings`; il job di build wasm32; il job cargo-deny e REUSE.
- [x] I fallimenti della prima esecuzione su Windows sono stati corretti (dimensione dello stack, test solo POSIX, accesso al log di collaborazione) e annotati in `docs/LIMITATIONS.md` (L-04).
- [x] Il job wasm32 passa (compilazione e test Node con il modulo reale). **Aperto:** esegui l'elenco di verifica di `docs/WASM.md` in un vero browser e aggiorna L-01 / L-02.

## 4. Prove con persone reali

- [ ] Dai lo studio a 3-5 animatori per una settimana con un breve compito (fare il rig di un personaggio, realizzare un ciclo di camminata, esportare una GIF). Registra dove si bloccano; apri una issue per ciascun caso.
- [ ] Prova su browser reali (Chrome, Firefox, Safari), su un dispositivo touch e in una sessione solo da tastiera.
- [ ] Su un vero **Windows 11** e su un vero **Mac**: decomprimi l'archivio, fai doppio clic su `osatura.exe` (Windows) e su `osatura-studio`, fai il rig e l'esportazione di un progetto e segnala che cosa non funziona (da L-74 a L-76).
- [ ] Prova le funzioni di rilevamento del volto / mocap / voce con dati reali; riporta onestamente che sono limitate (L-12, L-13).

## 5. Sicurezza del deployment (se ospiti lo studio o la collaborazione)

- [ ] Mettilo dietro un reverse proxy che termina il TLS (ricetta: `docs/SECURITY.md` sezione 12); non esporre mai su internet la porta HTTP in chiaro.
- [ ] Usa token per partecipante (`collab-token`), imposta una scadenza, fai il bind su localhost dietro il proxy.
- [ ] Commissiona una revisione di sicurezza indipendente prima di offrire la collaborazione ospitata a sconosciuti.

## 6. Artefatti di release

- [x] I binari di release sono compilati su runner ospitati da GitHub (`release.yml`) e `SHA256SUMS.txt` viene generato nella bozza. **Aperto:** controlla la bozza `v0.1.0-alpha.3`, spunta "pre-release", pubblicala e crea il tag sul commit (il tag viene creato con la release).
- [ ] Decidi sulla firma del codice (certificato Authenticode per Windows, Apple Developer ID e notarizzazione); fino ad allora i download sono segnalati come non firmati (L-74).
- [ ] Allega il sorgente corrispondente per le parti AGPL (il tag stesso basta; tienilo disponibile finché distribuisci i binari).
- [x] `docs/THIRD_PARTY_NOTICES.md` è rigenerato (24 ott 2026) e ogni archivio lo contiene; rigeneralo di nuovo con `scripts/licence-audit.sh` prima di ogni release successiva.
- [ ] Crates.io / npm / registri dei pacchetti: verifica che i nomi siano liberi e che i campi di licenza corrispondano a `LICENSING.md`.

## 7. Annuncio e donazioni

- [ ] Prima schermata del README: cosa funziona, cosa no (collega `docs/LIMITATIONS.md`); non dichiarare "di classe Moho" senza precisazioni.
- [ ] Mostra le schermate in `docs/img/` e una breve GIF realizzata con il progetto demo.
- [ ] Aggiungi i link per le donazioni (Buy Me a Coffee, Patreon) al README e alla finestra Informazioni dello studio; non promettere funzionalità in cambio di donazioni a meno che tu possa mantenere la promessa.
- [ ] Pubblica dove stanno gli animatori (community di rigging 2D / animazione indipendente); chiedi segnalazioni di bug, non elogi.
- [ ] Riserva del tempo per rispondere in fretta alle prime issue; le prime due settimane decidono se i contributori restano.
