# ABI de C de OSATURA

*Versión de ABI 1 (`OSATURA_ABI_VERSION`, `osatura_abi_version()`). Crate `osatura-capi`, cabecera `osatura-capi/osatura.h`. Destinatarios: integradores que quieren cargar y reproducir proyectos de OSATURA desde C, C++, Python, Go, C#, Swift o cualquier lenguaje con una interfaz de funciones externas de C. Documentación CC-BY-4.0; ejemplos de código MPL-2.0.*

La ABI de C es un envoltorio pequeño y en C puro sobre la fachada estable de Rust ([API.md](API.md)). Con ella, un programa anfitrión puede abrir un proyecto, leer sus datos y las transformaciones de los huesos, renderizar fotogramas a RGBA y ejecutar **máquinas de estados** (reproducción interactiva, desplazamiento, nivel de detalle). Desde la fachada 1.4 también puede hacer **ediciones** básicas (crear capas y huesos, establecer claves, hacer baking de física, deshacer) y **exportar a memoria** (GIF, glTF, Lottie, HTML, APNG, PNG, EXR, JSON de proyecto). No puede controlar versiones ni ejecutar las herramientas: para eso usa la API de Rust o la línea de comandos ([CLI_REFERENCE.md](CLI_REFERENCE.md)).

Contenido: [Compilación y enlazado](#compilación-y-enlazado) - [Convenciones](#convenciones) - [Referencia de funciones](#referencia-de-funciones) - [Propiedad y tiempo de vida](#propiedad-y-tiempo-de-vida) - [Tratamiento de errores](#tratamiento-de-errores) - [Hilos](#hilos) - [Ejemplo completo en C](#ejemplo-completo-en-c) - [Desplazamiento y nivel de detalle en C](#desplazamiento-y-nivel-de-detalle-en-c) - [Ejemplo en Python (ctypes)](#ejemplo-en-python-ctypes) - [Otros lenguajes](#otros-lenguajes) - [Versionado](#versionado) - [Particularidades conocidas](#particularidades-conocidas)

## Compilación y enlazado

El crate genera tres tipos de biblioteca (`crate-type = ["cdylib", "staticlib", "rlib"]`); depende de `osatura-core` con `default-features = false`, por lo que no contiene motor de scripting ni ningún crate de terceros MPL.

```sh
cargo build -p osatura-capi             # debug:   target/debug/libosatura_capi.{so,a}
cargo build --release -p osatura-capi   # release: target/release/libosatura_capi.{so,a}
```

| Plataforma | Biblioteca dinámica | Biblioteca estática |
|---|---|---|
| Linux | `libosatura_capi.so` | `libosatura_capi.a` |
| macOS | `libosatura_capi.dylib` | `libosatura_capi.a` |
| Windows | `osatura_capi.dll` (+ biblioteca de importación) | `osatura_capi.lib` |

**Probado:** Linux x86-64, gcc, tanto la biblioteca compartida como la estática (más abajo). macOS y Windows usan los nombres de salida estándar de Rust para esos destinos, pero el autor **no los compiló ni los ejecutó**. Un `.so` de depuración ocupa unos 150 MB (información de depuración); compila con `--release` para cualquier cosa que distribuyas (aquí no se midió el tamaño de la biblioteca de release).

Compila y enlaza con la biblioteca compartida:

```sh
cc -std=c99 -Wall -Wextra -Iosatura-capi -o example example.c -Ltarget/debug -losatura_capi
LD_LIBRARY_PATH=target/debug ./example project.json frame.ppm      # or install the .so and use rpath / ldconfig
```

Enlazado estático (verificado en Linux; la biblioteca estándar de Rust necesita estas bibliotecas del sistema):

```sh
cc -std=c99 -Iosatura-capi -o example example.c target/debug/libosatura_capi.a -lpthread -ldl -lm
```

`osatura.h` tiene guardas `extern "C"` para C++, solo necesita `<stddef.h>` y `<stdint.h>`, y es el único archivo que incluyes. Cópialo junto a tus fuentes. No hay archivo pkg-config, ni destino de instalación, ni SONAME versionado; incluye la biblioteca junto con tu aplicación.

Los pánicos se capturan en cada punto de entrada y se convierten en códigos de error, lo cual depende de que la biblioteca se compile con el valor por defecto `panic = "unwind"`. No la compiles con `panic = "abort"` (entonces un error en el motor abortaría tu proceso en lugar de devolver -1). El workspace no establece esa opción de perfil.

## Convenciones

* Las cadenas son UTF-8. Las cadenas de entrada (`const char *`) terminan en NUL, salvo el JSON del proyecto, que se pasa como puntero más longitud y no necesita terminador.
* Las funciones que devuelven `int` dan **0 si tienen éxito y un valor negativo si fallan**. `-1` es un error general; `-2` significa además «tu búfer de salida es demasiado pequeño» y solo se produce en las dos funciones de renderizado.
* Las funciones que rellenan un búfer de caracteres reciben `(char *out, size_t cap)` y devuelven la **longitud completa** del texto (sin contar el NUL). El texto se trunca a `cap - 1` bytes y se termina con NUL cuando `cap > 0`. Pasa `out == NULL` (con cualquier `cap`) para pedir solo la longitud. Un retorno negativo es un error (índice incorrecto, nombre desconocido).
* Los argumentos de coma flotante deben ser finitos; los fotogramas fuera de la línea de tiempo se limitan; `scale` debe estar entre 0.05 y 4.
* Cualquier argumento de tipo handle puede ser `NULL` en los getters (devuelven 0 y establecen el texto de error) y es un error en el resto.
* El RGBA de salida usa **alfa directo** (no premultiplicado), 8 bits por canal, orden por filas, fila superior primero, `width * height * 4` bytes.
* Las transformaciones de mundo de los huesos son seis floats por hueso `[a b c d tx ty]`; un punto local del hueso `(x, y)` se transforma en `(a*x + c*y + tx, b*x + d*y + ty)`.

## Referencia de funciones

53 funciones. `E` = `OsaturaEngine *`, `P` = `OsaturaPlayer *`.

### Biblioteca

| Función | Descripción |
|---|---|
| `const char *osatura_version(void)` | cadena de versión del crate `osatura-capi` (actualmente `"0.1.0"`); estática, nunca se libera |
| `uint32_t osatura_abi_version(void)` | versión de ABI de la biblioteca cargada; compárala con `OSATURA_ABI_VERSION` de la cabecera con la que compilaste |
| `const char *osatura_last_error(void)` | texto del último error **en este hilo**; cadena vacía si todavía no hubo ninguno; consulta [Tratamiento de errores](#tratamiento-de-errores) |

### Motor: abrir, consultar, renderizar

| Función | Descripción |
|---|---|
| `E *osatura_open(const char *json, size_t len)` | analiza y valida un proyecto (UTF-8, `len` bytes, de 1 a 256 MiB); devuelve `NULL` si hay error. La biblioteca copia lo que necesita; puedes liberar `json` de inmediato. Los archivos de imagen relativos del proyecto se buscan en el directorio actual del proceso, las rutas absolutas y `..` se rechazan (consulta [SECURITY.md](SECURITY.md)); las imágenes data URI funcionan siempre. |
| `void osatura_close(E *)` | libera el motor; se permite `NULL`; llámala exactamente una vez por handle |
| `uint32_t osatura_frame_count(const E *)` | número de fotogramas |
| `float osatura_fps(const E *)` | fotogramas por segundo |
| `uint32_t osatura_width(const E *)`, `osatura_height(const E *)` | tamaño del escenario en unidades del escenario, redondeado a un entero |
| `uint32_t osatura_bone_count(const E *)` | número de huesos |
| `int osatura_bone_name(const E *, uint32_t index, char *out, size_t cap)` | nombre del hueso `index` (orden del rig); longitud, o -1 si el índice es incorrecto |
| `int osatura_bone_world(const E *, float frame, float *out, size_t cap_floats)` | matrices de mundo de todos los huesos; `cap_floats` debe ser al menos `6 * bone_count`, de lo contrario -1 |
| `int osatura_render_rgba(const E *, float frame, float scale, uint8_t *out, size_t cap, uint32_t *w, uint32_t *h)` | renderiza un fotograma; tamaño de salida `round(width*scale) x round(height*scale)`; devuelve 0, **-2 si `cap` es menor que `w*h*4`** (y se establecen `*w` y `*h`, de modo que puedes reservar memoria y volver a llamar; pasa `out == NULL` para consultarlo), -1 en otros errores. `w` y `h` pueden ser `NULL`. |

### Player: máquinas de estados

Un proyecto puede contener `machines` (máquinas de estados de runtime) y `bindings` (valores externos que controlan propiedades). Un player ejecuta una máquina: establece entradas por nombre, avanza el tiempo, lee estados y eventos, renderiza la pose actual. Consulta [API.md](API.md) para la semántica (son los métodos de `Player`).

| Función | Descripción |
|---|---|
| `uint32_t osatura_machine_count(const E *)` | máquinas del proyecto |
| `int osatura_machine_name(const E *, uint32_t index, char *out, size_t cap)` | nombre de la máquina `index` (reglas del búfer de caracteres) |
| `P *osatura_player_open(const E *, const char *machine)` | inicia la máquina en el tiempo 0; `NULL` si hay error (máquina desconocida, ...). El player posee una **copia** del proyecto: sigue siendo válido tras `osatura_close` del motor. |
| `void osatura_player_close(P *)` | libera el player; se permite `NULL` |
| `int osatura_player_set_bool(P *, const char *name, int value)` | entrada booleana (`value != 0`) |
| `int osatura_player_set_number(P *, const char *name, double value)` | entrada numérica (finita) |
| `int osatura_player_fire(P *, const char *trigger)` | dispara un trigger; lo consume la transición que lo comprueba |
| `int osatura_player_set_value(P *, const char *path, double value)` | valor externo para los data bindings, p. ej. `"player.health"`; también controla una región de desplazamiento cuyo `input` es esta ruta (píxeles a lo largo de su eje principal) |
| `int osatura_player_advance(P *, double dt)` | avanza `dt` segundos, `0..=10`; determinista (los pasos largos se dividen en fragmentos de como máximo 1/60 s) |
| `int osatura_player_state(P *, const char *layer, char *out, size_t cap)` | nombre del estado actual de una capa de máquina; longitud, o -1 si la capa es desconocida |
| `int osatura_player_events(P *, char *out, size_t cap)` | eventos pendientes como array JSON `[{"time":..,"layer":..,"kind":"entered"\|"exited","state":..}]`. Devuelve la longitud completa. **Los eventos se consumen solo si el texto completo cupo (`cap` > longitud)**; de lo contrario se conservan, y vuelves a llamar con un búfer mayor. `out == NULL` consulta el tamaño y no consume nada. |
| `int osatura_player_render_rgba(P *, float scale, uint8_t *out, size_t cap, uint32_t *w, uint32_t *h)` | mismo contrato que `osatura_render_rgba`, para la pose actual; el tamaño sigue al lienzo del player |

### Player: maquetación, desplazamiento, nivel de detalle

| Función | Descripción |
|---|---|
| `int osatura_player_set_canvas(P *, float width, float height)` | maquetación adaptable: tamaño del lienzo en unidades del escenario (finito, positivo, como máximo 32768); los contenedores se vuelven a resolver, los renderizados adoptan este tamaño |
| `int osatura_player_canvas_size(P *, float *w, float *h)` | tamaño actual del lienzo; cualquiera de los punteros puede ser `NULL` |
| `uint32_t osatura_player_scroll_count(P *)` | regiones de desplazamiento del proyecto |
| `int osatura_player_scroll_id(P *, uint32_t index, char *out, size_t cap)` | id de la región `index` (reglas del búfer de caracteres) |
| `int osatura_player_scroll_drag(P *, const char *id, float x, float y)` | desplazamiento bruto del puntero en píxeles (contenido movido hacia arriba / la izquierda); más allá de los extremos se limita o rebota elásticamente |
| `int osatura_player_scroll_release(P *, const char *id, float vx, float vy)` | velocidad de suelta en px/s; `osatura_player_advance` entonces lanza, ajusta y recupera con muelle |
| `int osatura_player_scroll_offset(P *, const char *id, float *x, float *y)` | desplazamiento mostrado; -1 si el id es desconocido; los punteros pueden ser `NULL` |
| `int osatura_player_scroll_max(P *, const char *id, float *x, float *y)` | mayor desplazamiento útil al tamaño de lienzo actual |
| `int osatura_player_scroll_settled(P *, const char *id)` | 1 en reposo, 0 en movimiento, -1 id desconocido |
| `int osatura_player_set_screen(P *, float width_px, float height_px, float max_error_px)` | indica al player el tamaño con que se muestra; renderiza el nivel de detalle precalculado más grueso cuya cota de error declarada se mantiene dentro de `max_error_px` píxeles. Los niveles se construyen en el primer uso (esta llamada puede tardar un momento en un proyecto grande, mucho en una compilación de depuración). |
| `int osatura_player_lod_level(P *)` | nivel en uso (0 = original, también sin tamaño de pantalla); negativo para un player `NULL` |

### Edición y exportación a memoria (aditivo, fachada 1.4)

Las ediciones se validan y se pueden deshacer; una edición fallida no cambia nada, devuelve -1 y establece el texto de error. Reciben un `E *` no const (un hilo a la vez, como siempre). Los colores son `#rgb`, `#rrggbb` o `#rrggbbaa`. `parent` puede ser `NULL`.

| Función | Descripción |
|---|---|
| `int osatura_add_rect_layer(E *, name, x, y, w, h, fill, parent)` | añade al final una capa de rectángulo relleno; `parent` nombra una capa de grupo de nivel superior, `NULL` añade en el nivel superior |
| `int osatura_add_ellipse_layer(E *, name, cx, cy, rx, ry, fill, parent)` | lo mismo para una elipse |
| `int osatura_add_group_layer(E *, name)` | añade al final una capa de grupo vacía |
| `int osatura_add_bone(E *, name, parent, length, x, y, angle)` | añade al final un hueso; el padre debe existir (`NULL` = raíz); ángulo en grados |
| `int osatura_set_key(E *, target, prop, frame, value, interpolation)` | una clave en `target.prop`; interpolación 0 lineal, 1 step, 2 ease-in, 3 ease-out, 4 ease-in-out, 5 smooth |
| `int osatura_bake_physics(E *, chain, preset, overwrite)` | simula una cadena de huesos separados por comas (raíz primero) con `hair`, `tail`, `rope`, `antenna` o `cape`; devuelve los canales escritos (0 cuando se conservaron huesos que ya tenían claves) |
| `int osatura_undo(E *)`, `int osatura_redo(E *)` | 1 hecho, 0 nada que hacer, negativo si hay error |
| `uint32_t osatura_layer_count(const E *)`, `int osatura_layer_name(const E *, index, out, cap)` | rutas de capa (`group/child`), reglas del búfer de caracteres |
| `B *osatura_export(const E *, format, scale)` | exportación de toda la línea de tiempo, `B` = `OsaturaBlob *`; formato 0 GIF, 1 glTF binario, 2 Lottie JSON, 3 reproductor HTML, 4 APNG; `scale` de 0.05 a 4 para GIF y APNG; `NULL` si hay error |
| `B *osatura_render_png(const E *, frame, scale)` | un fotograma como PNG |
| `B *osatura_render_exr(const E *, frame, scale, float32)` | un fotograma como OpenEXR (lineal, premultiplicado; un `float32` distinto de cero almacena FLOAT, de lo contrario HALF) |
| `B *osatura_to_json(const E *)` | el proyecto (editado) como JSON UTF-8, sin terminador |
| `const uint8_t *osatura_blob_data(const B *)`, `size_t osatura_blob_size(const B *)` | los bytes de un blob, válidos hasta que se libera; `NULL` / 0 para un blob `NULL` |
| `void osatura_blob_free(B *)` | libera un blob exactamente una vez; se permite `NULL` |

## Propiedad y tiempo de vida

* **Las cadenas y los arrays se copian en búferes que tú proporcionas; los resultados de exportación son blobs que tú liberas.** Todo resultado de tipo cadena o array se copia en un búfer que proporcionas *tú*. La única memoria propiedad de la biblioteca es un `OsaturaBlob` devuelto por `osatura_export`, `osatura_render_png`, `osatura_render_exr` o `osatura_to_json`: léelo con `osatura_blob_data` / `osatura_blob_size` y libéralo una vez con `osatura_blob_free`. No existe un `osatura_free` genérico. Los únicos punteros que devuelve la biblioteca son `osatura_version()` (estático) y `osatura_last_error()` (véase más abajo), y los dos handles opacos.
* **Handles.** `osatura_open` devuelve un handle de motor; `osatura_player_open` devuelve un handle de player. Cada uno debe cerrarse exactamente una vez con su propio `_close`; cerrar `NULL` es válido; usar un handle después de cerrarlo, o cerrarlo dos veces, es comportamiento indefinido.
* **El motor y los players son independientes.** Un player posee una copia del proyecto, así que cerrar primero el motor está permitido y es habitual (el ejemplo lo hace). Se pueden abrir varios players desde un mismo motor.
* **Tus búferes.** Los punteros de entrada solo necesitan ser válidos durante la llamada; la biblioteca no conserva ningún puntero a ellos. Los búferes de salida deben tener realmente la capacidad que pasas; la biblioteca escribe como máximo `cap` bytes (cadenas) o `cap_floats` floats.
* **`osatura_last_error()`** devuelve un puntero a una cadena local al hilo. Sigue siendo válido hasta la siguiente llamada en el mismo hilo que *falle* (esa llamada la sobrescribe) o hasta que el hilo termine. Cópiala si la necesitas durante más tiempo. No la liberes.

## Tratamiento de errores

1. Comprueba siempre el valor de retorno (`NULL` en las dos funciones `_open` y en `osatura_player_open`, un `int` negativo en las demás, `0` de un getter al que se pasó un handle `NULL`).
2. Después lee `osatura_last_error()` en el mismo hilo para conocer el motivo: por ejemplo `invalid project JSON: ...`, `output buffer needs 1228800 bytes`, `invalid argument: input 'excited' is not a number`.
3. **El texto de error solo lo establecen los fallos y nunca lo borra un éxito.** Tras una llamada posterior con éxito sigue conteniendo el mensaje antiguo. No uses nunca «la cadena no está vacía» como prueba de fallo.
4. Los textos de error proceden de la fachada de Rust (`Display` de `ApiError`); son para personas, no un contrato. La ABI de C no expone el *tipo* de error.
5. Un pánico dentro del motor se captura y se notifica como `internal error (panic) inside osatura` con el retorno de fallo habitual; el handle sigue siendo utilizable en principio, pero trátalo como un error e infórmalo.

## Hilos

* Cada handle puede usarse desde **un hilo a la vez** (el contrato de la cabecera). Puedes mover un handle entre hilos siempre que las llamadas nunca se solapen; handles distintos pueden ejecutarse en paralelo en hilos distintos (los tipos de Rust subyacentes son `Send + Sync`, pero el contrato de C se mantiene conservador).
* La cadena del último error es **por hilo**: léela en el hilo que hizo la llamada fallida.
* No hay callbacks y la biblioteca no inicia hilos propios.

## Ejemplo completo en C

Carga un proyecto, imprime datos y posiciones de huesos, renderiza un fotograma a un archivo PPM (pidiendo primero el tamaño con `NULL` y reservando después la memoria exacta), ejecuta la primera máquina de estados del proyecto, lee sus eventos con el patrón de consulta de tamaño y muestra el camino de error. Verificado con gcc, también bajo AddressSanitizer/UBSan (sin hallazgos), tanto con la biblioteca compartida como con la estática.

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

Compilado y ejecutado con el proyecto de `osatura demo` ampliado con una máquina de estados (`hero`, entrada `excited`, estados `idle` y `wave`):

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

El repositorio incluye además `osatura-capi/tests/smoke.c` (abrir, renderizar, suma de comprobación) y las pruebas de Rust `osatura-capi/tests/abi.rs` (punteros nulos, índices incorrectos, tamaños de búfer, seguridad frente a pánicos).

## Desplazamiento y nivel de detalle en C

Un proyecto con un contenedor de maquetación y una región de desplazamiento (el proyecto de prueba de desplazamiento de las pruebas de Rust: diez filas de 20 unidades en una columna, un viewport anclado a los bordes izquierdo, superior e inferior). Verificado:

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

Salida: `scroll 'list': max=(0,80) dragged offset=(0,80) settled=1` (arrastrar hasta 500 se limita al máximo cuando la elasticidad está desactivada), luego `set_screen rc=0, lod level=7` (aceptando hasta 4 px de error, el nivel precalculado más grueso cumple en este proyecto diminuto; el número depende del proyecto) y `render size 100x120`. Observa la variable `rc` explícita en el código: el orden en que C evalúa los argumentos de una función no está especificado, así que nunca pongas en un mismo `printf` la llamada que cambia el player y la que lo lee.

## Ejemplo en Python (ctypes)

Solo biblioteca estándar. Declara `argtypes` y `restype` para cada función que llames: ctypes asume `int` en caso contrario y trunca los punteros de 64 bits. Verificado en Python 3 con la biblioteca de depuración; su salida PPM es idéntica byte a byte a la del ejemplo en C.

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

## Otros lenguajes

Cualquier lenguaje que pueda llamar a una función de C funciona de la misma manera: declara las 53 funciones, guarda los punteros opacos como `void *` y sigue las convenciones de búfer anteriores. Notas específicas de anfitriones habituales (no probadas aquí, solo consecuencias del contrato anterior): en C#/.NET usa `[DllImport]` con `CallingConvention.Cdecl` y convierte las cadenas en arrays de bytes UTF-8; en Go con cgo, mantén los punteros de Go fuera de las llamadas salvo para búferes que vivan durante la llamada; en Rust, prefiere usar directamente la fachada en lugar de esta ABI.

## Versionado

* `OSATURA_ABI_VERSION` / `osatura_abi_version()` es actualmente **1**. Solo cambia cuando se elimina una función existente o cambia su significado. Todas las adiciones hasta ahora (las funciones del player en la fachada 1.1, la maquetación, el desplazamiento y el nivel de detalle en la 1.2) fueron aditivas y la dejaron en 1, de modo que un programa compilado con una cabecera antigua sigue funcionando con una biblioteca más nueva, y una cabecera más nueva con una biblioteca más antigua falla en el enlazado o la carga por el símbolo ausente. Comprueba `osatura_abi_version()` al inicio y compárala con la macro.
* `osatura_version()` es la versión del crate `osatura-capi` (0.1.0), no la versión de la fachada. No hay ninguna función de C que informe de la versión de la fachada (`API_VERSION`).
* La fachada 1.4 añadió las funciones básicas de edición y de exportación a memoria descritas arriba (aditivo, la versión de ABI sigue siendo 1). No expuestos en la ABI de C: renderizado SVG, eliminación de claves, control de versiones, CI de animación, trazado, mocap, fuzzing, exportación con tamaño objetivo, scripts. Están disponibles en Rust y a través de la CLI.

## Particularidades conocidas

Los comentarios de la cabecera se corrigieron para que coincidan con el comportamiento (antes discrepaban); los puntos que conviene conocer:

* `osatura_render_rgba` escribe `*w` / `*h` en caso de éxito y con -2 (búfer demasiado pequeño), y también cuando se produce un error de renderizado posterior después de que `frame` y `scale` fueran aceptados. **No** los escribe cuando el motor es nulo o cuando `frame` / `scale` se rechazan (-1). Prueba de regresión: `documented_size_outputs_and_last_error_semantics` en `osatura-capi/tests/abi.rs`.
* El texto del último error es local al hilo, solo lo sobrescribe la siguiente llamada que *falle* y nunca lo borra un éxito. Cópialo si lo necesitas durante más tiempo.
* `osatura_width` / `osatura_height` redondean a un entero el tamaño del escenario en coma flotante; el tamaño real del renderizado es `round(size * scale)`, así que usa los `w` / `h` devueltos por la llamada de renderizado.
* La nota de la cabecera sobre las regiones de desplazamiento que siguen a `osatura_player_set_value` está situada entre las declaraciones de desplazamiento y de nivel de detalle; describe `osatura_player_set_value`.
