<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Indice della documentazione di OSATURA

*Parti da qui per trovare la pagina giusta. Tutti i documenti sono Markdown in inglese con licenza CC-BY-4.0, tranne i file di terze parti generati (vedi le loro intestazioni). Il README in rumeno è [../README.ro.md](../../../README.it.md).*

OSATURA è un motore di rigging e animazione vettoriale 2D in Rust. È un progetto alpha (le prime build sono preparate come pre-release in bozza `v0.1.0-alpha.3`; i crate sono ancora 0.1.0); leggi [LIMITATIONS.md](LIMITATIONS.md) prima di fare affidamento su una funzionalità.

Indice: [Tutti i documenti](#tutti-i-documenti) - [Percorsi di lettura per destinatario](#percorsi-di-lettura-per-destinatario) - [Documenti nella radice del repository](#documenti-nella-radice-del-repository) - [Dove trovare la risposta a ogni domanda](#dove-trovare-la-risposta-a-ogni-domanda) - [Convenzioni](#convenzioni-usate-nei-documenti)

## Tutti i documenti

| Documento | Cos'è | Destinatari |
|---|---|---|
| [GETTING_STARTED.md](GETTING_STARTED.md) | Installazione, primo progetto, primo render ed esportazione, prima sessione nello studio | tutti, prima lettura |
| [CLI_REFERENCE.md](CLI_REFERENCE.md) | Ogni comando `osatura`, opzione, codice di uscita ed esempio | utenti, autori di script, autori di CI |
| [STUDIO.md](STUDIO.md) | L'editor nel browser: pannelli, scorciatoie, collaborazione, anteprime di scrub | animatori, rigger |
| [RIGGING_GUIDE.md](RIGGING_GUIDE.md) | Costruire un rig: ossa, IK, skinning, deformatori, driver, macchine a stati, layout | rigger, technical artist |
| [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) | Chiavi, easing, clip e take, fisica, lip-sync, audio, scelte di esportazione | animatori |
| [TUTORIAL_SVG.md](TUTORIAL_SVG.md) | Da un disegno vettoriale a una GIF che cammina: importazione, rig da script, esportazione (comandi e output sono reali) | artisti |
| [COMPARISON.md](COMPARISON.md) | Un confronto onesto e datato con altri strumenti di animazione 2D | valutatori |
| [VERIFY_DOWNLOADS.md](VERIFY_DOWNLOADS.md) | Verifica degli archivi di release con le somme SHA-256 | chiunque installi |
| [GOOD_FIRST_TASKS.md](GOOD_FIRST_TASKS.md) | Piccoli compiti per un primo contributo | nuovi contributori |
| [PROJECT_FORMAT.md](PROJECT_FORMAT.md) | Il formato di progetto JSON campo per campo | autori di strumenti, rigger che modificano il JSON |
| [EXPORT_FORMATS.md](EXPORT_FORMATS.md) | Cosa trasporta e cosa perde ciascun formato di esportazione e importazione | autori di pipeline, integratori |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Crate, moduli, flusso dei dati, progettazione della valutazione e del rendering | contributori, revisori |
| [GLOSSARY.md](GLOSSARY.md) | Termini usati nel motore, nella documentazione e nella CLI | tutti |
| [FAQ.md](FAQ.md) | Risposte brevi alle domande frequenti | tutti |
| [API.md](API.md) | La facciata Rust stabile (versione 1.4.0): chiamate, errori, limiti, esempi | integratori Rust |
| [C_ABI.md](C_ABI.md) | L'interfaccia C (`osatura-capi`): funzioni, regole di memoria, esempi in C e Python | integratori C, C++, Python, Go, C# |
| [WASM.md](WASM.md) | Modulo WebAssembly e wrapper JavaScript. **Stato: compilato e testato sotto Node in CI; eseguito in Chromium headless, non ancora in Firefox o Safari**; include la checklist per la prima esecuzione in un browser | sviluppatori web |
| [CI.md](CI.md) | CI dell'animazione (`osatura ci`): manifest, soglie, fotogrammi di riferimento, report, esempi di pipeline | technical artist, build engineer |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Compilazione, test, convenzioni, checklist per un nuovo modulo, checklist di release, debug | contributori, maintainer |
| [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) | Passi a carico del titolare prima e dopo la pubblicazione della release in bozza | titolare |
| [SECURITY.md](SECURITY.md) | Modello delle minacce, protezioni integrate, lacune note, segnalazione, checklist di hardening | revisori della sicurezza, chi installa in produzione |
| [LIMITATIONS.md](LIMITATIONS.md) | Elenco numerato dei limiti noti con gravità, soluzione alternativa e id della roadmap | chiunque valuti OSATURA |
| [PATENT_NOTES.md](PATENT_NOTES.md) | Nota sullo stato dei brevetti (non è consulenza legale) | licenziatari, avvocati |
| [THIRD_PARTY.md](THIRD_PARTY.md) | Inventario generato dei crate di terze parti e delle licenze | licenziatari, packager |
| [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md) | Testi delle licenze delle dipendenze generati, da distribuire con i binari | packager |

## Percorsi di lettura per destinatario

Ogni percorso è un ordine, non un obbligo. Fermati quando hai ciò che ti serve.

**Nuovo utente (vuole vederlo funzionare).** [../README.md](../../../README.it.md) (avvio rapido) -> [GETTING_STARTED.md](GETTING_STARTED.md) -> [STUDIO.md](STUDIO.md) -> [FAQ.md](FAQ.md) -> [LIMITATIONS.md](LIMITATIONS.md).

**Rigger.** GETTING_STARTED -> [RIGGING_GUIDE.md](RIGGING_GUIDE.md) -> [PROJECT_FORMAT.md](PROJECT_FORMAT.md) (per i campi che modifichi a mano) -> [GLOSSARY.md](GLOSSARY.md) -> [CLI_REFERENCE.md](CLI_REFERENCE.md) (i comandi `lint`, `check`, `fuzz`, `autorig`, `machine`) -> [CI.md](CI.md) per proteggere il rig.

**Animatore.** GETTING_STARTED -> STUDIO -> [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) -> [EXPORT_FORMATS.md](EXPORT_FORMATS.md) (scegli il formato di output) -> CLI_REFERENCE (`export`, `export-fit`).

**Sviluppatore di pipeline o strumenti (script, altri formati).** CLI_REFERENCE -> PROJECT_FORMAT -> EXPORT_FORMATS -> [CI.md](CI.md) -> [SECURITY.md](SECURITY.md) sezioni da 4 a 7 (regole su asset e file).

**Integratore (incorpora il motore).** Panoramica di [ARCHITECTURE.md](ARCHITECTURE.md) -> scegli il livello: Rust ([API.md](API.md)), C e altri linguaggi ([C_ABI.md](C_ABI.md)), browser ([WASM.md](WASM.md), leggi prima il riquadro di stato) -> [LIMITATIONS.md](LIMITATIONS.md) sezioni 1 e 8 -> [SECURITY.md](SECURITY.md) sezione 8.

**Contributore.** [../CONTRIBUTING.md](CONTRIBUTING.md) (DCO) -> [DEVELOPMENT.md](DEVELOPMENT.md) -> ARCHITECTURE -> [../CLEANROOM.md](CLEANROOM.md) -> [../ROADMAP.md](ROADMAP.md) sezione 9 (cosa esiste e cosa no) -> LIMITATIONS (scegli qualcosa da correggere).

**Revisore della sicurezza o chi installa in produzione.** [SECURITY.md](SECURITY.md) -> [DEVELOPMENT.md](DEVELOPMENT.md) sezione 3 (cosa viene testato) -> LIMITATIONS sezioni 2 e 11 -> `osatura-cli/src/http.rs`, `studio.rs`, `collab_srv.rs`, `osatura-core/src/assets.rs`, `fsutil.rs` -> limiti di [CI.md](CI.md).

**Licenziatario, packager o avvocato.** [../LICENSING.md](LICENSING.md) -> [../COPYRIGHT](../../../COPYRIGHT) e [../LICENSES/](../../../LICENSES/) -> [THIRD_PARTY.md](THIRD_PARTY.md) -> [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md) -> [PATENT_NOTES.md](PATENT_NOTES.md) -> [../TRADEMARK.md](TRADEMARK.md) -> [../CLEANROOM.md](CLEANROOM.md) -> checklist di release in [DEVELOPMENT.md](DEVELOPMENT.md) sezione 7.

**Maintainer che prepara una release.** DEVELOPMENT sezione 7 -> [../CHANGELOG.md](CHANGELOG.md) -> LIMITATIONS (aggiorna) -> riquadro di stato del README.

## Documenti nella radice del repository

| File | Scopo |
|---|---|
| [../README.md](../../../README.it.md), [../README.ro.md](../../../README.it.md) | Panoramica, stato, avvio rapido, riepilogo delle licenze (in inglese e in rumeno) |
| [../CHANGELOG.md](CHANGELOG.md) | Cronologia ricostruita; formato Keep-a-Changelog |
| [../ROADMAP.md](ROADMAP.md) | Piano ordinato; **la sezione 9 è il registro datato di ciò che ogni voce fa e non fa** ed è l'autorità quando i documenti non concordano |
| [../FEATURES.md](FEATURES.md) | Matrice delle funzionalità rispetto ad altri strumenti (parti sono superate, vedi LIMITATIONS) |
| [../CLEANROOM.md](CLEANROOM.md) | Registro della provenienza e regole per l'implementazione indipendente |
| [../CONTRIBUTING.md](CONTRIBUTING.md) | Regole per i contributi, licenza in ingresso = licenza in uscita, DCO 1.1 |
| [../LICENSING.md](LICENSING.md), [../COPYRIGHT](../../../COPYRIGHT), [../LICENSES/](../../../LICENSES/) | Decisione sulla licenza (Variante A), testi |
| [../TRADEMARK.md](TRADEMARK.md) | Politica sul nome e sul logo |
| `../REUSE.toml`, `../deny.toml` | Metadati di licenza per i file non sorgente; politica sulle dipendenze |

## Dove trovare la risposta a ogni domanda

| Domanda | Pagina |
|---|---|
| Come lo installo e creo la mia prima animazione? | GETTING_STARTED |
| Cosa fa il comando X? | CLI_REFERENCE |
| Cosa significa questo campo nel JSON? | PROJECT_FORMAT, GLOSSARY |
| Quale formato di esportazione mantiene maschere / gradienti / audio? | EXPORT_FORMATS |
| Posso usarlo da Python / C / dal browser? | C_ABI / WASM (stato!) |
| È sicuro esporre lo studio a una rete? | SECURITY sezione 11 |
| Perché X non funziona? È noto? | LIMITATIONS, FAQ |
| Come viene compilato e testato? | DEVELOPMENT |
| Posso usarlo a scopo commerciale? Quali sono le licenze? | README (licenza), LICENSING, THIRD_PARTY |
| Cosa è cambiato? | CHANGELOG, ROADMAP sezione 9 |

## Convenzioni usate nei documenti

* **"Verificato"** significa che un comando o un programma è stato eseguito mentre venivano scritti i documenti, in un ambiente con Linux x86-64, senza GPU, senza webcam e con solo Chromium headless. **"Non testato"** significa esattamente questo. Le affermazioni su WebAssembly in un motore reale sono sempre non verificate.
* Gli esempi di codice in API.md e C_ABI.md sono i programmi che sono stati compilati ed eseguiti; gli esempi in WASM.md sono stati eseguiti solo contro il mock JavaScript.
* La sezione 9 della ROADMAP è l'autorità su ciò che una funzionalità fa e non fa. Se un documento non concorda con essa o con il codice, vince il codice; segnala la differenza.
* I percorsi sono relativi alla radice del repository, a meno che non inizino con `docs/`.
* I numeri (conteggi dei test, limiti) provengono dai sorgenti o da un'esecuzione del 22 ott 2026 e possono essere superati dopo la prossima modifica; fa fede il sorgente.
