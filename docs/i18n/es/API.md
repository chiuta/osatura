# API de Rust de OSATURA y política de estabilidad

*Versión de la fachada 1.4.0 (`osatura_core::api::API_VERSION`). Destinatarios: desarrolladores de Rust que integran el motor y mantenedores que modifican la fachada. Licencia de la documentación CC-BY-4.0; ejemplos de código MPL-2.0.*

Esta es la referencia de la **API de Rust admitida** de `osatura-core`: el módulo de fachada `osatura_core::api` y su importación de una sola línea `osatura_core::prelude`. Enumera todos los elementos públicos, las reglas que mantienen estable la API y ejemplos probados. Todos los ejemplos de este archivo se compilan y ejecutan con las fuentes actuales antes de publicarlo (consulta [Cómo se comprueban los ejemplos](#cómo-se-comprueban-los-ejemplos)).

Contenido: [Qué cubre](#qué-cubre) - [Obtención](#obtención-del-crate) - [Política de estabilidad](#política-de-estabilidad) - [Reglas de semver](#reglas-de-semver) - [Feature flags](#feature-flags) - [Seguridad entre hilos](#seguridad-entre-hilos) - [Determinismo](#determinismo) - [Límites](#límites-y-rangos-de-argumentos) - [Errores](#errores) - [Referencia: `Animation`](#referencia-animation) - [Referencia: `Player`](#referencia-player) - [Referencia: funciones libres](#referencia-funciones-libres-herramientas-control-de-versiones-ci) - [Referencia: tipos](#referencia-tipos) - [Ejemplos](#ejemplos) - [Otros lenguajes](#otros-lenguajes) - [Apéndice: JSON de máquinas y de bindings](#apéndice-json-de-máquinas-y-json-de-bindings)

## Qué cubre

Solo `osatura_core::api` y `osatura_core::prelude` son la API pública de Rust admitida. La fachada consta de

* `Animation`: un proyecto validado más su historial de deshacer (cargar, consultar, renderizar, exportar, editar, herramientas);
* `Player`: una máquina de estados en ejecución con maquetación, desplazamiento y nivel de detalle (1.1, 1.2);
* funciones libres para el control de versiones y la CI de animación (1.2 lectura, 1.3 escritura) y el trazado de mapas de bits;
* tipos de valor simples (`AnimationInfo`, `RenderSettings`, `Image`, `LodLevel`, `SizeFit`, `MocapReport`, `FuzzSummary`, `VcsCommit`, `VcsConflict`, `CiOutcome`, `CiInit`, ...) y las enumeraciones `Interpolation`, `ExportFormat`, `FitFormat`, `CiMode`, `VcsMergeOutcome`, `VcsChoice`;
* el tipo de error `ApiError` (implementa `std::error::Error` y `Display`).

La ABI de C (`osatura-capi`, cabecera `osatura.h`) y el módulo WebAssembly (`osatura-wasm`) están escritos únicamente sobre esta fachada y tienen sus propias versiones: [C_ABI.md](C_ABI.md), [WASM.md](WASM.md).

### Qué NO cubre

Todos los demás módulos de `osatura-core` (`engine`, `project`, `edit`, `svg`, `raster`, `physics`, `vcs`, `ci`, ...) y las reexportaciones de la raíz (`Engine`, `Project`, ...). Son `pub` y `#[doc(hidden)]` únicamente porque la herramienta de línea de comandos, el studio, las pruebas de la ABI de C y las pruebas unitarias los utilizan; pueden cambiar o desaparecer en cualquier versión, y los textos de sus `Err(String)` tampoco son un contrato. Si necesitas algo que solo es accesible allí, solicita que se añada a la fachada (una versión menor) en lugar de depender de los elementos internos.

El formato JSON de proyecto tiene su propio campo de versión `format` (actualmente 1); un proyecto escrito por una compilación más reciente se rechaza con `InvalidJson`. Consulta [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

## Obtención del crate

OSATURA no se publica en ningún registro; dependes de él por ruta (o git) desde tu workspace:

```toml
[dependencies]
osatura-core = { path = "../osatura/osatura-core", default-features = false }
```

`default-features = false` elimina el motor de scripting Rhai (consulta [Feature flags](#feature-flags)); la fachada no lo necesita. Después, `use osatura_core::prelude::*;`. La versión del crate (`0.1.0`) y la versión de la fachada (`API_VERSION`) son independientes: la fachada sigue semver, la versión del crate todavía no promete nada. No se declara una versión mínima admitida de Rust; el workspace se desarrolla y se prueba con Rust 1.97 (edición 2021).

## Política de estabilidad

1. **El archivo de superficie.** `osatura-core/src/api_surface.txt` enumera todos los elementos públicos de `src/api/` (tipos, campos, variantes, constantes y firmas de métodos, uno por línea). La prueba unitaria `api::tests::api_surface_snapshot` reconstruye esa lista a partir de `api/` y falla cuando difiere del archivo incluido en el repositorio. Por tanto, todo cambio de la superficie es una edición deliberada y revisada de ambos archivos.
2. **La prueba del prelude.** `osatura-core/tests/prelude_surface.rs` se compila como un crate *externo*. Importa los nombres de la fachada únicamente a través de `osatura_core::prelude` y llama a un subconjunto representativo (cargar, renderizar, editar, exportar, `Player`, las herramientas, los envoltorios de control de versiones y de CI, tratamiento de entradas erróneas). Si un nombre que usa deja de reexportarse desde el prelude, o una firma cambia de forma que esas llamadas lo detecten, esa prueba deja de compilar. También comprueba que `API_VERSION == "1.4.0"`, de modo que un cambio de versión también es allí una edición consciente. No enumera todos los nombres del prelude, así que añadir un nombre a la lista de reexportaciones sigue siendo un paso manual.
3. **Límites de la comprobación.** La instantánea es un análisis de texto de las firmas de una sola línea de `api/`, no una comparación a nivel de compilador como `cargo-public-api`: un cambio semántico que conserve el texto de la firma (por ejemplo un error distinto para la misma entrada) no se detecta, y una firma de varias líneas pasaría inadvertida (por eso el archivo mantiene las firmas en una sola línea). La lista de reexportaciones del prelude (`lib.rs`) la comprueba la prueba externa, no la instantánea.
4. **Oculto no significa admitido.** Los módulos `#[doc(hidden)] pub` existen para los consumidores dentro del workspace. Quedan fuera de la promesa aunque a ti te compilen.
5. **Cómo cambiar la fachada** (mantenedores; también en [DEVELOPMENT.md](DEVELOPMENT.md)):
   * edita `api/` (firmas de una línea, comentario de documentación, una doc-test para las nuevas funciones libres, pruebas unitarias);
   * añade los nuevos nombres públicos a la lista de reexportaciones de `prelude` en `lib.rs` y a `tests/prelude_surface.rs`;
   * incrementa `API_VERSION` (y `API_VERSION_MAJOR` si hay cambios incompatibles) según las reglas siguientes, y la versión comprobada en `prelude_surface.rs`;
   * regenera el archivo de superficie de forma deliberada: `OSATURA_BLESS=1 cargo test -p osatura-core api::` y revisa el diff de `api_surface.txt`;
   * actualiza este archivo, y `osatura-capi` / `osatura-wasm` solo si el nuevo elemento debe cruzar esas fronteras (su versión de ABI cambia por separado, las adiciones la mantienen).

## Reglas de semver

`API_VERSION` es una versión semántica solo de la fachada.

| Cambio | Versión |
|---|---|
| Corrección de errores, sin cambio de firma ni de contrato de comportamiento nuevo | parche (1.4.x) |
| Nueva función, método, tipo, variante de enumeración, campo de estructura o constante | menor (1.x.0) |
| Eliminar o renombrar cualquier cosa; cambiar una firma, un rango de argumento documentado de forma que rechace entradas antes válidas, o el significado de un resultado documentado | mayor (2.0.0) |

Consecuencias para quien llama:

* **Las enumeraciones y estructuras públicas de la fachada son `#[non_exhaustive]`.** Añade una rama comodín cuando hagas `match` sobre `ApiError`, `Interpolation`, `ExportFormat`, `FitFormat`, `CiMode`, `VcsMergeOutcome` o `VcsChoice`; una nueva variante es una versión menor. No puedes construir las estructuras con un literal fuera del crate (e `Image`, `SizeFit` y las estructuras de informe son resultados que solo lees). Los ajustes se construyen con constructores: `RenderSettings::new(scale).with_bones(true)`, `MocapSettings::new().with_smooth(2)`.
* **Las cadenas dentro de los resultados son datos, no contrato.** `VcsConflict::kind` es hoy uno de `"BothChanged"`, `"DeleteEdit"`, `"KeyRetimed"`, `"OrderConflict"`, `"ArrayConflict"` y *puede ganar valores en versiones menores*. Los informes JSON (`FuzzSummary::json`, `CiOutcome::json`) llevan sus propios nombres de esquema (`osatura-fuzz-report` v1, `osatura-ci/1`). Los textos de los mensajes de error son para personas.
* **Estabilidad del comportamiento.** El renderizado y la evaluación son deterministas (véase más abajo), pero la salida en píxeles puede cambiar en una versión menor cuando se corrige un error del rasterizador o de un efecto; los golden registrados por la CI de animación sirven de recordatorio. No calcules hashes de píxeles renderizados entre versiones de OSATURA ni entre máquinas sin una tolerancia.

Historial de versiones de la fachada: 1.0 (cargar, consultar, renderizar, exportar, editar, deshacer) - 1.1 (`Player`, máquinas de estados, data bindings) - 1.2 (maquetación, desplazamiento y nivel de detalle en `Player`; `lod_levels`, `fit_to_size`, `retarget_bvh`, `fuzz`, `from_image`, `trace_image_svg`, `vcs_log`, `ci_run`) - 1.3 (operaciones de escritura de control de versiones y de CI, `DirtyTree` / `MergeInProgress` / `Conflicts`, `CiOutcome::updated` / `blessed`) - 1.4 (creación de capas y huesos, `bake_physics`, exportación APNG y EXR, `vcs_branches` / `vcs_branch_delete` / `vcs_status` / `vcs_diff` / `vcs_show` / `vcs_gc` / `vcs_fsck`).

## Feature flags

`osatura-core` tiene una sola feature de Cargo:

| Feature | Por defecto | Efecto |
|---|---|---|
| `scripting` | activada | Compila el módulo de scripting en sandbox basado en Rhai (`script.rs`), usado únicamente por el comando `script` de la línea de comandos. Incorpora `rhai` y, a través de él, `smartstring` (MPL-2.0+, el único crate de terceros con copyleft débil del árbol). |

La fachada **no** tiene funcionalidad detrás de ninguna feature: todos los elementos documentados aquí existen con `default-features = false`. `osatura-capi`, `osatura-wasm` y `osatura-gpu` dependen del núcleo sin la feature, por lo que no contienen ningún crate de terceros MPL; `osatura-cli --no-default-features` hace lo mismo y entonces informa de que el comando `script` no está disponible. Los back ends de renderizado no son features: la fachada siempre usa el rasterizador de CPU; el renderizador de GPU opcional es un crate aparte (`osatura-gpu`) y una feature de la CLI (`gpu`), y no es accesible a través de esta API.

## Seguridad entre hilos

* `Animation`, `Player`, `ApiError` e `Image` son `Send + Sync` (comprobado en tiempo de compilación para este documento). La fachada no escribe estado global; solo lee dos ajustes de recursos de todo el proceso (el directorio de recursos por defecto y el interruptor de recursos externos) que nada en la fachada modifica.
* Los métodos que reciben `&self` (`info`, `bone_world`, `render_*`, `export`, `lod_levels`, `fit_to_size`, `fuzz`, `player`, `to_json`, ...) pueden ejecutarse simultáneamente en varios hilos sobre la misma `Animation`; los métodos que reciben `&mut self` (`set_key`, `delete_key`, `undo`, `redo`, `retarget_bvh`, cada entrada de `Player` y `advance`) necesitan acceso exclusivo, como es habitual en Rust. El ejemplo `render_in_parallel` de más abajo renderiza cada fotograma en su propio hilo y comprueba que las imágenes son iguales a las de un solo hilo.
* No hay bloqueo interno ni hilo en segundo plano. Un `Player` trabaja sobre su propia copia del proyecto, de modo que editar la `Animation` después nunca afecta a un reproductor en marcha.
* `Animation::from_json` resuelve los archivos de imagen relativos respecto al **directorio actual** del proceso y rechaza las rutas absolutas y `..` (consulta [SECURITY.md](SECURITY.md)); incrusta las imágenes como data URI, o establece el directorio de trabajo antes de cargar, si un proyecto referencia archivos.
* Las funciones de control de versiones y de CI trabajan sobre archivos: no son seguras frente a otro proceso que edite el mismo proyecto en el mismo instante (la fachada comprueba de antemano si hay un árbol sucio y una fusión pendiente, pero no hay ningún bloqueo).

## Determinismo

Mismo proyecto, mismos argumentos, misma compilación de OSATURA y misma máquina: mismos bytes. Esto se cumple para `render_rgba` / `render_png` / `render_svg`, `bone_world`, todas las exportaciones, `lod_levels` y `fuzz` (para una semilla dada), `fit_to_size` y para un `Player` (las mismas entradas y las mismas llamadas a `advance` dan la misma pose; `advance` divide el tiempo en fragmentos de como máximo 1/60 s, así que cómo dividas el tiempo importa solo por debajo de esa resolución). Las elecciones aleatorias dentro del motor (ruido, partículas, fuzzing, clústeres de trazado) usan generadores con semilla, nunca el reloj. Lo que *no* se promete: píxeles de borde con antialiasing idénticos entre sistemas operativos, CPU, compiladores o versiones de `libm` distintos, entre destinos de 32 y 64 bits, ni entre versiones de OSATURA. Los golden exactos al byte solo se verificaron entre procesos en una máquina Linux x86-64 (consulta [LIMITATIONS.md](LIMITATIONS.md)); usa tolerancias (la CI de animación lo hace) para cualquier comparación entre máquinas.

## Límites y rangos de argumentos

| Elemento | Regla |
|---|---|
| `MAX_JSON_BYTES` | 256 MiB; un texto más largo da `InvalidJson` |
| `MAX_RENDER_SIDE` | 16384 píxeles de salida por lado (`render_*` y `Player::render_*`) |
| Escala de renderizado / GIF | finita y dentro de `0.05..=4`, de lo contrario `InvalidArgument` |
| Posiciones de fotograma | deben ser finitas; los valores fuera de la línea de tiempo se **limitan** a `0..=frames-1` (sin error). Los fotogramas fraccionarios son válidos. |
| Claves (`set_key`, `delete_key`) | `frame` y `value` finitos; `prop` es un nombre de propiedad en snake_case del JSON del proyecto (`"bone_angle"`, `"layer_opacity"`, ...) |
| Proyecto | `fps` en `(0, 1000]`, `frames` en `1..=200000`, escenario de hasta 32768 unidades por lado, como máximo 4096 huesos, números más allá de +-1e7 rechazados al cargar; otros límites por función se validan al cargar y se devuelven como `InvalidProject` |
| `Player::advance(dt)` | `dt` finito, `0..=10` segundos |
| `Player::set_canvas_size` | ambos lados finitos, `> 0`, como máximo 32768 |
| `Player::set_screen_size` | ancho y alto finitos y `> 0`; `max_error_px` finito y `>= 0` |
| `Animation::lod_levels` | `tolerance_px > 0`; `scale` en `(0, 64]`; `levels` en `1..=12` |
| `Animation::fit_to_size` | `max_bytes` en `1000..=1_000_000_000` |
| `Animation::fuzz` | `samples` en `1..=100000` |
| `trace_image_svg`, `Animation::from_image` | `colors` en `2..=64`; bytes PNG o JPEG (se aplican límites de tamaño) |
| `vcs_log` | `limit` en `1..=10000` |

## Errores

Toda función que puede fallar devuelve `Result<_, ApiError>`. Una entrada errónea es un valor de error y **nunca provoca un pánico**; un pánico que llegue a producirse dentro del motor se captura en la frontera de la fachada y se devuelve como `ApiError::Internal` (por favor, infórmalo como un error). `ApiError` implementa `Display` y `std::error::Error`, de modo que `?` funciona con `Box<dyn Error>` y tipos al estilo de `anyhow`.

| Variante | Significado | Orígenes típicos |
|---|---|---|
| `InvalidJson(String)` | no es JSON, no es un proyecto, forma incorrecta, demasiado grande (> `MAX_JSON_BYTES`), o un `format` más reciente que esta compilación; también un valor JSON erróneo pasado a `vcs_resolve` | `Animation::from_json`, `validate_json`, `vcs_resolve` |
| `InvalidProject(Vec<String>)` | analizado, pero la validación falló; un mensaje por problema (hueso desconocido, rango incorrecto, ciclo, ...) | `from_json`, `from_image`, reconstrucciones del motor de `Player` |
| `InvalidArgument(String)` | se rechazó un argumento del llamador: número no finito, valor fuera de rango, propiedad desconocida, id de máquina / entrada / desplazamiento desconocido, BVH o imagen inutilizable, nombre de rama incorrecto, nada que confirmar, ningún repositorio | casi todas las funciones |
| `EditRejected(Vec<String>)` | se rechazó una edición (por ejemplo un canal o una capa bloqueados, o un resultado no válido); el proyecto no cambia | `set_key`, `delete_key`, `retarget_bvh` |
| `Export(String)` | falló un codificador o el formato no es posible para este proyecto | `export`, `render_png`, `fit_to_size` |
| `Internal(String)` | un pánico capturado | cualquiera |
| `DirtyTree(String)` | (1.3) el archivo de proyecto tiene cambios sin confirmar que la operación sobrescribiría | `vcs_checkout` (salvo `force`), `vcs_merge`, `vcs_cherry_pick`, `vcs_revert` |
| `MergeInProgress(String)` | (1.3) una fusión, cherry-pick o reversión está sin terminar; resuelve y confirma, o `vcs_abort` | las mismas funciones |
| `Conflicts(Vec<String>)` | (1.3) `vcs_commit` mientras quedan conflictos; un puntero por conflicto | `vcs_commit` |

Una comprobación fallida de la CI de animación **no** es un error: `ci_run` devuelve `Ok(CiOutcome)` con `exit_code` 1 (una comprobación falló) o 2 (una comprobación no pudo evaluarse); solo un manifiesto inutilizable es `Err`. Los conflictos de fusión son igualmente un resultado (`VcsMergeOutcome::Conflicted`), no un error.

## Referencia: `Animation`

Un proyecto cargado y validado más su historial de deshacer. Créala con `from_json`, `demo` o `from_image`. Todos los métodos que leen el proyecto reciben `&self`.

| Método | Devuelve | Notas |
|---|---|---|
| `Animation::from_json(&str)` | `Result<Animation, ApiError>` | analiza + valida. `InvalidJson` / `InvalidProject`. Aquí los campos JSON desconocidos se ignoran en silencio (el comando `check` de la CLI los informa). |
| `Animation::validate_json(&str)` | `Result<(), ApiError>` | las mismas comprobaciones, no conserva nada |
| `Animation::demo()` | `Animation` | pequeño rig integrado (6 huesos, 48 fotogramas a 24 fps, 640 x 480); útil para pruebas y primeros pasos |
| `Animation::from_image(&[u8], colors, name)` | `Result<Animation, ApiError>` | traza bytes PNG/JPEG en capas vectoriales rellenas sobre un escenario del tamaño de la imagen |
| `to_json(&self)` | `String` | el proyecto actual como JSON con sangrías (viaje de ida y vuelta mediante `from_json`) |
| `info(&self)` | `AnimationInfo` | `name`, `width`, `height` (unidades del escenario), `fps`, `frames` |
| `frame_count(&self)`, `fps(&self)` | `u32`, `f32` | |
| `bone_names(&self)` | `Vec<String>` | orden del rig = orden de `bone_world` |
| `layer_names(&self)` | `Vec<String>` | rutas de capa `group/child`, primero en profundidad |
| `bone_world(&self, frame)` | `Result<Vec<[f32; 6]>, ApiError>` | matriz de mundo por hueso `[a b c d tx ty]`; un punto local del hueso `(x, y)` se transforma en `(a*x + c*y + tx, b*x + d*y + ty)`. Fotograma limitado. Evalúa todo el rig (IK, restricciones, muelles, expresiones). |
| `machine_names(&self)` | `Vec<String>` | máquinas de estados de runtime almacenadas en el proyecto |
| `player(&self, machine)` | `Result<Player, ApiError>` | inicia la máquina indicada en el tiempo 0 sobre una copia del proyecto |
| `render_rgba(&self, frame, &RenderSettings)` | `Result<Image, ApiError>` | RGBA8 con alfa directo (no premultiplicado), orden por filas, fila superior primero |
| `render_png(&self, frame, &RenderSettings)` | `Result<Vec<u8>, ApiError>` | un archivo PNG en memoria |
| `render_svg(&self, frame, show_bones: bool)` | `Result<String, ApiError>` | un documento SVG (la salida vectorial no tiene escala) |
| `export(&self, ExportFormat, scale)` | `Result<Vec<u8>, ApiError>` | `Gif` (en bucle, toda la línea de tiempo; `scale` dentro de `0.05..=4`), `Glb` (glTF 2.0 binario), `Lottie` (bytes JSON), `Html` (una página de reproductor autocontenida); `scale` se ignora en los otros tres. Qué conserva o descarta cada formato: [EXPORT_FORMATS.md](EXPORT_FORMATS.md). |
| `set_key(&mut self, target, prop, frame, value, Interpolation)` | `Result<(), ApiError>` | establece o añade una clave en `target.prop`; validada, se puede deshacer, respeta los bloqueos de canal y de capa (`EditRejected`) |
| `delete_key(&mut self, target, prop, frame)` | `Result<bool, ApiError>` | `true` si se eliminó una clave |
| `add_rect_layer(&mut self, name, x, y, w, h, fill, parent)`, `add_ellipse_layer(&mut self, name, cx, cy, rx, ry, fill, parent)` | `Result<(), ApiError>` | (1.4) añade al final una capa de forma rellena (en el nivel superior, o dentro del grupo de nivel superior indicado por `parent`); nombre único sin `/`, tamaños finitos y positivos, un color válido; se puede deshacer |
| `add_group_layer(&mut self, name)` | `Result<(), ApiError>` | (1.4) añade al final una capa de grupo vacía |
| `add_bone(&mut self, name, parent, length, x, y, angle)` | `Result<(), ApiError>` | (1.4) añade al final un hueso (`parent` debe existir, ángulo en grados); se puede deshacer |
| `bake_physics(&mut self, chain, preset, overwrite)` | `Result<PhysicsBake, ApiError>` | (1.4) simula una cadena de huesos (raíz primero) con `hair`, `tail`, `rope`, `antenna` o `cape` y almacena canales de ángulo como un único paso de deshacer; con `overwrite = false` se conservan los huesos con claves manuales y se enumeran en `skipped` |
| `render_exr(&self, frame, &RenderSettings, float32)` | `Result<Vec<u8>, ApiError>` | (1.4) archivo OpenEXR (lineal, premultiplicado; HALF o FLOAT) del compositor en coma flotante |
| `undo(&mut self)`, `redo(&mut self)` | `bool` | `false` cuando no hay nada que deshacer / rehacer |
| `lod_levels(&self, tolerance_px, scale, levels)` | `Result<Vec<LodLevel>, ApiError>` | el nivel 0 es el original, el nivel k >= 1 se mantiene dentro de `tolerance_px * 2^(k-1)` píxeles de salida a `scale`; cada nivel lleva su cota declarada y un JSON de proyecto. Determinista; la cota cubre la reproducción de la línea de tiempo (no las poses de la máquina de estados). Lento en compilaciones sin optimizar (usa release). |
| `fit_to_size(&self, FitFormat, max_bytes)` | `Result<SizeFit, ApiError>` | la exportación más pequeña dentro del presupuesto, bajando paso a paso los parámetros de calidad; si es imposible, el resultado más pequeño con `met == false` y un `reason` |
| `retarget_bvh(&mut self, bvh_text, &MocapSettings)` | `Result<MocapReport, ApiError>` | retargeting de BVH como **una única edición reversible**; huesos emparejados por nombre y topología; los canales bloqueados se omiten y se informan |
| `fuzz(&self, samples, seed)` | `Result<FuzzSummary, ApiError>` | fuzzing de poses; la animación solo se lee; informe completo como JSON |

`Interpolation` del segmento que empieza en una clave: `Linear`, `Step` (mantener), `EaseIn`, `EaseOut`, `EaseInOut`, `Smooth` (Catmull-Rom a través de las claves vecinas). Las curvas más ricas del formato de proyecto (Bezier cúbica, bounce, back, spring) se crean mediante la CLI y el studio, no mediante `set_key`.

## Referencia: `Player`

Una máquina de estados en ejecución (reproducción interactiva). Se crea con `Animation::player`; trabaja sobre una copia del proyecto; determinista. Las máquinas de estados y los data bindings son JSON de proyecto simple (`machines`, `bindings`), resumidos en el [apéndice](#apéndice-json-de-máquinas-y-json-de-bindings).

| Método | Devuelve | Notas |
|---|---|---|
| `set_bool(name, bool)`, `set_number(name, f64)`, `fire(name)` | `Result<(), ApiError>` | establece una entrada por nombre; un tipo de entrada equivocado, o un nombre desconocido, da `InvalidArgument`. Un trigger permanece activado hasta que se dispara una transición que lo comprueba (se consume) o hasta el final del siguiente `advance`; `hold: true` lo mantiene hasta que se consume. |
| `set_value(path, f64)` | `Result<(), ApiError>` | publica un valor externo para los data bindings (`"player.health"`). Una región de desplazamiento cuyo `input` es esta ruta toma el valor como su desplazamiento en píxeles a lo largo de su eje principal (el desplazamiento queda entonces en reposo). |
| `advance(dt)` | `Result<(), ApiError>` | avanza la máquina y las regiones de desplazamiento `dt` segundos (`0..=10`) |
| `state(layer)` | `Option<String>` | estado actual de una capa de máquina; `None` para una capa desconocida |
| `events_json()` | `String` | eventos pendientes `[{"time":..,"layer":..,"kind":"entered"|"exited","state":..}]`, sin consumir |
| `take_events_json()`, `clear_events()` | `String`, `()` | consumen / descartan los eventos pendientes |
| `render_rgba(&RenderSettings)`, `render_svg(show_bones)`, `bone_world()` | como en `Animation` | la pose actual; la imagen tiene el tamaño de lienzo fijado con `set_canvas_size` por la escala |
| `set_canvas_size(w, h)`, `canvas_size()` | `Result<(), ApiError>`, `(f32, f32)` | (1.2) maquetación adaptable: los contenedores se vuelven a resolver, los límites de desplazamiento los siguen |
| `scroll_ids()` | `Vec<String>` | ids de las regiones de desplazamiento del proyecto |
| `scroll_drag(id, x, y)` | `Result<(), ApiError>` | desplazamiento bruto del puntero en píxeles (contenido movido hacia arriba / la izquierda esa cantidad); más allá de los extremos se limita o rebota elásticamente |
| `scroll_release(id, vx, vy)` | `Result<(), ApiError>` | velocidad de suelta en px/s; `advance` entonces lanza, ajusta y recupera con muelle |
| `scroll_offset(id)`, `scroll_max(id)`, `scroll_settled(id)` | `Option<[f32; 2]>`, `Option<[f32; 2]>`, `Option<bool>` | `None` para un id desconocido |
| `set_screen_size(w_px, h_px, max_error_px)` | `Result<(), ApiError>` | (1.2) elige el más grueso de ocho niveles de detalle precalculados cuya cota declarada, a la escala que ajusta el lienzo a esa pantalla, es como máximo `max_error_px`. Los niveles se construyen una vez en el primer uso (el nivel 1 acotado por 0.125 unidades de escenario, duplicando) y se almacenan en caché; con `max_error_px` 0 solo cumplen los niveles sin pérdida. |
| `clear_screen_size()`, `lod_level()`, `lod_bound_px()` | `()`, `usize`, `f32` | vuelve al original; nivel en uso (0 = original); su cota declarada en píxeles |

Notas: la cota del nivel de detalle cubre la geometría y las claves de la línea de tiempo tal como se documenta en `lod/mod.rs` (`ASSUMPTIONS`, `LIMITS`); las poses de la máquina de estados y los bindings no se modelan en ella, y los canales de pose que apuntan a una capa que un nivel eliminó se descartan. Los proxies de scrub del studio usan un mecanismo aparte con su propia cuadrícula de niveles. El `input` de una región de desplazamiento pasa por la tabla de valores del reproductor, no por el runtime de data bindings, de modo que las expresiones, el remapeo y el suavizado no se le aplican.

## Referencia: funciones libres (herramientas, control de versiones, CI)

| Función | Devuelve | Hace |
|---|---|---|
| `trace_image_svg(&[u8], colors)` | `Result<String, ApiError>` | traza un mapa de bits en un SVG de regiones rellenas |
| `vcs_log(path, limit)` | `Result<Vec<VcsCommit>, ApiError>` | (1.2, solo lectura) historial del más reciente al más antiguo; `path` es el archivo de proyecto o cualquier ruta dentro de su repositorio; un historial vacío da una lista vacía |
| `ci_run(manifest)` | `Result<CiOutcome, ApiError>` | (1.2) CI de animación en modo de verificación (no se vuelve a registrar nada); las rutas relativas del manifiesto se resuelven junto a él |
| `vcs_init(project_file, author)` | `Result<(), ApiError>` | (1.3) inicia un historial junto al archivo en la rama `main`, sin confirmaciones todavía; falla si ya existe uno |
| `vcs_commit(path, message, author)` | `Result<String, ApiError>` | id de la confirmación; termina una fusión una vez que no queda ningún conflicto; `Conflicts` mientras queden; nada que confirmar o sin mensaje es `InvalidArgument` |
| `vcs_branch(path, name, at)`, `vcs_tag(path, name, at)` | `Result<String, ApiError>` | crean una rama / etiqueta (por defecto: la confirmación actual); no se cambia a ella; las etiquetas nunca se mueven; se devuelve el id de la confirmación |
| `vcs_checkout(path, target, force)` | `Result<String, ApiError>` | cambia el archivo a una rama (se queda en ella) o a cualquier revisión (detached); `DirtyTree` salvo `force`; `MergeInProgress` |
| `vcs_merge(path, rev, no_ff, message, author)` | `Result<VcsMergeOutcome, ApiError>` | `UpToDate`, `FastForward(id)`, `Merged(id)` o `Conflicted(Vec<VcsConflict>)` |
| `vcs_cherry_pick(path, rev, author)`, `vcs_revert(path, rev, author)` | `Result<VcsMergeOutcome, ApiError>` | reaplican / deshacen el cambio de una confirmación |
| `vcs_conflicts(path)` | `Result<Vec<VcsConflict>, ApiError>` | los conflictos aún pendientes (vacío cuando no hay ninguno) |
| `vcs_resolve(path, conflict_path, &VcsChoice)` | `Result<usize, ApiError>` | resuelve un conflicto con `Ours`, `Theirs` o `Value(json text)`; devuelve cuántos quedan |
| `vcs_abort(path)` | `Result<(), ApiError>` | descarta una fusión / cherry-pick / reversión sin terminar y restaura la última confirmación |
| `vcs_branches(path)` | `Result<Vec<VcsBranch>, ApiError>` | (1.4) las ramas con su confirmación y si están activas |
| `vcs_branch_delete(path, name)` | `Result<(), ApiError>` | (1.4) elimina una rama; la rama activa y los nombres desconocidos se rechazan; los objetos permanecen hasta `vcs_gc` |
| `vcs_status(path)` | `Result<VcsStatus, ApiError>` | (1.4) rama, HEAD, cambios semánticos sin confirmar, fusión sin terminar, rutas de conflicto |
| `vcs_diff(path, a, b)` | `Result<Vec<VcsChange>, ApiError>` | (1.4) diferencias semánticas entre dos revisiones; `None` es el archivo de trabajo |
| `vcs_show(path, rev)` | `Result<VcsShow, ApiError>` | (1.4) una confirmación y sus cambios respecto a su primer padre |
| `vcs_gc(path)` | `Result<VcsGc, ApiError>` | (1.4) elimina los objetos inalcanzables; se aborta sin eliminar nada si no se puede leer el historial |
| `vcs_fsck(path)` | `Result<Vec<String>, ApiError>` | (1.4) problemas de integridad, vacío cuando todo está sano |
| `ci_init(project, manifest, write_goldens)` | `Result<CiInit, ApiError>` | escribe un manifiesto calibrado por defecto (nunca sobrescribe un archivo existente; nombre por defecto `osatura-ci.json` junto al proyecto); opcionalmente registra ya los golden |
| `ci_run_mode(manifest, CiMode)` | `Result<CiOutcome, ApiError>` | `Verify`, `Update` (vuelve a registrar todos los golden de las comprobaciones de renderizado), `Bless` (acepta solo los que fallan o faltan) |
| `ci_bless(manifest)` | `Result<CiOutcome, ApiError>` | abreviatura de `Bless` |

El historial se almacena junto al archivo de proyecto (`.osatura-vcs/`); los mensajes de confirmación y los autores son texto plano. Un conflicto contiene el puntero de identidad `path`, un `kind`, una `description` y los valores `base` / `ours` / `theirs` como texto JSON (`None` = ausente allí). Tras una fusión con conflictos, el archivo de proyecto contiene el estado fusionado con los elementos en conflicto con el valor de la rama actual, y se registra un estado pendiente hasta que resuelvas todos los conflictos y hagas `vcs_commit`, o `vcs_abort`. Los recursos binarios que referencia un proyecto no se versionan, solo sus cadenas de ruta. La fusión semántica, sus tipos de conflicto y sus límites se describen en [ROADMAP.md](ROADMAP.md) sección 9 (elemento I-01) y [LIMITATIONS.md](LIMITATIONS.md).

No incluido en la fachada: invalidaciones del mapeo de huesos de mocap, ajuste fino del trazado más allá del número de colores, opciones de fit más allá del formato y del presupuesto, renderizado en GPU, el studio, la colaboración, el servidor de agentes, los numerosos comandos de creación y de baking (física, foot lock, magnets, autorig, ...), eliminación / listado de ramas, diff, show, status, gc, fsck, y opciones de CI como prune, `--only` y artefactos. Usa la CLI para ello ([CLI_REFERENCE.md](CLI_REFERENCE.md)).

## Referencia: tipos

| Tipo | Campos / variantes |
|---|---|
| `AnimationInfo` | `name: String`, `width: f32`, `height: f32` (unidades del escenario), `fps: f32`, `frames: u32` |
| `RenderSettings` | `scale: f32` (0.05..=4), `show_bones: bool`, `transparent: bool`; `new(scale)`, `with_bones(bool)`, `with_transparent(bool)`; `Default` = escala 1, ambos desactivados |
| `Image` | `width: u32`, `height: u32`, `pixels: Vec<u8>` (RGBA8, alfa directo) |
| `ExportFormat` | `Gif`, `Glb`, `Lottie`, `Html`, `Apng` (1.4) |
| `PhysicsBake` (1.4) | `added: usize`, `skipped: Vec<String>` |
| `VcsBranch` (1.4) | `name`, `commit`, `current: bool` |
| `VcsChange` (1.4) | `kind` (`"Added"`, `"Removed"`, `"Changed"`, `"Moved"`, `"Retimed"`), `path`, `description` |
| `VcsStatus` (1.4) | `branch`, `head`, `changes: Vec<VcsChange>`, `merge`, `conflicts: Vec<String>` |
| `VcsShow` (1.4) | `commit: VcsCommit`, `changes: Vec<VcsChange>` |
| `VcsGc` (1.4) | `kept`, `removed` |
| `Interpolation` | `Linear`, `Step`, `EaseIn`, `EaseOut`, `EaseInOut`, `Smooth` |
| `LodLevel` | `level`, `declared_px: f64`, `bound_stage: f64`, `path_anchors`, `keys`, `layers`, `json: String`; `bound_at(screen_scale) -> f64` |
| `FitFormat` | `Lottie`, `Html`, `Gif`, `Apng`, `Project` |
| `SizeFit` | `met: bool`, `size: usize`, `bytes: Vec<u8>`, `ssim_min: f64`, `reason: Option<String>` |
| `MocapSettings` | `in_place`, `mirror`, `smooth` (radio de la media móvil en fotogramas); `new()`, `with_in_place`, `with_mirror`, `with_smooth`; `Default` |
| `MocapReport` | `matched: Vec<(String, String)>` (articulación BVH, hueso del rig), `unmatched_joints`, `unmatched_bones`, `frames`, `bones_used`, `blocked` (`target.prop` bloqueado), `notes` |
| `FuzzSummary` | `samples`, `failures`, `failure_rate: f32`, `max_severity: f32`, `json: String` |
| `VcsCommit` | `id`, `author`, `time: u64` (segundos Unix), `message` |
| `VcsConflict` | `path`, `kind`, `description`, `base`, `ours`, `theirs: Option<String>` |
| `VcsMergeOutcome` | `UpToDate`, `FastForward(String)`, `Merged(String)`, `Conflicted(Vec<VcsConflict>)` |
| `VcsChoice` | `Ours`, `Theirs`, `Value(String)` |
| `CiMode` | `Verify`, `Update`, `Bless` |
| `CiOutcome` | `total`, `passed`, `failed`, `errors`, `exit_code: i32` (0 correcto, 1 fallido, 2 no evaluable), `json: String`, `updated`, `blessed` |
| `CiInit` | `manifest: String`, `checks: Vec<(String, String)>` (tipo, nombre), `recorded: Option<CiOutcome>` |
| Constantes | `API_VERSION: &str`, `API_VERSION_MAJOR: u32`, `MAX_RENDER_SIDE: u32`, `MAX_JSON_BYTES: usize` |

## Ejemplos

Todos estos ejemplos están completos y se ejecutaron. Suponen `use osatura_core::prelude::*;`.

### Cargar, evaluar, renderizar, exportar

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Load a project, query it, render one frame and export the whole animation.
pub fn quick(project_path: &str) -> Result<(), Box<dyn Error>> {
    let json = std::fs::read_to_string(project_path)?;
    let mut anim = Animation::from_json(&json)?;           // parse + validate; typed error otherwise

    // Facts about the project.
    let info = anim.info();
    println!("{}: {}x{} @ {} fps, {} frames", info.name, info.width, info.height, info.fps, info.frames);

    // Evaluate: world matrices of every bone at frame 10 (six floats per bone: a b c d tx ty).
    let names = anim.bone_names();
    let world = anim.bone_world(10.0)?;
    let m = world[0];
    // A bone-local point (x, y) lands at (a*x + c*y + tx, b*x + d*y + ty).
    let (x, y) = (m[0] * 10.0 + m[2] * 0.0 + m[4], m[1] * 10.0 + m[3] * 0.0 + m[5]);
    println!("tip of {} (10 units along its x axis) is at ({x:.1}, {y:.1})", names[0]);

    // Edit through the undoable editor, then render.
    anim.set_key(&names[0], "bone_angle", 10.0, 25.0, Interpolation::EaseInOut)?;
    let png = anim.render_png(10.0, &RenderSettings::new(1.0).with_bones(true))?;
    std::fs::write("frame10.png", &png)?;
    let svg = anim.render_svg(10.0, false)?;
    std::fs::write("frame10.svg", svg)?;

    // Raw pixels: straight-alpha RGBA8, row-major, top row first.
    let img = anim.render_rgba(10.0, &RenderSettings::new(0.5).with_transparent(true))?;
    assert_eq!(img.pixels.len(), img.width as usize * img.height as usize * 4);

    // Whole-animation exports.
    let glb = anim.export(ExportFormat::Glb, 1.0)?;
    let gif = anim.export(ExportFormat::Gif, 0.5)?;        // `scale` only matters for GIF
    std::fs::write("anim.glb", &glb)?;
    std::fs::write("anim.gif", &gif)?;
    println!("{} bones, {} bytes of glb, {} bytes of gif", names.len(), glb.len(), gif.len());
    Ok(())
}
```

### Edición, deshacer y tratamiento de errores

```rust
use osatura_core::prelude::*;

/// Edits are validated, undoable and all-or-nothing.
pub fn edit_and_errors() {
    let mut anim = Animation::demo();                       // small built-in project
    let bone = anim.bone_names()[0].clone();
    let before = anim.to_json();

    anim.set_key(&bone, "bone_angle", 4.0, 30.0, Interpolation::Smooth).unwrap();
    assert!(anim.delete_key(&bone, "bone_angle", 4.0).unwrap());      // true: a key was removed
    assert!(!anim.delete_key(&bone, "bone_angle", 77.0).unwrap());    // false: nothing there
    assert!(anim.undo() && anim.undo());                    // each successful edit is one undo step
    assert_eq!(anim.to_json(), before);
    assert!(anim.redo());

    // Errors are values. Match on them with a wildcard arm: the enum is #[non_exhaustive].
    match anim.set_key(&bone, "no_such_prop", 0.0, 1.0, Interpolation::Linear) {
        Err(ApiError::InvalidArgument(why)) => println!("rejected argument: {why}"),
        Err(ApiError::EditRejected(problems)) => println!("edit refused: {problems:?}"),
        Err(other) => println!("other failure: {other}"),   // Display is implemented for every variant
        Ok(()) => unreachable!(),
    }
    // A rejected edit leaves the project exactly as it was.
    let snapshot = anim.to_json();
    assert!(anim.set_key("no_such_bone", "bone_angle", 1.0, 1.0, Interpolation::Linear).is_err());
    assert_eq!(anim.to_json(), snapshot);

    // Bad input never panics.
    assert!(matches!(Animation::from_json("{ nope"), Err(ApiError::InvalidJson(_))));
    assert!(anim.render_rgba(f32::NAN, &RenderSettings::default()).is_err());
    assert!(anim.render_rgba(0.0, &RenderSettings::new(50.0)).is_err()); // scale must be 0.05..=4
}
```

### Reproducción interactiva con una máquina de estados

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Run a state machine of a project (machine.json has machine "hero", layer "body", bool input "excited").
pub fn play(project_path: &str) -> Result<(), Box<dyn Error>> {
    let anim = Animation::from_json(&std::fs::read_to_string(project_path)?)?;
    let machine = anim.machine_names()[0].clone();         // e.g. "hero"
    let mut player = anim.player(&machine)?;               // works on a copy of the project

    player.set_bool("excited", true)?;                     // also set_number, fire (trigger), set_value (binding path)
    for _ in 0..30 {
        player.advance(1.0 / 30.0)?;                       // deterministic; dt in seconds, 0..=10
    }
    println!("body is in state {:?}", player.state("body"));  // Some("wave")
    println!("{}", player.take_events_json());             // [{"time":..,"layer":"body","kind":"entered","state":..}, ..]

    let frame = player.render_rgba(&RenderSettings::new(0.5))?;
    println!("pose rendered at {}x{}", frame.width, frame.height);
    let svg = player.render_svg(false)?;
    assert!(svg.starts_with("<svg"));

    // Same inputs, same advance calls, same pose: a second player is bit-for-bit identical.
    let mut again = anim.player(&machine)?;
    again.set_bool("excited", true)?;
    for _ in 0..30 {
        again.advance(1.0 / 30.0)?;
    }
    assert_eq!(again.render_rgba(&RenderSettings::new(0.5))?, frame);

    // Wrong calls are typed errors, not panics.
    assert!(matches!(player.set_number("excited", 1.0), Err(ApiError::InvalidArgument(_))));
    assert!(matches!(anim.player("no such machine"), Err(ApiError::InvalidArgument(_))));
    Ok(())
}
```

### Maquetación, desplazamiento y nivel de detalle

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Layout, scrolling and level of detail on a Player (scroll.json has one scroll region "list").
pub fn layout_and_lod(project_path: &str) -> Result<(), Box<dyn Error>> {
    let anim = Animation::from_json(&std::fs::read_to_string(project_path)?)?;
    let mut player = anim.player(&anim.machine_names()[0])?;

    player.set_canvas_size(100.0, 120.0)?;                 // containers re-solve; renders get this size
    for id in player.scroll_ids() {
        let max = player.scroll_max(&id).unwrap();         // about [0, 80] here: 200 units of content in a 120 unit view
        player.scroll_drag(&id, 0.0, 50.0)?;               // raw pointer offset in px (clamped / rubber-banded)
        assert_eq!(player.scroll_offset(&id), Some([0.0, 50.0]));
        player.scroll_release(&id, 0.0, 900.0)?;           // fling at 900 px/s
        while player.scroll_settled(&id) == Some(false) {
            player.advance(1.0 / 60.0)?;                   // steps the machine and every scroll region
        }
        println!("{id}: max {max:?}, rests at {:?}", player.scroll_offset(&id).unwrap());
    }

    // Pick the coarsest level of detail whose declared error stays within 0.5 px at this screen size.
    player.set_screen_size(100.0, 120.0, 0.5)?;
    println!("level {} (declared bound {} px)", player.lod_level(), player.lod_bound_px());
    player.clear_screen_size();                            // back to the original
    assert_eq!(player.lod_level(), 0);

    // A scroll region whose `input` names a path follows set_value(path, px) instead of a drag.
    player.set_value("ui.scroll", 200.0)?;
    Ok(())
}
```

### Renderizado en varios hilos

```rust
use osatura_core::prelude::*;

/// `Animation` and `Player` are `Send + Sync`: read-only calls (`&self`) may run on several threads.
pub fn render_in_parallel() -> Result<(), ApiError> {
    let anim = Animation::demo();
    let frames = anim.frame_count();
    let settings = RenderSettings::new(0.25);
    let (a, s) = (&anim, &settings);                        // shared references are Copy: move them into the threads
    let images: Vec<Result<Image, ApiError>> = std::thread::scope(|scope| {
        let jobs: Vec<_> = (0..frames).map(|f| scope.spawn(move || a.render_rgba(f as f32, s))).collect();
        jobs.into_iter().map(|j| j.join().expect("render thread")).collect()
    });
    // Mutating calls (`&mut self`: set_key, undo, retarget_bvh ...) need exclusive access.
    for (f, img) in images.into_iter().enumerate() {
        assert_eq!(img?, anim.render_rgba(f as f32, &settings)?);   // threads do not change the result
    }
    Ok(())
}
```

### Control de versiones y CI de animación

```rust
use osatura_core::prelude::*;

/// Version control of a project file and animation CI, from Rust.
pub fn history(dir: &std::path::Path) -> Result<(), ApiError> {
    let file = dir.join("hero.json");
    std::fs::write(&file, Animation::demo().to_json()).unwrap();
    let path = file.to_str().unwrap();

    // A tiny helper: re-key the first bone at frame 5 and save the file.
    let edit = |angle: f32| -> Result<(), ApiError> {
        let mut a = Animation::from_json(&std::fs::read_to_string(path).unwrap())?;
        let bone = a.bone_names()[0].clone();
        a.set_key(&bone, "bone_angle", 5.0, angle, Interpolation::Linear)?;
        std::fs::write(path, a.to_json()).unwrap();
        Ok(())
    };

    vcs_init(path, Some("ana"))?;                           // branch `main`, no commit yet
    vcs_commit(path, Some("base"), None)?;
    vcs_branch(path, "other", None)?;                       // created, not switched to
    edit(20.0)?;
    vcs_commit(path, Some("main edit"), None)?;
    vcs_checkout(path, "other", false)?;                    // DirtyTree if the file has uncommitted changes
    edit(40.0)?;                                            // the same key, another value
    vcs_commit(path, Some("other edit"), None)?;
    vcs_checkout(path, "main", false)?;

    match vcs_merge(path, "other", false, None, None)? {
        VcsMergeOutcome::Conflicted(conflicts) => {         // a normal result, not an error
            for c in &conflicts {
                println!("{} ({}): {}", c.path, c.kind, c.description);
                vcs_resolve(path, &c.path, &VcsChoice::Theirs)?;
            }
            vcs_commit(path, None, None)?;                  // the merge commit
        }
        other => println!("merged without conflicts: {other:?}"),
    }
    for c in vcs_log(path, 10)? {
        println!("{} {} {}", &c.id[..8], c.author, c.message);
    }

    // Animation CI: write a calibrated manifest, record goldens, then verify.
    let init = ci_init(path, None, true)?;                  // never overwrites an existing manifest
    let run = ci_run(&init.manifest)?;                      // verify mode: nothing is re-recorded
    println!("{} checks, {} passed, exit code {}", run.total, run.passed, run.exit_code);
    if run.exit_code == 1 {
        // A failing check is a result, not an Err. After reviewing the report, accept the new look:
        ci_bless(&init.manifest)?;
    }
    Ok(())
}
```

### Mocap, fuzzing, trazado, nivel de detalle y exportación con tamaño objetivo

```rust
use osatura_core::prelude::*;

const RIG: &str = r#"{"format":1,"name":"rig","width":400.0,"height":400.0,"fps":30.0,"frames":10,"background":null,
 "camera":{"position":null,"zoom":1.0,"rotation":0.0},
 "bones":[{"name":"hips","parent":null,"length":20.0,"rest_offset":[200.0,250.0],"rest_angle":-90.0},
          {"name":"spine","parent":"hips","length":40.0,"rest_offset":[20.0,0.0],"rest_angle":0.0},
          {"name":"head","parent":"spine","length":30.0,"rest_offset":[40.0,0.0],"rest_angle":0.0}],
 "ik":[],"constraints":[],"springs":[],"layers":[],"channels":[]}"#;

fn mini_bvh() -> String {
    let mut s = String::from("HIERARCHY\nROOT Hips\n{\n OFFSET 0 100 0\n CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation\n");
    s.push_str(" JOINT Spine\n {\n  OFFSET 0 12 0\n  CHANNELS 3 Zrotation Xrotation Yrotation\n  JOINT Head\n  {\n   OFFSET 0 12 0\n   CHANNELS 3 Zrotation Xrotation Yrotation\n   End Site\n   {\n    OFFSET 0 8 0\n   }\n  }\n }\n}\nMOTION\nFrames: 4\nFrame Time: 0.0333333\n");
    for f in 0..4 {
        s.push_str(&format!("0 100 0 0 0 0 {} 0 0 0 0 0\n", f * 10));
    }
    s
}

/// Motion capture, pose fuzzing, tracing, level of detail and size-targeted export.
pub fn tools() -> Result<(), ApiError> {
    // BVH retargeting: one undoable edit; bones are matched by name and topology.
    let mut rig = Animation::from_json(RIG)?;
    let report = rig.retarget_bvh(&mini_bvh(), &MocapSettings::new().with_smooth(1))?;
    println!("matched {:?}; {} frames on {} bones", report.matched, report.frames, report.bones_used);
    assert!(rig.undo());                                    // the whole retarget is one step

    // Pose fuzzing: where does the rig break?  (deterministic for a given seed)
    let demo = Animation::demo();
    let fuzz = demo.fuzz(50, 1)?;
    println!("{} of {} samples failed (max severity {:.2})", fuzz.failures, fuzz.samples, fuzz.max_severity);
    let _full_report: &str = &fuzz.json;                    // schema "osatura-fuzz-report" v1

    // Bitmap tracing: PNG/JPEG bytes -> SVG, or straight into an animation.
    let png = demo.render_png(0.0, &RenderSettings::new(0.25))?;
    let svg = trace_image_svg(&png, 6)?;
    let traced = Animation::from_image(&png, 6, "traced")?;
    println!("traced SVG: {} bytes; project has {} layers", svg.len(), traced.layer_names().len());

    // Level of detail with a declared error bound, one project JSON per level (level 0 = original).
    for l in demo.lod_levels(0.5, 1.0, 3)? {
        println!("L{}: bound {:.3} px, {} anchors, {} keys", l.level, l.declared_px, l.path_anchors, l.keys);
        let _reduced = Animation::from_json(&l.json)?;
    }

    // Smallest export within a byte budget.
    let fit = demo.fit_to_size(FitFormat::Lottie, 250_000)?;
    println!("met={} {} bytes, worst SSIM {:.3}", fit.met, fit.size, fit.ssim_min);
    Ok(())
}
```

### Cómo se comprueban los ejemplos

El código anterior es el código que se ejecutó: este archivo se genera a partir de las fuentes de los ejemplos, que se compilaron con `cargo` contra `osatura-core` y se ejecutaron (los archivos de proyecto utilizados son la salida de `osatura demo`, `take-record`, `clip add` y `machine add`, más un proyecto con un contenedor y una región de desplazamiento). Las doc-tests de `api/` (`cargo test --doc -p osatura-core`: 19 doc-tests, 18 sobre funciones de la fachada en `api/` y el ejemplo a nivel de crate en `lib.rs`; unos 170 s en una compilación de depuración porque cada una compila un programa) cubren los envoltorios de control de versiones y de CI en el propio repositorio.

## Otros lenguajes

La fachada es la única fuente de verdad para los demás bindings. Cobertura:

| Capacidad | Fachada de Rust | ABI de C (`osatura.h`) | WebAssembly / JS |
|---|---|---|---|
| Cargar, validar, info, nombres de huesos / matrices de mundo | sí | `osatura_open`, `_bone_*`, `_frame_count`, ... | `arm_open`, `load()` (sin matrices de huesos) |
| Renderizar RGBA | sí | `osatura_render_rgba` | `arm_render_rgba`, `rgba()` |
| Renderizar SVG / PNG / formatos de exportación | sí | no | SVG sí (`svg()`), PNG / exportación no |
| Editar claves, deshacer / rehacer | sí | no | no |
| `Player`: entradas, avance, estado, eventos, renderizado | sí | `osatura_player_*` | `arm_player_*`, `startPlayer()`, ... |
| Maquetación, desplazamiento, nivel de detalle | sí | `osatura_player_set_canvas`, `_scroll_*`, `_set_screen`, `_lod_level` | `arm_player_set_canvas / layout / scroll_* / set_screen` |
| Herramientas, control de versiones, CI | sí | no | no |

La versión de la ABI de C sigue siendo 1 porque todas las adiciones hasta ahora han sido aditivas. El binario wasm se compila y se prueba con Node en CI, pero no se ha ejecutado en un navegador; consulta [WASM.md](WASM.md).

## Apéndice: JSON de máquinas y JSON de bindings

Breves resúmenes de las dos secciones del proyecto que ejecuta el `Player` (formato completo: [PROJECT_FORMAT.md](PROJECT_FORMAT.md)).

### JSON de máquina (resumen)

```json
{"name":"hero",
 "inputs":[{"name":"moving","kind":"bool"},{"name":"speed","kind":"number"},{"name":"jump","kind":"trigger"}],
 "layers":[{"name":"body","mask":["arm_*"],"entry":"idle",
   "states":[{"name":"idle","source":{"kind":"clip","clip":"idle"},"blend_in":0.2},
             {"name":"loco","source":{"kind":"blend","input":"speed","children":[
                {"threshold":0,"source":{"kind":"clip","clip":"walk"}},{"threshold":1,"source":{"kind":"clip","clip":"run"}}]}},
             {"name":"jump","source":{"kind":"clip","clip":"jump"},"mode":"once"}],
   "transitions":[{"from":"idle","to":"loco","when":[{"input":"moving"}],"duration":0.25,"ease":{"type":"ease_in_out"}},
                  {"from":"*","to":"jump","when":[{"input":"jump"}],"duration":0.1},
                  {"from":"jump","to":"idle","exit_time":1.0}]}]}
```

Las condiciones son `{"input","op":"eq|ne|lt|le|gt|ge","value"}` (por defecto `eq`, `1`; todas deben cumplirse). `from: "*"` coincide con cualquier estado excepto el destino. Gana la primera transición coincidente en orden de declaración. `exit_time` es la fracción reproducida (0..1) del estado. Fuentes: `clip` (una entrada de `Project.clips`), `segment` (fotogramas `start`, `end` de los canales de modo reemplazo con claves del proyecto), `region` (un marcador con final), `blend` (1D, dos o más fuentes hoja con umbrales crecientes). Las claves desconocidas en una transición o condición son un error. CLI: `machine add|list|remove|validate`, `machine-sim <project> <script.json> <out_dir|out.json>` (script: `{"machine":"hero","duration":2,"events":[{"t":0.5,"set":{"moving":true},"trigger":["jump"],"value":{"player.health":40}}]}`).

### JSON de binding (resumen)

```json
{"name":"hp","path":"player.health","expr":"value / 100","remap":{"in_min":0,"in_max":1,"out_min":-30,"out_max":30},
 "smooth":0.15,"target":{"kind":"prop","target":"arm_up","prop":"bone_angle"}}
```

Pipeline: valor, luego `expr` (variable `value`; las funciones de expresión del motor, fuente de como máximo 4 KB y anidamiento de como máximo 64), luego `remap` (limitado por defecto; `curve` suaviza), luego el suavizado (constante de tiempo en segundos), y por último el destino: `{"kind":"prop",...}` (cualquier propiedad de canal, incluidas `layer_opacity` y `layer_switch`) o `{"kind":"input","machine","input"}` (una entrada de tipo trigger se dispara en el flanco ascendente). CLI: `bind add|list|remove`, `bind-eval`.

### Maquetación y mapas de normales por capa

Los contenedores de maquetación y las regiones de desplazamiento son los campos de proyecto `containers` / `scrolls` (se omiten cuando están vacíos): CLI `container add|list|remove|solve`, `scroll add|list|remove|eval`, `layout-preview <project> <out.svg> --size WxH`, y `--size WxH` en `render`, `frames` y `export` (los contenedores se vuelven a resolver para ese tamaño de lienzo; el archivo no cambia). `Layer.normal_map` (`data` PNG en base64 o `path`, `fit` stretch / tile / exact, `strength`, `green_down`) alimenta el efecto de posprocesado `relight` para los píxeles que cubre esa capa, o un hijo sin mapa propio (CLI `layer-normal`, `fx ... --layer-normals`); no forma parte de la fachada.
