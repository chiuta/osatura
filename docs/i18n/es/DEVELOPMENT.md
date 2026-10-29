<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Guía de desarrollo

*Para quienes compilan, modifican, prueban y publican OSATURA. Documentación CC-BY-4.0. Todo lo que aparece aquí procede del árbol de código fuente, de la sección 9 de `ROADMAP.md`, de [CI.md](CI.md), [CLEANROOM.md](CLEANROOM.md), [CONTRIBUTING.md](CONTRIBUTING.md) y [LICENSING.md](LICENSING.md); los comandos marcados con "run" se ejecutaron al escribir esta página (Linux x86-64, rustc 1.97.0, registro sin conexión).*

Contenido: [Entorno](#1-entorno) - [Comandos de compilación y prueba](#2-comandos-de-compilación-y-prueba) - [Las suites de pruebas](#3-las-suites-de-pruebas) - [Convenciones de código](#4-convenciones-de-código) - [Lista de comprobación para un módulo o comando nuevo](#5-lista-de-comprobación-para-un-módulo-o-comando-nuevo) - [Reglas clean-room y DCO](#6-reglas-clean-room-y-dco) - [Lista de comprobación de publicación](#7-lista-de-comprobación-de-publicación) - [Integración continua](#8-integración-continua) - [Depuración](#9-depuración) - [Perfilado](#10-perfilado)

## 1. Entorno

| Necesidad | Motivo | Notas |
|---|---|---|
| Rust 1.97.0 (workspace con edición 2021) | todo | fijado en `rust-toolchain.toml` (rustup lo instala la primera vez que se usa; `rust-version = "1.97"` en cada crate); `Cargo.lock` está en el repositorio, las compilaciones son compatibles con `--locked`. Las compilaciones de release tienen `overflow-checks = true`: un desbordamiento de enteros provoca un panic en lugar de dar la vuelta, así que prueba también el perfil release (`cargo test --release`) |
| Un compilador de C (`cc`) | el ejemplo o prueba de la ABI de C (opcional) | véase [C_ABI.md](C_ABI.md) |
| `ffmpeg` / `ffprobe` en el `PATH` | exportación de vídeo, decodificación de audio mp3/ogg, `dof-export` a .mp4/.webm, `soundtrack` | **nunca se incluye ni es necesario para compilar**; los comandos que lo necesitan lo indican y fallan sin mostrar mensaje de éxito |
| Node.js >= 22 (el que usa la CI) y `playwright-core` + Chromium | las cinco pruebas de humo del studio; la prueba del wrapper WASM | valores por defecto: `/opt/npm-tools/node_modules/playwright-core`, `/opt/pw-browsers/chromium-1194/chrome-linux/chrome`; se cambian con `PLAYWRIGHT_CORE`, `CHROME` |
| `cargo-deny` | política de licencias/prohibiciones/fuentes | `scripts/licence-audit.sh` lo ejecuta si está instalado (`--no-deny` lo omite) |
| `python3` | `scripts/licence-audit.sh` genera la documentación con un script en línea | se ejecuta con `python3 -I` |
| Un adaptador Vulkan o GL | solo para obtener cifras con GPU real de `osatura-gpu` | las pruebas recurren a un adaptador por software; véase la sección 2 |
| `rustup target wasm32-unknown-unknown`, `wasm-opt` | solo para [WASM.md](WASM.md) | nunca se instalaron en el entorno de desarrollo; la CI los instala |

Estructura del workspace (el diseño se describe en [ARCHITECTURE.md](ARCHITECTURE.md)):

| Crate | Licencia | Función |
|---|---|---|
| `osatura-core` | MPL-2.0 | motor, formatos, herramientas, la fachada estable `api` ([API.md](API.md)); features: `scripting` (por defecto, Rhai) |
| `osatura-cli` | AGPL-3.0-or-later | el binario `osatura`, servidor del studio, servidor de colaboración; features: `scripting` (por defecto), `gpu` |
| `osatura-capi` | MPL-2.0 | ABI de C ([C_ABI.md](C_ABI.md)) |
| `osatura-wasm` | MPL-2.0 | ABI de WebAssembly + wrapper de JS ([WASM.md](WASM.md)) |
| `osatura-gpu` | MPL-2.0 | renderizador wgpu opcional; **no** es miembro por defecto |
| `osatura-gui` | AGPL-3.0-or-later | `osatura-studio`, el estudio en una ventana nativa (`tao` + `wry`, solo Windows y macOS; un stub en el resto); **no** es miembro por defecto, `publish = false` |

Archivos del nivel del repositorio: `ROADMAP.md` (la sección 9 es el registro fechado de funciones y revisiones), `FEATURES.md`, `CLEANROOM.md`, `LICENSING.md`, `REUSE.toml`, `deny.toml`, `LICENSES/`, `scripts/`.

## 2. Comandos de compilación y prueba

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

Features y partes opcionales:

```sh
cargo build -p osatura-cli --no-default-features         # no Rhai (no `script` command, no smartstring)
cargo build -p osatura-core --no-default-features        # what capi, wasm and gpu use
cargo build -p osatura-cli --features gpu                # adds `render-gpu` (wgpu)
cargo test  -p osatura-gpu                               # GPU tests (31: 18 unit + 13 parity); NOT in the workspace default
OSATURA_GPU_BACKEND=vulkan osatura render-gpu ...     # restrict the wgpu backend: vulkan or gl
```

Reglas prácticas: usa compilaciones de depuración para las suites de pruebas (el repositorio se desarrolló teniendo en cuenta el espacio en disco: no copies `target/`); usa `--release` solo para cuestiones de tiempos. La instantánea de la fachada se bendice con `OSATURA_BLESS=1 cargo test -p osatura-core api::` (sección 5). `OSATURA_UI_CONFIG` selecciona en las pruebas el archivo de ajustes de la interfaz del studio, `OSATURA_AUTHOR` el nombre del autor en los commits del control de versiones, y `OSATURA_E2E_TIMINGS=<file>` hace que la prueba e2e escriba los tiempos de cada etapa.

## 3. Las suites de pruebas

Cifras de la sección 9 de `ROADMAP.md` y de una ejecución completa de `cargo test --workspace --offline` el 22 de octubre de 2026 para esta página (perfil de depuración, en una máquina compartida con otros trabajos, por lo que los tiempos son orientativos y **serán más lentos que en una máquina inactiva**):

| Suite | Pruebas | Tiempo (esta ejecución) | Qué cubre |
|---|---:|---:|---|
| `osatura-core` unit (`src/lib.rs`) | 1167 | 349 s | todos los módulos; incluye `api::tests::api_surface_snapshot` |
| `osatura-core/tests/prelude_surface.rs` | 6 | 0.7 s | la fachada desde fuera del crate (`#[non_exhaustive]`, correspondencia de errores, un subconjunto representativo de llamadas) |
| `osatura-cli` unit (`src/usage_tests.rs`, `src/args.rs`, studio, collab, http) | 80 | 42 s | análisis de opciones, USAGE frente al analizador, servidor HTTP, política, registro de colaboración |
| `tests/e2e.rs` | 2 | 214 s | todo el pipeline mediante el binario real (unos 19 s en una compilación de depuración con la máquina inactiva y 40 s en release según el ROADMAP; aquí la máquina estaba cargada) |
| `tests/fixes.rs`, `tests/wave4.rs`, `tests/robust.rs` | 7, 15, 10 | 19 s, 1.2 s, 1.2 s | regresiones de las oleadas de revisión y pruebas con entradas hostiles |
| `tests/doc_mismatch.rs` | 20 | 1.8 s | regresiones de documentación/comportamiento: salidas atómicas, opciones estrictas, la matriz de aceptación de `--json`/`--quiet`/`--allow-empty`, reglas de escala de `golden` |
| `tests/source_offer.rs` | 3 | 0.2 s | oferta de código fuente `/source` de la sección 13 de la AGPL |
| `tests/face_cli.rs` | 1 | 0.8 s | CLI de seguimiento facial |
| `osatura-core/tests/wasm_safety.rs` | 2 | 0.1 s | ningún uso directo de `Instant` / `SystemTime` / `process::id` / `thread::spawn` fuera de `clock.rs` (análisis del código fuente) |
| `osatura-cli/tests/wave7.rs` | 7 | 7 s | oleada 7: opciones por subcomando, `follow` / `make-loop`, `color-space`, `collab-token`, archivo de secreto del studio, rango de fotogramas de `dof` / `render-hdr` / `fx` |
| `tests/spdx_audit.rs` | 2 | 0.01 s | cada `.rs/.js/.mjs/.sh` tiene una línea SPDX igual a la licencia de su crate; REUSE.toml cubre el resto |
| `osatura-capi/tests/abi.rs` | 7 | 1.1 s | la ABI de C a través del lado Rust (prueba de humo en C: `osatura-capi/tests/smoke.c`, compilada a mano, véase [C_ABI.md](C_ABI.md)) |
| `osatura-wasm` unit | 8 | 1.6 s | las funciones de la ABI en el host |
| `osatura-gpu` unit + `tests/parity.rs` | 18 + 13 | 0.2 s + 6.5 s | paridad GPU frente a CPU (adaptador por software cuando no hay GPU) |
| Doc-tests (`osatura-core`) | 19 | 171 s (cada uno compila un programa) | los ejemplos de la fachada (`api/`, `lib.rs`); `osatura-capi` y `osatura-wasm` no tienen ninguno |

Total: 1343 pruebas en los binarios unitarios y de integración y 19 doc-tests, contadas el 24 de octubre de 2026 con `cargo test --workspace --exclude osatura-gpu --exclude osatura-gui` (las 1368 de la frase siguiente incluían también las pruebas del crate de GPU; recontada a partir de una ejecución completa de `cargo test --workspace` el 22 de octubre de 2026 tras la oleada 7; los tiempos por suite de arriba proceden de una ejecución anterior), más 19 doc-tests; el conjunto de `cargo test --workspace --offline` tardó 836 s en la máquina compartida.

Suites fuera de `cargo test`:

* **Pruebas de humo del studio (Chromium).** Cinco scripts de Node manejan el studio real en Chromium sin interfaz con `playwright-core`: `studio_ui_smoke.js` (128 comprobaciones), `studio_collab_smoke.js` (60, dos páginas en una misma máquina), `studio_overlays_smoke.js` (89), `studio_suggest_smoke.js` (35) y `studio_demo_smoke.js` (25: la demo guiada carga los 13 pasos, nunca escribe, restaura el proyecto byte a byte y se recupera de una pestaña cerrada); los cinco pasaron el 24 de octubre de 2026 (cifras tal como las imprimen los scripts). **No** los ejecuta `cargo test` y no forman parte de la CI por defecto. Ejecución:

  ```sh
  cargo build -p osatura-cli
  node osatura-cli/tests/studio_ui_smoke.js        # same for collab / overlays / suggest / demo
  ```

  Entorno: `OSATURA_BIN` (por defecto `target/debug/osatura`), `PLAYWRIGHT_CORE`, `CHROME` y, para el script de overlays, `SHOTS` (directorio de capturas de pantalla). Los scripts limpian sus archivos temporales.
* **Prueba del wrapper WASM.** `node osatura-wasm/js/test.mjs` (solo con mock, a menos que indiques la ruta de un proyecto y hayas compilado el módulo; véase [WASM.md](WASM.md)).
* **Auditoría de licencias.** `scripts/licence-audit.sh`: reconstruye `docs/THIRD_PARTY.md` y `docs/THIRD_PARTY_NOTICES.md` a partir de `cargo metadata --locked --all-features` (dependencias normales y de compilación, sin las de desarrollo), comprueba la línea SPDX de cada archivo fuente con la `license` de su crate y después ejecuta `cargo deny check licenses bans sources`, salvo con `--no-deny`. `deny.toml` permite licencias permisivas y MPL-2.0 solo para crates concretos.
* **clippy.** `cargo clippy --workspace --all-targets -- -D warnings` termina con 0 y sin avisos (oleada 8) y es una condición de la CI. Cuando un lint no es adecuado para el caso, el código lleva un `#[allow(clippy::...)]` acotado con un comentario (bucles con índices sobre arrays paralelos, `too_many_arguments` en firmas estables, `manual_clamp` donde el tratamiento de NaN difiere, nombres inherentes `next` / `mul` que son API pública); no añadas allows a nivel de crate.
* **Integración con la CI.** El repositorio incluye `osatura ci` para comprobaciones de *animación* ([CI.md](CI.md)); es algo distinto de la CI del código. El proyecto no tiene ninguna configuración de CI de código en el árbol; la sección 8 ofrece una receta mínima.

Cómo añadir pruebas: coloca las pruebas del motor junto al código (`#[cfg(test)]`), las pruebas de comportamiento de la CLI en `osatura-cli/tests/`, y cada error corregido recibe una prueba de regresión (las oleadas de revisión de [CHANGELOG.md](CHANGELOG.md) enumeran qué se corrigió y dónde). Las pruebas no deben dejar archivos temporales (usa guardas de drop) ni depender del reloj ni de la red. Los presupuestos de tiempo se prueban con un reloj de pruebas determinista.

## 4. Convenciones de código

* **Sin pánicos ante la entrada.** Todo lo que lee un proyecto, un archivo, un argumento o un byte de red devuelve un error. La indexación, el troceado, `unwrap` y el desbordamiento aritmético con valores no fiables son errores. Además, la fachada (`api/`) captura los pánicos y devuelve `ApiError::Internal` (solo donde el destino hace unwinding), pero es la última línea de defensa, no un diseño.
* **Política de valores finitos.** Los números procedentes de archivos y argumentos se comprueban frente a NaN e infinito y frente a magnitudes razonables (los rangos de la fachada están en [API.md](API.md): magnitudes hasta 1e7, fps en (0, 1000], fotogramas 1..=200000, escenario hasta 32768, 4096 huesos, lado de renderizado hasta 16384 px). Rechaza, no recortes, salvo que la función esté documentada como recortadora (los fotogramas se recortan a la línea de tiempo).
* **Comprueba los tamaños antes de reservar memoria.** Los búferes de píxeles, los tamaños de malla y el número de muestras se calculan con aritmética comprobada y se rechazan antes de `vec![..]`.
* **Escrituras atómicas.** Escribe los resultados con `fsutil::write_atomic` (archivo temporal en el mismo directorio, `fsync`, renombrado; rechaza los enlaces simbólicos salvo con `--follow-symlinks`; recurre a una sobrescritura in situ con `fsync`, documentada como no atómica, cuando el directorio no admite escritura). Nunca uses `std::fs::write` directamente sobre un archivo del usuario. Los directorios temporales proceden de `fsutil` (modo 0700, archivos 0600). Los bytes aleatorios proceden de `fsutil::random_bytes` (`/dev/urandom`).
* **Los recursos pasan por `assets::resolve`.** Las rutas relativas de imágenes, mapas de normales y sonidos se resuelven respecto al directorio de recursos de todo el proceso (`assets::default_dir`, fijado desde la carpeta del proyecto de entrada); las rutas absolutas y `..` se rechazan salvo con `--allow-external-assets`; los archivos de imagen se limitan a 64 MB y los de audio a 256 MB. Los comandos que escriben un proyecto derivado validan respecto al directorio del proyecto de *entrada* y reescriben las rutas de recursos relativas o avisan sobre ellas (`write_json`).
* **Aleatoriedad determinista.** Todo comportamiento aleatorio (fuzzing, ruido, jitter, simulación) usa el generador con semilla del propio motor, con una opción `--seed`, y es reproducible bit a bit. Nada de crate `rand` ni de hora del sistema como semilla.
* **Println y tuberías cerradas.** La CLI sustituye `println!`/`print!` por macros que terminan sin ruido cuando la tubería está cerrada (`osatura ... | head` no debe entrar en pánico). Sigue usándolas en los módulos declarados después de las macros en `main.rs`.
* **Los errores dicen qué hacer.** Los mensajes indican qué estaba mal, qué valor y el rango permitido o la solución (las pruebas de la fachada comprueban este estilo).
<!-- REUSE-IgnoreStart -->
* **Cabeceras SPDX.** Todo archivo `.rs`, `.js`, `.mjs` y `.sh` empieza con `SPDX-License-Identifier:` igual a la licencia de su crate (MPL-2.0 para core/capi/wasm/gpu/scripts, AGPL-3.0-or-later para la CLI) y una línea de copyright; los documentos Markdown llevan el comentario de cabecera CC-BY-4.0, como este archivo. Los demás archivos (JSON, imágenes, los textos de licencia) los cubre `REUSE.toml`; cuando añadas un patrón de ruta nuevo, añádelo ahí. En caso contrario falla `cargo test -p osatura-cli --test spdx_audit`.
<!-- REUSE-IgnoreEnd -->
* **Dependencias.** Pocas, en Rust puro y con licencia permisiva. Un crate nuevo necesita una comprobación en `deny.toml` (`scripts/licence-audit.sh`), una línea en `docs/THIRD_PARTY.md` (regenerado) y una justificación en la entrada del ROADMAP. `osatura-core` se compila sin features por defecto desde capi, wasm, gpu y la CLI, así que todo lo pesado va detrás de una feature.
* **Primero la fachada.** Toda capacidad nueva del motor visible para el usuario a la que deban poder acceder otros lenguajes va a `osatura-core/src/api/` con una línea de instantánea, un doc-test y una prueba unitaria; las capas de C y wasm solo envuelven la fachada.

## 5. Lista de comprobación para un módulo o comando nuevo

1. **Módulo** en `osatura-core/src/<name>.rs`, registrado en `lib.rs`; cabecera SPDX; pruebas unitarias (válidas, límites, entradas hostiles, determinismo); sin pánicos.
2. **Comando de CLI**: una entrada en `USAGE` (ortografía exacta de cada opción) en `osatura-cli/src/usage.rs`, la rama del despacho en uno de `osatura-cli/src/cmds_1.rs` a `cmds_6.rs` (o en `main.rs` para `ui`), y las tablas de opciones en `osatura-cli/src/args.rs`: toda opción que reciba un valor debe figurar en `VALUE_OPTS`; los flags que no reciben valor, en la entrada `CMD_OPTS` del comando cuando la tabla global sería incorrecta (por ejemplo `ci --prune` es un flag, `export-oca --layers` y `autotrace --png` reciben valores, `keyedit --free` es un flag). Las pruebas unitarias de USAGE frente al analizador fallan si el texto y el analizador no coinciden. Admite los flags globales (`--json`, `--quiet`, `--allow-empty`, `--allow-external-assets`, `--follow-symlinks`, `--max-input-bytes`), `<command> --help` y `--key=value`.
3. **Escritura de la salida** solo con `fsutil::write_atomic` o `write_json`; los comandos que no producen nada terminan con código distinto de cero salvo con `--allow-empty`.
4. **Documentación**: el comando en [CLI_REFERENCE.md](CLI_REFERENCE.md) si añades uno, una mención en la guía donde corresponda y la entrada en la sección 9 de `ROADMAP.md` (qué se añadió, qué *no* está hecho: esa lista alimenta [LIMITATIONS.md](LIMITATIONS.md)).
5. **Fila en CLEANROOM.md**: por cada algoritmo nuevo, registra la fuente pública a partir de la cual se implementó (libro de texto, norma, artículo) y la derivación independiente; sin fila, no hay merge.
6. **Volver a bendecir la superficie de la fachada** si `api/` cambió: ejecuta `OSATURA_BLESS=1 cargo test -p osatura-core api::`, revisa el diff de `osatura-core/src/api_surface.txt` (una línea eliminada o modificada es un cambio incompatible y exige una versión mayor de la fachada; una línea añadida es una menor) y vuelve a ejecutar las pruebas sin la variable. Actualiza [API.md](API.md), y los wrappers de C/WASM y sus cabeceras si deben exponerlo.
7. **De extremo a extremo**: añade el comando al pipeline e2e si pertenece a la ruta de producción; añade una comprobación de humo a los scripts del studio si tiene interfaz.
8. **Ejecución**: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/licence-audit.sh` (cuando cambien las dependencias o las cabeceras).

## 6. Reglas clean-room y DCO

El proyecto es una implementación independiente. En resumen (el texto vinculante es [CLEANROOM.md](CLEANROOM.md) y [CONTRIBUTING.md](CONTRIBUTING.md)):

* Implementa a partir de normas públicas, libros de texto y artículos, y cítalos. No leas, descompiles, copies ni aplices ingeniería inversa al código, los archivos o las interfaces de herramientas de animación propietarias, y no leas ni pegues código fuente de herramientas de la competencia mientras escribes una función. No pegues código de proyectos con una licencia incompatible.
* Registra cada método nuevo en la tabla de `CLEANROOM.md` con su fuente pública.
* Las notas sobre patentes están en [PATENT_NOTES.md](PATENT_NOTES.md); una función que afecte a un área sensible en materia de patentes se discute primero allí. Esto no es asesoramiento jurídico.
* Las contribuciones se aceptan bajo la licencia de la parte que modifican (entrada igual a salida): MPL-2.0 para las bibliotecas y los scripts, AGPL-3.0-or-later para `osatura-cli`, CC-BY-4.0 para la documentación y los recursos sintéticos.
* **Sign-off.** Cada commit necesita `Signed-off-by: Name <email>` (`git commit -s`), que certifica el Developer Certificate of Origin 1.1 (texto en CONTRIBUTING.md). El sign-off certifica únicamente el origen: **no** concede ningún derecho de relicenciamiento, y no es posible ninguna licencia comercial o dual para el código aportado mientras no exista un CLA.
* El nombre y el logotipo están cubiertos por [TRADEMARK.md](TRADEMARK.md).

## 7. Lista de comprobación de publicación

La última versión preliminar publicada es `v0.1.0-alpha.4`; la siguiente se prepara como `v0.1.0-alpha.5` (todos los crates siguen en 0.1.0; las etiquetas de versión "v0.5/v0.6/v0.7" de README, FEATURES y ROADMAP son niveles de funciones, no versiones de crates; véase [CHANGELOG.md](CHANGELOG.md)). Para preparar una:

1. **Versión.** Decide la versión de cada crate. La fachada tiene su propia versión (`API_VERSION`, hoy 1.4.0: mayor al eliminar o cambiar una línea de la superficie, menor al añadir) y `OSATURA_ABI_VERSION` / `arm_abi_version()` (ambas 1) cambian únicamente ante un cambio incompatible de la ABI de C o de wasm. Alinea las versiones de `Cargo.toml`, la salida de `osatura --version`, el recuadro de estado del README y el CHANGELOG.
2. **Controles en verde.** `cargo test --workspace`, `cargo test --doc -p osatura-core`, `cargo test -p osatura-cli --test spdx_audit`, `cargo clippy --workspace --all-targets -- -D warnings`, las cinco pruebas de humo de Chromium, `scripts/e2e.sh` (release), `cargo test -p osatura-gpu` si cambió el código de GPU.
3. **Avisos de terceros.** Ejecuta `scripts/licence-audit.sh`: regenera `docs/THIRD_PARTY.md` (inventario) y `docs/THIRD_PARTY_NOTICES.md` (textos de licencia de todo lo distribuido), comprueba las líneas SPDX y ejecuta `cargo deny`. Haz commit de los archivos regenerados. Una distribución binaria debe incluir el paquete de avisos junto al binario.
4. **Archivos de licencia.** `COPYRIGHT`, `LICENSES/` (textos de todas las licencias utilizadas; los archivos `LICENSE-*` de la raíz son copias literales de ellos) y `REUSE.toml` están sincronizados con [LICENSING.md](LICENSING.md); la atribución CC-BY-4.0 de la documentación está intacta.
5. **Sin ffmpeg.** No incluyas, enlaces ni descargues ffmpeg en los artefactos de publicación. Se detecta en el `PATH` en tiempo de ejecución y la documentación indica que lo instala el usuario.
6. **Oferta de código fuente AGPL.** El binario de `osatura-cli` sirve su propia oferta de código fuente en `/source` (sección 13 de la AGPL) e imprime un aviso con `--version`; `release.yml` fija `OSATURA_SOURCE_URL` en tiempo de compilación con `<server>/<repository>/tree/<commit>` para que los binarios de la versión nombren el código fuente exacto desde el que se compilaron (la prueba de humo del flujo de trabajo comprueba que `--version` lo muestre; un simple `cargo build` no tiene ninguna dirección integrada); quienes operen servidores del studio modificados fijan `--source-url` a la dirección donde esté su código fuente modificado. Pruebas: `tests/source_offer.rs`. Para una publicación binaria, proporciona el código fuente correspondiente (etiqueta, tarball) en la URL a la que puedan acceder los usuarios.
7. **Documentación.** Actualiza el recuadro de estado del README, [LIMITATIONS.md](LIMITATIONS.md) y el CHANGELOG (traslada *Unreleased* bajo la nueva versión con una fecha).
8. **Compila el borrador en GitHub.** `.github/workflows/release.yml` se ejecuta **solo** al enviar una etiqueta `vMAJOR.MINOR.PATCH[-pre]` (o a mano desde `main`, lo que compila pero no crea ninguna versión). Un primer trabajo comprueba la grafía de la etiqueta y que su commit esté en `main`; después compila los archivos de Windows, macOS y Linux (glibc y musl) y el módulo web, firma todo con Sigstore, certifica la procedencia, genera el SBOM y `SHA256SUMS.txt`, y crea una *versión preliminar en borrador* (un borrador anterior con la misma etiqueta se sustituye). Revisa el borrador, compruébalo con [VERIFY_DOWNLOADS.md](VERIFY_DOWNLOADS.md) y luego publícalo a mano.
9. **Etiqueta y publicación.** Envía una etiqueta **firmada** (`git tag -s v0.1.0-alpha.5`) desde un commit que ya esté en `main`; el envío inicia el paso 8. No publiques crates con nombres que no sean tuyos; las reglas de TRADEMARK.md se aplican a los forks.

## 8. Integración continua

Dos cosas distintas comparten la palabra (un tercer workflow, `.github/workflows/release.yml`, compila los archivos de la versión y el borrador de la publicación, véase la sección 7; un cuarto, `codeql.yml`, analiza el código):

* **CI de animación** (`osatura ci`): comprobaciones de *proyectos* (lint, goldens de renderizado, rendimiento, determinismo). Se describe por completo en [CI.md](CI.md): manifiesto `osatura-ci.json`, umbrales, goldens (`--update` frente a `bless`), informes, ejemplos de GitHub Actions y de shell simple (marcados allí como "example, not run").
* **CI del código** de este repositorio: `.github/workflows/ci.yml` tiene cuatro tipos de trabajo: `test` (matriz ubuntu / windows / macos: `cargo test --workspace --exclude osatura-gpu --locked` y clippy con `-D warnings`), `wasm32` (compila `osatura-wasm` para `wasm32-unknown-unknown`, copia el módulo junto a `osatura.js` y ejecuta `node osatura-wasm/js/test.mjs` con un proyecto de demostración, incluida su parte con el módulo real), `gpu parity` (informativo; se omite si no existe ningún adaptador, se ejecuta en el adaptador de software DX12 de Windows) y `licences` (`cargo-deny check`, que ahora incluye avisos de seguridad, bans y crates retirados, y `reuse lint`). El toolchain está fijado en `rust-toolchain.toml` (1.97.0) para que los nuevos lints de clippy no rompan la compilación; además, cada trabajo ejecuta `rustup toolchain install` como paso de seguridad, y los workflows llevan grupos `concurrency` para que un push más reciente cancele la ejecución de CI anterior (nunca una publicación). A propósito no hay `paths-ignore`: un workflow omitido deja una comprobación obligatoria pendiente para siempre. El trabajo `test` también compila `osatura-gui` y ejecuta `osatura-studio --selftest` (una autoprueba real en Windows y macOS). **Primera ejecución completamente verde: 2026-10-24** en runners alojados en GitHub. Si hay un fallo, los pasos imprimen las pruebas fallidas como anotaciones del workflow (legibles sin acceso a los registros). Windows omite las pruebas exclusivas de POSIX (enlaces simbólicos, permisos, `/dev/zero`, separadores de ruta POSIX).

  ```sh
  cargo test --workspace --locked
  cargo test --doc -p osatura-core --locked
  cargo clippy --workspace --all-targets --locked -- -D warnings
  scripts/licence-audit.sh            # needs python3 and (optional) cargo-deny
  ```

  Guarda en caché `~/.cargo/registry` y `target/` entre ejecuciones, concede a la prueba e2e varios minutos en compilaciones de depuración y ejecuta las pruebas de humo de Chromium solo en runners que tengan Chromium y `playwright-core`.

## 9. Depuración

* **Backtraces.** `RUST_BACKTRACE=1 osatura ...` ante cualquier pánico (un pánico ante la entrada es un error; infórmalo junto con el proyecto que lo provoca).
* **Salida legible por máquina.** `--json` en los comandos que lo admiten; `--quiet` para scripts. Códigos de salida: 0 si hay éxito, 1 si hay fallo (comprobación fallida, presupuesto incumplido, uso incorrecto; run: un comando desconocido y un argumento ausente terminan ambos con 1); `osatura ci` distingue 1 (una comprobación falló) de 2 (error).
* **Inspeccionar un proyecto.** `osatura check`, `osatura lint` (termina con 1 si hay errores) y `osatura diff a.json b.json` ofrecen estructura y problemas sin renderizar; `osatura render project.json N out.png` renderiza un fotograma, y `out.svg` produce una salida vectorial que puedes abrir en un navegador (`--bones` muestra el esqueleto).
* **Studio.** Ejecuta `osatura studio project.json --port 0` (imprime el puerto real). Las herramientas para desarrolladores del navegador muestran la API JSON: las peticiones son simples `POST /api/...` con `Content-Type: application/json`. Un 421 indica una cabecera Host que no está en `--allow-host`; un 403 indica que falló una comprobación de Origin o de `Sec-Fetch-Site` (usa `--public-origin` detrás de un proxy); 415 indica el tipo de contenido; 429 es el retroceso de tokens.
* **Etapas de la demo.** La demo guiada del Studio (y la demo paso a paso del sitio del proyecto) se generan a partir de `osatura demo-person` con `scripts/build-studio-tour.mjs` y `scripts/lib/person-stages.mjs`; la compilación falla si la demo terminada ya no coincide con las cifras citadas en los textos (`EXPECT` en la biblioteca). Tras cambiar `osatura-core/src/demo.rs`, vuelve a ejecutar el script y recompila la CLI.
* **Registro de colaboración.** El registro de una sesión compartida (`--collab-log`) es texto, con una línea `<hash> <json>` por operación; una cadena rota se notifica al cargar.
* **Una prueba falla solo a veces.** Busca el reloj, los nombres de directorios temporales y el orden de iteración de `HashMap`; el código del motor es determinista por construcción, así que los fallos intermitentes son casi siempre un problema de higiene de las pruebas.
* **Falla la instantánea de la fachada.** Has cambiado una firma pública: véase la sección 5, punto 6.
* **GPU.** `osatura render-gpu <project> <out.png> --adapter software|auto --bench N` (solo en compilaciones con `--features gpu`) compara con el renderizador de CPU (un adaptador por software imprime que sus tiempos no significan nada); `OSATURA_GPU_BACKEND` selecciona el backend.
* **Sanitizers para la ABI de C.** Compila `smoke.c` o tu propio llamador con `-fsanitize=address,undefined` contra la biblioteca estática (hecho durante el trabajo de documentación, sin problemas; véase [C_ABI.md](C_ABI.md)).

## 10. Perfilado

* **Tiempos por etapa del pipeline de producción:** `scripts/e2e.sh` imprime una tabla (`OSATURA_E2E_TIMINGS` indica el archivo); las cifras de release de la sección 9 de `ROADMAP.md` proceden de ella.
* **GPU frente a CPU:** `osatura render-gpu <project> <out.png> --bench N`.
* **Presupuesto de tamaño y velocidad:** `osatura size-report`, `osatura export-fit --max-seconds` (el presupuesto de tiempo se comprueba dentro de los bucles de candidatos; una llamada a ffmpeg o al cuantizador no se puede interrumpir).
* **Coste de renderizado de un proyecto:** `osatura ci` con un umbral de rendimiento ([CI.md](CI.md)) registra el tiempo por fotograma en tu máquina; los tiempos dependen de la máquina, así que compara siempre en la misma.
* **Herramientas generales:** compila con `--release` y usa `perf record`/`perf report` o `cargo flamegraph` (no forman parte del proyecto; instálalas aparte). Las compilaciones de depuración son aproximadamente un orden de magnitud más lentas y no dicen nada sobre la velocidad real.
