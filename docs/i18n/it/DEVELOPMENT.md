<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Guida allo sviluppo

*Per chi compila, modifica, collauda e rilascia OSATURA. Documentazione CC-BY-4.0. Tutto ciò che segue è ricavato dall'albero dei sorgenti, da `ROADMAP.md` sezione 9, da [CI.md](CI.md), [CLEANROOM.md](CLEANROOM.md), [CONTRIBUTING.md](CONTRIBUTING.md) e [LICENSING.md](LICENSING.md); i comandi contrassegnati con "run" sono stati eseguiti durante la stesura di questa pagina (Linux x86-64, rustc 1.97.0, registry offline).*

Indice: [Ambiente](#1-ambiente) - [Comandi di build e test](#2-comandi-di-build-e-test) - [Le suite di test](#3-le-suite-di-test) - [Convenzioni di codifica](#4-convenzioni-di-codifica) - [Checklist per un nuovo modulo o comando](#5-checklist-per-un-nuovo-modulo-o-comando) - [Regole clean-room e DCO](#6-regole-clean-room-e-dco) - [Checklist di rilascio](#7-checklist-di-rilascio) - [Integrazione continua](#8-integrazione-continua) - [Debug](#9-debug) - [Profilazione](#10-profilazione)

## 1. Ambiente

| Requisito | Perché | Note |
|---|---|---|
| Rust (stable, workspace edition 2021) | tutto | sviluppato con rustc 1.97.0; `Cargo.lock` è nel repository, le build sono compatibili con `--locked` |
| Un compilatore C (`cc`) | l'esempio/test della C ABI (opzionale) | vedi [C_ABI.md](C_ABI.md) |
| `ffmpeg` / `ffprobe` nel `PATH` | esportazione video, decodifica audio di mp3/ogg, `dof-export` verso .mp4/.webm, `soundtrack` | **mai incluso né richiesto per la compilazione**; i comandi che ne hanno bisogno lo dicono e falliscono senza un messaggio di successo |
| Node.js >= 18 e `playwright-core` + Chromium | i cinque smoke test dello studio; il test del wrapper WASM | valori predefiniti: `/opt/npm-tools/node_modules/playwright-core`, `/opt/pw-browsers/chromium-1194/chrome-linux/chrome`; si modificano con `PLAYWRIGHT_CORE`, `CHROME` |
| `cargo-deny` | policy su licenze/ban/sorgenti | `scripts/licence-audit.sh` lo esegue se installato (`--no-deny` lo salta) |
| `python3` | `scripts/licence-audit.sh` genera i documenti con uno script inline | viene eseguito con `python3 -I` |
| Un adattatore Vulkan o GL | solo per misure su GPU reale con `osatura-gpu` | i test ripiegano su un adattatore software; vedi sezione 2 |
| `rustup target wasm32-unknown-unknown`, `wasm-opt` | solo per [WASM.md](WASM.md) | mai installati nell'ambiente di sviluppo; la CI li installa |

Struttura del workspace (per il progetto vedi [ARCHITECTURE.md](ARCHITECTURE.md)):

| Crate | Licenza | Ruolo |
|---|---|---|
| `osatura-core` | MPL-2.0 | motore, formati, strumenti, la facciata stabile `api` ([API.md](API.md)); feature: `scripting` (predefinita, Rhai) |
| `osatura-cli` | AGPL-3.0-or-later | il binario `osatura`, il server dello studio, il server di collaborazione; feature: `scripting` (predefinita), `gpu` |
| `osatura-capi` | MPL-2.0 | C ABI ([C_ABI.md](C_ABI.md)) |
| `osatura-wasm` | MPL-2.0 | ABI WebAssembly + wrapper JS ([WASM.md](WASM.md)) |
| `osatura-gpu` | MPL-2.0 | renderer wgpu opzionale; **non** è un membro predefinito |
| `osatura-gui` | AGPL-3.0-or-later | `osatura-studio`, lo studio in una finestra nativa (`tao` + `wry`, solo Windows e macOS; uno stub altrove); **non** è un membro predefinito, `publish = false` |

File a livello di repository: `ROADMAP.md` (la sezione 9 è il registro datato di funzionalità e revisioni), `FEATURES.md`, `CLEANROOM.md`, `LICENSING.md`, `REUSE.toml`, `deny.toml`, `LICENSES/`, `scripts/`.

## 2. Comandi di build e test

```sh
cargo build                                  # default members: core, cli, capi, wasm (gpu is opt-in)
cargo test --workspace                       # run: 1343 tests + 19 doc-tests without gpu and gui (section 3)
cargo test -p osatura-core                  # one crate (the engine and the facade tests)
cargo test -p osatura-core api::            # a module or a name filter
cargo test -p osatura-cli --test e2e        # one integration test file
cargo test --doc -p osatura-core            # doc-tests only (the facade examples)
cargo test -p osatura-cli --test spdx_audit # SPDX header audit (0.01 s)
cargo clippy --workspace --all-targets -- -D warnings   # zero warnings (gated in CI since wave 8)
scripts/licence-audit.sh                     # regenerate THIRD_PARTY*, SPDX check, cargo deny
scripts/e2e.sh                               # the end-to-end pipeline in release mode (--debug for debug)
```

Feature e parti opzionali:

```sh
cargo build -p osatura-cli --no-default-features         # no Rhai (no `script` command, no smartstring)
cargo build -p osatura-core --no-default-features        # what capi, wasm and gpu use
cargo build -p osatura-cli --features gpu                # adds `render-gpu` (wgpu)
cargo test  -p osatura-gpu                               # GPU tests (31: 18 unit + 13 parity); NOT in the workspace default
OSATURA_GPU_BACKEND=vulkan osatura render-gpu ...     # restrict the wgpu backend: vulkan or gl
```

Regole pratiche: usa build di debug per le suite di test (il repository è stato sviluppato badando allo spazio su disco: non copiare `target/`); usa `--release` solo per questioni di tempi. Lo snapshot della facciata si aggiorna con `OSATURA_BLESS=1 cargo test -p osatura-core api::` (sezione 5). `OSATURA_UI_CONFIG` seleziona nei test il file delle impostazioni dell'interfaccia dello studio, `OSATURA_AUTHOR` il nome dell'autore nei commit del controllo di versione, `OSATURA_E2E_TIMINGS=<file>` fa scrivere al test e2e i tempi per fase.

## 3. Le suite di test

Conteggi tratti da `ROADMAP.md` sezione 9 e da un'esecuzione completa di `cargo test --workspace --offline` del 22 ott 2026 per questa pagina (profilo debug, macchina condivisa con altri job: i tempi sono quindi indicativi e **saranno più lenti che su una macchina inattiva**):

| Suite | Test | Tempo (questa esecuzione) | Cosa copre |
|---|---:|---:|---|
| `osatura-core` unit (`src/lib.rs`) | 1167 | 349 s | ogni modulo; include `api::tests::api_surface_snapshot` |
| `osatura-core/tests/prelude_surface.rs` | 6 | 0.7 s | la facciata dall'esterno del crate (`#[non_exhaustive]`, mappatura degli errori, un sottoinsieme rappresentativo di chiamate) |
| `osatura-cli` unit (`src/usage_tests.rs`, `src/args.rs`, studio, collab, http) | 80 | 42 s | parsing delle opzioni, USAGE rispetto al parser, server HTTP, policy, log di collaborazione |
| `tests/e2e.rs` | 2 | 214 s | l'intera pipeline attraverso il binario reale (circa 19 s in una build debug su macchina inattiva e 40 s in release secondo la ROADMAP; qui la macchina era sotto carico) |
| `tests/fixes.rs`, `tests/wave4.rs`, `tests/robust.rs` | 7, 15, 10 | 19 s, 1.2 s, 1.2 s | regressioni delle ondate di revisione e test con input ostili |
| `tests/doc_mismatch.rs` | 20 | 1.8 s | regressioni documentazione/comportamento: output atomici, opzioni rigorose, la matrice di accettazione di `--json`/`--quiet`/`--allow-empty`, regole di scala di `golden` |
| `tests/source_offer.rs` | 3 | 0.2 s | l'offerta `/source` della sezione 13 dell'AGPL |
| `tests/face_cli.rs` | 1 | 0.8 s | CLI del face tracking |
| `osatura-core/tests/wasm_safety.rs` | 2 | 0.1 s | nessun uso diretto di `Instant` / `SystemTime` / `process::id` / `thread::spawn` fuori da `clock.rs` (scansione dei sorgenti) |
| `osatura-cli/tests/wave7.rs` | 7 | 7 s | ondata 7: opzioni per sotto-comando, `follow` / `make-loop`, `color-space`, `collab-token`, file segreto dello studio, intervallo di fotogrammi di `dof` / `render-hdr` / `fx` |
| `tests/spdx_audit.rs` | 2 | 0.01 s | ogni `.rs/.js/.mjs/.sh` ha una riga SPDX uguale alla licenza del proprio crate; REUSE.toml copre il resto |
| `osatura-capi/tests/abi.rs` | 7 | 1.1 s | la C ABI attraverso il lato Rust (smoke test in C: `osatura-capi/tests/smoke.c`, compilato a mano, vedi [C_ABI.md](C_ABI.md)) |
| `osatura-wasm` unit | 8 | 1.6 s | le funzioni dell'ABI sull'host |
| `osatura-gpu` unit + `tests/parity.rs` | 18 + 13 | 0.2 s + 6.5 s | parità GPU/CPU (adattatore software in assenza di GPU) |
| Doc-test (`osatura-core`) | 19 | 171 s (ognuno compila un programma) | gli esempi della facciata (`api/`, `lib.rs`); `osatura-capi` e `osatura-wasm` non ne hanno |

Totale: 1343 test nei binari di unit e integration test e 19 doc-test, contati il 24 ott 2026 con `cargo test --workspace --exclude osatura-gpu --exclude osatura-gui` (i 1368 della frase successiva includevano anche i test del crate GPU; riconteggiato da un'esecuzione completa di `cargo test --workspace` del 22 ott 2026 dopo l'ondata 7, i tempi per suite qui sopra provengono da un'esecuzione precedente), più 19 doc-test; l'intero `cargo test --workspace --offline` ha richiesto 836 s sulla macchina condivisa.

Suite esterne a `cargo test`:

* **Smoke test dello studio (Chromium).** Cinque script Node pilotano lo studio reale in Chromium headless con `playwright-core`: `studio_ui_smoke.js` (128 controlli), `studio_collab_smoke.js` (60, due pagine sulla stessa macchina), `studio_overlays_smoke.js` (89), `studio_suggest_smoke.js` (35) e `studio_demo_smoke.js` (25: la demo guidata carica tutti i 13 passi, non scrive mai, ripristina il progetto byte per byte e recupera una scheda chiusa); tutti e cinque sono passati il 24 ott 2026 (conteggi come stampati dagli script). **Non** vengono eseguiti da `cargo test` e di default non fanno parte della CI. Esecuzione:

  ```sh
  cargo build -p osatura-cli
  node osatura-cli/tests/studio_ui_smoke.js        # same for collab / overlays / suggest / demo
  ```

  Ambiente: `OSATURA_BIN` (predefinito `target/debug/osatura`), `PLAYWRIGHT_CORE`, `CHROME` e, per lo script degli overlay, `SHOTS` (cartella degli screenshot). Gli script ripuliscono i propri file temporanei.
* **Test del wrapper WASM.** `node osatura-wasm/js/test.mjs` (solo mock, a meno che si passi il percorso di un progetto e si sia compilato il modulo; vedi [WASM.md](WASM.md)).
* **Audit delle licenze.** `scripts/licence-audit.sh`: ricostruisce `docs/THIRD_PARTY.md` e `docs/THIRD_PARTY_NOTICES.md` da `cargo metadata --locked --all-features` (dipendenze normali e di build, escluse quelle di sviluppo), verifica la riga SPDX di ogni file sorgente rispetto alla `license` del suo crate, poi esegue `cargo deny check licenses bans sources` a meno di `--no-deny`. `deny.toml` consente le licenze permissive e MPL-2.0 solo per crate indicati per nome.
* **clippy.** `cargo clippy --workspace --all-targets -- -D warnings` termina con codice 0 senza avvisi (ondata 8) ed è un requisito bloccante in CI. Dove un lint è errato per il caso specifico, il codice contiene un `#[allow(clippy::...)]` mirato con un commento (cicli con indice su array paralleli, `too_many_arguments` su firme stabili, `manual_clamp` dove la gestione dei NaN differisce, nomi inerenti `next` / `mul` che fanno parte dell'API pubblica); non aggiungere allow a livello di crate.
* **Configurazione della CI.** Il repository fornisce `osatura ci` per i controlli sulle *animazioni* ([CI.md](CI.md)); è cosa diversa dalla CI del codice. Il progetto non ha alcuna configurazione di CI del codice nell'albero; la sezione 8 fornisce una ricetta minima.

Aggiungere test: i test del motore vanno accanto al codice (`#[cfg(test)]`), i test di comportamento della CLI in `osatura-cli/tests/`, e ogni bug corretto ottiene un test di regressione (le ondate di revisione in [CHANGELOG.md](CHANGELOG.md) elencano cosa è stato corretto e dove). I test non devono lasciare file temporanei (usa drop guard) e non devono dipendere dall'orologio né dalla rete. I budget di tempo si verificano con un orologio di test deterministico.

## 4. Convenzioni di codifica

* **Nessun panic sull'input.** Tutto ciò che legge un progetto, un file, un argomento o un byte di rete restituisce un errore. Indicizzazione, slicing, `unwrap` e overflow aritmetico su valori non fidati sono bug. La facciata (`api/`) inoltre intercetta i panic e restituisce `ApiError::Internal` (solo dove il target esegue l'unwinding), ma è l'ultima linea di difesa, non un criterio di progetto.
* **Politica dei valori finiti.** I numeri provenienti da file e argomenti vengono controllati per NaN e infinito e per grandezze ragionevoli (gli intervalli della facciata sono in [API.md](API.md): grandezze fino a 1e7, fps in (0, 1000], fotogrammi 1..=200000, stage fino a 32768, 4096 ossa, lato di render fino a 16384 px). Rifiuta, non limitare (clamp), a meno che la funzione sia documentata come clamping (i fotogrammi vengono limitati alla timeline).
* **Controlla le dimensioni prima di allocare.** Buffer di pixel, dimensioni delle mesh e numero di campioni sono calcolati con aritmetica controllata e rifiutati prima di `vec![..]`.
* **Scritture atomiche.** Scrivi i risultati con `fsutil::write_atomic` (file temporaneo nella stessa directory, `fsync`, rinomina; rifiuta i collegamenti simbolici salvo `--follow-symlinks`; ripiega su una sovrascrittura sul posto con `fsync`, documentata come non atomica, quando la directory non è scrivibile). Non usare mai direttamente `std::fs::write` su un file dell'utente. Le directory temporanee provengono da `fsutil` (modo 0700, file 0600). I byte casuali provengono da `fsutil::random_bytes` (`/dev/urandom`).
* **Gli asset passano per `assets::resolve`.** I percorsi relativi di immagini, normal map e suoni si risolvono rispetto alla directory degli asset valida per tutto il processo (`assets::default_dir`, impostata dalla cartella del progetto in ingresso); i percorsi assoluti e `..` sono rifiutati salvo `--allow-external-assets`; i file immagine sono limitati a 64 MB e l'audio a 256 MB. I comandi che scrivono un progetto derivato validano rispetto alla directory del progetto *in ingresso* e riscrivono i percorsi relativi degli asset o avvisano al riguardo (`write_json`).
* **Casualità deterministica.** Qualsiasi comportamento casuale (fuzzing, rumore, jitter, simulazione) usa il generatore con seed del motore, con un'opzione `--seed`, ed è riproducibile bit per bit. Niente crate `rand`, niente orologio di sistema come seed.
* **Println e pipe chiuse.** La CLI sostituisce `println!`/`print!` con macro che escono silenziosamente su una pipe chiusa (`osatura ... | head` non deve andare in panic). Continua a usarle nei moduli dichiarati dopo le macro in `main.rs`.
* **Gli errori dicono cosa fare.** I messaggi dicono cosa non andava, quale valore, e l'intervallo ammesso o il rimedio (i test della facciata verificano questo stile).
<!-- REUSE-IgnoreStart -->
* **Intestazioni SPDX.** Ogni file `.rs`, `.js`, `.mjs` e `.sh` inizia con `SPDX-License-Identifier:` uguale alla licenza del suo crate (MPL-2.0 per core/capi/wasm/gpu/scripts, AGPL-3.0-or-later per la CLI) e una riga di copyright; i documenti Markdown portano il commento di intestazione CC-BY-4.0, come questo file. Gli altri file (JSON, immagini, i testi delle licenze) sono coperti da `REUSE.toml`; quando aggiungi un nuovo pattern di percorso, aggiungilo lì. In caso contrario `cargo test -p osatura-cli --test spdx_audit` fallisce.
<!-- REUSE-IgnoreEnd -->
* **Dipendenze.** Poche, in puro Rust, con licenza permissiva. Un nuovo crate richiede un controllo `deny.toml` (`scripts/licence-audit.sh`), una riga in `docs/THIRD_PARTY.md` (rigenerato) e una motivazione nella voce della ROADMAP. `osatura-core` è compilato senza feature predefinite da capi, wasm, gpu e dalla CLI, quindi tutto ciò che è pesante va dietro una feature.
* **Prima la facciata.** Ogni nuova capacità del motore visibile all'utente che altri linguaggi devono poter raggiungere va in `osatura-core/src/api/` con una riga di snapshot, un doc-test e un test unitario; i livelli C e wasm incapsulano solo la facciata.

## 5. Checklist per un nuovo modulo o comando

1. **Modulo** in `osatura-core/src/<name>.rs`, registrato in `lib.rs`; intestazione SPDX; test unitari (validi, al limite, input ostili, determinismo); nessun panic.
2. **Comando CLI**: una voce `USAGE` (grafia esatta di ogni opzione) in `osatura-cli/src/usage.rs`, il ramo di dispatch in uno dei file da `osatura-cli/src/cmds_1.rs` a `cmds_6.rs` (o in `main.rs` per `ui`), e le tabelle delle opzioni in `osatura-cli/src/args.rs`: ogni opzione che accetta un valore deve essere elencata in `VALUE_OPTS`; i flag che non accettano valore nella voce `CMD_OPTS` del comando dove la tabella globale sarebbe errata (per esempio `ci --prune` è un flag, `export-oca --layers` e `autotrace --png` accettano valori, `keyedit --free` è un flag). I test unitari USAGE-contro-parser falliscono se il testo e il parser non concordano. Supporta i flag globali (`--json`, `--quiet`, `--allow-empty`, `--allow-external-assets`, `--follow-symlinks`, `--max-input-bytes`), `<command> --help` e `--key=value`.
3. **Scrittura dell'output** solo con `fsutil::write_atomic` o `write_json`; i comandi che non producono nulla terminano con codice diverso da zero salvo `--allow-empty`.
4. **Documentazione**: il comando in [CLI_REFERENCE.md](CLI_REFERENCE.md) se ne aggiungi uno, una menzione in una guida dove pertinente, e la voce in `ROADMAP.md` sezione 9 (cosa è stato aggiunto, cosa *non* è stato fatto: quell'elenco alimenta [LIMITATIONS.md](LIMITATIONS.md)).
5. **Riga di CLEANROOM.md**: per ogni nuovo algoritmo registra la fonte pubblica da cui è stato implementato (manuale, standard, articolo) e la derivazione indipendente; niente riga, niente merge.
6. **Aggiornamento (re-bless) della superficie della facciata** se `api/` è cambiato: esegui `OSATURA_BLESS=1 cargo test -p osatura-core api::`, esamina il diff di `osatura-core/src/api_surface.txt` (una riga rimossa o modificata è una breaking change e richiede una versione major della facciata; una riga aggiunta è una minor), poi esegui di nuovo i test senza la variabile. Aggiorna [API.md](API.md) e i wrapper C/WASM e i relativi header se devono esporla.
7. **End-to-end**: aggiungi il comando alla pipeline e2e se appartiene al percorso di produzione; aggiungi un controllo smoke agli script dello studio se ha un'interfaccia.
8. **Esegui**: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/licence-audit.sh` (quando dipendenze o intestazioni sono cambiate).

## 6. Regole clean-room e DCO

Il progetto è un'implementazione indipendente. In breve (il testo vincolante è [CLEANROOM.md](CLEANROOM.md) e [CONTRIBUTING.md](CONTRIBUTING.md)):

* Implementa a partire da standard pubblici, manuali e articoli, e citali. Non leggere, decompilare, copiare né fare reverse engineering di codice, file o interfacce di strumenti di animazione proprietari, e non leggere né incollare codice sorgente di strumenti concorrenti mentre scrivi una funzionalità. Non incollare codice da progetti con una licenza incompatibile.
* Registra ogni nuovo metodo nella tabella di `CLEANROOM.md` con la sua fonte pubblica.
* Le note sui brevetti si trovano in [PATENT_NOTES.md](PATENT_NOTES.md); una funzionalità che tocca un'area sensibile dal punto di vista dei brevetti va prima discussa lì. Questo non è un parere legale.
* I contributi sono accettati con la licenza della parte che modificano (inbound uguale a outbound): MPL-2.0 per le librerie e gli script, AGPL-3.0-or-later per `osatura-cli`, CC-BY-4.0 per la documentazione e gli asset sintetici.
* **Sign-off.** Ogni commit richiede `Signed-off-by: Name <email>` (`git commit -s`), che certifica il Developer Certificate of Origin 1.1 (testo in CONTRIBUTING.md). Il sign-off certifica solo l'origine: **non** concede alcun diritto di cambiare licenza, e finché non esiste un CLA non è possibile alcuna licenza commerciale o doppia per il codice contribuito.
* Il nome e il logo sono coperti da [TRADEMARK.md](TRADEMARK.md).

## 7. Checklist di rilascio

Le prime build sono preparate come pre-release in bozza `v0.1.0-alpha.3` (tutti i crate sono ancora 0.1.0; le etichette di versione "v0.5/v0.6/v0.7" in README, FEATURES e ROADMAP sono livelli di funzionalità, non versioni dei crate; vedi [CHANGELOG.md](CHANGELOG.md)). Per prepararne una:

1. **Versione.** Decidi la versione di ciascun crate. La facciata ha una propria versione (`API_VERSION`, oggi 1.4.0: major in caso di rimozione o modifica di una riga della superficie, minor in caso di aggiunta) e `OSATURA_ABI_VERSION` / `arm_abi_version()` (entrambe 1) cambiano solo con una breaking change della ABI C o wasm. Allinea le versioni in `Cargo.toml`, l'output di `osatura --version`, il riquadro di stato del README e il CHANGELOG.
2. **Controlli verdi.** `cargo test --workspace`, `cargo test --doc -p osatura-core`, `cargo test -p osatura-cli --test spdx_audit`, `cargo clippy --workspace --all-targets -- -D warnings`, i cinque smoke test Chromium, `scripts/e2e.sh` (release), `cargo test -p osatura-gpu` se il codice GPU è cambiato.
3. **Avvisi di terze parti.** Esegui `scripts/licence-audit.sh`: rigenera `docs/THIRD_PARTY.md` (inventario) e `docs/THIRD_PARTY_NOTICES.md` (testi delle licenze di tutto ciò che viene distribuito), verifica le righe SPDX ed esegue `cargo deny`. Fai il commit dei file rigenerati. Una distribuzione binaria deve includere il pacchetto degli avvisi accanto al binario.
4. **File di licenza.** `LICENSE`, `LICENSES/` (testi di ogni licenza usata) e `REUSE.toml` sono allineati con [LICENSING.md](LICENSING.md); l'attribuzione CC-BY-4.0 per la documentazione è integra.
5. **Niente ffmpeg.** Non includere, collegare né scaricare ffmpeg negli artefatti di rilascio. Viene rilevato nel `PATH` a tempo di esecuzione e la documentazione dice che lo installa l'utente.
6. **Offerta dei sorgenti AGPL.** Il binario di `osatura-cli` serve la propria offerta dei sorgenti su `/source` (sezione 13 dell'AGPL) e stampa un avviso con `--version`; `release.yml` imposta `OSATURA_SOURCE_URL` in fase di compilazione su `<server>/<repository>/tree/<commit>` in modo che i binari di release indichino l'esatto sorgente da cui sono stati compilati (il test di fumo del workflow controlla che `--version` lo riporti; un semplice `cargo build` non ha un indirizzo incorporato); chi gestisce server dello studio modificati imposta `--source-url` sul luogo in cui si trovano i propri sorgenti modificati. Test: `tests/source_offer.rs`. Per un rilascio binario, fornisci i sorgenti corrispondenti (tag, tarball) a un URL raggiungibile dagli utenti.
7. **Documentazione.** Aggiorna il riquadro di stato nel README, [LIMITATIONS.md](LIMITATIONS.md) e il CHANGELOG (sposta *Unreleased* sotto la nuova versione con una data).
8. **Compila la bozza su GitHub.** `.github/workflows/release.yml` compila gli archivi Windows, macOS e Linux e il modulo web a ogni push su `main`; un commit su `main` il cui messaggio contiene `[release vX.Y.Z-label]` (o un tag `v*` inviato) crea anche una *pre-release in bozza* con gli archivi e `SHA256SUMS.txt` (una bozza precedente con lo stesso tag viene sostituita). Controlla la bozza, poi pubblicala a mano.
9. **Tag e pubblicazione** con un tag firmato; allega i checksum. Non pubblicare crate con nomi che non sono tuoi; le regole di TRADEMARK.md valgono per i fork.

## 8. Integrazione continua

Due cose diverse condividono la parola (un terzo workflow, `.github/workflows/release.yml`, compila gli archivi di release e la release in bozza, vedi la sezione 7):

* **CI delle animazioni** (`osatura ci`): controlli sui *progetti* (lint, golden di render, prestazioni, determinismo). Descritta per intero in [CI.md](CI.md): manifest `osatura-ci.json`, soglie, golden (`--update` contro `bless`), report, esempi per GitHub Actions e per shell semplice (contrassegnati lì come "example, not run").
* **CI del codice** per questo repository: `.github/workflows/ci.yml` ha quattro tipi di job: `test` (matrice ubuntu / windows / macos: `cargo test --workspace --exclude osatura-gpu --locked` e clippy con `-D warnings`), `wasm32` (compila `osatura-wasm` per `wasm32-unknown-unknown`, copia il modulo accanto a `osatura.js` ed esegue `node osatura-wasm/js/test.mjs` con un progetto demo, compresa la parte con il modulo reale), `gpu parity` (informativo; viene saltato se non esiste alcun adattatore, gira sull'adattatore software DX12 di Windows) e `licences` (`cargo-deny check` e `reuse lint`). La toolchain è fissata (1.97.0) in modo che i nuovi lint di clippy non rompano la build. Il job `test` compila anche `osatura-gui` ed esegue `osatura-studio --selftest` (un vero auto-test su Windows e macOS). **Prima esecuzione completamente verde: 2026-10-24** su runner ospitati da GitHub. In caso di errore i passaggi stampano i test falliti come annotazioni del workflow (leggibili senza accesso ai log). Windows salta i test solo POSIX (symlink, permessi, `/dev/zero`, separatori di percorso POSIX).

  ```sh
  cargo test --workspace --locked
  cargo test --doc -p osatura-core --locked
  cargo clippy --workspace --all-targets --locked -- -D warnings
  scripts/licence-audit.sh            # needs python3 and (optional) cargo-deny
  ```

  Metti in cache `~/.cargo/registry` e `target/` tra le esecuzioni, concedi al test e2e diversi minuti nelle build debug ed esegui gli smoke test Chromium solo sui runner che hanno Chromium e `playwright-core`.

## 9. Debug

* **Backtrace.** `RUST_BACKTRACE=1 osatura ...` per qualsiasi panic (un panic sull'input è un bug; segnalalo insieme al progetto che lo provoca).
* **Output leggibile da macchina.** `--json` sui comandi che lo supportano; `--quiet` per gli script. Codici di uscita: 0 in caso di successo, 1 in caso di errore (controllo fallito, budget non rispettato, uso errato; run: un comando sconosciuto e un argomento mancante terminano entrambi con 1); `osatura ci` distingue 1 (un controllo è fallito) da 2 (errore).
* **Ispezionare un progetto.** `osatura check`, `osatura lint` (codice 1 in caso di errori) e `osatura diff a.json b.json` forniscono struttura e problemi senza fare il render; `osatura render project.json N out.png` rende un fotogramma e `out.svg` produce un output vettoriale apribile in un browser (`--bones` mostra lo scheletro).
* **Studio.** Esegui `osatura studio project.json --port 0` (stampa la porta effettiva). Gli strumenti per sviluppatori del browser mostrano l'API JSON: le richieste sono semplici `POST /api/...` con `Content-Type: application/json`. Un 421 indica un header Host non presente in `--allow-host`; un 403 indica che un controllo di Origin o `Sec-Fetch-Site` è fallito (usa `--public-origin` dietro un proxy); 415 riguarda il tipo di contenuto; 429 è il back-off del token.
* **Fasi della demo.** La demo guidata dello Studio (e la demo passo per passo sul sito del progetto) sono generate da `osatura demo-person` tramite `scripts/build-studio-tour.mjs` e `scripts/lib/person-stages.mjs`; la compilazione fallisce se la demo finita non corrisponde più alle cifre citate nei testi (`EXPECT` nella libreria). Dopo aver modificato `osatura-core/src/demo.rs`, riesegui lo script e ricompila la CLI.
* **Log di collaborazione.** Il log di una sessione condivisa (`--collab-log`) è testo, una riga `<hash> <json>` per operazione; una catena interrotta viene segnalata al caricamento.
* **Un test fallisce solo a volte.** Cerca l'orologio, i nomi delle directory temporanee e l'ordine di iterazione di `HashMap`; il codice del motore è deterministico per costruzione, quindi i fallimenti intermittenti sono quasi sempre un problema di igiene dei test.
* **Lo snapshot della facciata fallisce.** Hai cambiato una firma pubblica: vedi sezione 5, punto 6.
* **GPU.** `osatura render-gpu <project> <out.png> --adapter software|auto --bench N` (solo nelle build con `--features gpu`) confronta con il renderer CPU (un adattatore software stampa che i suoi tempi non hanno significato); `OSATURA_GPU_BACKEND` seleziona il backend.
* **Sanitizer per la ABI C.** Compila `smoke.c` o un tuo chiamante con `-fsanitize=address,undefined` contro la libreria statica (fatto durante il lavoro sulla documentazione, pulito; vedi [C_ABI.md](C_ABI.md)).

## 10. Profilazione

* **Tempi per fase della pipeline di produzione:** `scripts/e2e.sh` stampa una tabella (`OSATURA_E2E_TIMINGS` indica il file); i numeri release in `ROADMAP.md` sezione 9 provengono da lì.
* **GPU contro CPU:** `osatura render-gpu <project> <out.png> --bench N`.
* **Budget di dimensione e velocità:** `osatura size-report`, `osatura export-fit --max-seconds` (il budget di tempo è controllato dentro i cicli sui candidati; una singola chiamata a ffmpeg o al quantizzatore non è interrompibile).
* **Costo di render di un progetto:** `osatura ci` con una soglia di prestazioni ([CI.md](CI.md)) registra il tempo per fotogramma sulla tua macchina; i tempi dipendono dalla macchina, quindi confronta sulla stessa macchina.
* **Strumenti generali:** compila con `--release` e usa `perf record`/`perf report` o `cargo flamegraph` (non fanno parte del progetto; installali separatamente). Le build debug sono circa un ordine di grandezza più lente e non dicono nulla sulla velocità reale.
