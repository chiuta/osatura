# ABI-ul C OSATURA

*Versiunea ABI 1 (`OSATURA_ABI_VERSION`, `osatura_abi_version()`). Crate `osatura-capi`, header `osatura-capi/osatura.h`. Public țintă: integratorii care vor să încarce și să redea proiecte OSATURA din C, C++, Python, Go, C#, Swift sau orice limbaj cu o interfață C de apel de funcții externe (FFI). Documentație CC-BY-4.0; exemple de cod MPL-2.0.*

ABI-ul C este un wrapper mic, în C pur, peste fațada Rust stabilă ([API.md](API.md)). Cu el, un program gazdă poate deschide un proiect, citi datele lui și transformările oaselor, randa cadre în RGBA și rula **mașini de stări** (redare interactivă, derulare, nivel de detaliu). Începând cu fațada 1.4 poate face și **editări** de bază (crearea de straturi și oase, setarea de chei, baking de fizică, anulare) și **export în memorie** (GIF, glTF, Lottie, HTML, APNG, PNG, EXR, JSON de proiect). Nu poate face control al versiunilor și nu poate rula uneltele: pentru acestea folosește API-ul Rust sau linia de comandă ([CLI_REFERENCE.md](CLI_REFERENCE.md)).

Cuprins: [Build și linkare](#build-și-linkare) - [Convenții](#convenții) - [Referința funcțiilor](#referința-funcțiilor) - [Proprietate și durată de viață](#proprietate-și-durată-de-viață) - [Tratarea erorilor](#tratarea-erorilor) - [Threading](#threading) - [Exemplu complet în C](#exemplu-complet-în-c) - [Derulare și nivel de detaliu în C](#derulare-și-nivel-de-detaliu-în-c) - [Exemplu Python (ctypes)](#exemplu-python-ctypes) - [Alte limbaje](#alte-limbaje) - [Versionare](#versionare) - [Particularități cunoscute](#particularități-cunoscute)

## Build și linkare

Crate-ul construiește trei tipuri de biblioteci (`crate-type = ["cdylib", "staticlib", "rlib"]`); depinde de `osatura-core` cu `default-features = false`, deci nu conține motor de scripting și niciun crate terț MPL.

```sh
cargo build -p osatura-capi             # debug:   target/debug/libosatura_capi.{so,a}
cargo build --release -p osatura-capi   # release: target/release/libosatura_capi.{so,a}
```

| Platformă | Bibliotecă dinamică | Bibliotecă statică |
|---|---|---|
| Linux | `libosatura_capi.so` | `libosatura_capi.a` |
| macOS | `libosatura_capi.dylib` | `libosatura_capi.a` |
| Windows | `osatura_capi.dll` (+ bibliotecă de import) | `osatura_capi.lib` |

**Testat:** Linux x86-64, gcc, atât biblioteca partajată, cât și cea statică (mai jos). macOS și Windows sunt numele standard de ieșire Rust pentru acele ținte, dar **nu au fost construite sau rulate** de autor. Un `.so` de debug are aproximativ 150 MB (informații de debug); construiește `--release` pentru tot ce livrezi (dimensiunea bibliotecii release nu a fost măsurată aici).

Compilare și linkare cu biblioteca partajată:

```sh
cc -std=c99 -Wall -Wextra -Iosatura-capi -o example example.c -Ltarget/debug -losatura_capi
LD_LIBRARY_PATH=target/debug ./example project.json frame.ppm      # or install the .so and use rpath / ldconfig
```

Linkare statică (verificată pe Linux; biblioteca standard Rust are nevoie de aceste biblioteci de sistem):

```sh
cc -std=c99 -Iosatura-capi -o example example.c target/debug/libosatura_capi.a -lpthread -ldl -lm
```

`osatura.h` are gărzi `extern "C"` pentru C++, are nevoie doar de `<stddef.h>` și `<stdint.h>` și este singurul fișier pe care îl incluzi. Copiază-l lângă sursele tale. Nu există fișier pkg-config, țintă de instalare sau SONAME versionat; livrează biblioteca împreună cu aplicația ta.

Panic-urile sunt prinse la fiecare punct de intrare și transformate în coduri de eroare, ceea ce se bazează pe construirea bibliotecii cu `panic = "unwind"` implicit. Nu o construi cu `panic = "abort"` (altfel un bug din motor ar opri procesul tău în loc să returneze -1). Workspace-ul nu setează această opțiune de profil.

## Convenții

* Șirurile sunt UTF-8. Șirurile de intrare (`const char *`) se termină cu NUL, cu excepția JSON-ului proiectului, care se transmite ca pointer plus lungime și nu are nevoie de terminator.
* Funcțiile care returnează `int` dau **0 la succes și o valoare negativă la eșec**. `-1` este o eroare generală; `-2` înseamnă suplimentar „bufferul tău de ieșire este prea mic” și apare doar în cele două funcții de randare.
* Funcțiile care umplu un buffer de caractere primesc `(char *out, size_t cap)` și returnează **lungimea completă** a textului (fără NUL). Textul este trunchiat la `cap - 1` octeți și terminat cu NUL când `cap > 0`. Transmite `out == NULL` (cu orice `cap`) pentru a cere doar lungimea. O valoare negativă returnată este o eroare (index greșit, nume necunoscut).
* Argumentele float trebuie să fie finite; cadrele din afara timeline-ului sunt limitate; `scale` trebuie să fie între 0.05 și 4.
* Orice argument de tip handle poate fi `NULL` pentru getteri (aceștia returnează 0 și setează textul erorii) și este o eroare pentru restul.
* RGBA-ul de ieșire este cu **alfa neprefixat (straight)** (nu premultiplicat), 8 biți pe canal, pe rânduri, primul rând sus, `width * height * 4` octeți.
* Transformările de lume ale oaselor sunt șase float-uri pe os `[a b c d tx ty]`; un punct local osului `(x, y)` se transformă în `(a*x + c*y + tx, b*x + d*y + ty)`.

## Referința funcțiilor

53 de funcții. `E` = `OsaturaEngine *`, `P` = `OsaturaPlayer *`.

### Bibliotecă

| Funcție | Descriere |
|---|---|
| `const char *osatura_version(void)` | șirul de versiune al crate-ului `osatura-capi` (în prezent `"0.1.0"`); static, nu se eliberează niciodată |
| `uint32_t osatura_abi_version(void)` | versiunea ABI a bibliotecii încărcate; compar-o cu `OSATURA_ABI_VERSION` din header-ul cu care ai compilat |
| `const char *osatura_last_error(void)` | textul ultimei erori **pe acest thread**; șir gol dacă nu a existat încă niciuna; vezi [Tratarea erorilor](#tratarea-erorilor) |

### Engine: deschidere, interogare, randare

| Funcție | Descriere |
|---|---|
| `E *osatura_open(const char *json, size_t len)` | parsează și validează un proiect (UTF-8, `len` octeți, între 1 și 256 MiB); returnează `NULL` la eroare. Biblioteca copiază ce îi trebuie; poți elibera `json` imediat. Fișierele de imagine relative din proiect sunt căutate în directorul curent al procesului, căile absolute și `..` sunt refuzate (vezi [SECURITY.md](SECURITY.md)); imaginile data-URI funcționează întotdeauna. |
| `void osatura_close(E *)` | eliberează engine-ul; `NULL` este permis; apelează exact o dată pentru fiecare handle |
| `uint32_t osatura_frame_count(const E *)` | numărul de cadre |
| `float osatura_fps(const E *)` | cadre pe secundă |
| `uint32_t osatura_width(const E *)`, `osatura_height(const E *)` | dimensiunea scenei în unități de scenă, rotunjită la întreg |
| `uint32_t osatura_bone_count(const E *)` | numărul de oase |
| `int osatura_bone_name(const E *, uint32_t index, char *out, size_t cap)` | numele osului `index` (ordinea din rig); lungimea, sau -1 pentru un index greșit |
| `int osatura_bone_world(const E *, float frame, float *out, size_t cap_floats)` | matricile de lume ale fiecărui os; `cap_floats` trebuie să fie cel puțin `6 * bone_count`, altfel -1 |
| `int osatura_render_rgba(const E *, float frame, float scale, uint8_t *out, size_t cap, uint32_t *w, uint32_t *h)` | randează un cadru; dimensiunea ieșirii `round(width*scale) x round(height*scale)`; returnează 0, **-2 dacă `cap` este mai mic decât `w*h*4`** (iar `*w`, `*h` sunt setate, deci poți aloca și apela din nou; transmite `out == NULL` pentru a întreba), -1 la alte erori. `w` și `h` pot fi `NULL`. |

### Player: mașini de stări

Un proiect poate conține `machines` (mașini de stări de runtime) și `bindings` (valori externe care comandă proprietăți). Un player rulează o mașină: setezi intrările după nume, avansezi timpul, citești stările și evenimentele, randezi poziția curentă. Vezi [API.md](API.md) pentru semantică (sunt metodele `Player`).

| Funcție | Descriere |
|---|---|
| `uint32_t osatura_machine_count(const E *)` | mașinile din proiect |
| `int osatura_machine_name(const E *, uint32_t index, char *out, size_t cap)` | numele mașinii `index` (regulile bufferului de caractere) |
| `P *osatura_player_open(const E *, const char *machine)` | pornește mașina la momentul 0; `NULL` la eroare (mașină necunoscută, ...). Player-ul deține o **copie** a proiectului: rămâne valid după `osatura_close` al engine-ului. |
| `void osatura_player_close(P *)` | eliberează player-ul; `NULL` permis |
| `int osatura_player_set_bool(P *, const char *name, int value)` | intrare bool (`value != 0`) |
| `int osatura_player_set_number(P *, const char *name, double value)` | intrare numerică (finită) |
| `int osatura_player_fire(P *, const char *trigger)` | declanșează un trigger; este consumat de tranziția care îl testează |
| `int osatura_player_set_value(P *, const char *path, double value)` | valoare externă pentru data bindings, de ex. `"player.health"`; comandă și o regiune de derulare al cărei `input` este această cale (pixeli de-a lungul axei sale principale) |
| `int osatura_player_advance(P *, double dt)` | avansează cu `dt` secunde, `0..=10`; determinist (pașii lungi sunt împărțiți în bucăți de cel mult 1/60 s) |
| `int osatura_player_state(P *, const char *layer, char *out, size_t cap)` | numele stării curente a unui strat-mașină; lungimea, sau -1 pentru un strat necunoscut |
| `int osatura_player_events(P *, char *out, size_t cap)` | evenimentele în așteptare ca tablou JSON `[{"time":..,"layer":..,"kind":"entered"\|"exited","state":..}]`. Returnează lungimea completă. **Evenimentele sunt consumate doar dacă tot textul a încăput (`cap` > lungime)**; altfel sunt păstrate, iar tu apelezi din nou cu un buffer mai mare. `out == NULL` interoghează dimensiunea și nu consumă nimic. |
| `int osatura_player_render_rgba(P *, float scale, uint8_t *out, size_t cap, uint32_t *w, uint32_t *h)` | același contract ca `osatura_render_rgba`, pentru poziția curentă; dimensiunea urmează canvasul player-ului |

### Player: layout, derulare, nivel de detaliu

| Funcție | Descriere |
|---|---|
| `int osatura_player_set_canvas(P *, float width, float height)` | layout adaptiv: dimensiunea canvasului în unități de scenă (finită, pozitivă, cel mult 32768); containerele se rezolvă din nou, randările primesc această dimensiune |
| `int osatura_player_canvas_size(P *, float *w, float *h)` | dimensiunea curentă a canvasului; oricare dintre pointeri poate fi `NULL` |
| `uint32_t osatura_player_scroll_count(P *)` | regiunile de derulare ale proiectului |
| `int osatura_player_scroll_id(P *, uint32_t index, char *out, size_t cap)` | id-ul regiunii `index` (regulile bufferului de caractere) |
| `int osatura_player_scroll_drag(P *, const char *id, float x, float y)` | decalaj brut al pointerului în pixeli (conținutul mutat în sus / spre stânga); dincolo de capete este limitat sau elastic (rubber-band) |
| `int osatura_player_scroll_release(P *, const char *id, float vx, float vy)` | viteza la eliberare în px/s; `osatura_player_advance` apoi lansează, aliniază și revine elastic |
| `int osatura_player_scroll_offset(P *, const char *id, float *x, float *y)` | decalajul afișat; -1 pentru un id necunoscut; pointerii pot fi `NULL` |
| `int osatura_player_scroll_max(P *, const char *id, float *x, float *y)` | cel mai mare decalaj util la dimensiunea curentă a canvasului |
| `int osatura_player_scroll_settled(P *, const char *id)` | 1 în repaus, 0 în mișcare, -1 id necunoscut |
| `int osatura_player_set_screen(P *, float width_px, float height_px, float max_error_px)` | spune player-ului cât de mare este afișat; acesta randează cel mai grosier nivel de detaliu precalculat a cărui limită de eroare declarată rămâne în `max_error_px` pixeli. Nivelurile se construiesc la prima utilizare (acest apel poate dura un moment pe un proiect mare, mult într-un build de debug). |
| `int osatura_player_lod_level(P *)` | nivelul folosit (0 = original, și fără o dimensiune de ecran); negativ pentru un player `NULL` |

### Editare și export în memorie (aditiv, fațada 1.4)

Editările sunt validate și anulabile; o editare eșuată nu schimbă nimic, returnează -1 și setează textul erorii. Primesc un `E *` non-const (un singur thread la un moment dat, ca întotdeauna). Culorile sunt `#rgb`, `#rrggbb` sau `#rrggbbaa`. `parent` poate fi `NULL`.

| Funcție | Descriere |
|---|---|
| `int osatura_add_rect_layer(E *, name, x, y, w, h, fill, parent)` | adaugă la sfârșit un strat dreptunghi umplut; `parent` numește un strat-grup de nivel superior, `NULL` adaugă la nivelul superior |
| `int osatura_add_ellipse_layer(E *, name, cx, cy, rx, ry, fill, parent)` | la fel pentru o elipsă |
| `int osatura_add_group_layer(E *, name)` | adaugă la sfârșit un strat-grup gol |
| `int osatura_add_bone(E *, name, parent, length, x, y, angle)` | adaugă la sfârșit un os; părintele trebuie să existe (`NULL` = rădăcină); unghi în grade |
| `int osatura_set_key(E *, target, prop, frame, value, interpolation)` | o cheie pe `target.prop`; interpolare 0 linear, 1 step, 2 ease-in, 3 ease-out, 4 ease-in-out, 5 smooth |
| `int osatura_bake_physics(E *, chain, preset, overwrite)` | simulează un lanț de oase separate prin virgulă (rădăcina prima) cu `hair`, `tail`, `rope`, `antenna` sau `cape`; returnează canalele scrise (0 când oasele existente cu chei au fost păstrate) |
| `int osatura_undo(E *)`, `int osatura_redo(E *)` | 1 făcut, 0 nimic de făcut, negativ la eroare |
| `uint32_t osatura_layer_count(const E *)`, `int osatura_layer_name(const E *, index, out, cap)` | căi de straturi (`group/child`), regulile bufferului de caractere |
| `B *osatura_export(const E *, format, scale)` | export al întregului timeline, `B` = `OsaturaBlob *`; format 0 GIF, 1 glTF binar, 2 Lottie JSON, 3 player HTML, 4 APNG; `scale` între 0.05 și 4 pentru GIF și APNG; `NULL` la eroare |
| `B *osatura_render_png(const E *, frame, scale)` | un cadru ca PNG |
| `B *osatura_render_exr(const E *, frame, scale, float32)` | un cadru ca OpenEXR (liniar, premultiplicat; un `float32` nenul stochează FLOAT, altfel HALF) |
| `B *osatura_to_json(const E *)` | proiectul (editat) ca JSON UTF-8, fără terminator |
| `const uint8_t *osatura_blob_data(const B *)`, `size_t osatura_blob_size(const B *)` | octeții unui blob, valizi până la eliberarea lui; `NULL` / 0 pentru un blob `NULL` |
| `void osatura_blob_free(B *)` | eliberează un blob exact o dată; `NULL` este permis |

## Proprietate și durată de viață

* **Șirurile și tablourile sunt copiate în buffere pe care le oferi tu; rezultatele exportului sunt blob-uri pe care le eliberezi tu.** Fiecare rezultat de tip șir sau tablou este copiat într-un buffer oferit *de tine*. Singura memorie deținută de bibliotecă este un `OsaturaBlob` de la `osatura_export`, `osatura_render_png`, `osatura_render_exr` sau `osatura_to_json`: citește-l cu `osatura_blob_data` / `osatura_blob_size`, apoi eliberează-l o singură dată cu `osatura_blob_free`. Nu există un `osatura_free` generic. Singurii pointeri pe care îi returnează biblioteca sunt `osatura_version()` (static) și `osatura_last_error()` (vezi mai jos), plus cele două handle-uri opace.
* **Handle-uri.** `osatura_open` returnează un handle de engine; `osatura_player_open` returnează un handle de player. Fiecare trebuie închis exact o dată cu propriul `_close`; închiderea lui `NULL` este în regulă; folosirea unui handle după închidere sau închiderea lui de două ori este comportament nedefinit.
* **Engine-ul și player-ele sunt independente.** Un player deține o copie a proiectului, deci închiderea engine-ului mai întâi este permisă și frecventă (exemplul o face). Din același engine se pot deschide mai multe player-e.
* **Bufferele tale.** Pointerii de intrare trebuie să fie valizi doar pe durata apelului; biblioteca nu păstrează niciun pointer către ei. Bufferele de ieșire trebuie să aibă într-adevăr capacitatea pe care o transmiți; biblioteca scrie cel mult `cap` octeți (șiruri) sau `cap_floats` float-uri.
* **`osatura_last_error()`** returnează un pointer către un șir thread-local. Rămâne valid până la următorul apel de pe același thread care *eșuează* (acel apel îl suprascrie) sau până la ieșirea thread-ului. Copiază-l dacă ai nevoie de el mai mult timp. Nu-l elibera.

## Tratarea erorilor

1. Testează întotdeauna valoarea returnată (`NULL` pentru cele două funcții `_open` și pentru `osatura_player_open`, un `int` negativ în rest, `0` de la un getter căruia i s-a dat un handle `NULL`).
2. Apoi citește `osatura_last_error()` pe același thread pentru motiv: de exemplu `invalid project JSON: ...`, `output buffer needs 1228800 bytes`, `invalid argument: input 'excited' is not a number`.
3. **Textul erorii este setat doar de eșecuri și nu este șters niciodată de un succes.** După un apel reușit ulterior încă mai conține vechiul mesaj. Nu folosi niciodată „șirul este nevid” ca test de eșec.
4. Textele erorilor provin din fațada Rust (`Display` al `ApiError`); sunt pentru oameni, nu un contract. ABI-ul C nu expune *tipul* erorii.
5. Un panic în motor este prins și raportat ca `internal error (panic) inside osatura`, cu returnarea obișnuită de eșec; handle-ul rămâne în principiu utilizabil, dar tratează cazul ca pe un bug și raportează-l.

## Threading

* Fiecare handle poate fi folosit de pe **un singur thread la un moment dat** (contractul header-ului). Poți muta un handle între thread-uri atâta timp cât apelurile nu se suprapun; handle-uri diferite pot rula în paralel pe thread-uri diferite (tipurile Rust de dedesubt sunt `Send + Sync`, dar contractul C rămâne conservator).
* Șirul ultimei erori este **per thread**: citește-l pe thread-ul care a făcut apelul eșuat.
* Nu există callback-uri, iar biblioteca nu pornește thread-uri proprii.

## Exemplu complet în C

Încarcă un proiect, afișează date și poziții de oase, randează un cadru într-un fișier PPM (cerând întâi dimensiunea cu `NULL`, apoi alocând exact), rulează prima mașină de stări a proiectului, citește evenimentele ei cu tiparul de interogare a dimensiunii și arată calea erorii. Verificat cu gcc, inclusiv sub AddressSanitizer/UBSan (fără constatări), atât cu biblioteca partajată, cât și cu cea statică.

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

Construit și rulat pe proiectul `osatura demo` extins cu o mașină de stări (`hero`, intrarea `excited`, stările `idle` și `wave`):

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

Repository-ul include și `osatura-capi/tests/smoke.c` (deschidere, randare, sumă de control) și testele Rust `osatura-capi/tests/abi.rs` (pointeri nuli, indecși greșiți, dimensiuni de buffer, siguranță la panic).

## Derulare și nivel de detaliu în C

Un proiect cu un container de layout și o regiune de derulare (proiectul de test de derulare al testelor Rust: zece rânduri de 20 de unități într-o coloană, un viewport fixat la marginile din stânga, de sus și de jos). Verificat:

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

Ieșire: `scroll 'list': max=(0,80) dragged offset=(0,80) settled=1` (tragerea până la 500 limitează la maxim când elasticitatea este dezactivată), apoi `set_screen rc=0, lod level=7` (cu acceptarea a până la 4 px de eroare, cel mai grosier nivel precalculat se califică pe acest proiect minuscul; numărul depinde de proiect) și `render size 100x120`. Observă variabila explicită `rc` din sursă: ordinea în care C evaluează argumentele unei funcții este nespecificată, deci nu pune niciodată apelul care modifică player-ul și apelul care îl citește într-un singur `printf`.

## Exemplu Python (ctypes)

Doar biblioteca standard. Declară `argtypes` și `restype` pentru fiecare funcție pe care o apelezi: altfel ctypes presupune `int` și trunchiază pointerii pe 64 de biți. Verificat pe Python 3 cu biblioteca de debug; ieșirea sa PPM este identică octet cu octet cu cea a exemplului C.

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

## Alte limbaje

Orice poate apela o funcție C funcționează la fel: declară cele 53 de funcții, păstrează pointerii opaci ca `void *` și urmează convențiile de buffer de mai sus. Note specifice gazdelor uzuale (netestate aici, doar consecințe ale contractului de mai sus): în C#/.NET folosește `[DllImport]` cu `CallingConvention.Cdecl` și marshalează șirurile ca tablouri de octeți UTF-8; în Go cu cgo, ține pointerii Go în afara apelurilor, cu excepția bufferelor care există pe durata apelului; în Rust preferă fațada direct, în loc de acest ABI.

## Versionare

* `OSATURA_ABI_VERSION` / `osatura_abi_version()` este în prezent **1**. Se schimbă doar când o funcție existentă este eliminată sau semnificația ei se schimbă. Toate adăugirile de până acum (funcțiile player din fațada 1.1, layout, derulare și nivel de detaliu din 1.2) au fost aditive și au lăsat-o la 1, deci un program construit cu un header mai vechi continuă să funcționeze cu o bibliotecă mai nouă, iar un header mai nou cu o bibliotecă mai veche eșuează la linkare sau la încărcare din cauza simbolului lipsă. Verifică `osatura_abi_version()` la pornire și compar-o cu macroul.
* `osatura_version()` este versiunea crate-ului `osatura-capi` (0.1.0), nu versiunea fațadei. Nu există nicio funcție C care să raporteze versiunea fațadei (`API_VERSION`).
* Fațada 1.4 a adăugat funcțiile de bază pentru editare și de export în memorie de mai sus (aditiv, versiunea ABI rămâne 1). Nu sunt expuse în ABI-ul C: randarea SVG, ștergerea cheilor, controlul versiunilor, CI de animație, trasarea, mocap, fuzzing, exportul cu țintă de dimensiune, scripturile. Sunt disponibile în Rust și prin CLI.

## Particularități cunoscute

Comentariile din header au fost corectate pentru a corespunde comportamentului (înainte nu se potriveau); punctele care merită cunoscute:

* `osatura_render_rgba` scrie `*w` / `*h` la succes și la -2 (buffer prea mic), precum și atunci când apare o eroare de randare ulterioară după ce `frame` și `scale` au fost acceptate. **Nu** le scrie când engine-ul este nul sau când `frame` / `scale` sunt respinse (-1). Test de regresie: `documented_size_outputs_and_last_error_semantics` în `osatura-capi/tests/abi.rs`.
* Textul ultimei erori este thread-local, suprascris doar de următorul apel *eșuat* și niciodată șters de un succes. Copiază-l dacă ai nevoie de el mai mult timp.
* `osatura_width` / `osatura_height` rotunjesc dimensiunea float a scenei la un întreg; dimensiunea reală de randare este `round(size * scale)`, deci folosește `w` / `h` returnate de apelul de randare.
* Nota din header despre regiunile de derulare care urmează `osatura_player_set_value` se află între declarațiile de derulare și cele de nivel de detaliu; ea descrie `osatura_player_set_value`.
