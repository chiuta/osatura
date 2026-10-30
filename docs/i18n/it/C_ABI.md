# ABI C di OSATURA

*ABI versione 1 (`OSATURA_ABI_VERSION`, `osatura_abi_version()`). Crate `osatura-capi`, header `osatura-capi/osatura.h`. Destinatari: integratori che vogliono caricare e riprodurre progetti OSATURA da C, C++, Python, Go, C#, Swift o da qualsiasi linguaggio con un'interfaccia C per funzioni esterne (FFI). Documentazione CC-BY-4.0; esempi di codice MPL-2.0.*

L'ABI C è un piccolo wrapper in puro C sopra la facciata Rust stabile ([API.md](API.md)). Con essa un programma ospite può aprire un progetto, leggerne i dati e le trasformazioni degli ossi, renderizzare fotogrammi in RGBA ed eseguire **macchine a stati** (riproduzione interattiva, scorrimento, livello di dettaglio). A partire dalla facciata 1.4 può anche eseguire **modifiche** di base (creare livelli e ossi, impostare chiavi, fare il baking della fisica, annullare) ed **esportare in memoria** (GIF, glTF, Lottie, HTML, APNG, PNG, EXR, JSON del progetto). Non può gestire il controllo di versione né eseguire gli strumenti: per questi usa l'API Rust o la riga di comando ([CLI_REFERENCE.md](CLI_REFERENCE.md)).

Indice: [Compilazione e linking](#compilazione-e-linking) - [Convenzioni](#convenzioni) - [Riferimento delle funzioni](#riferimento-delle-funzioni) - [Proprietà e durata](#proprietà-e-durata) - [Gestione degli errori](#gestione-degli-errori) - [Threading](#threading) - [Esempio completo in C](#esempio-completo-in-c) - [Scorrimento e livello di dettaglio in C](#scorrimento-e-livello-di-dettaglio-in-c) - [Esempio in Python (ctypes)](#esempio-in-python-ctypes) - [Altri linguaggi](#altri-linguaggi) - [Versionamento](#versionamento) - [Particolarità note](#particolarità-note)

## Compilazione e linking

Il crate produce tre tipi di libreria (`crate-type = ["cdylib", "staticlib", "rlib"]`); dipende da `osatura-core` con `default-features = false`, quindi non contiene alcun motore di scripting né crate MPL di terze parti.

```sh
cargo build -p osatura-capi             # debug:   target/debug/libosatura_capi.{so,a}
cargo build --release -p osatura-capi   # release: target/release/libosatura_capi.{so,a}
```

| Piattaforma | Libreria dinamica | Libreria statica |
|---|---|---|
| Linux | `libosatura_capi.so` | `libosatura_capi.a` |
| macOS | `libosatura_capi.dylib` | `libosatura_capi.a` |
| Windows | `osatura_capi.dll` (+ libreria di importazione) | `osatura_capi.lib` |

**Testato:** Linux x86-64, gcc, sia la libreria condivisa sia quella statica (vedi sotto). Per macOS e Windows si tratta dei nomi di output standard di Rust per quei target, ma l'autore **non li ha compilati né eseguiti**. Un `.so` di debug pesa circa 150 MB (informazioni di debug); per qualsiasi cosa distribuisci compila con `--release` (la dimensione della libreria release qui non è stata misurata).

Compilare e collegare con la libreria condivisa:

```sh
cc -std=c99 -Wall -Wextra -Iosatura-capi -o example example.c -Ltarget/debug -losatura_capi
LD_LIBRARY_PATH=target/debug ./example project.json frame.ppm      # or install the .so and use rpath / ldconfig
```

Linking statico (verificato su Linux; la libreria standard di Rust richiede queste librerie di sistema):

```sh
cc -std=c99 -Iosatura-capi -o example example.c target/debug/libosatura_capi.a -lpthread -ldl -lm
```

`osatura.h` ha le guardie `extern "C"` per C++, richiede solo `<stddef.h>` e `<stdint.h>` ed è l'unico file da includere. Copialo accanto ai tuoi sorgenti. Non esistono un file pkg-config, un target di installazione né un SONAME versionato; distribuisci la libreria insieme alla tua applicazione.

I panic vengono intercettati a ogni punto di ingresso e trasformati in codici di errore, il che presuppone che la libreria sia compilata con il `panic = "unwind"` predefinito. Non compilarla con `panic = "abort"` (altrimenti un bug nel motore terminerebbe il tuo processo invece di restituire -1). Il workspace non imposta quell'opzione di profilo.

## Convenzioni

* Le stringhe sono UTF-8. Le stringhe in ingresso (`const char *`) terminano con NUL, tranne il JSON del progetto, che viene passato come puntatore più lunghezza e non richiede terminatore.
* Le funzioni che restituiscono `int` danno **0 in caso di successo e un valore negativo in caso di errore**. `-1` è un errore generico; `-2` significa inoltre "il tuo buffer di output è troppo piccolo" e si verifica solo nelle due funzioni di rendering.
* Le funzioni che riempiono un buffer di caratteri prendono `(char *out, size_t cap)` e restituiscono la **lunghezza completa** del testo (escluso il NUL). Il testo viene troncato a `cap - 1` byte e terminato con NUL quando `cap > 0`. Passa `out == NULL` (con qualsiasi `cap`) per chiedere solo la lunghezza. Un valore di ritorno negativo è un errore (indice errato, nome sconosciuto).
* Gli argomenti float devono essere finiti; i fotogrammi fuori dalla timeline vengono limitati all'intervallo; `scale` deve essere compresa tra 0.05 e 4.
* Ogni argomento handle può essere `NULL` per i getter (restituiscono 0 e impostano il testo dell'errore) ed è un errore per tutte le altre funzioni.
* L'RGBA in output ha **alfa non premoltiplicato**, 8 bit per canale, per righe, riga superiore per prima, `width * height * 4` byte.
* Le trasformazioni mondo degli ossi sono sei float per osso `[a b c d tx ty]`; un punto locale all'osso `(x, y)` viene mappato in `(a*x + c*y + tx, b*x + d*y + ty)`.

## Riferimento delle funzioni

53 funzioni. `E` = `OsaturaEngine *`, `P` = `OsaturaPlayer *`.

### Libreria

| Funzione | Descrizione |
|---|---|
| `const char *osatura_version(void)` | stringa di versione del crate `osatura-capi` (attualmente `"0.1.0"`); statica, non va mai liberata |
| `uint32_t osatura_abi_version(void)` | versione ABI della libreria caricata; confrontala con `OSATURA_ABI_VERSION` dell'header con cui hai compilato |
| `const char *osatura_last_error(void)` | testo dell'ultimo errore **su questo thread**; stringa vuota se non ce ne sono ancora stati; vedi [Gestione degli errori](#gestione-degli-errori) |

### Engine: aprire, interrogare, renderizzare

| Funzione | Descrizione |
|---|---|
| `E *osatura_open(const char *json, size_t len)` | analizza e valida un progetto (UTF-8, `len` byte, da 1 a 256 MiB); restituisce `NULL` in caso di errore. La libreria copia ciò che le serve; puoi liberare subito `json`. I file immagine relativi nel progetto vengono cercati nella directory corrente del processo, i percorsi assoluti e `..` vengono rifiutati (vedi [SECURITY.md](SECURITY.md)); le immagini come data URI funzionano sempre. |
| `void osatura_close(E *)` | libera l'engine; `NULL` è ammesso; da chiamare esattamente una volta per handle |
| `uint32_t osatura_frame_count(const E *)` | numero di fotogrammi |
| `float osatura_fps(const E *)` | fotogrammi al secondo |
| `uint32_t osatura_width(const E *)`, `osatura_height(const E *)` | dimensione del palco in unità del palco, arrotondata a un intero |
| `uint32_t osatura_bone_count(const E *)` | numero di ossi |
| `int osatura_bone_name(const E *, uint32_t index, char *out, size_t cap)` | nome dell'osso `index` (ordine del rig); lunghezza, oppure -1 per un indice errato |
| `int osatura_bone_world(const E *, float frame, float *out, size_t cap_floats)` | matrici mondo di ogni osso; `cap_floats` deve essere almeno `6 * bone_count`, altrimenti -1 |
| `int osatura_render_rgba(const E *, float frame, float scale, uint8_t *out, size_t cap, uint32_t *w, uint32_t *h)` | renderizza un fotogramma; dimensione dell'output `round(width*scale) x round(height*scale)`; restituisce 0, **-2 se `cap` è minore di `w*h*4`** (e `*w`, `*h` vengono impostati, così puoi allocare e richiamare; passa `out == NULL` per chiedere), -1 per altri errori. `w` e `h` possono essere `NULL`. |

### Player: macchine a stati

Un progetto può contenere `machines` (macchine a stati di runtime) e `bindings` (valori esterni che pilotano le proprietà). Un player esegue una macchina: imposta gli input per nome, fai avanzare il tempo, leggi stati ed eventi, renderizza la posa corrente. Per la semantica vedi [API.md](API.md) (sono i metodi di `Player`).

| Funzione | Descrizione |
|---|---|
| `uint32_t osatura_machine_count(const E *)` | macchine nel progetto |
| `int osatura_machine_name(const E *, uint32_t index, char *out, size_t cap)` | nome della macchina `index` (regole dei buffer di caratteri) |
| `P *osatura_player_open(const E *, const char *machine)` | avvia la macchina al tempo 0; `NULL` in caso di errore (macchina sconosciuta, ...). Il player possiede una **copia** del progetto: resta valido dopo `osatura_close` dell'engine. |
| `void osatura_player_close(P *)` | libera il player; `NULL` ammesso |
| `int osatura_player_set_bool(P *, const char *name, int value)` | input bool (`value != 0`) |
| `int osatura_player_set_number(P *, const char *name, double value)` | input numerico (finito) |
| `int osatura_player_fire(P *, const char *trigger)` | fa scattare un trigger; viene consumato dalla transizione che lo verifica |
| `int osatura_player_set_value(P *, const char *path, double value)` | valore esterno per i data binding, ad es. `"player.health"`; pilota anche una regione di scorrimento il cui `input` è questo percorso (pixel lungo il suo asse principale) |
| `int osatura_player_advance(P *, double dt)` | avanza di `dt` secondi, `0..=10`; deterministico (i passi lunghi vengono suddivisi in pezzi di al massimo 1/60 s) |
| `int osatura_player_state(P *, const char *layer, char *out, size_t cap)` | nome dello stato corrente di un livello macchina; lunghezza, oppure -1 per un livello sconosciuto |
| `int osatura_player_events(P *, char *out, size_t cap)` | eventi in sospeso come array JSON `[{"time":..,"layer":..,"kind":"entered"\|"exited","state":..}]`. Restituisce la lunghezza completa. **Gli eventi vengono consumati solo se l'intero testo ci sta (`cap` > lunghezza)**; altrimenti vengono conservati e richiami con un buffer più grande. `out == NULL` chiede la dimensione e non consuma nulla. |
| `int osatura_player_render_rgba(P *, float scale, uint8_t *out, size_t cap, uint32_t *w, uint32_t *h)` | stesso contratto di `osatura_render_rgba`, per la posa corrente; la dimensione segue il canvas del player |

### Player: layout, scorrimento, livello di dettaglio

| Funzione | Descrizione |
|---|---|
| `int osatura_player_set_canvas(P *, float width, float height)` | layout responsivo: dimensione del canvas in unità del palco (finita, positiva, al massimo 32768); i contenitori vengono risolti di nuovo, i render prendono questa dimensione |
| `int osatura_player_canvas_size(P *, float *w, float *h)` | dimensione corrente del canvas; ciascun puntatore può essere `NULL` |
| `uint32_t osatura_player_scroll_count(P *)` | regioni di scorrimento del progetto |
| `int osatura_player_scroll_id(P *, uint32_t index, char *out, size_t cap)` | id della regione `index` (regole dei buffer di caratteri) |
| `int osatura_player_scroll_drag(P *, const char *id, float x, float y)` | offset grezzo del puntatore in pixel (contenuto spostato in alto / a sinistra); oltre le estremità viene limitato o rimbalza elasticamente |
| `int osatura_player_scroll_release(P *, const char *id, float vx, float vy)` | velocità di rilascio in px/s; `osatura_player_advance` poi lancia, aggancia e fa rimbalzare indietro come una molla |
| `int osatura_player_scroll_offset(P *, const char *id, float *x, float *y)` | offset visualizzato; -1 per un id sconosciuto; i puntatori possono essere `NULL` |
| `int osatura_player_scroll_max(P *, const char *id, float *x, float *y)` | offset massimo utile alla dimensione corrente del canvas |
| `int osatura_player_scroll_settled(P *, const char *id)` | 1 a riposo, 0 in movimento, -1 id sconosciuto |
| `int osatura_player_set_screen(P *, float width_px, float height_px, float max_error_px)` | comunica al player quanto grande viene mostrato; esso renderizza il livello di dettaglio precostruito più grossolano il cui limite di errore dichiarato resta entro `max_error_px` pixel. I livelli vengono costruiti al primo uso (questa chiamata può richiedere un po' su un progetto grande, molto in una build di debug). |
| `int osatura_player_lod_level(P *)` | livello in uso (0 = originale, anche senza una dimensione dello schermo); negativo per un player `NULL` |

### Modifica ed esportazione in memoria (additivo, facciata 1.4)

Le modifiche sono validate e annullabili; una modifica fallita non cambia nulla, restituisce -1 e imposta il testo dell'errore. Prendono un `E *` non const (un thread alla volta, come sempre). I colori sono `#rgb`, `#rrggbb` o `#rrggbbaa`. `parent` può essere `NULL`.

| Funzione | Descrizione |
|---|---|
| `int osatura_add_rect_layer(E *, name, x, y, w, h, fill, parent)` | aggiunge in coda un livello rettangolo riempito; `parent` nomina un livello gruppo di primo livello, `NULL` aggiunge al livello superiore |
| `int osatura_add_ellipse_layer(E *, name, cx, cy, rx, ry, fill, parent)` | lo stesso per un'ellisse |
| `int osatura_add_group_layer(E *, name)` | aggiunge in coda un livello gruppo vuoto |
| `int osatura_add_bone(E *, name, parent, length, x, y, angle)` | aggiunge in coda un osso; il genitore deve esistere (`NULL` = radice); angolo in gradi |
| `int osatura_set_key(E *, target, prop, frame, value, interpolation)` | una chiave su `target.prop`; interpolazione 0 lineare, 1 step, 2 ease-in, 3 ease-out, 4 ease-in-out, 5 smooth |
| `int osatura_bake_physics(E *, chain, preset, overwrite)` | simula una catena di ossi separati da virgole (radice per prima) con `hair`, `tail`, `rope`, `antenna` o `cape`; restituisce i canali scritti (0 quando gli ossi con chiavi esistenti sono stati conservati) |
| `int osatura_undo(E *)`, `int osatura_redo(E *)` | 1 eseguito, 0 niente da fare, negativo in caso di errore |
| `uint32_t osatura_layer_count(const E *)`, `int osatura_layer_name(const E *, index, out, cap)` | percorsi dei livelli (`group/child`), regole dei buffer di caratteri |
| `B *osatura_export(const E *, format, scale)` | esportazione dell'intera timeline, `B` = `OsaturaBlob *`; formato 0 GIF, 1 glTF binario, 2 JSON Lottie, 3 player HTML, 4 APNG; `scale` da 0.05 a 4 per GIF e APNG; `NULL` in caso di errore |
| `B *osatura_render_png(const E *, frame, scale)` | un fotogramma come PNG |
| `B *osatura_render_exr(const E *, frame, scale, float32)` | un fotogramma come OpenEXR (lineare, premoltiplicato; un `float32` diverso da zero memorizza FLOAT, altrimenti HALF) |
| `B *osatura_to_json(const E *)` | il progetto (modificato) come JSON UTF-8, senza terminatore |
| `const uint8_t *osatura_blob_data(const B *)`, `size_t osatura_blob_size(const B *)` | i byte di un blob, validi finché non viene liberato; `NULL` / 0 per un blob `NULL` |
| `void osatura_blob_free(B *)` | libera un blob esattamente una volta; `NULL` è ammesso |

## Proprietà e durata

* **Stringhe e array vengono copiati in buffer che fornisci tu; i risultati delle esportazioni sono blob che liberi tu.** Ogni risultato di tipo stringa o array viene copiato in un buffer che fornisci *tu*. L'unica memoria di proprietà della libreria è un `OsaturaBlob` proveniente da `osatura_export`, `osatura_render_png`, `osatura_render_exr` o `osatura_to_json`: leggilo con `osatura_blob_data` / `osatura_blob_size`, poi rilascialo una volta con `osatura_blob_free`. Non esiste un `osatura_free` generico. Gli unici puntatori che la libreria restituisce sono `osatura_version()` (statico) e `osatura_last_error()` (vedi sotto), più i due handle opachi.
* **Handle.** `osatura_open` restituisce un handle di engine; `osatura_player_open` restituisce un handle di player. Ciascuno va chiuso esattamente una volta con il proprio `_close`; chiudere `NULL` va bene; usare un handle dopo averlo chiuso, o chiuderlo due volte, è un comportamento indefinito.
* **Engine e player sono indipendenti.** Un player possiede una copia del progetto, quindi chiudere prima l'engine è ammesso e comune (l'esempio lo fa). Da un solo engine si possono aprire più player.
* **I tuoi buffer.** I puntatori in ingresso devono essere validi solo per la durata della chiamata; la libreria non conserva alcun puntatore a essi. I buffer di output devono avere davvero la capacità che passi; la libreria scrive al massimo `cap` byte (stringhe) o `cap_floats` float.
* **`osatura_last_error()`** restituisce un puntatore a una stringa thread-local. Resta valido fino alla successiva chiamata sullo stesso thread che *fallisce* (quella chiamata la sovrascrive) o fino alla terminazione del thread. Copiala se ti serve più a lungo. Non liberarla.

## Gestione degli errori

1. Verifica sempre il valore di ritorno (`NULL` per le due funzioni `_open` e per `osatura_player_open`, un `int` negativo per le altre, `0` da un getter che riceve un handle `NULL`).
2. Poi leggi `osatura_last_error()` sullo stesso thread per conoscere il motivo: per esempio `invalid project JSON: ...`, `output buffer needs 1228800 bytes`, `invalid argument: input 'excited' is not a number`.
3. **Il testo dell'errore viene impostato solo dai fallimenti e non viene mai cancellato da un successo.** Dopo una successiva chiamata riuscita contiene ancora il vecchio messaggio. Non usare mai "la stringa non è vuota" come test di fallimento.
4. I testi degli errori provengono dalla facciata Rust (`Display` di `ApiError`); sono destinati alle persone, non sono un contratto. L'ABI C non espone il *tipo* di errore.
5. Un panic all'interno del motore viene intercettato e segnalato come `internal error (panic) inside osatura` con il consueto valore di ritorno di fallimento; in linea di principio l'handle resta utilizzabile, ma trattalo come un bug e segnalalo.

## Threading

* Ogni handle può essere usato da **un solo thread alla volta** (il contratto dell'header). Puoi spostare un handle tra thread purché le chiamate non si sovrappongano mai; handle diversi possono essere eseguiti in parallelo su thread diversi (i tipi Rust sottostanti sono `Send + Sync`, ma il contratto C resta prudente).
* La stringa dell'ultimo errore è **per thread**: leggila sul thread che ha effettuato la chiamata fallita.
* Non ci sono callback e la libreria non avvia thread propri.

## Esempio completo in C

Carica un progetto, stampa i dati e le posizioni degli ossi, renderizza un fotogramma in un file PPM (chiedendo prima la dimensione con `NULL`, poi allocando esattamente), esegue la prima macchina a stati del progetto, legge i suoi eventi con il modello della richiesta di dimensione e mostra il percorso di errore. Verificato con gcc, anche con AddressSanitizer/UBSan (nessun rilievo), sia con la libreria condivisa sia con quella statica.

```c
/* OSATURA C ABI example: load, inspect, render a PPM, run a state machine.
 * Build:  cc -std=c99 -Wall -Wextra -Iosatura-capi -o example example.c -Ltarget/debug -losatura_capi
 * Run:    LD_LIBRARY_PATH=target/debug ./example project.json frame.ppm [machine-name]
 */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "osatura.h"

static char *read_file(const char *path, size_t *len) {
    FILE *f = fopen(path, "rb");
    if (!f) return NULL;
    fseek(f, 0, SEEK_END);
    long n = ftell(f);
    fseek(f, 0, SEEK_SET);
    char *buf = malloc(n > 0 ? (size_t)n : 1);
    if (!buf || fread(buf, 1, (size_t)n, f) != (size_t)n) { fclose(f); free(buf); return NULL; }
    fclose(f);
    *len = (size_t)n;
    return buf;
}

int main(int argc, char **argv) {
    if (argc < 3) { fprintf(stderr, "usage: %s project.json out.ppm [machine]\n", argv[0]); return 2; }
    if (osatura_abi_version() != OSATURA_ABI_VERSION) { fprintf(stderr, "header and library disagree\n"); return 2; }
    printf("osatura %s, ABI %u\n", osatura_version(), osatura_abi_version());

    size_t len = 0;
    char *json = read_file(argv[1], &len);
    if (!json) { fprintf(stderr, "cannot read %s\n", argv[1]); return 3; }
    OsaturaEngine *e = osatura_open(json, len);
    free(json);                                   /* the engine keeps its own copy */
    if (!e) { fprintf(stderr, "open failed: %s\n", osatura_last_error()); return 4; }
    printf("%ux%u, %u frames @ %.1f fps\n", osatura_width(e), osatura_height(e), osatura_frame_count(e), osatura_fps(e));

    /* bones: copy the name, then the world matrices of frame 10 */
    uint32_t nb = osatura_bone_count(e);
    float *world = malloc(sizeof(float) * 6 * (nb ? nb : 1));
    if (osatura_bone_world(e, 10.0f, world, 6 * (size_t)nb) != 0) { fprintf(stderr, "%s\n", osatura_last_error()); return 5; }
    for (uint32_t i = 0; i < nb; i++) {
        char name[64];
        osatura_bone_name(e, i, name, sizeof name);
        printf("  bone %-10s tx=%7.2f ty=%7.2f\n", name, world[i * 6 + 4], world[i * 6 + 5]);
    }
    free(world);

    /* render: ask for the size first (out == NULL), then allocate exactly */
    uint32_t w = 0, h = 0;
    int rc = osatura_render_rgba(e, 10.0f, 0.5f, NULL, 0, &w, &h);
    if (rc != -2) { fprintf(stderr, "unexpected rc %d: %s\n", rc, osatura_last_error()); return 6; }
    size_t need = (size_t)w * h * 4;
    uint8_t *px = malloc(need);
    if (osatura_render_rgba(e, 10.0f, 0.5f, px, need, &w, &h) != 0) { fprintf(stderr, "%s\n", osatura_last_error()); return 6; }
    FILE *out = fopen(argv[2], "wb");
    if (!out) return 7;
    fprintf(out, "P6\n%u %u\n255\n", w, h);       /* RGBA straight alpha -> RGB over white */
    for (size_t i = 0; i < (size_t)w * h; i++) {
        unsigned a = px[i * 4 + 3];
        for (int c = 0; c < 3; c++) fputc((px[i * 4 + c] * a + 255 * (255 - a)) / 255, out);
    }
    fclose(out);
    free(px);
    printf("wrote %s (%ux%u)\n", argv[2], w, h);

    /* state machine (only when the project has one) */
    if (osatura_machine_count(e) > 0) {
        char mname[64];
        osatura_machine_name(e, 0, mname, sizeof mname);
        const char *wanted = argc > 3 ? argv[3] : mname;
        OsaturaPlayer *p = osatura_player_open(e, wanted);
        if (!p) { fprintf(stderr, "player: %s\n", osatura_last_error()); return 8; }
        osatura_close(e);                        /* allowed: the player owns a copy of the project */
        osatura_player_set_bool(p, "excited", 1);
        for (int i = 0; i < 30; i++) osatura_player_advance(p, 1.0 / 30.0);
        char state[64];
        osatura_player_state(p, "body", state, sizeof state);
        printf("machine %s: layer body is in state '%s'\n", wanted, state);
        int need_ev = osatura_player_events(p, NULL, 0);   /* query the JSON size; nothing consumed */
        char *ev = malloc((size_t)need_ev + 1);
        osatura_player_events(p, ev, (size_t)need_ev + 1); /* fits: events are consumed now */
        printf("events: %s\n", ev);
        free(ev);
        /* a wrong call reports through osatura_last_error() */
        if (osatura_player_set_number(p, "excited", 1.0) != 0) printf("expected error: %s\n", osatura_last_error());
        osatura_player_close(p);
    } else {
        osatura_close(e);
    }
    return 0;
}
```

Compilato ed eseguito sul progetto `osatura demo` esteso con una macchina a stati (`hero`, input `excited`, stati `idle` e `wave`):

```text
osatura 0.1.0, ABI 1
640x480, 48 frames @ 24.0 fps
  bone arm_up     tx= 150.00 ty= 330.00
  ...
wrote frame.ppm (320x240)
machine hero: layer body is in state 'wave'
events: [{"time":0.0,"layer":"body","kind":"entered","state":"idle"},{"time":0.016666666666666666,"layer":"body","kind":"exited","state":"idle"},{"time":0.016666666666666666,"layer":"body","kind":"entered","state":"wave"}]
expected error: invalid argument: input 'excited' is not a number
```

Il repository include anche `osatura-capi/tests/smoke.c` (apertura, rendering, checksum) e i test Rust `osatura-capi/tests/abi.rs` (puntatori nulli, indici errati, dimensioni dei buffer, sicurezza rispetto ai panic).

## Scorrimento e livello di dettaglio in C

Un progetto con un contenitore di layout e una regione di scorrimento (il progetto di test dello scorrimento dei test Rust: dieci righe da 20 unità in una colonna, un viewport ancorato ai bordi sinistro, superiore e inferiore). Verificato:

```c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "osatura.h"
int main(int argc, char **argv) {
    FILE *f = fopen(argv[1], "rb"); fseek(f, 0, SEEK_END); long n = ftell(f); fseek(f, 0, SEEK_SET);
    char *json = malloc(n); if (fread(json, 1, n, f) != (size_t)n) return 1; fclose(f);
    OsaturaEngine *e = osatura_open(json, n); free(json);
    if (!e) { printf("%s\n", osatura_last_error()); return 2; }
    OsaturaPlayer *p = osatura_player_open(e, "hero");
    osatura_close(e);
    if (!p) { printf("%s\n", osatura_last_error()); return 3; }
    osatura_player_set_canvas(p, 100.0f, 120.0f);               /* containers re-solve for this size */
    char id[32];
    for (uint32_t i = 0; i < osatura_player_scroll_count(p); i++) {
        osatura_player_scroll_id(p, i, id, sizeof id);
        float mx, my, ox, oy;
        osatura_player_scroll_max(p, id, &mx, &my);
        osatura_player_scroll_drag(p, id, 0.0f, 500.0f);        /* far past the end: rubber band / clamp */
        osatura_player_scroll_offset(p, id, &ox, &oy);
        printf("scroll '%s': max=(%g,%g) dragged offset=(%g,%g) settled=%d\n", id, mx, my, ox, oy, osatura_player_scroll_settled(p, id));
        osatura_player_scroll_release(p, id, 0.0f, 0.0f);
        for (int k = 0; k < 60; k++) osatura_player_advance(p, 1.0 / 60.0);
        osatura_player_scroll_offset(p, id, &ox, &oy);
        printf("  after release: offset=(%g,%g) settled=%d\n", ox, oy, osatura_player_scroll_settled(p, id));
    }
    int rc = osatura_player_set_screen(p, 100.0f, 120.0f, 4.0f);   /* accept up to 4 px of error */
    printf("set_screen rc=%d, lod level=%d\n", rc, osatura_player_lod_level(p));
    uint32_t w, h; osatura_player_render_rgba(p, 1.0f, NULL, 0, &w, &h);
    printf("render size %ux%u\n", w, h);
    osatura_player_close(p);
    return 0;
}
```

Output: `scroll 'list': max=(0,80) dragged offset=(0,80) settled=1` (trascinare fino a 500 limita l'offset al massimo quando l'elasticità è disattivata), poi `set_screen rc=0, lod level=7` (accettando fino a 4 px di errore, il livello precostruito più grossolano si qualifica su questo minuscolo progetto; il numero dipende dal progetto) e `render size 100x120`. Nota la variabile `rc` esplicita nel sorgente: l'ordine in cui C valuta gli argomenti di una funzione non è specificato, quindi non mettere mai nello stesso `printf` la chiamata che modifica il player e quella che lo legge.

## Esempio in Python (ctypes)

Solo libreria standard. Dichiara `argtypes` e `restype` per ogni funzione che chiami: altrimenti ctypes assume `int` e tronca i puntatori a 64 bit. Verificato con Python 3 sulla libreria di debug; il suo output PPM è identico byte per byte a quello dell'esempio in C.

```python
#!/usr/bin/env python3
"""OSATURA C ABI from Python with ctypes (standard library only).

    python3 example.py libosatura_capi.so project.json out.ppm
"""
import ctypes as C
import sys

lib = C.CDLL(sys.argv[1])

# --- declare every signature you use (ctypes defaults to int and would truncate pointers) ---
lib.osatura_version.restype = C.c_char_p
lib.osatura_abi_version.restype = C.c_uint32
lib.osatura_last_error.restype = C.c_char_p
lib.osatura_open.argtypes = [C.c_char_p, C.c_size_t]
lib.osatura_open.restype = C.c_void_p                      # opaque handle
lib.osatura_close.argtypes = [C.c_void_p]
for name, res in (("frame_count", C.c_uint32), ("fps", C.c_float), ("width", C.c_uint32),
                  ("height", C.c_uint32), ("bone_count", C.c_uint32), ("machine_count", C.c_uint32)):
    f = getattr(lib, "osatura_" + name)
    f.argtypes = [C.c_void_p]
    f.restype = res
lib.osatura_bone_name.argtypes = [C.c_void_p, C.c_uint32, C.c_char_p, C.c_size_t]
lib.osatura_bone_name.restype = C.c_int
lib.osatura_bone_world.argtypes = [C.c_void_p, C.c_float, C.POINTER(C.c_float), C.c_size_t]
lib.osatura_bone_world.restype = C.c_int
lib.osatura_render_rgba.argtypes = [C.c_void_p, C.c_float, C.c_float, C.c_void_p, C.c_size_t,
                                     C.POINTER(C.c_uint32), C.POINTER(C.c_uint32)]
lib.osatura_render_rgba.restype = C.c_int


def last_error() -> str:
    return (lib.osatura_last_error() or b"").decode("utf-8", "replace")


class Engine:
    """Owns one handle; use as a context manager so osatura_close runs exactly once."""

    def __init__(self, json_bytes: bytes):
        self.h = lib.osatura_open(json_bytes, len(json_bytes))
        if not self.h:
            raise ValueError(last_error())

    def close(self):
        if self.h:
            lib.osatura_close(self.h)
            self.h = None

    __enter__ = lambda self: self
    __exit__ = lambda self, *a: self.close()

    def bones(self):
        out = []
        for i in range(lib.osatura_bone_count(self.h)):
            buf = C.create_string_buffer(128)
            lib.osatura_bone_name(self.h, i, buf, len(buf))
            out.append(buf.value.decode())
        return out

    def bone_world(self, frame: float):
        n = lib.osatura_bone_count(self.h)
        arr = (C.c_float * (6 * n))()
        if lib.osatura_bone_world(self.h, frame, arr, len(arr)) != 0:
            raise RuntimeError(last_error())
        return [tuple(arr[i * 6:i * 6 + 6]) for i in range(n)]

    def render(self, frame: float, scale: float = 1.0):
        w, h = C.c_uint32(), C.c_uint32()
        rc = lib.osatura_render_rgba(self.h, frame, scale, None, 0, C.byref(w), C.byref(h))
        if rc != -2:                                       # -2 = "buffer too small", and w/h are filled in
            raise RuntimeError(last_error())
        buf = (C.c_uint8 * (w.value * h.value * 4))()
        if lib.osatura_render_rgba(self.h, frame, scale, buf, len(buf), C.byref(w), C.byref(h)) != 0:
            raise RuntimeError(last_error())
        return w.value, h.value, bytes(buf)


if __name__ == "__main__":
    print("osatura", lib.osatura_version().decode(), "ABI", lib.osatura_abi_version())
    data = open(sys.argv[2], "rb").read()
    try:
        Engine(b"{ not json")
    except ValueError as e:
        print("expected error:", e)
    with Engine(data) as e:
        print(lib.osatura_width(e.h), "x", lib.osatura_height(e.h), lib.osatura_frame_count(e.h), "frames")
        print("bones:", e.bones())
        print("arm_up world @10:", [round(v, 2) for v in e.bone_world(10.0)[0]])
        w, h, rgba = e.render(10.0, 0.5)
        with open(sys.argv[3], "wb") as f:
            f.write(b"P6\n%d %d\n255\n" % (w, h))
            for i in range(0, len(rgba), 4):
                r, g, b, a = rgba[i:i + 4]
                f.write(bytes((c * a + 255 * (255 - a)) // 255 for c in (r, g, b)))
        print("wrote", sys.argv[3], w, h)
```

## Altri linguaggi

Tutto ciò che può chiamare una funzione C funziona allo stesso modo: dichiara le 53 funzioni, tieni i puntatori opachi come `void *` e segui le convenzioni sui buffer descritte sopra. Note specifiche per gli ambienti ospite più comuni (non testate qui, sono solo conseguenze del contratto precedente): in C#/.NET usa `[DllImport]` con `CallingConvention.Cdecl` e converti le stringhe come array di byte UTF-8; in Go con cgo, tieni i puntatori Go fuori dalle chiamate, tranne per i buffer che vivono per tutta la durata della chiamata; in Rust preferisci la facciata direttamente invece di questa ABI.

## Versionamento

* `OSATURA_ABI_VERSION` / `osatura_abi_version()` è attualmente **1**. Cambia solo quando una funzione esistente viene rimossa o il suo significato cambia. Tutte le aggiunte finora (le funzioni del player nella facciata 1.1, layout, scorrimento e livello di dettaglio nella 1.2) sono state additive e l'hanno lasciata a 1, quindi un programma compilato con un header più vecchio continua a funzionare con una libreria più recente, mentre un header più recente con una libreria più vecchia fallisce in fase di link o di caricamento per il simbolo mancante. Controlla `osatura_abi_version()` all'avvio e confrontala con la macro.
* `osatura_version()` è la versione del crate `osatura-capi` (0.1.0), non la versione della facciata. Non esiste alcuna funzione C che riporti la versione della facciata (`API_VERSION`).
* La facciata 1.4 ha aggiunto le funzioni di base per la modifica e per l'esportazione in memoria descritte sopra (additivo, versione ABI ancora 1). Non esposti nell'ABI C: rendering SVG, eliminazione delle chiavi, controllo di versione, CI delle animazioni, vettorizzazione, mocap, fuzzing, esportazione con dimensione obiettivo, script. Sono disponibili in Rust e tramite la CLI.

## Particolarità note

I commenti dell'header sono stati corretti per corrispondere al comportamento (prima non coincidevano); i punti da conoscere:

* `osatura_render_rgba` scrive `*w` / `*h` in caso di successo e con -2 (buffer troppo piccolo), e anche quando si verifica un errore di rendering successivo dopo che `frame` e `scale` sono stati accettati. **Non** li scrive quando l'engine è nullo o quando `frame` / `scale` vengono rifiutati (-1). Test di regressione: `documented_size_outputs_and_last_error_semantics` in `osatura-capi/tests/abi.rs`.
* Il testo dell'ultimo errore è thread-local, viene sovrascritto solo dalla successiva chiamata *fallita* e non viene mai cancellato da un successo. Copialo se ti serve più a lungo.
* `osatura_width` / `osatura_height` arrotondano a un intero la dimensione float del palco; la dimensione reale del rendering è `round(size * scale)`, quindi usa i valori `w` / `h` restituiti dalla chiamata di rendering.
* La nota dell'header sulle regioni di scorrimento che seguono `osatura_player_set_value` si trova tra le dichiarazioni dello scorrimento e quelle del livello di dettaglio; descrive `osatura_player_set_value`.
