<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# OSATURA

**Un motor de rigging y animación vectorial 2D en Rust: huesos, IK, skinning, deformadores, un runtime de máquinas de estados, una caja de herramientas de línea de comandos, un estudio en el navegador y bibliotecas integrables, todo en un único workspace.** Versión en rumano: [README.ro.md](README.ro.md).

![Un personaje de perfil (cabeza, oreja, coleta, tronco, brazos con dedos, piernas, pies) camina en el sitio sobre un suelo que se desplaza; salida real de osatura export con el proyecto de demostración incorporado](docs/img/walk.gif)

*Lo que produce: la persona que camina incluida, exportada con `osatura demo-person` y `osatura export` (todo el proyecto es un archivo JSON). Pruébalo en dos minutos: [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md).*

## Resumen y filosofía

OSATURA permite construir un personaje o una escena a partir de formas vectoriales e imágenes, dotarlo de rig con huesos y restricciones, animarlo con claves, clips y física, y exportarlo como GIF, vídeo, Lottie, glTF, un reproductor HTML autónomo o un runtime interactivo. Está formado por piezas pequeñas y deterministas: el mismo proyecto y la misma seed producen los mismos píxeles, cada herramienta lee y escribe un proyecto JSON documentado y cada comando funciona desde un terminal, de modo que la animación puede automatizarse con scripts, revisarse, compararse con diff y probarse como el código fuente.

Principios que dan forma al proyecto:

* **Implementación independiente.** Escrita a partir de especificaciones públicas, artículos científicos y matemáticas de manual. Se consultaron las listas de funciones de la competencia para determinar el *alcance*; no se copió código fuente de terceros y el proyecto no lee ni escribe ningún formato propietario. No es un proceso clean-room estricto (véase [CLEANROOM.md](docs/i18n/es/CLEANROOM.md), incluida su divulgación de la investigación) y no está afiliado a Moho ni a ningún otro producto comercial, ni deriva de ellos, ni es compatible con ellos. El código fue escrito en gran parte con un modelo de IA.
* **Estado honesto.** Cada función se describe junto con lo que *no* hace ([docs/LIMITATIONS.md](docs/i18n/es/LIMITATIONS.md), [ROADMAP.md](docs/i18n/es/ROADMAP.md) sección 9).
* **Seguro por defecto.** Sin panics ante la entrada, comprobaciones de valores finitos, límites de recursos, escrituras atómicas, rutas de recursos confinadas a la carpeta del proyecto ([docs/SECURITY.md](docs/i18n/es/SECURITY.md)).
* **Formatos abiertos primero.** OpenRaster, OCA, SVG, Lottie, glTF, APNG, WebM, EXR; ningún formato de archivo propietario por diseño.
* **Pocas dependencias**, con una política de licencias comprobada mediante pruebas y `cargo deny`.

## Estado

*Nombre: el proyecto se llamó ARMATURA hasta el 24 oct 2026 y pasó a llamarse OSATURA tras una búsqueda de marca ([docs/LIMITATIONS.md](docs/i18n/es/LIMITATIONS.md) L-78). También se cambiaron el nombre del repositorio y la dirección del sitio (los enlaces antiguos al repositorio redirigen).*

> **Alfa. Primeras compilaciones preparadas como versión preliminar en borrador: `v0.1.0-alpha.3`.** Los ejecutables para Windows x86-64, macOS arm64 y Linux x86-64 y el módulo WebAssembly los compila GitHub Actions y se adjuntan a una versión preliminar *en borrador* en <https://github.com/chiuta/osatura/releases> (un borrador solo lo ve el propietario hasta que lo publica a mano). Todos los crates siguen en la versión `0.1.0`; "v0.5 / v0.6 / v0.7" son *niveles de funciones* del historial, no números de versión ([CHANGELOG.md](docs/i18n/es/CHANGELOG.md)).
>
> | Verificado | No verificado |
> |---|---|
> | **En local (Linux x86-64, 24 oct 2026):** pasan 1338 pruebas de `cargo test --workspace --exclude osatura-gpu --exclude osatura-gui` y 19 doc-tests (el crate de GPU necesita un adaptador y el crate de ventana un escritorio; antes, el recuento que incluía el crate de GPU era 1368). **En GitHub Actions (24 oct 2026, toolchain 1.97.0):** la batería de pruebas y `clippy -D warnings` pasan en Linux, macOS (arm64) y Windows; `cargo deny` y `reuse lint` pasan; el módulo WebAssembly se compila para `wasm32-unknown-unknown` y la prueba del envoltorio pasa contra el módulo real con Node 20; los binarios de la versión superan una prueba de humo (`--version`, `demo`, `check`, `render`) y `osatura-studio --selftest` se ejecuta en Windows y macOS. Cuatro pruebas de humo del estudio con Chromium sin interfaz pasan; la CLI, la fachada de Rust y la ABI de C (también con ASan/UBSan) se ejecutaron con los ejemplos de la documentación | **Ninguna persona ha usado los ejecutables en un escritorio real de Windows o macOS**: el lanzador de doble clic de Windows y la aplicación de ventana `osatura-studio` (WebView2 / WKWebView) solo se compilaron y se autoprobaron en CI; los binarios **no están firmados** (SmartScreen "Editor desconocido", Gatekeeper de macOS); el binario de Linux se compiló solo en un runner de Ubuntu; **el módulo WebAssembly nunca se ha ejecutado en un navegador** y no se ha medido su velocidad ni su memoria; el estudio nunca fue usado por una persona en producción, ni en un navegador distinto de Chromium; nunca se usó una GPU real (solo un adaptador por software), ni una webcam, ni una fotografía, ni un archivo de captura de movimiento; sin auditoría de seguridad, revisión por un abogado ni búsqueda de patentes |
>
> Lee [docs/LIMITATIONS.md](docs/i18n/es/LIMITATIONS.md) (78 limitaciones numeradas con su gravedad y alternativa) antes de depender de cualquier cosa.

## Descarga

Los archivos precompilados, una vez que el propietario haya publicado la versión en borrador, están en la [página de Releases](https://github.com/chiuta/osatura/releases) del proyecto:

| Archivo | Contenido |
|---|---|
| `osatura-windows-x86_64.zip` | `osatura.exe` (línea de comandos + servidor del estudio), `osatura-studio.exe` (aplicación de ventana), biblioteca y cabecera de C, licencias, avisos de terceros |
| `osatura-macos-arm64.tar.gz` | `osatura`, `osatura-studio` (solo Apple silicon), biblioteca y cabecera de C, licencias, avisos |
| `osatura-linux-x86_64.tar.gz` | `osatura`, biblioteca y cabecera de C, licencias, avisos (sin aplicación de ventana: usa `osatura studio` y un navegador) |
| `osatura-web.tar.gz` | `osatura_wasm.wasm`, `osatura.js`, `index.html` de demostración (sin verificar en navegadores, véase [docs/WASM.md](docs/i18n/es/WASM.md)) |
| `SHA256SUMS.txt` | comprueba cada descarga con él |

* **Windows:** haz doble clic en `osatura.exe` para crear `Documents\OSATURA\first-rig.json` y abrir el estudio en tu navegador, o haz doble clic en `osatura-studio.exe` para tener una ventana propia. Ambos programas no están firmados: en SmartScreen elige *Más información* y después *Ejecutar de todas formas*. Véase [docs/GETTING_STARTED.md](docs/i18n/es/GETTING_STARTED.md).
* **macOS:** aprueba el programa en *Ajustes del Sistema > Privacidad y seguridad* la primera vez.
* **Código fuente:** cada archivo indica el commit a partir del cual se compiló; las partes AGPL se sirven con su oferta de código fuente (`osatura --version`, `/source`).

## Funciones por área

Detalles y opciones exactas: [docs/CLI_REFERENCE.md](docs/i18n/es/CLI_REFERENCE.md); qué es parcial o falta: [docs/LIMITATIONS.md](docs/i18n/es/LIMITATIONS.md).

* **Modelo de dibujo.** Trazados vectoriales, rellenos, contornos (guiones, ancho variable), degradados lineales y radiales, máscaras, modos de fusión, efectos por capa (desenfoque, sombra, tinte, relleno), capas de imagen con deformación de malla, metaballs líquidas, booleanas de polígonos, partículas deterministas, cámara multiplano con tomas guardadas, profundidad de campo.
* **Rigging.** Huesos con cinemática directa; IK de dos huesos, suave, con estiramiento y FABRIK; límites, restricciones, muelles; skinning con herramientas de pesos; deformadores de lattice, spine, morph, wobble y cage; trazado de malla a partir del alfa; pivotes de deformación y glue; drivers (deslizadores, joysticks, cambio de vista); conjuntos de skin; auto-rig a partir de nombres de capa; recorte de ojos; contenedores de maquetación y regiones de desplazamiento.
* **Animación.** Claves con suavizado Bézier/escalón/muelle/rebote, expresiones y canales dirigidos, capas de conmutación, takes y una pila de clips por capas, conversión a bucle y a sitio, bloqueo de pies, imanes, seguimiento de trayectoria, generadores de reposo y de solapamiento, física de cadenas/pelo/tela, lip-sync a partir de audio o de una transcripción, movimiento de cabeza adaptado al habla, retargeting de BVH, entrada en vivo (MIDI, ratón, gamepad, puntos de referencia de webcam que tú proporcionas), pistas de audio con recorte, fundido y mezcla.
* **Runtime.** Máquinas de estados con blend trees 1-D, enlace de datos, física de desplazamiento, nivel de detalle con cota de error, un sandbox de scripting Rhai, un servidor de agente (MCP) sobre operaciones de parche.
* **Salida.** SVG, PNG, secuencias PNG y hojas de sprites, GIF, APNG, MP4/WebM (mediante ffmpeg), reproductor HTML autónomo, Lottie y dotLottie, glTF 2.0 binario, OpenEXR/PNG de 16 bits, OCA; exportación con tamaño objetivo (`export-fit`). Importación: SVG, OpenRaster, Lottie, BVH, secuencias PNG/GIF/APNG.
* **Herramientas de calidad.** `check`, `lint`, `diff` semántico, fuzzing de poses con mapas de calor, CI de animación con diff perceptual y fotogramas de referencia ([docs/CI.md](docs/i18n/es/CI.md)), control de versiones con merge semántico (`vcs`), comprobaciones de seguridad para destellos y movimiento.
* **Estudio.** Editor en el navegador servido por la CLI: gizmos en el lienzo, línea de tiempo, dope sheet y editor de curvas, overlays, paleta de comandos y atajos, deshacer/rehacer, edición multiusuario en tiempo real con token opcional y registro persistente ([docs/STUDIO.md](docs/i18n/es/STUDIO.md)).
* **Bibliotecas.** Fachada estable de Rust ([docs/API.md](docs/i18n/es/API.md)), ABI de C ([docs/C_ABI.md](docs/i18n/es/C_ABI.md)), módulo WebAssembly y envoltorio de JavaScript (compilado y probado con Node en CI, aún no ejecutado en un navegador; [docs/WASM.md](docs/i18n/es/WASM.md)), renderizador wgpu opcional.

## Inicio rápido

Requisitos: una toolchain estable reciente de Rust (o un archivo precompilado de la sección [Descarga](#descarga): omite la línea `cargo build` y usa el `osatura` descomprimido). `ffmpeg` solo se necesita para exportar MP4/WebM y para audio mp3/ogg; nunca se incluye en el paquete. Todos los comandos siguientes se ejecutaron para este README (compilación debug, Linux). Las compilaciones release (`cargo build --release`) son mucho más rápidas para renderizar y se usan igual con `target/release`.

```sh
cargo build -p osatura-cli                       # builds target/debug/osatura
export PATH="$PWD/target/debug:$PATH"             # so that `osatura` is found

osatura --version                                # version, licence, source offer, support links
osatura demo demo.json                           # write the built-in rig demo (640x480, 48 frames)
osatura check demo.json                          # validate: ok: 'OSATURA demo' 640x480 @ 24 fps, 48 frames, 6 bones...
osatura lint demo.json                           # static rig/animation checks: 0 finding(s), 0 error(s)
osatura render demo.json 12 frame12.png --bones  # one frame, with the skeleton overlay
osatura render demo.json 12 frame12.svg          # the same frame as vector output
osatura frames demo.json frames_out --png        # every frame as a PNG
osatura export demo.json demo.gif                # also .html (player), .json (Lottie), .lottie, .mp4 (ffmpeg)
osatura demo-fx fx.json                          # second demo: liquid, booleans, particles
osatura lipsync demo.json voice.wav mouth out.json   # mouth shapes from a WAV file (switch layer "mouth")
osatura studio demo.json --port 8080             # browser editor at http://127.0.0.1:8080
osatura ci init demo.json && osatura ci run     # animation CI: goldens + checks (about 40 s in a debug build)
```

`voice.wav` es cualquier archivo WAV mono o estéreo que proporciones. El estudio escucha solo en `127.0.0.1` y rechaza las peticiones dirigidas a otros nombres de host; para compartirlo, lee antes [docs/SECURITY.md](docs/i18n/es/SECURITY.md).

Usa las bibliotecas:

```sh
cargo build -p osatura-capi                      # target/debug/libosatura_capi.so and .a
gcc -std=c99 -o smoke osatura-capi/tests/smoke.c -Ltarget/debug -losatura_capi
LD_LIBRARY_PATH=target/debug ./smoke demo.json    # abi 1, 48 frames, 320x240, rc=0, checksum=...
```

La fachada de Rust es `osatura_core::prelude` ([docs/API.md](docs/i18n/es/API.md)); hay más ejemplos en C y Python en [docs/C_ABI.md](docs/i18n/es/C_ABI.md).

Pruebas (unos 14 minutos en una compilación debug en una máquina compartida; véase [docs/DEVELOPMENT.md](docs/i18n/es/DEVELOPMENT.md)):

```sh
cargo test --workspace
cargo test --doc -p osatura-core
cargo test -p osatura-cli --test spdx_audit
```

## Estructura del repositorio

| Ruta | Contenido | Licencia |
|---|---|---|
| `osatura-core/` | motor, formatos, herramientas, fachada estable `api` | MPL-2.0 |
| `osatura-cli/` | el binario `osatura`, estudio y servidor de colaboración, pruebas de extremo a extremo y de humo | AGPL-3.0-or-later |
| `osatura-gui/` | `osatura-studio`, el estudio en una ventana nativa (Windows y macOS; un stub en Linux) | AGPL-3.0-or-later |
| `osatura-capi/` | ABI de C (`osatura.h`, biblioteca compartida y estática) | MPL-2.0 |
| `osatura-wasm/` | ABI de WebAssembly, envoltorio de JavaScript, página de demostración (compilado en CI) | MPL-2.0 |
| `osatura-gpu/` | renderizador wgpu opcional (no es un miembro por defecto) | MPL-2.0 |
| `docs/` | documentación (CC-BY-4.0) y archivos generados de terceros | CC-BY-4.0 |
| `.github/workflows/` | CI (`ci.yml`) y compilaciones de versiones (`release.yml`) | MPL-2.0 |
| `scripts/` | `e2e.sh`, `licence-audit.sh` | MPL-2.0 |
| `LICENSES/`, `REUSE.toml`, `deny.toml` | textos de licencia, metadatos de licencia, política de dependencias | - |
| `ROADMAP.md`, `FEATURES.md`, `CLEANROOM.md`, `LICENSING.md`, `TRADEMARK.md`, `CONTRIBUTING.md`, `CHANGELOG.md` | documentos del proyecto | CC-BY-4.0 |

## Mapa de la documentación

Empieza en [docs/INDEX.md](docs/i18n/es/INDEX.md) (todos los documentos, destinatarios, rutas de lectura).

| Página | Para |
|---|---|
| [docs/GETTING_STARTED.md](docs/i18n/es/GETTING_STARTED.md) | primer proyecto, renderizado, exportación |
| [docs/CLI_REFERENCE.md](docs/i18n/es/CLI_REFERENCE.md) | todos los comandos y opciones |
| [docs/STUDIO.md](docs/i18n/es/STUDIO.md) | el editor en el navegador |
| [docs/RIGGING_GUIDE.md](docs/i18n/es/RIGGING_GUIDE.md) | construcción de rigs |
| [docs/ANIMATION_GUIDE.md](docs/i18n/es/ANIMATION_GUIDE.md) | animación, clips, física, audio |
| [docs/PROJECT_FORMAT.md](docs/i18n/es/PROJECT_FORMAT.md) | el formato de proyecto JSON |
| [docs/EXPORT_FORMATS.md](docs/i18n/es/EXPORT_FORMATS.md) | qué conserva y qué descarta cada formato |
| [docs/ARCHITECTURE.md](docs/i18n/es/ARCHITECTURE.md) | diseño y módulos |
| [docs/GLOSSARY.md](docs/i18n/es/GLOSSARY.md) | términos |
| [docs/FAQ.md](docs/i18n/es/FAQ.md) | preguntas frecuentes |
| [docs/API.md](docs/i18n/es/API.md) | fachada de Rust 1.4 |
| [docs/C_ABI.md](docs/i18n/es/C_ABI.md) | interfaz de C |
| [docs/WASM.md](docs/i18n/es/WASM.md) | WebAssembly (compilado en CI, no ejecutado en un navegador) |
| [docs/CI.md](docs/i18n/es/CI.md) | CI de animación |
| [docs/DEVELOPMENT.md](docs/i18n/es/DEVELOPMENT.md) | compilación, pruebas, convenciones, publicación |
| [docs/SECURITY.md](docs/i18n/es/SECURITY.md) | modelo de amenazas y refuerzo de la seguridad |
| [docs/LIMITATIONS.md](docs/i18n/es/LIMITATIONS.md) | limitaciones conocidas |
| [docs/PATENT_NOTES.md](docs/i18n/es/PATENT_NOTES.md), [docs/THIRD_PARTY.md](docs/i18n/es/THIRD_PARTY.md), [docs/THIRD_PARTY_NOTICES.md](docs/THIRD_PARTY_NOTICES.md) | patentes, licencias de las dependencias |
| [CHANGELOG.md](docs/i18n/es/CHANGELOG.md), [ROADMAP.md](docs/i18n/es/ROADMAP.md), [FEATURES.md](docs/i18n/es/FEATURES.md) | historial, plan y estado detallado, matriz de funciones |
| [CLEANROOM.md](docs/i18n/es/CLEANROOM.md), [LICENSING.md](docs/i18n/es/LICENSING.md), [TRADEMARK.md](docs/i18n/es/TRADEMARK.md), [CONTRIBUTING.md](docs/i18n/es/CONTRIBUTING.md) | procedencia, licencias, política del nombre, normas de contribución |

## Resumen de licencias

* **MPL-2.0** para las bibliotecas `osatura-core`, `osatura-capi`, `osatura-wasm`, `osatura-gpu` y para `scripts/`: puedes enlazarlas en tu propio programa, abierto o cerrado; los cambios en esos archivos deben publicarse bajo la MPL.
* **AGPL-3.0-or-later** para `osatura-cli` (la línea de comandos y el estudio) y `osatura-gui` (la ventana que lo envuelve). Si lo modificas y permites que otros interactúen con él a través de una red, la sección 13 te obliga a ofrecerles tu código fuente: `osatura studio --source-url URL` publica dónde se encuentra; `/source` y `osatura --version` incluyen la oferta.
* **CC-BY-4.0** para la documentación y los recursos sintéticos (incluido el modelo sintético de detección de rostros).
* **Tus proyectos y renders siguen siendo tuyos.**
* El nombre y el logotipo no están cubiertos por estas licencias ([TRADEMARK.md](docs/i18n/es/TRADEMARK.md)). El estado de las patentes no se ha evaluado ([docs/PATENT_NOTES.md](docs/i18n/es/PATENT_NOTES.md)). Una licencia comercial o dual para el código aportado no es posible sin un acuerdo de licencia de colaborador, y no existe ninguno.
* Crates de terceros: 363 en el inventario del 24 oct 2026 (todas las plataformas y targets juntos, lo que incluye los crates solo de Windows / macOS / Linux de la aplicación de ventana; un binario concreto contiene un subconjunto), todos permisivos salvo seis crates MPL-2.0 de copyleft débil: `smartstring` tras la función opcional `scripting`, y cinco (`cssparser`, `cssparser-macros`, `dtoa-short`, `selectors`, `option-ext`) que solo la aplicación de ventana `osatura-gui` puede incorporar ([docs/THIRD_PARTY.md](docs/i18n/es/THIRD_PARTY.md)). ffmpeg nunca se incluye en el paquete.

Véanse [LICENSE](LICENSE), [LICENSES/](LICENSES/) y [LICENSING.md](docs/i18n/es/LICENSING.md). Nada de esto es asesoramiento jurídico; sigue pendiente la revisión por un abogado.

## Contribuir

Las contribuciones son bienvenidas bajo la licencia de la parte que modifiquen. Lee [CONTRIBUTING.md](docs/i18n/es/CONTRIBUTING.md) (cada commit necesita una línea `Signed-off-by`, DCO 1.1), [CLEANROOM.md](docs/i18n/es/CLEANROOM.md) (implementa solo a partir de fuentes públicas; no copies de código propietario ni con una licencia incompatible) y [docs/DEVELOPMENT.md](docs/i18n/es/DEVELOPMENT.md) (convenciones, lista de comprobación para un módulo nuevo, pruebas). Buenas primeras contribuciones: una corrección de [docs/LIMITATIONS.md](docs/i18n/es/LIMITATIONS.md), una ejecución real de la lista de comprobación de WebAssembly, una prueba en un escritorio real de los ejecutables de Windows / macOS, un benchmark en una GPU real.

Problemas de seguridad: no abras una incidencia pública; sigue [docs/SECURITY.md](docs/i18n/es/SECURITY.md) sección 10.

## Apoyo

OSATURA está desarrollado por Alexandru-Ionuț Chiuță (Alexio). Si te resulta útil, puedes apoyar el trabajo:

* Buy Me a Coffee: <https://buymeacoffee.com/echo.of.the.strings>
* Patreon: <https://patreon.com/c/alexio_tf>

Contacto del mantenedor (también lo imprime `osatura --version`): alexio@trom.tf.

## Agradecimientos

* Los autores de los artículos públicos y los libros de texto que siguen los algoritmos, citados por módulo en [CLEANROOM.md](docs/i18n/es/CLEANROOM.md) (matemáticas de Bézier y splines, cinemática inversa, skinning por mezcla lineal, coordenadas de valor medio, ajuste de curvas y otros).
* Las especificaciones abiertas que OSATURA lee y escribe: SVG, OpenRaster, OCA, Lottie/dotLottie, glTF, APNG, WebM, OpenEXR, BVH, PNG, GIF.
* Los crates de Rust en los que se apoya, enumerados con sus licencias en [docs/THIRD_PARTY.md](docs/i18n/es/THIRD_PARTY.md), en particular `tiny-skia`, `serde`, `png`, `zip`, `rhai` y `wgpu`.
* Todas las personas que apoyan el proyecto a través de los enlaces anteriores.
