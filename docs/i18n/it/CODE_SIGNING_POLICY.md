<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Politica di firma del codice

Firma del codice gratuita fornita da [SignPath.io](https://about.signpath.io/), certificato della [SignPath Foundation](https://signpath.org/).

*Stato (25 ott 2026): il progetto ha richiesto, o sta per richiedere, un certificato gratuito della SignPath Foundation. Finché una release non dirà altro, i file per Windows **non** sono firmati con Authenticode; ogni file di release porta già una firma Sigstore e un’attestazione di provenienza della build di GitHub (vedi [Verifica](#verifica)).*

## Cosa viene firmato

Vengono firmati solo i file costruiti da questo repository ([github.com/chiuta/osatura](https://github.com/chiuta/osatura)) dal suo workflow di rilascio ([`.github/workflows/release.yml`](../../../.github/workflows/release.yml)) su runner ospitati da GitHub. Nulla di costruito su un computer personale viene mai firmato e nessun binario di terze parti viene firmato.

| File (in `osatura-windows-x86_64.zip`) | Cos’è | Licenza |
|---|---|---|
| `osatura.exe` | strumento da riga di comando e studio nel browser | AGPL-3.0-or-later |
| `osatura-studio.exe` | lo studio in una finestra nativa (WebView2) | AGPL-3.0-or-later |
| `lib/osatura_capi.dll` | libreria C | MPL-2.0 |

Ogni file firmato ha il nome di prodotto `OSATURA` e la versione della release nelle proprietà del file (incorporati da `build.rs` in ogni crate e verificati dal workflow prima di qualsiasi firma). La configurazione degli artefatti SignPath è conservata in [`.signpath/artifact-configuration.xml`](../../../.signpath/artifact-configuration.xml).

## Ruoli del team

Il progetto ha attualmente un solo manutentore, che ricopre tutti e tre i ruoli:

| Ruolo | Chi |
|---|---|
| Committer e revisori | [Alexandru-Ionuț Chiuță (Alexio), @chiuta](https://github.com/chiuta) |
| Approvatori | [Alexandru-Ionuț Chiuță (Alexio), @chiuta](https://github.com/chiuta) |

* Le modifiche di chiunque non abbia accesso in scrittura arrivano come pull request e vengono riviste da un committer prima di essere unite ([CONTRIBUTING.md](CONTRIBUTING.md)).
* Una richiesta di firma viene creata solo dal workflow di rilascio per un tag di versione (`vMAJOR.MINOR.PATCH[-pre]`) il cui commit si trova su `main`, e ogni richiesta viene approvata a mano in SignPath da un approvatore.
* Chiunque abbia uno di questi ruoli è tenuto a usare l’autenticazione a più fattori per GitHub e SignPath (condizione della SignPath Foundation; il manutentore la conferma per ogni account quando il certificato viene concesso).
* Ogni modifica a questi ruoli viene registrata in questo file.

## Informativa sulla privacy

Questo programma non trasferirà alcuna informazione ad altri sistemi in rete, a meno che non sia richiesto esplicitamente dall’utente o dalla persona che lo installa o lo gestisce. Dettagli: [PRIVACY.md](PRIVACY.md).

## Modifiche al tuo sistema e disinstallazione

OSATURA non ha un installer, non scrive nulla nel registro e non installa servizi, driver o voci di avvio. I pochi file che può creare e come rimuovere tutto sono elencati in [UNINSTALL.md](../../UNINSTALL.md).

## Licenze

Tutto il codice è open source con licenze approvate dall’OSI: MPL-2.0 (librerie) e AGPL-3.0-or-later (strumento da riga di comando, studio e app a finestra); la documentazione è CC-BY-4.0 ([COPYRIGHT](../../../COPYRIGHT), [LICENSING.md](LICENSING.md)). I file firmati non contengono codice proprietario e OSATURA non è offerto con alcuna licenza commerciale o proprietaria (nessuna doppia licenza).

## Verifica

* **Authenticode** (quando disponibile): clic destro su un file in Esplora risorse di Windows > Proprietà > Firme digitali; il firmatario è SignPath Foundation.
* **Origine di ogni file di release**: `gh attestation verify <file> -R chiuta/osatura`, oppure `cosign verify-blob` con il bundle `.sigstore.json` ([VERIFY_DOWNLOADS.md](../../VERIFY_DOWNLOADS.md)).

## Segnalare un problema

Se pensi che un file firmato per OSATURA sia dannoso, non sia stato costruito da questo repository o che questa politica sia stata violata, scrivi a <alexio@trom.tf> o apri una issue; le segnalazioni di sicurezza seguono [SECURITY.md](SECURITY.md). I problemi con il certificato stesso possono essere segnalati anche alla [SignPath Foundation](https://signpath.org/).
