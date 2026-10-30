# CI de animación (`osatura ci`)

Trata una animación como código comprobable: un manifiesto enumera las comprobaciones, un trabajo de CI las ejecuta, y un cambio
visual que falla viene con un informe que muestra qué se ha movido. Todo es local, funciona sin conexión y es
determinista; las métricas son las clásicas (CIEDE2000, SSIM, MSE/PSNR, un hash DCT).

```
osatura ci init  <project.json> [--out osatura-ci.json] [--no-goldens]
osatura ci run   [manifest] [--update] [--bless] [--prune] [--only a,b]
                  [--junit out.xml] [--html out.html] [--json out.json] [--artifacts dir] [--quiet]
osatura ci bless [manifest] [--only a,b] [--prune]
osatura ci diff  a.png b.png [--metric ssim|de2000|de76|mse] [--tol T] [--pixel-de D] [--aa N]
                  [--ignore x,y,w,h;x,y,w,h] [--out heat.png] [--side s.png] [--overlay o.png] [--flicker f.gif]
```

Códigos de salida: **0** todas las comprobaciones han pasado (o solo hay info / actualizado / bendecido), **1** al menos una comprobación
ha fallado (o `ci diff` encontró una diferencia), **2** algo no se pudo evaluar (proyecto ausente,
manifiesto incorrecto, golden ilegible o manipulado, opción desconocida).

## Inicio rápido

```
osatura ci init animation.json        # writes osatura-ci.json and goldens/ next to the project
osatura ci run --html report.html     # verify; exit 1 and a report on regressions
osatura ci bless                      # after reviewing the report: accept the new look
git add osatura-ci.json goldens       # commit the goldens with the change that caused them
```

`init` calibra el manifiesto con el proyecto tal como está ahora: unos seis fotogramas equiespaciados a una
escala de proxy (lado más largo de 480 px como máximo), una comprobación con el renderizador de coma flotante en dos de ellos, determinismo,
validación, un techo de tasa de fallos del pose fuzzing (la tasa medida más 2 puntos; el rig de la demo falla en la mayoría de las
muestras de rango completo, así que su techo es del 100 % y la comprobación no protege nada hasta que lo bajes),
límites con un 50 % de margen, un presupuesto de tamaño de GIF (+50 %), una comprobación de la costura del bucle (solo cuando el proyecto
hace bucle limpiamente hoy) y un presupuesto informativo de velocidad de renderizado (4 x la mediana medida, que
depende de la máquina y de si la compilación es de depuración o de release).

## Manifiesto (`osatura-ci.json`)

```json
{
  "schema": "osatura-ci/1",
  "project": "animation.json",
  "goldens": "goldens",
  "defaults": { "max_de": 6.0 },
  "checks": [
    { "kind": "validate", "name": "validate" },
    { "kind": "render", "name": "frames", "frames": [0, 12, 24, 47], "scale": 0.5,
      "thresholds": { "min_ssim": 0.995 }, "ignore": [[500, 0, 140, 40]] },
    { "kind": "render", "name": "tablet", "step": 12, "size": "1024x768", "scale": 0.5 },
    { "kind": "render", "name": "svg-text", "frames": [0], "renderer": "svg" },
    { "kind": "determinism", "name": "determinism", "frames": [0, 47] },
    { "kind": "fuzz", "name": "pose-fuzz", "samples": 200, "max_fail_rate": 0.05 },
    { "kind": "limits", "name": "limits", "max_layers": 200, "max_file_bytes": 2000000 },
    { "kind": "export-size", "name": "gif", "format": "gif", "max_bytes": 800000, "scale": 0.5 },
    { "kind": "loop", "name": "loop-seam" },
    { "kind": "perf", "name": "speed", "budget_ms": 40, "tolerance": 0.5, "enforce": false }
  ]
}
```

Las rutas son relativas al manifiesto. Los campos y tipos desconocidos son errores; todos los problemas de un manifiesto
se notifican juntos (los nombres deben tener de 1 a 64 caracteres entre letras, dígitos, `-`, `_`, `.`; los nombres son
únicos; cada comprobación necesita un proyecto; se comprueban los rangos).

| kind | qué comprueba | falla cuando |
| --- | --- | --- |
| `render` | fotogramas (lista `frames`, o `step`; por defecto primero/central/último), `scale`, `size` (los containers de diseño se resuelven de nuevo), `renderer`: `raster` (tiny-skia), `float` (rasterizador f32, cuantizado a 8 bits) o `svg` (comparación exacta de texto) | la imagen difiere del golden más allá de los umbrales, no existe golden o el tamaño ha cambiado |
| `validate` | el proyecto compila; hallazgos del linter con la severidad `lint` (`error` por defecto, `warn`, `info`, `off`) | cualquier hallazgo de ese tipo o un error de compilación |
| `fuzz` | el pose fuzzer (`samples`, `seed`, `frame`) | tasa de fallos por encima de `max_fail_rate` (por defecto 0). La comprobación **bloquea por defecto** (`gate` sin definir significa `true`); solo se informa de ella, con estado `info` y sin fallar nunca, cuando `"gate": false` o `max_fail_rate` es 1 o más (un límite que no puede fallar nunca se muestra como aprobado). El comando `osatura fuzz` es una herramienta aparte que solo informa, salvo que se le dé `--gate` o `--max-fail-rate` |
| `limits` | `max_layers`, `max_bones`, `max_channels`, `max_frames`, `max_canvas_pixels`, `max_file_bytes` | un recuento supera su límite |
| `export-size` | bytes de `gif`, `apng`, `lottie`, `html` o `svg-frames` (exportadores existentes) | mayor que `max_bytes` |
| `determinism` | cada fotograma se renderiza dos veces con el mismo motor y con un motor recién cargado; el SHA-256 de los píxeles (o del texto SVG) debe coincidir; `renderers` por defecto raster, float, svg | cualquier resumen difiere |
| `loop` | costura entre el último y el primer fotograma frente al cambio medio entre fotogramas (diferencia media absoluta de canal en porcentaje); `max_seam_ratio` por defecto 2, `max_seam_mad` opcional | costura mayor de lo permitido; 600 fotogramas como máximo |
| `perf` | mediana de ms por fotograma, `budget_ms` con `tolerance` relativa (por defecto 0.5) | solo cuando `enforce` es true; en caso contrario estado `info` |

### Umbrales de renderizado

Se definen en `defaults` (todas las comprobaciones de render) y se sobrescriben en cada comprobación en `thresholds`.

| clave | por defecto | significado |
| --- | --- | --- |
| `metric` | `de2000` | `de2000` (CIEDE2000) o `de76` (CIE76, euclídea en Lab) |
| `pixel_de` | 1.0 | un píxel cuenta como diferente por encima de esta diferencia de color |
| `max_de` | 8.0 | mayor diferencia de color permitida entre los píxeles contados |
| `min_ssim` | 0.99 | menor SSIM medio permitido |
| `max_diff_fraction` | 0.001 | mayor fracción permitida de píxeles contados (0.1 %) |
| `aa_isolated` | 2 | tolerancia de antialiasing: los grupos contados de como máximo este número de píxeles conectados en 8 direcciones, todos situados en un borde, se perdonan (0 = desactivado) |
| `aa_edge_de` | 0 (desactivado) | los píxeles de borde que difieren como máximo en esto se perdonan (más laxo que `pixel_de` solo en los bordes) |
| `max_phash_distance` | no se comprueba | mayor distancia de Hamming de los hashes DCT de 64 bits |

Un fotograma cuyos píxeles son idénticos byte a byte al golden pasa sin ninguna métrica. En caso contrario, el
fotograma se compara píxel a píxel; cada píxel se compone sobre blanco y sobre negro y se usa la mayor
diferencia, de modo que un cambio solo de alfa es visible. Un píxel es un píxel de borde cuando dos vecinos
opuestos (limitados en el borde de la imagen) en cualquiera de las dos imágenes difieren en más de 40 en algún canal; el valor del propio
píxel se ignora, así que un píxel aislado en una zona plana es una diferencia real y el contorno de una forma
no lo es. Los rectángulos `ignore` son `[x, y, w, h]` en unidades del escenario; se igualan en ambas imágenes antes de
cualquier métrica, por lo que tampoco cuentan en SSIM, MSE ni en el hash.

A modo de orientación, en el proyecto de demostración el rasterizador de CPU y el rasterizador de coma flotante (dos caminos de código
distintos para la misma escena) difieren en SSIM 0.9997, 0.83 % de los píxeles por encima de la diferencia de color 1 (386 más
perdonados como antialiasing), con una diferencia de color máxima de 11.3 en bordes de formas. Un movimiento de 40 unidades de la capa
del sol da SSIM 0.993, 6.4 % de los píxeles, diferencia de color máxima de 19.6 en una región.

### Goldens: `--update` frente a `bless`

Los goldens viven en `goldens/` como archivos PNG con el nombre de los primeros 24 dígitos hexadecimales de un SHA-256 del tamaño y
los píxeles (los fotogramas idénticos comparten archivo), más `goldens/index.json`, que asigna comprobación y fotograma a archivo.
Los goldens SVG son archivos `.svg` con el nombre formado del mismo modo. Un golden se comprueba contra su nombre cuando se
lee, así que un archivo editado a mano o dañado es un error (salida 2), no un aprobado silencioso.

* `ci run --update` reescribe todos los goldens de las comprobaciones de render seleccionadas a partir del renderizado actual y
  olvida los fotogramas que ya no figuran en la lista. Úsalo cuando haya cambiado un manifiesto o tras un cambio
  global intencionado.
* `ci bless` (o `ci run --bless`) acepta solo los fotogramas que fallan actualmente o que no tienen golden y
  deja intactos los que pasan, de modo que el ruido de tolerancia no remueva los archivos. El informe los marca
  como `blessed`.
* `--prune` elimina los archivos golden a los que nada hace referencia. Confirma `goldens/` junto con el cambio que lo provocó.

### Informes

`--json` es el resultado completo (esquema `osatura-ci-report/1`; métricas, regiones, umbrales usados).
`--junit` es XML JUnit con un caso de prueba por comprobación. `--html` es una página autocontenida (sin recursos
externos): resumen, tabla de comprobaciones y, para cada fotograma que falla, el golden, el renderizado actual y un
mapa de diferencias como URI de datos PNG en línea, un botón de intercambio para alternar entre el golden y el actual, y
la lista de regiones que difieren. `--artifacts dir` escribe además `dir/<check>/fNNNNN-{golden,actual,diff}.png`
para los fotogramas que fallan. El mapa de diferencias muestra el golden atenuado, las diferencias contadas en una rampa de amarillo a
rojo oscuro según la diferencia de color, el antialiasing perdonado en azul claro, las zonas ignoradas rayadas en gris
y los recuadros de región en magenta. `ci diff` también puede escribir una tira lado a lado, una superposición rojo/cian y un
GIF de parpadeo de dos fotogramas; usa `--flicker` con un par golden/actual para obtener un GIF de antes/después.

## GitHub Actions (ejemplo, no ejecutado)

Solo documentación: este flujo de trabajo se escribió para que coincida con la CLI, pero **no** se ha ejecutado en un servicio de
CI real. Ajusta la instalación de Rust y las rutas a tu repositorio.

```yaml
name: animation-ci
on:
  pull_request:
  push:
    branches: [main]
jobs:
  animation:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        run: rustup toolchain install stable --profile minimal
      - name: Build osatura
        run: cargo build --release -p osatura-cli
      - name: Animation checks
        run: |
          ./target/release/osatura ci run animation/osatura-ci.json \
            --junit ci-out/junit.xml --html ci-out/report.html --json ci-out/report.json \
            --artifacts ci-out/frames
      - name: Upload report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: animation-ci-report
          path: ci-out/
      - name: Publish test results
        if: always()
        uses: mikepenz/action-junit-report@v4   # any JUnit consumer works
        with:
          report_paths: ci-out/junit.xml
```

Los códigos de salida 1 (regresión visual o un presupuesto) y 2 (no se pudo evaluar) hacen fallar el trabajo; el informe
se sube en ambos casos. Para aceptar un cambio intencionado, ejecuta `osatura ci bless` en local, revisa el
diff de `goldens/` y confírmalo.

## Script de shell genérico (ejemplo, no ejecutado)

```sh
#!/bin/sh
# ci.sh - run animation checks; works in any CI that can run a shell and keep files.
set -u
MANIFEST="${1:-animation/osatura-ci.json}"
OUT="${OUT:-ci-out}"
mkdir -p "$OUT"

osatura ci run "$MANIFEST" \
  --junit "$OUT/junit.xml" --html "$OUT/report.html" --json "$OUT/report.json" \
  --artifacts "$OUT/frames"
code=$?

case $code in
  0) echo "animation checks passed" ;;
  1) echo "animation regression: open $OUT/report.html" ;;
  *) echo "animation CI could not run (exit $code); see the log above" ;;
esac
exit $code
```

## Determinismo y otras máquinas

Lo que se verificó (una máquina Linux x86-64, el rasterizador de CPU `tiny-skia` y el rasterizador f32):

* renderizar el mismo fotograma dos veces en un proceso, y desde un motor recién cargado, da bytes
  idénticos (la comprobación `determinism`; raster, float y texto SVG);
* los goldens escritos por un proceso coincidieron byte a byte con los de procesos posteriores (`init` y luego `run`,
  `bless` y luego `run`), con ambos renderizadores;
* comprobaciones independientes de las métricas: los 24 pares CIEDE2000 de Sharma, Wu y Dalal (2005) usados en las
  pruebas unitarias se reproducen con una precisión de 5e-5; en los fotogramas de la demo, SSIM (una vez corregido por la región de borde que
  scikit-image recorta), el mayor valor CIEDE2000 y el número de píxeles que difieren coinciden
  con scikit-image 0.26 (0.99330 frente a 0.99330 tras la corrección del borde; 19.648 frente a 19.648; 11088
  frente a 11088 píxeles), y MSE con numpy.

Lo que **no** se verificó: otros sistemas operativos, CPU, compiladores o versiones de Rust. Los renderizadores
usan coma flotante (trigonometría, aplanamiento de curvas, acumulación de cobertura), por lo que una libm distinta, una
contracción FMA o un camino SIMD diferente pueden mover un píxel de borde con antialiasing uno o dos niveles. Por tanto, los hashes exactos
solo sirven con una compilación de la herramienta en un tipo de máquina; entre máquinas confía en las
tolerancias (`pixel_de`, `aa_isolated`, `max_diff_fraction`) y mantén los goldens producidos por la
misma imagen de CI que los verifica. El renderizador de GPU no forma parte de este arnés.

## Límites

* No hay ningún rasterizador de SVG en el workspace, así que una comprobación de render no puede rasterizar la salida SVG y
  comparar píxeles; el renderizador `svg` compara el texto SVG exactamente (cualquier reordenación de atributos es un
  fallo).
* Ninguna comprobación de CI renderiza las superposiciones de huesos; el audio no se compara.
* La fusión de regiones y la regla de antialiasing son heurísticas (prueba de borde con un paso fijo de 40, límite de tamaño
  de grupo); un desplazamiento subpíxel de un borde largo forma un grupo largo y delgado que se notifica, no se perdona.
* Los umbrales perceptuales son valores por defecto, no están calibrados con el criterio de animadores.
* La comprobación perf mide tiempo de reloj en una máquina compartida; mantenla informativa salvo que el
  runner sea dedicado.
* `ci init` renderiza, hace fuzzing y exporta una vez para calibrar, lo cual es lento en compilaciones de depuración.
* No hay integración con pull requests (ni comentario de bot, ni subida automática de GIF de antes/después): las piezas
  (`--html`, `--artifacts`, `ci diff --flicker`) están, pero el pegamento con un servicio de alojamiento no.
