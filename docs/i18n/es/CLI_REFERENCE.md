<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# Referencia de la línea de comandos de OSATURA

Referencia del binario `osatura` (crate `osatura-cli`, AGPL-3.0-or-later; enlaza el motor `osatura-core`, con licencia MPL-2.0).
Versión documentada: **osatura 0.1.0**. Este texto se publica bajo licencia CC-BY-4.0.

Cada sinopsis de abajo está copiada del propio texto de uso del programa (`osatura` sin argumentos, o
`osatura <command> --help`). Cada ejemplo se ejecutó con el proyecto de demostración integrado y la salida mostrada
es un extracto de la salida real. Cuando el texto de uso y el comportamiento real discrepan, este documento describe el
comportamiento real y enumera la discrepancia en [Discrepancias conocidas entre documentación y comportamiento](#discrepancias-conocidas-entre-la-documentación-y-el-comportamiento) al final.

Documentos relacionados: [EXPORT_FORMATS.md](EXPORT_FORMATS.md) (qué conserva y qué descarta cada formato de exportación/importación),
[CI.md](CI.md) (manifiestos de CI de animación), [API.md](API.md) (biblioteca/ABI de C), [PATENT_NOTES.md](PATENT_NOTES.md)
(estado de códecs y patentes, no es asesoramiento jurídico).

## Contenido

1. [Convenciones comunes a todos los comandos](#1-convenciones-comunes-a-todos-los-comandos)
2. [Primeros pasos: el proyecto de demostración](#2-primeros-pasos-el-proyecto-de-demostración)
3. [Creación e inspección de proyectos](#3-creación-e-inspección-de-proyectos)
4. [Renderizado y exportación](#4-renderizado-y-exportación)
5. [Rigging, skins, IK y utilidades de maquetación](#5-rigging-skins-ik-y-utilidades-de-maquetación)
6. [Animación, keys, línea de tiempo y generadores](#6-animación-keys-línea-de-tiempo-y-generadores)
7. [Física, tela y deformadores](#7-física-tela-y-deformadores)
8. [Audio, lip-sync y sonidos](#8-audio-lip-sync-y-sonidos)
9. [Importación (SVG, OpenRaster, Lottie, BVH, GIF/APNG, secuencias PNG, mapas de bits)](#9-importación)
10. [Entrada en directo, MIDI, seguimiento facial y captura de movimiento](#10-entrada-en-vivo-midi-seguimiento-facial-y-captura-de-movimiento)
11. [Máquinas de estados, enlaces de datos, contenedores y regiones de desplazamiento](#11-máquinas-de-estados-enlaces-de-datos-contenedores-y-regiones-de-desplazamiento)
12. [Efectos, profundidad de campo, onion skin, trails y cámara](#12-efectos-profundidad-de-campo-onion-skin-trayectorias-y-cámara)
13. [Nivel de detalle, informes de tamaño y export-fit](#13-nivel-de-detalle-informes-de-tamaño-y-export-fit)
14. [Control de calidad: check, lint, ci, golden, fuzz, diff](#14-control-de-calidad)
15. [Control de versiones (`vcs`)](#15-control-de-versiones-vcs)
16. [Estudio, colaboración, servidor de agente MCP y registro de la interfaz](#16-estudio-colaboración-servidor-de-agente-mcp-y-registro-de-la-interfaz)
17. [Base de datos de poses, sugerencias y otras utilidades](#17-base-de-datos-de-poses-sugerencias-y-otras-utilidades)
18. [Discrepancias conocidas entre documentación y comportamiento](#discrepancias-conocidas-entre-la-documentación-y-el-comportamiento)
19. [Índice alfabético de comandos](#índice-alfabético-de-comandos)

---

## 1. Convenciones comunes a todos los comandos

### 1.1 Forma de la línea de comandos

```
osatura <command> [sub-command] <positional arguments...> [options]
```

* Las opciones pueden aparecer **en cualquier lugar** de la línea, también después del subcomando y después de los argumentos posicionales
  (`osatura render demo.json 0 o.png --scale 2` y `osatura --scale 2 render ...` son equivalentes).
* Las **opciones con valor** toman la palabra siguiente (`--scale 2`) o la forma `--key=value` (`--scale=2`). La forma con `=` funciona para
  toda opción con valor. Un valor que contenga a su vez `=` es válido con la forma separada por espacio (`--remap a=b`).
* Los **flags** (opciones booleanas como `--bones`) no toman valor. `--bones=1` se rechaza:
  `--bones is a flag and takes no value: write --bones (without =), got '--bones=1'`.
* **Opciones estrictas.** Una opción (flag u opción con valor) que el comando no documenta es un error con una
  sugerencia de «quizá quisiste decir» (salida 1) en lugar de ignorarse:
  ```
  $ osatura render demo.json 0 o.svg --bone
  unknown option --bone for 'render' (did you mean --bones?); see: osatura render --help
  ```
  La comprobación se aplica a todo comando que tenga una línea de uso, incluidos `check`, `stats`, `lint`, `coach`, `contracts` y
  `machine list` (las versiones anteriores ignoraban en silencio los flags sobrantes en ellos). `--json`, `--quiet` y `--allow-empty` **no** son
  globales: los rechaza todo comando cuya línea de uso no los enumere.
  **La comprobación se hace por subcomando (por forma de sinopsis), no por comando.** Una opción enumerada para una subforma se rechaza en
  sus hermanas, y el mensaje nombra la subforma que la acepta:
  ```
  $ osatura ci diff a.png b.png --quiet
  unknown option --quiet for 'ci diff' (--quiet applies to: ci run); see: osatura ci --help
  ```
  Así, `--quiet` es solo para `ci run`, `--json` solo para `vcs log|status|diff|show`, `ui commands|shortcuts` y la
  forma `trail ... --target`, `--tol` solo para `golden check` (y `ci diff`), `--scale` solo para `golden write`, y
  cada edición de `keyedit` acepta únicamente las opciones escritas tras su nombre en la sinopsis. (`export` sigue agrupando sus dos
  líneas de sinopsis, porque el formato se elige por la extensión del archivo; `--codec`, `--columns` y `--scale` que no
  se apliquen a la extensión elegida los notifica el propio `export`.)
* Algunos comandos más antiguos toman **palabras** `key=value` en lugar de `--options` (`cloth`, `physics`, `cage`, `warp-pivot`,
  `soft-scale`, `mesh-lock`, `glue`, `audio-react`, `beats`, `import-seq`, `import-anim`, `layer-pivot`, `chan-meta`,
  `waveform`). Son posicionales y se escriben exactamente como en la sinopsis (`gravity=900`, no `--gravity 900`).
* Un **fotograma** es un número (se admiten fracciones donde tenga sentido). Un proyecto con `frames = N` tiene los fotogramas `0..N-1`.
  Las capas se direccionan por **ruta** (`group/child`), los huesos por **nombre**; la cámara tiene el destino vacío `""`.
* Los **nombres de propiedad** (usados por `--prop`, `slider`, `drive`, archivos de mapeo, ...) son los nombres en snake_case:
  `bone_angle bone_scale bone_offset_x bone_offset_y layer_opacity layer_x layer_y layer_rotation layer_scale_x
  layer_scale_y layer_skew layer_switch layer_wobble camera_x camera_y camera_zoom camera_rotation camera_focus
  camera_blur ik_mix constraint_weight path_morph point_dx point_dy lattice_dx lattice_dy spine_dx spine_dy pin_dx
  pin_dy cage_dx cage_dy fx_blur fx_dx fx_dy fx_soft fx_amount grad_dx grad_dy`.  Algunos comandos imprimen las mismas propiedades en CamelCase (`BoneAngle`) en sus mensajes; los valores de las opciones son siempre snake_case.

### 1.2 Opciones globales

| Opción | Significado |
|---|---|
| `--allow-external-assets` | Permite que un proyecto nombre archivos por ruta absoluta o fuera de su propio directorio (sonidos, imágenes, mapas de normales...). Por defecto: se rechaza. |
| `--follow-symlinks` | Escribe en el **destino** de un enlace simbólico dado como archivo de salida. Por defecto: se rechaza. |
| `--max-input-bytes N` | Tamaño máximo de archivo de proyecto que se lee. Por defecto `268435456` (256 MiB). Debe ser un número entero positivo. |
| `<command> --help` | Imprime en stdout las líneas de uso de un comando (salida 0). |
| `<command> --version`, `--version` | `osatura 0.1.0`; sin comando, además la línea de licencia y la oferta de código fuente AGPL. |
| `--json`, `--quiet`, `--allow-empty` | Opciones propias de cada comando, **rechazadas** (`unknown option`) por los comandos cuya sinopsis no las enumera (`--json`: `pick`, `select-box`, `timeline`, `trail`, `size-report`, `collab-sim`, `vcs`, `ui`, `face-detect`, `pose-suggest`; `--allow-empty`: `physics`, `live-map`, `face-bake`; `--quiet`: `ci run`). Los scripts que las pasaban de forma uniforme a todos los comandos deben dejar de hacerlo. |

Ejemplos (verificados):

```
$ osatura --max-input-bytes 100 check demo.json
cannot read demo.json: 51450 bytes exceed the input limit of 100 bytes (raise it with --max-input-bytes N)
[exit 1]

$ osatura check /dev/zero
cannot read /dev/zero: not a regular file (devices, pipes and directories are refused)
[exit 1]

$ ln -s real_target.svg link.svg                       # dangling link
$ osatura render demo.json 0 link.svg
cannot write link.svg: link.svg: refusing to write through a symbolic link; pass --follow-symlinks to write to the link's target, or name the target file yourself
[exit 1]
```

### 1.3 Lectura de archivos de entrada

* Los archivos de entrada deben ser **archivos regulares** de tamaño no superior a `--max-input-bytes`. Los dispositivos, tuberías, directorios y
  sustituciones de procesos (`<(...)`) se rechazan: escribe primero los datos en un archivo.
* Los archivos de proyecto son JSON. Un proyecto con claves desconocidas se **acepta** y `check` (y `lint`, `studio`) notifica las claves
  desconocidas como `warning: unknown field bogus_key (ignored)`. Los archivos auxiliares que redactas para un comando
  (contratos, mapeos, máquinas, contenedores, regiones de desplazamiento, manifiestos de ci, mapas de drive ...) se analizan de forma **estricta**: una clave
  desconocida es un error y no se aplica nada (`unknown field(s): elastic.damping (typo? nothing was applied)`).
* Un proyecto se valida siempre que se carga para renderizar o exportar (`<file> is not a valid project:` seguido de
  un problema por línea). Los comandos de edición también recompilan el proyecto que van a escribir y se niegan a escribir uno que ya no valide.
* Las **rutas de assets** dentro de un proyecto (archivos de sonido, de imagen, mapas de normales) se resuelven respecto al **directorio del
  archivo de proyecto**. Las rutas absolutas y cualquier ruta que contenga `..` se rechazan salvo que se indique `--allow-external-assets`:
  ```
  $ osatura sound add s.json o.json x /etc/passwd
  sound add: asset '/etc/passwd': absolute paths are not allowed (a project may only reference files inside its own directory; use --allow-external-assets to override) (nothing was added)
  ```
  El flag debe indicarse de nuevo en cada comando posterior que cargue ese proyecto (`check ext.json` falla sin él y funciona con él).

### 1.4 Escritura de archivos de salida

* **Escrituras atómicas.** La mayoría de los comandos escriben mediante un archivo temporal en el **mismo directorio** (`.<name>.tmp-<pid>-<random>`), le hacen `fsync`,
  copian los permisos del archivo anterior y le aplican `rename` sobre el destino, de modo que un fallo o un disco lleno nunca deja un archivo a medio escribir.
  Si no se puede crear un archivo temporal (directorio de solo lectura pero archivo escribible), el archivo se sobrescribe en el sitio, lo que **no** es atómico.
  Los FIFO y los archivos especiales del tipo `/dev/stdout` se escriben directamente.
* Los **enlaces simbólicos** como salida se rechazan salvo que se indique `--follow-symlinks` (entonces se escribe el destino del enlace; un enlace roto sigue siendo un error).
  Un directorio dado como archivo de salida es un error (`is a directory`).
* **Nunca sobre la entrada.** `render`, `render-hdr`, `dof`, `dof-export`, `export` (también el archivo auxiliar `.json` de hoja de sprites de un `.png`
  y `--report`), `layout-preview`, `render-gpu`, `export-fit`, `fuzz-grid`, `onion`, `trail` y `golden` rechazan una salida que sea el mismo archivo
  que el proyecto de entrada: `refusing to overwrite the input project demo.json with output of the same name`.
* Los **comandos de edición** (los que tienen un par `<project.json> <out.json>`: `reparent`, `keys`, `physics`, ...) pueden escribir **en el sitio**
  (`osatura reparent p.json p.json hand none` funciona y reemplaza `p.json`). `import-lottie` rechaza una salida (o `fx.json`) que sea
  el mismo archivo que su entrada; los demás comandos de importación leen un tipo de archivo distinto del que escriben.
* **Las salidas existentes se sobrescriben en silencio** en todos los comandos. No hay `--force`.
* **Toda salida dirigida al usuario** (incluidos los informes y goldens de `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, los archivos `face-*`,
  los archivos de poses, las instantáneas, `vcs` y el registro de colaboración del estudio) pasa por el mismo escritor atómico que rechaza enlaces simbólicos. Las
  salidas `.mp4`/`.webm` que escribe ffmpeg se comprueban en busca de un enlace simbólico antes de que ffmpeg arranque. Carpetas superiores: `lod`, `export-fit`, `ci` y `face-*`
  crean las carpetas que falten; `fuzz`, `fuzz-grid` y `export` no (`No such file or directory`).

### 1.5 stdout, stderr y códigos de salida

* Los resultados («wrote out.json», tablas, JSON) van a **stdout**. Los errores y la mayoría de las advertencias van a **stderr**. `import-svg` imprime sus
  advertencias en stdout; `export` imprime las advertencias de Lottie/GIF/GLB en stderr.
* El código de salida **0** significa éxito. El código de salida **1** significa cualquier error *o* una comprobación fallida: un archivo ilegible o no válido, argumentos incorrectos,
  `lint` con errores, `diff` con diferencias, un contrato incumplido, un golden que no coincide, una tasa de fallos de fuzz por encima del límite,
  un presupuesto de `export-fit` que no se pudo cumplir, `lod --verify` por encima del límite, divergencia de `collab-sim`, una comprobación de `ci` fallida.
  El código de salida **2** lo usa solo `ci` (no se pudo evaluar nada: falta el proyecto o el manifiesto, golden manipulado).
* Sin argumentos, con un comando desconocido, con un número incorrecto de argumentos posicionales o con `osatura --help`, se imprime el uso completo en
  **stderr** y se sale con **1**. `osatura <command> --help` imprime las líneas de uso de ese comando en stdout y sale con 0.
  **Excepción (Windows, desde 0.1.0-alpha.2):** cuando `osatura.exe` se inicia con un doble clic desde el Explorador (la ventana de consola se creó para él y ningún otro proceso la comparte) y no tiene argumentos, no imprime el uso: crea `Documents\OSATURA\first-rig.json` (a partir de `demo-person`) si no existe, ejecuta `studio` sobre ese archivo en `127.0.0.1:8080`, abre el navegador y mantiene la consola abierta hasta que la cierres o pulses Enter tras un error. Iniciado desde un terminal, se comporta como se ha descrito arriba. El resto del programa se ejecuta en un hilo con pila de 256 MiB (el hilo principal de Windows solo tiene 1 MiB).

### 1.6 Programas externos

`ffmpeg` (encontrado en `PATH`) es necesario para: la exportación `.mp4`/`.webm` y `dof-export` a esos formatos, `sound mix`,
el audio en `.html` (mp3) y `face-track` sobre un archivo de vídeo. Codificadores necesarios: `libx264` (mp4), `libvpx-vp9` / `libaom-av1` (webm),
`aac` (sonido mp4), `libopus` (sonido webm), `libvorbis`, `libmp3lame` (sound mix). No se incluye ningún código ni binario de ffmpeg; véase
[PATENT_NOTES.md](PATENT_NOTES.md). No se ejecuta nada más.

### 1.7 Nota sobre rendimiento

Todos los tiempos de este documento proceden de una compilación **debug** y son entre 10 y 30 veces más lentos que una compilación release. Los comandos que consumen muchos fotogramas
(`safety`, `golden`, `ci`, `export-fit`, `proxy-bench`) se ejecutaron sobre una copia reducida de 8 fotogramas de la demo (`small.json`, hecha estableciendo
`"frames": 8`).

---

## 2. Primeros pasos: el proyecto de demostración

```
$ osatura demo demo.json
wrote demo.json
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels
$ osatura render demo.json 12 f12.png --bones --scale 2
wrote f12.png
$ osatura export demo.json demo.gif
wrote demo.gif
```

La demo tiene los huesos `arm_up arm_low hand thigh shin foot_ctrl`, una cadena IK `leg` y las capas de primer nivel
`sky sun hills ground shadow arm leg target blush mouth(switch: closed/open/smile) ink`. Todos los ejemplos de esta referencia la usan
salvo que se indique lo contrario.

---

## 3. Creación e inspección de proyectos

### `demo`

**Sinopsis:** `osatura demo <out.json>`

Escribe como archivo de proyecto el rig de demostración integrado (640x480, 24 fps, 48 fotogramas, 6 huesos, pierna IK, 14 capas).
**Salida:** `wrote <out.json>`. **Errores:** falta un argumento (uso, salida 1); ruta no escribible. **Relacionado:** `demo-fx`, `check`.
**Límites:** la demo es un recurso fijo; es la base de todos los ejemplos de este documento.

```
$ osatura demo demo.json
wrote demo.json
```

### `demo-person`

**Sinopsis:** `osatura demo-person <out.json>`

Escribe como archivo de proyecto la demo de la persona caminando (640x480, 24 fps, 32 fotogramas, 42 huesos, 45 capas): una persona vista de lado con cabeza, oreja y coleta, cuello, tronco, dos brazos con manos y cinco dedos cada uno, y dos piernas con pies y dedos de los pies. El ciclo de marcha dura dos pasos y se repite sin costuras: la pelvis sube y baja y se inclina, el tronco se inclina y contrarrota, la cabeza se mantiene nivelada con un pequeño cabeceo retardado, los brazos se balancean contra las piernas con los codos doblándose más en el balanceo hacia delante, y las manos, los dedos, la oreja y la coleta siguen un poco más tarde. Cada pie aterriza sobre el talón, rueda hasta apoyarse plano y sale por la planta; el suelo, la valla y la sombra se desplazan exactamente a la velocidad del pie apoyado, de modo que los pies no se deslizan. Este es el proyecto que los lanzadores de Windows (`osatura.exe`, `osatura-studio.exe`) crean como `first-rig.json` en el primer inicio; `osatura demo` sigue siendo la pequeña base de los ejemplos de este documento.
**Salida:** `wrote <out.json>`. **Errores:** falta un argumento (uso, salida 1); ruta no escribible. **Relacionado:** `demo`, `studio`.
**Límites:** las claves se muestrean cada dos fotogramas con una clave adicional antes del fotograma 0 y otra después del último fotograma, de modo que el bucle es suave; `lint` informa de esas dos claves como fuera de la línea de tiempo (información, no un error). La figura es una vista lateral, por lo que no hay giros ni profundidad.

```
$ osatura demo-person person.json && osatura check person.json
wrote person.json
ok: 'OSATURA walking person' 640×480 @ 24 fps, 32 frames, 42 bones, 0 ik, 45 layers, 48 channels
```

### `demo-fx`

**Sinopsis:** `osatura demo-fx <out.json>`

Escribe la demo «líquido / booleana / partículas» (640x480, 72 fotogramas, 3 huesos, 3 capas) que ejercita las capas procedurales.

```
$ osatura demo-fx fx.json && osatura check fx.json
wrote fx.json
ok: 'OSATURA fx demo' 640×480 @ 24 fps, 72 frames, 3 bones, 0 ik, 3 layers, 3 channels
```

### `check`

**Sinopsis:** `osatura check <project.json>`

Carga y valida un proyecto. Imprime líneas `warning:` (campos desconocidos, problemas de degradados, problemas de keys de switch y las advertencias de datos del editor del código de lint L013: zoom de cámara <= 0, opacidad de capa fuera de 0..1, referencias o tiempos incorrectos en skins, comps, marcadores, eventos, shots, channel_meta, bone_vis, clips y lods) y una línea de resumen `ok:`. Solo los valores que no pueden existir en un archivo guardado (NaN o infinito en esas estructuras) son errores, de modo que los proyectos antiguos siguen cargándose.
Las pistas de sonido también se abren: un archivo de audio ilegible o inutilizable es un **error**. **Salida:** 0 válido; 1 no válido.

```
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels

$ osatura check unk.json                      # a project with two unknown keys
warning: unknown field bogus_key (ignored)
warning: unknown field bones[0].lenght (ignored)
ok: 'small' 640×480 @ 24 fps, 8 frames, 6 bones, 1 ik, 14 layers, 14 channels

$ osatura check bad3.json                     # IK chain whose target bone does not exist
bad3.json is not a valid project:
  - ik 'leg': unknown target bone 'nope'
[exit 1]

$ osatura check bad.json                      # {"format":1,"name":"x"}
invalid JSON in bad.json: missing field `width`
[exit 1]
```

**Relacionado:** `lint` (calidad del rig/animación), `ci` (puerta de regresión). **Ojo:** `check` no renderiza nada; un proyecto puede pasar `check` y aun así
fallar más tarde en un fotograma (las poses `NaN` las detectan `render`/`export`, que evalúan antes todos los fotogramas).

### `stats`

**Sinopsis:** `osatura stats <project.json>`

Imprime recuentos como JSON: `name, size, fps, frames, seconds, bones, ik_chains, constraints, springs, layers, channels, keyed_channels, keys, markers`.
La salida es siempre JSON; no existe la opción `--json` (se rechaza como cualquier opción desconocida).

```
$ osatura stats demo.json
{ "name": "OSATURA demo", "size": [640.0, 480.0], "fps": 24.0, "frames": 48, "seconds": 2.0, "bones": 6,
  "ik_chains": 1, "constraints": 0, "springs": 0, "layers": 14, "channels": 14, "keyed_channels": 8, "keys": 37, "markers": 0 }
```
(la salida real se imprime con formato legible en varias líneas.) Ten en cuenta que `layers` cuenta todos los nodos, incluidos los hijos de grupos/switches, mientras que `check` imprime el mismo número.

### `diff`

**Sinopsis:** `osatura diff <a.json> <b.json>`

Diferencia semántica entre dos proyectos: `+` añadido, `-` eliminado, `~` cambiado. No depende del orden en las listas direccionadas por nombre.
**Salida:** 0 idénticos (`no differences`); **1** cuando hay diferencias (el recuento `N difference(s)` va a stderr), de modo que puede servir de puerta en un script.

```
$ osatura diff demo.json demo.json
no differences
$ osatura diff demo.json d2.json              # fps 24->30, arm_up longer, layer 'sun' deleted
~ fps: 24 → 30
~ bone 'arm_up' changed (length)
- layer 'sun' removed
3 difference(s)            (stderr)
[exit 1]
```
**Relacionado:** `vcs diff` (historial), `compare` (diferencia de píxeles), `ci diff` (diferencia de imágenes).

### `tag`

**Sinopsis:** `osatura tag <project.json>`

Propone nombres de rol estándar (`arm.upper.l`, `hand.l`, `face.mouth` ...) para huesos y capas a partir de sus nombres, con un nivel de confianza, y enumera los roles estándar para los que no se encontró ningún hueso. Solo lectura; usa los roles como guía para `mocap-match` o `autorig`.

```
$ osatura tag demo.json
bone  arm_up                   -> arm.upper.l    (45%)
bone  hand                     -> hand.l         (45%)
layer mouth                    -> face.mouth     (90%)
no bone found for: head, neck, torso, hips, face.eye.l, ...
```

### `explain`

**Sinopsis:** `osatura explain <project.json> <bone-or-layer> <frame>`

«¿Por qué está aquí?»: enumera las contribuciones ordenadas (pose de reposo, cada canal con su modo y peso, restricciones, IK) que producen la transformación de un hueso o capa en un fotograma, y el ángulo resultante de cabeza/punta/mundo.

```
$ osatura explain demo.json hand 12
- bone 'hand' (length 45), parent 'arm_low'
- ancestry: hand ← arm_low ← arm_up
- rest pose: offset (100.00, 0.00), angle 0.00°
- channel #2 BoneAngle: keys → 20.000 (5 key(s), Hold outside range); mode replace, weight 1.00
- channel #3 BoneAngle: expression `6 * sin(time * 2 * pi * 2)`; mode add, weight 1.00
- result at frame 12: head (174.6, 126.4), tip (203.5, 92.0), world angle -50.0°
```
**Errores:** nombre de hueso/capa desconocido; fotograma fuera de la animación.

### `pick`

**Sinopsis:** `osatura pick <project.json> --frame N --x X --y Y [--tol PX] [--root] [--bones] [--points a,b] [--locked a,b] [--json]`

Qué hay bajo un punto del lienzo, el superior primero (pruebas de impacto como las hace el estudio).

| Opción | Significado |
|---|---|
| `--frame N`, `--x X`, `--y Y` | Obligatorios: fotograma y coordenadas del lienzo. |
| `--tol PX` | Tolerancia de selección en píxeles del lienzo (por defecto 3). |
| `--root` | Notifica el ancestro de primer nivel (`mouth`) en lugar de la capa más interna (`mouth/closed`). |
| `--bones` | Considera también los huesos. |
| `--points a,b` | Capas cuyos puntos de control (anclas/tiradores/vértices de malla) se pueden seleccionar. |
| `--locked a,b` | Capas que se omiten (se tratan como bloqueadas). |
| `--json` | `{"hit":{...},"hits":[...]}`. |

```
$ osatura pick demo.json --frame 0 --x 209 --y 104 --tol 4
hit: layer ink (distance 0.00)
  under: layer mouth/closed (distance 0.00)
  under: layer arm (distance 0.00)
  under: layer sky (distance 0.00)
$ osatura pick demo.json --frame 0 --x 209 --y 104 --tol 4 --root --locked mouth
hit: layer ink (distance 0.00)
  under: layer arm (distance 0.00)
  under: layer sky (distance 0.00)
$ osatura pick demo.json --frame 0 --x 300 --y 240 --json
{"hit":{"distance":0.0,"target":{"kind":"layer","path":"sky"}},"hits":[{"distance":0.0,"target":{"kind":"layer","path":"sky"}}]}
```
**Ojo:** la capa de fondo (`sky`) es un objetivo de impacto normal, de modo que casi siempre hay al menos un impacto.

### `select-box`

**Sinopsis:** `osatura select-box <project.json> --frame N (--rect x0,y0,x1,y1 | --lasso x,y;x,y;x,y) [--mode bbox|contain|intersect] [--root] [--bones] [--json]`

Selección por marco/lazo. `--mode` decide cuándo una capa cuenta como interior: `bbox` (su caja delimitadora toca la región), `contain` (completamente dentro), `intersect` (por defecto: su geometría dibujada interseca la región). `--root` notifica los ancestros de primer nivel, `--bones` incluye los huesos.
Entrecomilla el valor de `--lasso` en un shell (el `;` separa los puntos).

```
$ osatura select-box demo.json --frame 0 --rect 200,95,220,115 --root
layer sky
layer arm
layer mouth
layer ink
$ osatura select-box demo.json --frame 0 --rect 0,0,640,480 --mode contain
layer sun
layer shadow
...
$ osatura select-box demo.json --frame 0 --lasso "100,100;300,100;300,300" --bones --json
{"targets":[{"kind":"bone","name":"arm_low"},{"kind":"bone","name":"hand"},{"kind":"layer","path":"sky"}, ...]}
```

### `timeline`

**Sinopsis:** `osatura timeline <project.json> [--json]`

La vista de dope sheet como texto: pistas por canal agrupadas por hueso/capa, con los fotogramas clave (`keys @ ...`), canales `expr` / `driven`, bloqueos y colores. Los números `#N` son los índices de canal usados por `keyedit --channels` y por las selecciones `#N@F0..F1`.

```
$ osatura timeline demo.json
timeline: 48 frames at 24 fps, 14 channels
Bones
  arm_up
    #0 bone_angle (keys @ 0 12 24 36 48)
    arm_low
      #1 bone_angle (keys @ 0 12 24 36 48)
      hand
        #2 bone_angle (keys @ 0 12 24 36 48)
        #3 bone_angle (expr)
  foot_ctrl
    #4 bone_offset_x (keys @ 0 12 24 36 48)
...
```
`--json` da grupos con ids, padres, hijos, pistas, `frames`, `locked`.

### `timecode`

**Sinopsis:** `osatura timecode <frame|HH:MM:SS;FF> [--fps 24] [--drop] [--nodrop]`

Conversión de código de tiempo SMPTE. Un número solo es un fotograma e imprime un código de tiempo; un argumento que contenga `:` o `;` se analiza e imprime el número de fotograma.
29,97, 59,94 y 119,88 fps usan por defecto código de tiempo con drop-frame (`;` antes de los fotogramas); `--nodrop` cuenta 30 (o 60/120) en su lugar; `--drop` fuerza el drop-frame (error para otras frecuencias).
`--drop` y `--nodrop` se excluyen mutuamente.

```
$ osatura timecode 100
00:00:04:04
$ osatura timecode 100 --fps 29.97
00:00:03;10
$ osatura timecode 1800 --fps 29.97 --drop
00:01:00;02
$ osatura timecode 1800 --fps 29.97 --nodrop
00:01:00:00
$ osatura timecode 00:00:04:04
100
$ osatura timecode 00:00:04;04 --fps 24
drop-frame timecode is only defined for 29.97, 59.94 and 119.88 fps         [exit 1]
```
No lee ningún proyecto; `--fps` vale 24 por defecto (no la frecuencia de fotogramas de un proyecto).

### `snapshot`

**Sinopsis:**
```
osatura snapshot save <project.json> <dir> [--keep 20]
osatura snapshot list <project.json> <dir>
osatura snapshot restore <project.json> <dir> <out.json> [--number N]
osatura snapshot recover <project.json> <dir> <out.json>
```
`--keep` pertenece solo a `save` y `--number` solo a `restore`; cualquier otra combinación se rechaza (`unknown option --keep for 'snapshot list' (applies to: snapshot save)`), igual que un argumento de archivo ausente o sobrante.

Un historial numerado de un proyecto en un directorio (independiente de `vcs`). `save` guarda el proyecto actual como `rev-NNNNNN-<hash>.json` y conserva solo las `--keep` revisiones más recientes (por defecto 20, los archivos más antiguos se eliminan); guardar un proyecto idéntico a la última revisión imprime `unchanged since the last revision` y no guarda nada. `list` imprime las revisiones. `restore` escribe la revisión `--number N` en `<out.json>`; `recover` escribe la revisión más reciente.

```
$ osatura snapshot save s.json snaps --keep 3
saved revision 1 (4922f48a86f24554)
$ osatura snapshot list s.json snaps
     1  4922f48a86f24554  snaps/rev-000001-4922f48a86f24554.json
$ osatura snapshot restore s.json snaps restored.json --number 1
wrote restored.json
$ osatura snapshot recover s.json snaps recovered.json
wrote recovered.json
```
`list` sobre un directorio inexistente no imprime nada y sale con 0. **Relacionado:** `vcs` para ramas y fusiones.

---

## 4. Renderizado y exportación

### `render`

**Sinopsis:** `osatura render <project.json> <frame> <out.svg|out.png> [--bones] [--scale N] [--clamp] [--proxy N]`

`--proxy N` renderiza un proxy de scrub (N = 1..4): un `.png` es un mapa de bits a resolución 1/2^N, un `.svg` son vectores simplificados; ambos descartan los trazos finos, los degradados y las mezclas.

Renderiza un fotograma. Un `<frame>` igual o posterior a la longitud del proyecto es un error (`frame 99 is outside the animation ... valid frames are 0..N-1 ... pass --clamp`); `--clamp` renderiza en su lugar el último fotograma e imprime una advertencia en stderr. (Solo `render` tiene esta comprobación; `dof`, `render-hdr` y `fx` no.) La extensión elige el formato (`.svg` vectorial, `.png` ráster RGBA de 8 bits); cualquier otra extensión se rechaza **antes** de escribir nada.

| Opción | Por defecto | Significado |
|---|---|---|
| `--bones` | desactivado | Dibuja la superposición del esqueleto (cabezas como puntos, huesos como líneas). |
| `--scale N` | 1 | Escala de salida, un número en (0, 16]. Para `.png` multiplica el tamaño en píxeles; para `.svg` el documento conserva su viewBox. |
| `--proxy N` | desactivado | Proxy de scrub, N = 1..4: `.png` = mapa de bits a resolución 1/2^N, `.svg` = vectores simplificados; ambos descartan los trazos finos, los degradados y los modos de mezcla. Pensado para el scrubbing interactivo, no para entregables. |

```
$ osatura render demo.json 12 f12.png --bones --scale 2
wrote f12.png
$ osatura render demo.json 0 /tmp/x.xyz
render writes .png or .svg; '/tmp/x.xyz' has another extension (nothing was written)       [exit 1]
$ osatura render demo.json 0 o.svg --scale 0
--scale needs a number in (0, 16]                                                         [exit 1]
$ osatura render demo.json abc o.svg
bad frame number 'abc'                                                                    [exit 1]
$ osatura render demo.json 2 px.png --proxy 2        # 640x480 project -> 160x120 PNG
wrote px.png
```
**Errores:** proyecto ausente o no válido, pose no finita en ese fotograma, salida igual al proyecto de entrada. **Relacionado:** `frames`, `render-hdr`, `onion`, `compare`, `fx`.
**Límites:** el rasterizador PNG es el propio del motor (8 bits, sRGB; la mezcla no tiene gestión de color salvo que el proyecto la active con `color-space ... linear`). Los efectos de capa, las máscaras y los modos de mezcla los dibujan ambos.

### `frames`

**Sinopsis:** `osatura frames <project.json> <out_dir> [--png] [--bones] [--scale N]`

Renderiza **todos** los fotogramas en un directorio (se crea si no existe) como `frame_0000.svg` ... (o `.png` con `--png`). Antes se comprueba que todos los fotogramas tengan poses finitas.

```
$ osatura frames demo.json fr --png
wrote 48 frames to fr
$ ls fr | head -2
frame_0000.png
frame_0001.png
```
**Ojo:** los archivos existentes con los mismos nombres se sobrescriben; los archivos ajenos del directorio no se tocan. Úsalo como entrada para `ffmpeg` o `import-seq`.

### `export`

**Sinopsis:**
```
osatura export <project.json> <out.gif> [--scale N] [--report r.json]
osatura export <project.json> <out.html|out.htm>
osatura export <project.json> <out.json|out.lottie> [--report r.json]
osatura export <project.json> <out.mp4> [--scale N]
osatura export <project.json> <out.apng> [--scale N]
osatura export <project.json> <out.webm> [--scale N] [--codec vp9|av1]
osatura export <project.json> <out.png> [--scale N] [--columns N]
osatura export <project.json> <out.glb> [--report r.json]
```
(una forma por formato; la extensión de salida, sin distinguir mayúsculas, elige la forma. Una opción que la forma no enumera se rechaza: `unknown option --scale for 'export .json' (applies to: export .gif, export .mp4, ...)`. Con una extensión desconocida o ausente se notifica en su lugar el error de formato.)

La extensión de salida selecciona el formato:

| Extensión | Resultado | Necesita ffmpeg | Opciones aplicables |
|---|---|---|---|
| `.gif` | GIF en bucle a la frecuencia de fotogramas del proyecto (véase [EXPORT_FORMATS.md](EXPORT_FORMATS.md)) | no | `--scale`, `--report` |
| `.html`, `.htm` | un reproductor HTML autocontenido (fotogramas SVG en línea, reproducir/pausar, barra de desplazamiento, audio incrustado opcional) | solo para el audio | - |
| `.json` | Lottie (Bodymovin) | no | `--report` |
| `.lottie` | contenedor dotLottie (zip: `manifest.json` + `a/animation.json`) | no | `--report` |
| `.mp4` | H.264 yuv420p (+ AAC cuando el proyecto tiene sonidos) | sí (`libx264`, `aac`) | `--scale` |
| `.apng` | PNG animado | no | `--scale` |
| `.webm` | VP9 (por defecto) o AV1 con alfa (+ sonido Opus) | sí (`libvpx-vp9` / `libaom-av1`, `libopus`) | `--scale`, `--codec vp9\|av1` |
| `.png` | hoja de sprites **más** metadatos `<out>.atlas.json` (nunca `<out>.json`) | no | `--scale`, `--columns N` |
| `.glb` | glTF 2.0 binario (huesos, animación, rellenos/trazos con skinning, degradados, máscaras, imágenes, morph targets) | no | `--report` |

Opciones:

| Opción | Significado |
|---|---|
| `--scale N` | Escala del ráster (0, 16] para `.gif .mp4 .apng .webm .png`. En otros formatos se rechaza (`unknown option --scale for 'export .json'`); antes de la ola 7 era una advertencia. |
| `--report FILE` | Escribe como JSON la lista de cosas que el formato no pudo transportar (`.gif .json .lottie .glb`): `{"format","count","warnings":[{"code","layer","message"}]}`. Las mismas advertencias se imprimen siempre en stderr. El archivo no puede ser el proyecto de entrada. |
| `--codec vp9\|av1` | Solo `.webm` (por defecto `vp9`); cualquier otro valor: `unknown --codec 'foo' (vp9 or av1)`. |
| `--columns N` | Solo la hoja de sprites `.png`: teselas por fila (por defecto: una cuadrícula casi cuadrada, ceil(sqrt(frames)): 3 columnas para 8 fotogramas). |

Ejemplos (todos verificados sobre una copia de 8 fotogramas de la demo, `--scale 0.5`):

```
$ osatura export small.json out.gif --scale 0.5
wrote out.gif
$ osatura export small.json out.json --report rep.json
wrote out.json
warning: [variable-width-stroke] ink: variable-width stroke profiles are not supported by Lottie; the constant base width is used
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
$ cat rep.json
{ "count": 2, "format": "lottie", "warnings": [ {"code": "variable-width-stroke", "layer": "ink", ...}, {"code": "gradient-anisotropic", ...} ] }
$ osatura export small.json out3.png --columns 4 --scale 0.25
wrote out3.json
wrote out3.png                          # 4 columns x 2 rows of 160x120 tiles; out3.json lists {index,x,y,w,h} per frame
$ osatura export small.json out.glb
wrote out.glb
warning: [glb-limit] blend mode Screen of 'sun' is not exported (glTF materials only alpha-blend); it is drawn as normal
warning: [glb-limit] animated opacity of 'target' is not exported (frozen at frame 0)
$ osatura export small.json out.mp4 --scale 0.5       # 320x240 h264 yuv420p, 8 frames
wrote out.mp4
$ osatura export small.json out.webm --codec av1      # needs a libaom-av1 build of ffmpeg
```

Errores y rechazos:

```
$ osatura export small.json z.xyz
unknown export format '.xyz' (use .gif .html .htm .json .lottie .mp4 .apng .webm .png .glb)
$ osatura export small.json small.json
refusing to overwrite the input project small.json with output of the same name
$ osatura export ora.json ora.glb                       # a project of image layers and no bones
the project has no bones to export
$ PATH=/nonexistent osatura export small.json y.mp4
cannot run ffmpeg (...); install it or export a GIF/HTML instead                          [exit 1, nothing written]
```
Otros rechazos: `ffmpeg failed` (falta un codificador), `export would write about N million values ...` (la protección de tamaño de Lottie/HTML/GLB: el límite es de 40 millones de valores = fotogramas x (anclas + huesos + capas); acorta la animación o simplifica las rutas), poses no finitas en cualquier fotograma, e `image too large for GIF`.

**Salidas / efectos secundarios:** `.png` escribe además los metadatos de la hoja de sprites en `<out-name>.atlas.json` (p. ej. `out.png` -> `out.atlas.json`), de modo que ya no puede reemplazar un archivo Lottie `out.json`; el archivo auxiliar está sujeto a las mismas protecciones de entrada/enlace simbólico. `.mp4`/`.webm` renderizan fotogramas PNG en un directorio temporal privado que se elimina después.
**Sonido:** `.mp4` y `.webm` mezclan las pistas de sonido del proyecto (inicio, ganancia, recorte, fundidos; cortadas a la longitud del proyecto); `.html` incrusta un mp3 mezclado (avisa por encima de 3 MiB, se descarta por encima de 16 MiB con una advertencia); `.gif` y `.apng` imprimen `note: GIF cannot carry audio; ...` y quedan en silencio; Lottie avisa con `[audio]`.
**Transparencia:** un proyecto con `"background": null` se renderiza con píxeles transparentes donde no se dibuja nada. `.apng`, la hoja de sprites `.png` y `.webm` (`yuva420p`, etiqueta de contenedor `alpha_mode=1`) conservan el canal alfa; `.gif` conserva transparencia de 1 bit (verificado: el indicador de color transparente está activado); `.mp4` no tiene alfa. Un proyecto con color de fondo (la demo tiene `#f6f1e7`) es opaco.
**Relacionado:** `export-fit` (ajustar a un presupuesto de bytes), `size-report`, `lod`, `dof-export`, `export-oca`, `render-hdr`. Detalle completo de los formatos: [EXPORT_FORMATS.md](EXPORT_FORMATS.md).

### `export-oca`

**Sinopsis:** `osatura export-oca <project.json> <out_dir> [--layers]`

Escribe una carpeta **Open Cel Animation**: `<out_dir>/<project name>.oca` (JSON, `ocaVersion` 1.1) más celdas PNG `layerN/fNNNN.png`. Sin `--layers` hay una sola capa `render` que contiene los fotogramas compuestos; con `--layers` cada capa de **primer nivel** se renderiza sola sobre transparencia en su propia capa OCA (11 capas para la demo). `--scale N` multiplica el tamaño de las celdas.

```
$ osatura export-oca small.json oca2 --layers
wrote oca2/small.oca
$ ls oca2
layer0 layer1 ... layer10 small.oca
```
**Límites:** las celdas son ráster (no sobreviven vectores, huesos ni canales); la opacidad y la mezcla de capa no se escriben (toda capa es `paintlayer`, opacidad 1); los sonidos no se exportan; la carpeta se crea si hace falta y los archivos que contiene se sobrescriben.

### `render-hdr`

**Sinopsis:** `osatura render-hdr <project.json> <frame> <out.exr|out.png> [linear] [--precision half|float] [--float32] [--8bit] [--scale N] [--clamp]`

Renderizado en coma flotante (32 bits) (un `<frame>` fuera del proyecto es un error, `--clamp` toma el fotograma válido más cercano, como en `render`): `.exr` escribe un **OpenEXR** de una sola parte, por líneas de barrido y sin comprimir (canales HALF por defecto, FLOAT con `--precision float` o `--float32`), en luz lineal y con alfa premultiplicado; `.png` escribe un **PNG RGBA de 16 bits**, codificado en sRGB salvo que se dé la palabra literal `linear` como quinto argumento. El fondo del proyecto **no** se dibuja (la imagen es transparente donde no se dibuja nada). `--8bit` usa el rasterizador heredado de 8 bits como fuente (no se puede combinar con salida float32). `--scale N` se aplica.

```
$ osatura render-hdr small.json 3 f.exr
wrote f.exr (640x480)                       # 465,611 bytes: HALF
$ osatura render-hdr small.json 3 f2.exr linear --precision float
wrote f2.exr (640x480)                      # 923,211 bytes: FLOAT
$ osatura render-hdr small.json 3 f4.png
wrote f4.png (640x480)                      # PNG 16-bit/colour RGBA
$ osatura render-hdr small.json 3 f6.exr --precision bogus
unknown precision 'bogus' (use half or float)
$ osatura render-hdr small.json 3 f7.jpg
output must end in .exr or .png
```
`ffprobe` lee el EXR como `exr 640x480 gbrapf32le`. Para `.exr`, la palabra `linear` no tiene efecto (EXR siempre es lineal). **Límites:** sin compresión, un fotograma por archivo, sin capas/AOV; el arte de origen es color sRGB corriente, así que el archivo contiene valores de rango de visualización con mayor precisión, no luz HDR referida a la escena. Sin gestión de color más allá de la conversión sRGB<->lineal.

### `render-gpu`

**Sinopsis:** `osatura render-gpu <project.json> <out.png> [--frame N] [--adapter software|auto] [--scale S] [--bench N]`

Renderizado por GPU (wgpu) de un fotograma. Solo está disponible en compilaciones hechas con `--features gpu`; la compilación por defecto responde:

```
$ osatura render-gpu small.json g.png
render-gpu needs a build with the gpu feature: cargo build -p osatura-cli --features gpu       [exit 1]
```
En una compilación con GPU, las escenas no admitidas (imágenes, huesos) se notifican como errores, nunca se dibujan mal. `--bench N` es una opción de benchmark. Este comando no pudo ejecutarse en la compilación de pruebas (sin la característica gpu), por lo que sus flags proceden únicamente del texto de uso.

### `compare`

**Sinopsis:** `osatura compare <a.json> <b.json> <frame> <out.png> [--gain 4 --scale N]`

Escribe un PNG: **antes | después | diferencia** lado a lado (el tercer panel es un mapa de calor, negro = idéntico, rojo -> amarillo = cambio mayor, multiplicado por `--gain`, por defecto 4). Ambos proyectos deben renderizarse al mismo tamaño (`frame sizes differ (...); compare projects of the same size`).

```
$ osatura compare small.json d2.json 3 cmp.png --gain 4 --scale 0.25
wrote cmp.png
```
**Relacionado:** `diff` (estructura), `ci diff` (dos PNG, métricas perceptuales), `golden`.

### `proxy-bench`

**Sinopsis:** `osatura proxy-bench <project.json> [--frames N]`

Milisegundos por fotograma, un solo hilo, a calidad completa y en cada nivel de proxy de scrub (véase `render --proxy`).

```
$ osatura proxy-bench small.json --frames 2
2 frames of small (640x480), ms per frame, single thread:
  evaluate only           0.08
  svg (no raster)         0.48   vector proxy: level 0 0.32, level 2 0.24, level 4 0.24
  raster level 0       1360.87   (full quality)
  raster level 1        319.14   (4.3x faster than level 0)
  ...
  raster level 4          6.84   (199.0x faster than level 0)
```
(cifras de una compilación debug). 

---

## 5. Rigging, skins, IK y utilidades de maquetación

Todos los comandos de esta sección siguen el patrón `osatura <cmd> <project.json> <out.json> ...`: leen un proyecto, lo modifican, **lo revalidan**
y escriben `<out.json>` (que puede ser igual a la entrada). Si hay un error, no se escribe nada.

### `autorig`

**Sinopsis:** `osatura autorig <project.json> <out.json> [seed]`

Añade automatización estándar deducida de los **nombres** de huesos/capas: cadenas IK (mix 0 para que FK siga funcionando), un bucle de respiración (necesita un hueso de torso) y parpadeo (necesita capas de ojos). Imprime una línea por decisión: `+` añadido, `-` omitido con el motivo. También ejecuta `eye-clip`.

```
$ osatura autorig small.json auto.json 7
+ IK arm_l -> arm_l_ctrl (mix 0, bend +1)
- arm_r: upper/lower bone not found
- leg_l: already has an IK chain
- breathing: no torso bone
- blink: no eye layers
wrote auto.json
```
`seed` (posicional opcional) inicializa la temporización aleatoria del parpadeo. **Límites:** se basa puramente en los nombres (`tag` muestra cómo se interpretan los nombres); nunca crea huesos.

### `slider`

**Sinopsis:** `osatura slider <project.json> <out.json> <name> <poses>` donde `poses` es `at:target.prop=val,target.prop=val;at:...`

Crea un **hueso controlador** `ctrl_<name>` cuyo `offset_x` gobierna varias propiedades entre poses almacenadas (un slider de poses). Anima el `bone_offset_x` del controlador impreso.

```
$ osatura slider small.json sl.json pose "0:arm_up.bone_angle=0;24:arm_up.bone_angle=30"
controller bone: ctrl_pose (animate its offset_x)
wrote sl.json
```
`target.prop` usa los nombres de propiedad en snake_case (`arm_up.angle` falla con `unknown property 'angle'`). El último punto separa el destino de la propiedad, así que funcionan rutas de capa como `grp/child.layer_opacity`.

### `view-swap`

**Sinopsis:** `osatura view-swap <project.json> <out.json> <switch_layer> <bone.prop> <from:child,from:child,...>`

Gobierna una capa switch a partir de una propiedad de un hueso con una función escalón: los pares `from:child` se ordenan por `from`, y el switch muestra el hijo del último par cuyo `from` no supera el valor del controlador (`-40:0,0:1,40:2` = hijo 0 por debajo de 0, hijo 1 desde 0, hijo 2 desde 40). Úsalo para vistas de giro (frontal/lateral/trasera) gobernadas por un hueso de guiñada. Los índices de hijo deben existir (`child index 5 is out of range (the layer has 3)`); la capa debe ser un switch.

```
$ osatura view-swap small.json vs.json mouth foot_ctrl.bone_offset_x "-40:0,0:1,40:2"
wrote vs.json
```

### `align` y `distribute`

**Sinopsis:**
`osatura align <project.json> <out.json> <mode> <layer,layer,...|layer> [stage]`  (mode: `left right top bottom center-x center-y`)
`osatura distribute <project.json> <out.json> <x|y> <gaps|centers> <layer,layer,...>`

`align` mueve capas de modo que las cajas delimitadoras de sus curvas aplanadas queden alineadas (entre sí o, para una sola capa seguida de la palabra `stage`, con el lienzo). `distribute` reparte tres o más capas uniformemente a lo largo de un eje, igualando bien los **huecos** (gaps) entre cajas, bien los **centros** (centers).

```
$ osatura align small.json al.json center-x sun,hills stage
wrote al.json
$ osatura align small.json al2.json top ground
select at least two layers (or one with --stage)                 [exit 1]
$ osatura distribute small.json di.json x gaps sun,ground,hills
wrote di.json
```
Estos comandos editan las posiciones de las capas (transformación estática), así que los canales `layer_x`/`layer_y` con keys siguen prevaleciendo sobre ellas. (El mensaje de error menciona `--stage`; el argumento real es la palabra sola `stage`.)

### `trim`

**Sinopsis:** `osatura trim <project.json> <out.json> <layer> <path_index> <start> <end> [offset]`

Hornea un *trim path*: reemplaza la ruta número `path_index` de una capa vectorial por solo la parte entre `start` y `end` (fracciones 0..1 de su longitud, ambas desplazadas por `offset`; las rutas cerradas dan la vuelta). El resultado es una **polilínea abierta con el mismo trazo y sin relleno**: es destructivo, así que conserva el original. Para un efecto de dibujado progresivo, hornea una copia recortada por fotograma.

```
$ osatura trim small.json tr.json ink 0 0 0.5 0.25
wrote tr.json
```

### `reparent`

**Sinopsis:** `osatura reparent <project.json> <out.json> <bone> <new_parent|none>`

Cambia el padre de un hueso **sin moverlo**: el desplazamiento y el ángulo de reposo se recalculan para que la pose de reposo no cambie (verificado: después solo difieren `parent`, `rest_angle`, `rest_offset`). `none` lo convierte en raíz. Se rechaza un padre que crearía un ciclo.

```
$ osatura reparent small.json rp.json hand none
wrote rp.json
$ osatura diff small.json rp.json
~ bone 'hand' changed (parent, rest_angle, rest_offset)
```

### `parent-switch`

**Sinopsis:** `osatura parent-switch <project.json> <out.json> <bone> <frame:parent|none,...>`

Hornea un reemparentado animado (una mano que agarra un objeto): en cada fotograma de cambio se captura y se mantiene la pose en el mundo del hueso relativa al nuevo padre (seguimiento rígido), o se congela en el espacio del mundo para `none`. El hueso pasa a ser raíz y su pose en el mundo se escribe como keys simples de desplazamiento/ángulo para cada fotograma; sus canales antiguos permanecen en el archivo pero los horneados los anulan. **Límites:** se rechaza para huesos implicados en IK, restricciones o huesos de muelle; la escala no se transfiere.

```
$ osatura parent-switch demo.json ps.json hand 0:none,24:arm_up
wrote ps.json
```

### `skin`

**Sinopsis (subcomandos):**
```
osatura skin <project.json> <out.json> add <group> <name> <layer,layer,...>
osatura skin <project.json> <out.json> rig <group> <name> <constraint,...|-> <ik,...|->
osatura skin <project.json> <out.json> bones <group> <name> <bone,...|->
osatura skin <project.json> <out.json> apply <group> <name>
osatura skin <project.json> <out.json> schedule <group> <frame:name,frame:name,...>
```

Un *conjunto de skins* se identifica por una etiqueta `<group>` (un conjunto de «vestuario» o de «expresión»; en el ejemplo coincide con el nombre de un grupo de capas). Cada skin con nombre muestra un conjunto de capas, dadas por **ruta de capa** (p. ej. `skins/box`); aplicar una skin muestra sus capas y oculta las capas de todas las demás skins del mismo grupo.
`add` define una skin; `rig` enumera las restricciones y cadenas IK que solo están activas mientras lo está esa skin; `bones` enumera los huesos que solo existen mientras está activa (de lo contrario se colapsan a escala 0); `apply` activa una skin de forma estática; `schedule` aplica keys a la skin activa a lo largo del tiempo como keys escalonadas sobre la **opacidad de capa** (por lo que no debes animar de otro modo la opacidad de esas capas). `-` significa «ninguno».

```
$ osatura skin sc.json sk1.json add skins red skins/box
wrote sk1.json
$ osatura skin sk1.json sk2.json add skins green skins/box2
$ osatura skin sk2.json sk3.json apply skins green
$ osatura skin sk3.json sk4.json schedule skins 0:red,4:green
$ osatura skin sk4.json sk5.json bones skins red -
$ osatura skin sk4.json sk5.json rig skins red - leg
$ osatura skin sc.json x.json add skins red box
no layer 'box'                                                  [exit 1]     (use the full path skins/box)
```
**Límites (ROADMAP §9):** un hueso que existe solo en una skin nunca se elimina del rig evaluado (los índices se mantienen; se escala a 0); una capa ponderada a un hueso exclusivo de una skin pero que no está en esa skin desaparece con él, y esto no se comprueba.

### `bone-vis`

**Sinopsis:** `osatura bone-vis <project.json> <out.json> <bone> <from> <to>|clear`

Muestra el hueso en las **superposiciones** del estudio solo durante un rango de fotogramas (control del desorden visual). No cambia el renderizado ni la exportación. `clear` elimina el rango (`range removed`).

### `chan-meta`

**Sinopsis:** `osatura chan-meta <project.json> <out.json> <target> <prop|*> [color=#rrggbb] [shy=on|off] [solo=on|off] [lock=on|off]`

Metadatos de dope sheet de un canal: color de visualización, *shy* (oculto en las listas), *solo* (previsualizar solo esos canales en el estudio) y **lock** (rechaza ediciones: `keys` omite los canales bloqueados, `keyedit`, `audio-react` y `live-map` los rechazan, y `clip bake` nunca los toca). La evaluación y la exportación ignoran los cuatro. `*` selecciona todas las propiedades del destino.

```
$ osatura chan-meta small.json cm.json arm_up bone_angle 'color=#ff0000' shy=on lock=on
wrote cm.json
```

### `reference`

**Sinopsis:** `osatura reference <project.json> <out.json> <layer> <on|off>`

Marca una capa como **referencia de calco**: se muestra en el estudio y nunca se exporta.

### `parallax`

**Sinopsis:** `osatura parallax <project.json> <out.json> <far_layer,...,near_layer> <far_value> <near_value>`

Asigna profundidad multiplano (valores de `parallax` repartidos entre `far_value` y `near_value`) a las capas de primer nivel indicadas, en el orden dado (la primera = la más lejana). Con movimiento de cámara las capas se desplazan en cantidades distintas; `dof` desenfoca según `|parallax - focus|`.

```
$ osatura parallax small.json pa.json sky,hills,ground 0.2 1.0
wrote pa.json
```

### `eye-clip`

**Sinopsis:** `osatura eye-clip <project.json> <out.json>`

Enmascara cada capa de iris/pupila con la capa de forma del ojo del mismo lado (encontrada por nombre, véase `tag`) mediante una máscara de capa ordinaria, de modo que las pupilas nunca salen del ojo mientras se mueven. También lo ejecuta `autorig`. Cuando no se encuentran nombres coincidentes, el archivo se escribe igualmente, sin cambios.

### `comp`

**Sinopsis:**
```
osatura comp capture|apply <project.json> <out.json> <name>
osatura comp list <project.json>
```

Estados de capa con nombre («comps»: visibilidad, opacidad y el hijo seleccionado de las capas switch, por ruta de capa; **no** las transformaciones). `capture` guarda el estado actual como `name`, `apply` lo restaura, `list` imprime los nombres (`list` no toma archivo de salida y no escribe nada; todavía se acepta, e ignora, una cuarta palabra heredada, `comp list p.json p.json`).

```
$ osatura comp capture small.json c1.json pose1
wrote c1.json
$ osatura comp list c1.json c1.json
pose1 (14 layers)
$ osatura comp apply c1.json c2.json pose1
wrote c2.json
```

### `retarget`

**Sinopsis:** `osatura retarget <source.json> <dest.json> <out.json> --pairs srcChain=dstChain[,..]`

Retargeting por *intención*: la animación del **efector** IK (hueso destino) de cada cadena de origen se escala según la razón entre los alcances de las dos extremidades (suma de las longitudes de los huesos) y se reorienta hacia el hueso destino de la cadena de destino. Los canales de efector sin keys no pueden escalarse; se transmiten con otro nombre y se notifican como líneas `note:`. `--pairs srcChain=dstChain[,...]` es obligatorio. En el ejemplo el origen y el destino son el mismo proyecto, lo cual solo sirve como prueba de humo.

```
$ osatura retarget demo.json demo.json rt.json --pairs leg=leg
wrote rt.json (+14 channels)
```
**Relacionado:** `mocap-retarget` para datos BVH; `walk`, `mirror`.

---

## 6. Animación, keys, línea de tiempo y generadores

### `keys`

**Sinopsis:**
```
osatura keys shift <project.json> <out.json> --by N [--from F --to F --target NAME --prop PROP]
osatura keys scale <project.json> <out.json> --factor X [--pivot F --from F --to F --target NAME --prop PROP]
osatura keys ease <project.json> <out.json> --ease linear|step|ease_in|ease_out|ease_in_out|smooth [--from F --to F --target NAME --prop PROP]
osatura keys reverse <project.json> <out.json> [--from F --to F --target NAME --prop PROP]
```
Cada operación acepta solo sus propias opciones (`keys shift --factor 2` se rechaza); `--by`, `--factor` y `--ease` son obligatorios para su operación (antes, omitir `--by` o `--factor` era una operación nula silenciosa).

Operaciones masivas sobre keys de **todos los canales con keys** (filtradas por `--target` / `--prop`; un rango `--from/--to` limita las keys que se tocan). Los canales bloqueados se omiten con `skipped locked channel ...`.

| Operación | Necesita | Efecto |
|---|---|---|
| `shift` | `--by N` | Desplaza las keys N fotogramas. |
| `scale` | `--pivot F --factor X` | Escala los tiempos de las keys respecto al fotograma pivote. |
| `ease` | `--ease E` | Establece la suavización de las keys (suavizaciones con nombre; usa `keyedit ease` para `bezier:`). |
| `reverse` | - | Invierte el orden de las keys dentro del rango. |

```
$ osatura keys shift small.json k1.json --target arm_up --prop bone_angle --by 2
wrote k1.json (5 key(s) changed)
$ osatura keys scale small.json k2.json --pivot 0 --factor 0.5 --from 0 --to 48
wrote k2.json (37 key(s) changed)
$ osatura keys reverse small.json k4.json
wrote k4.json (37 key(s) changed)
```
Las keys desplazadas fuera de `0..frames-1` permanecen en el archivo (lint notifica `L007`); este comando no amplía la longitud del proyecto. **Relacionado:** `keyedit` (preciso, basado en selección), `loop-solve`.

### `keyedit`

**Sinopsis:** `osatura keyedit <project.json> <out.json|-> <sub-command> [options]`

Ediciones de dope sheet como **un único lote seguro para deshacer**. Con `-` como salida no se escribe nada y la operación planificada se imprime como JSON (ejecución en seco). Subcomandos y opciones:

```
osatura keyedit <project.json> <out.json|-> move --sel SEL [--dt N] [--dv N] [--collision replace|merge|refuse] [--no-snap] [--free]
osatura keyedit <project.json> <out.json|-> scale --sel SEL --pivot F --factor X | dup --sel SEL --offset N | delete --sel SEL | ease --sel SEL --ease E | value --sel SEL --value V | smooth --sel SEL [--kind monotone|catmull_rom]
osatura keyedit <project.json> <out.json|-> copy --sel SEL --clip clip.json | paste --clip clip.json --at F [--remap from_target:prop=to_target:prop]
osatura keyedit <project.json> <out.json|-> add --target T --prop P --at F [--value V] [--ease E] | ripple --at F --by N --mode insert|delete [--scope 0,1] [--adjust-frames]
osatura keyedit <project.json> <out.json|-> offset --channels 0,1 --dt N | extrapolate --channels 0,1 --extrapolate loop | loop --channels 0,1 [--end F] | handle --sel SEL --side in|out --t F --v V [--mode free|aligned|auto] | lock --target T [--prop P] [--value on|off]
```

* **SEL** son elementos `TARGET:PROP[@F0..F1]` unidos por `;`, o `#CHANNEL[@F0..F1]` (índice de canal de `timeline`), o `all`. Entrecomíllalo (`;`, `@`, `#`).
* **E** (suavización) es `linear`, `step`, `ease_in`, `ease_out`, `ease_in_out`, `smooth`, `bounce`, `bezier:x1,y1,x2,y2` o un objeto JSON.
* `--collision` decide qué ocurre cuando una key desplazada cae sobre una key existente (`replace` por defecto, `merge`, `refuse`); `--no-snap` conserva los fotogramas fraccionarios; `--free` levanta la restricción de fotogramas enteros.
* Línea de salida: `<op>: N keys changed, wrote <out>`. `copy` con una salida real imprime `copy: 0 keys changed, project unchanged, clip saved`.
* Las operaciones que dejarían una key fuera de `0..=frames` se rechazan con un mensaje preciso (`duplicate: frame 52 is outside the project range 0..=48`) salvo que `ripple --adjust-frames` amplíe el proyecto.

```
$ osatura keyedit demo.json - move --sel "arm_up:bone_angle@12..36" --dt 2        # dry run: prints the batch JSON
{ "op": { "op": "batch", "ops": [ { "op": "set", "path": "/channels/0/track", ... } ] } }
$ osatura keyedit demo.json k1.json move --sel "arm_up:bone_angle@12..36" --dt 2 --collision merge
move: 3 keys changed, wrote k1.json
$ osatura keyedit demo.json k5.json ease --sel "arm_up:bone_angle" --ease bezier:0.4,0,0.2,1
ease: 5 keys changed, wrote k5.json
$ osatura keyedit demo.json k7.json smooth --sel "arm_up:bone_angle" --kind monotone
smooth: 4 keys changed, wrote k7.json
$ osatura keyedit demo.json k8.json copy --sel "arm_up:bone_angle@0..12" --clip c2.json
copy: 0 keys changed, project unchanged, clip saved
$ osatura keyedit demo.json k8.json paste --clip c2.json --at 30 --remap arm_up:bone_angle=arm_low:bone_angle
paste: 2 keys changed, wrote k8.json
$ osatura keyedit demo.json k9.json add --target arm_up --prop bone_angle --at 6 --value 5 --ease linear
add: 1 keys changed, wrote k9.json
$ osatura keyedit demo.json k10.json ripple --at 12 --by 6 --mode insert --adjust-frames
ripple: 28 keys changed, wrote k10.json            # project now 54 frames
$ osatura keyedit demo.json k1.json move --sel "arm_up:bone_angle@12..36" --dt 12 --collision refuse
a key already exists at frame 48                    [exit 1]
$ osatura keyedit demo.json k1.json move --sel "zzz:bone_angle" --dt 1
nothing matches 'zzz:bone_angle'                    [exit 1]
$ osatura keyedit demo.json k15.json lock --target arm_up --prop bone_angle --value on
lock: 1 keys changed, wrote k15.json
```
**Ojo:** `delete` con `#0@12..12` elimina exactamente una key; `ripple --mode delete --scope 0,1` limita el desplazamiento a los canales 0 y 1; `copy` y `paste` intercambian un archivo de clip, no el portapapeles del sistema. Los canales bloqueados rechazan `move/scale/...` (usa antes `lock --value off`).

### `marker`

**Sinopsis:** `osatura marker <project.json> <out.json> <name> <frame> [--end F]`

Añade un marcador a la línea de tiempo (un punto con nombre o, con `--end`, un rango). Un marcador con el mismo nombre se reemplaza. Los marcadores se ordenan por fotograma.

```
$ osatura marker small.json m1.json hit 4 --end 6
wrote m1.json (marker 'hit' at 4 = 00:00:00:04)
```

### `event`

**Sinopsis:** `osatura event <project.json> <out.json> <name> <frame> [--data JSON]`

Añade un evento con nombre (un disparador en tiempo de ejecución para los reproductores) en un fotograma, con una carga JSON opcional. JSON no válido: `--data is not valid JSON: ...`.

```
$ osatura event small.json ev1.json boom 3 --data '{"power":2}'
wrote ev1.json (1 events)
```

### `idle`

**Sinopsis:**
```
osatura idle blink <project.json> <out.json> --target T --prop P [--interval S --jitter X --closed F --seed N]
osatura idle breath <project.json> <out.json> --target T --prop P [--amp A --period F]
osatura idle sway <project.json> <out.json> --target T --prop P [--amp A --freq HZ --seed N]
```
Cada generador acepta solo sus propias opciones (`idle breath --seed 2` se rechaza).

Genera un canal procedural de reposo (idle) y lo **añade al final** (los canales existentes se conservan).

| Generador | Opciones (valores por defecto) | Destino típico |
|---|---|---|
| `blink` | `--interval` segundos medios entre parpadeos, `--jitter`, `--closed` fotogramas cerrado, `--seed` | un `layer_switch` (ojo abierto/cerrado) o `layer_scale_y` |
| `breath` | `--amp` (0.03), `--period` fotogramas (3 s a la frecuencia de fotogramas del proyecto) | `bone_scale` de un torso |
| `sway` | `--amp` (2), `--freq` Hz (0.3), `--seed` (1) | `bone_angle` |

```
$ osatura idle blink small.json i1.json --target mouth --prop layer_switch --interval 1 --seed 3
wrote i1.json
$ osatura idle sway small.json i3.json --target arm_up --prop bone_angle --amp 3 --freq 0.5
wrote i3.json
```
`--target` y `--prop` son obligatorios (`--target is required`).

### `follow` (dos formas)

**Follow-through (opciones):** `osatura follow <project.json> <out.json> --target SRC --prop P --chain a,b,c [--delay F --decay X]`
Añade copias retardadas y amortiguadas de un canal existente (`--target`/`--prop` nombran el origen) a los huesos de la cadena: acción superpuesta / follow-through (`--delay` fotogramas por eslabón, por defecto 3; `--decay` por eslabón, por defecto 0.7). Una copia retardada se corta en el último fotograma del proyecto (no se escribe ninguna key después del fotograma `frames`, por lo que `lint` L007 no se dispara); los fotogramas entre la última key conservada y el corte se muestrean para que la curva retardada sea exacta en cada fotograma entero.

```
$ osatura follow small.json f1.json --target arm_up --prop bone_angle --chain arm_low,hand --delay 2 --decay 0.6
wrote f1.json (+2 channels)
```

**Seguimiento de ruta (posicional):** `osatura follow <project.json> <out.json> <layer|bone:NAME> <layer_with_path> <start_frame> <end_frame> [no-orient]`
Hornea una capa (o el hueso raíz `bone:NAME`) que recorre la primera ruta de otra capa entre dos fotogramas, girando según la tangente de la ruta salvo que se indique `no-orient`.

```
$ osatura follow small.json fp.json target ink 0 7
wrote fp.json
$ osatura follow small.json fp2.json bone:hand ink 0 7 no-orient
path following needs a root bone (no parent)                      [exit 1]
```

### `take-record`, `take-apply`, `take-list`

**Sinopsis:**
```
osatura take-record <project.json> <library.json> <name> <from> <to> [target_prefix,...]
osatura take-apply  <project.json> <out.json> <library.json> <name> <at> [from=to,...] [speed] [--blend FRAMES]
osatura take-list   <library.json>
```
Un **take** es un fragmento reutilizable de animación almacenado en un archivo de biblioteca. `take-record` captura los fotogramas `from..to` de los canales cuyo destino empieza por uno de los prefijos (todos los canales si se omite); la biblioteca se crea si no existe y un take del mismo nombre se reemplaza. `take-apply` pega el take en el fotograma `at`, con la posibilidad de **retargetear** (los pares `from=to` renombran destinos) y de cambiar la `speed`; los argumentos se comprueban estrictamente: una entrada sin `=`, un nombre de origen para el que el take no tiene pista, o una velocidad que no es un número positivo es un error, y una biblioteca ilegible nombra el archivo (`cannot read the take library ...`); `--blend N` hace un fundido cruzado con la animación existente durante N fotogramas en ambos extremos.

```
$ osatura take-record demo.json lib.json wave 0 24 arm_
recorded 2 tracks into lib.json
$ osatura take-list lib.json
wave (24 frames, 2 tracks)
$ osatura take-apply demo.json ta.json lib.json wave 10 arm_up=arm_low 1.5 --blend 4
applied 2 tracks; wrote ta.json
```

### `clip`

**Sinopsis:**
```
osatura clip add <project.json> <out.json> <library.json> <take> <clip> [--at F --speed S --loops N --mode replace|additive|multiply --weight W
                   --include a_*,b --exclude c --track-weight arm_*=1,leg_*=0.2 --rename a=b --fade-in F --fade-out F]
osatura clip list <project.json>
osatura clip remove <project.json> <out.json> <clip>
osatura clip set-weight <project.json> <out.json> <clip> [W] [--keys frame:w,frame:w] [--track PATTERN[:prop]]
osatura clip bake <project.json> <out.json>
```
Una **pila de clips** por capas (animación no lineal): los clips hacen referencia a takes de una biblioteca, se reproducen en `--at` con `--speed`/`--loops` y se mezclan en modo `replace`, `additive` o `multiply` con un peso (constante, con keys en el tiempo, o por patrón de pista). `clip add/set-weight/remove` solo editan la pila; **`clip bake` regenera los canales pertenecientes a la pila** (los canales hechos a mano y los bloqueados nunca se tocan).

```
$ osatura clip add demo.json cl1.json lib.json wave w1 --at 6 --speed 1 --loops 2 --mode additive --weight 0.8 --fade-in 2 --include 'arm_*' --rename arm_up=hand
added clip 'w1' (1 clips); run clip bake to generate channels
$ osatura clip list cl1.json
1. w1 take 'wave' frames 6..30 x1 speed 1 Replace weight 1
0 baked channels
$ osatura clip set-weight cl1.json cl2.json w1 0.5
updated clip 'w1'; run clip bake to update channels
$ osatura clip set-weight cl1.json cl3.json w1 --keys 0:0,12:1
$ osatura clip bake cl2.json cl5.json
baked 1 clips into 4 channels (0 replaced); wrote cl5.json
$ osatura clip remove cl2.json cl6.json w1
removed clip 'w1'; run clip bake to update channels
```
(El ejemplo de `clip list` muestra el segundo `clip add` con el mismo nombre, que reemplazó al primero.) **Límites (ROADMAP §9):** los canales horneados se muestrean por fotograma entero; no existe un `clip refresh` para volver a traer un take de biblioteca editado.

### `make-loop`

**Sinopsis:** `osatura make-loop <project.json> <out.json> [targets|all] [props] [--inclusive]`

Hace que los canales `replace` con keys manuales terminen donde empezaron: cada curva se muestrea por fotograma, se resta la recta que une su primer y su último valor (eliminación de deriva) y se vuelve a sumar el primer valor. Aplicado al desplazamiento X de un hueso que viaja, convierte un paseo en un paseo en el sitio. El periodo del bucle es N = los `frames` del proyecto, de modo que reproducir `0..N-1` repetidamente no muestra ningún fotograma de costura duplicado. `--inclusive` es la convención heredada en la que el último fotograma es igual al primero. Sin argumentos se procesan todos los canales con keys. Imprime una línea `removed drift` por cada canal modificado.

```
$ osatura make-loop demo.json ml2.json foot_ctrl bone_offset_x
foot_ctrl BoneOffsetX: removed drift 0.79
wrote ml2.json
$ osatura make-loop demo.json ml.json all bone_angle --inclusive
```
**Límites:** es un detrending lineal de manual, no un reajuste de tiempos del movimiento; los canales gobernados por expresiones no se tocan. **El desplazamiento se elimina a propósito** (un proyecto que reproduce `0..N-1` repetidamente no puede viajar eternamente, así que el bucle es en el sitio): un canal cuyo cambio entero era desplazamiento pasa a ser constante, y el comando imprime para él una línea `note: ... is now constant`; `lint` notifica entonces ese canal como L005 (información), lo cual es lo esperado. Nombra los destinos y propiedades para dejar intacto el desplazamiento de otros canales, o usa `loop-solve` para un movimiento que ya es cíclico.

### `loop-solve`

**Sinopsis:** `osatura loop-solve <project.json> <out.json> [--window 24] [--inclusive]`

Resuelve la **costura**: mezcla el final de los canales con keys con su inicio a lo largo de una ventana para que la animación haga bucle sin saltos. Imprime el error de costura antes y después.

```
$ osatura loop-solve demo.json ls.json --window 24
wrote ls.json: 8 channel(s) adjusted, seam 0.000 -> 0.000
```
`--window` se expresa en fotogramas (por defecto 24). Compáralo con el contrato `loop` de `lint` (`contracts`).

### `walk`

**Sinopsis:** `osatura walk <project.json> <out.json> --thigh-l B --thigh-r B [--shin-l B --shin-r B --arm-l B --arm-r B --hips B --period N --stride DEG --knee DEG --bob PX --swing DEG]`

Genera un ciclo de caminata en bucle como canales con keys en los huesos que nombres (se requiere al menos una opción de hueso: `name at least one bone (--thigh-l, --thigh-r, ...)`). Valores por defecto: `--period 24` fotogramas, `--stride 28`, `--knee 40`, `--bob 4`, `--swing 22`. Los canales usan extrapolación en bucle y se **añaden al final**.

```
$ osatura walk small.json wk.json --thigh-l arm_up --thigh-r arm_low --period 8 --stride 20
wrote wk.json (+2 channels)
```

### `mirror`

**Sinopsis:** `osatura mirror <project.json> <out.json>`

Refleja todos los canales con keys respecto al eje vertical: los destinos intercambian lados (prefijos `L_`/`R_`, sufijos `_L`/`_R` y `_l`/`_r`, las palabras `left`/`right`), y las rotaciones y traslaciones horizontales cambian de signo. Los canales reflejados **reemplazan** a los canales del proyecto (no se añaden junto a los originales). La línea de resultado es `wrote X (N of M channel target(s) swapped sides)`; cuando ningún destino tiene un nombre izquierdo/derecho, o el proyecto no tiene canales, una advertencia en stderr lo indica (solo se invierten los signos).

### `foot-lock`, `stretch`, `magnet`

```
osatura foot-lock <project.json> <out.json> <ctrl_bone> <ground_y> [tolerance] [min_frames]
osatura stretch   <project.json> <out.json> <follower_bone> <leader_bone> [min=1] [max=3]
osatura magnet    <project.json> <out.json> <ctrl_bone> <bone:NAME|x,y> <radius> <strength 0..1>
```

* **`foot-lock`** muestrea la posición en el mundo del controlador en cada fotograma; las rachas de al menos `min_frames` fotogramas en que está en el suelo (el eje y del escenario apunta hacia abajo: el pie está abajo cuando `y >= ground_y - tolerance`) son *contactos*, y un canal de corrección **aditivo** fija el controlador donde empezó cada contacto. La animación original queda intacta; elimina los canales añadidos para deshacerlo.
  ```
  $ osatura foot-lock demo.json fl.json foot_ctrl 385 2 2
  contact frames 23..47 pinned at (479.2, 384.1)
  wrote fl.json
  ```
* **`stretch`** hace que `follower_bone` apunte a la cabeza de `leader_bone` y se escale (uniformemente, entre `min` y `max`, valores por defecto 1 y 3) para que su punta la alcance. Se hornea como un canal de ángulo aditivo más un canal de escala multiplicativo; los hijos del seguidor se escalan con él. El líder no puede depender del seguidor (`the leader depends on the follower (it is one of its descendants)`).
  ```
  $ osatura stretch p.json st.json flw lead 1 2
  corrected 8 frames
  wrote st.json
  ```
* **`magnet`** hornea una atracción por proximidad de un controlador hacia la cabeza de un hueso (seguida fotograma a fotograma) o hacia un punto fijo: caída suave, con `strength` completa (0..1) en el destino y cero en `radius`. Se hornea como canales de desplazamiento aditivos.
  ```
  $ osatura magnet p.json mg.json lead 200,100 200 0.8
  pulled on 8 frames
  wrote mg.json
  ```
**Límites (ROADMAP §9):** los tres son resultados horneados a partir de un muestreo por fotograma: vuelve a ejecutarlos tras editar la animación de origen. `stretch` usa una escala uniforme. `foot-lock` y `magnet` esperan un hueso controlador raíz.

### `shot`, `camera-seq`

```
osatura shot       <project.json> <out.json> <name> <x> <y> <zoom> [rotation]
osatura camera-seq <project.json> <out.json> <frame:shot[:cut],...>
```
`shot` guarda un **plano de cámara** con nombre (centro x, y, zoom, rotación opcional en grados). `camera-seq` anima con keys la cámara a través de los planos guardados: en cada `frame` la cámara se mueve al plano (con suavizado) o, con el sufijo `:cut`, salta a él. Escribe canales `camera_x/y/zoom/rotation`.

```
$ osatura shot small.json sh.json wide 320 240 1 15
wrote sh.json (1 shots)
$ osatura shot sh.json sh2.json close 400 200 2
wrote sh2.json (2 shots)
$ osatura camera-seq sh2.json cs.json 0:wide,4:close:cut
wrote cs.json
```

### `spring-fit`

**Sinopsis:** `osatura spring-fit --start V --samples a,b,c,... [--fps 24]`

Ajusta la **frecuencia (Hz) y la razón de amortiguamiento** de un muelle a una curva de respuesta dibujada (muestras por fotograma de un escalón desde `--start` hasta el último valor). Imprime el par ajustado y el JSON que pegar en los `springs` de un proyecto. No lee ningún proyecto.

```
$ osatura spring-fit --start 0 --samples 0.5,0.9,1.1,1.0,0.98,1.0 --fps 24
frequency 4.68 Hz, damping 0.61 (rms error 0.028 of the step size)
add to the project: {"bone": "<bone>", "frequency": 4.68, "damping": 0.61}
```

### `script`

**Sinopsis:** `osatura script <project.json> <out.json> <script.rhai>`

Ejecuta un **script Rhai en sandbox** que edita el proyecto mediante peticiones validadas (una petición rechazada deja todo sin cambios:
`request 5 (box.LayerOpacity at 0) was rejected, nothing changed: ...`). Funciones integradas: `fps() frames() bones() layers() set_key(target, prop, frame, value[, ease]) add_bone(name, parent (empty = root), length, x, y, angle) add_rect add_circle add_polygon|add_blob add_line add_gradient_rect add_ik add_aim add_copy_rotation add_pin add_group(name[, [layers]]) add_switch move_layer set_parent delete_layer delete_bone set_layer set_bone add_channel_expr add_noise add_marker add_event bone_info layer_info world print`.
Un script que no aplica ninguna petición imprime `warning: script: 0 requests were applied`. Las compilaciones sin la característica `scripting` responden `this build has no scripting support`.

```
$ cat t.rhai
add_rect("box", 10.0, 10.0, 50.0, 50.0, "#ff0000");
add_rect("box2", 80.0, 10.0, 50.0, 50.0, "#00ff00");
add_group("skins", ["box", "box2"]);
set_key("skins/box", "layer_opacity", 0, 1.0);
print(fps());
$ osatura script small.json sc.json t.rhai
24.0
4 request(s) applied; wrote sc.json
```
Las capas dentro de un grupo se direccionan por ruta (`skins/box`). `print` va a stdout.

---

## 7. Física, tela y deformadores

Todos estos comandos **hornean** una simulación o un resultado geométrico en canales con keys ordinarios: el proyecto sigue siendo determinista y el resultado se reproduce en cualquier sitio (incluidos Lottie/GLB donde corresponda). Vuelve a ejecutar el comando tras cambiar la animación de origen.
Toman `<project.json> <out.json>` y palabras `key=value` (sin `--`).

### `physics`

**Sinopsis:**
```
osatura physics <project.json> <out.json> <bone,bone,...> [gravity=900] [wind=x,y] [gust=0.5] [damping=0.985]
    [circle=x,y,r[@bone];...] [capsule=x1,y1,x2,y2,r[@bone];...] [follow=bone] [--allow-empty]
    [drive=bone:gain;...] [vortex=bone|x,y:swirl[:radial[:radius]];...] [preset=hair|tail|rope|antenna|cape]
    [merge=keep|replace] [lateral=0..1]
```
Hornea una **simulación de cadena** de Verlet (pelo, cola, cuerda, antena, capa) sobre los huesos indicados (primero la raíz de la cadena) como keys de `bone_angle`, una por fotograma.

| Palabra | Significado |
|---|---|
| `gravity=` | Aceleración hacia abajo (por defecto 900 unidades de escenario/s^2). |
| `wind=x,y`, `gust=`, | Vector de viento constante; amplitud de ráfaga de un viento modulado por ruido. |
| `damping=` | Amortiguamiento de la velocidad por paso (por defecto 0.985). |
| `circle=x,y,r[@bone];...`, `capsule=x1,y1,x2,y2,r[@bone];...` | Colisionadores, opcionalmente unidos a un hueso (lo siguen). Se aceptan cápsulas (verificado con `capsule=100,300,300,300,20`; una nota antigua de «capsule colliders NOT done» en ROADMAP §9 está desactualizada). |
| `follow=bone` | La raíz de la cadena se arrastra con el movimiento de este hueso (inercia). |
| `drive=bone:gain;...` | Cuerpos que empujan la cadena (su velocidad multiplicada por la ganancia). |
| `vortex=bone\|x,y:swirl[:radial[:radius]];...` | Campos de torbellino alrededor de un hueso o punto. |
| `preset=` | Valores iniciales para `hair tail rope antenna cape`; las palabras escritas **después** de `preset=` lo anulan. |
| `merge=keep\|replace` | `keep` (por defecto) deja intactos los huesos que ya tienen animación; `replace` los sobrescribe. |
| `lateral=0..1` | Con varias cadenas (`bone,bone/bone,bone`) une las hebras vecinas entre sí. |
| `--allow-empty` | Acepta una ejecución que no horneó nada. |

```
$ osatura physics p.json ph.json tail1,tail2,tail3 gravity=500 preset=tail
baked 3 bones; wrote ph.json
$ osatura physics p.json ph4.json tail1,tail2/tail3 lateral=0.5            # two strands
baked 3 bones; wrote ph4.json
$ osatura physics p.json ph3.json tail1,tail2,tail3 follow=arm_up circle=300,300,40 wind=10,0 gust=0.5 lateral=0.3 drive=lead:0.5
baked 3 bones; wrote ph3.json
$ osatura physics demo.json ph.json arm_low,hand gravity=500 preset=tail   # these bones are keyed already
kept existing animation on 'arm_low' (use merge=replace to overwrite)
kept existing animation on 'hand' (use merge=replace to overwrite)
physics: nothing baked - 2 bone(s) already have animation and merge=keep (the default) leaves it alone; nothing was written. Use merge=replace to overwrite the existing animation, or --allow-empty to accept an unchanged file     [exit 1]
$ osatura physics demo.json ph.json hand preset=bogus
unknown preset 'bogus' (try hair, tail, rope, antenna, cape)                [exit 1]
```
(`p.json` es la demo más tres huesos sin keys `tail1..tail3` creados con un script Rhai; véase `script`.) **Ojo:** un horneado que no cambia nada es un error salvo con `--allow-empty`.
**Límites (ROADMAP §9):** los colisionadores ignoran la escala de los huesos (solo transformación rígida); los círculos estáticos colisionan solo en puntos (los círculos unidos también a lo largo de los eslabones); no hay colisión cápsula-cápsula ni autocolisión.

### `cloth`

**Sinopsis:**
```
osatura cloth <project.json> <out.json> <layer> [pin=top,left@bone,row:N,col:N,v:N,corners,none] [gravity=900]
    [wind=x,y] [gust=0.5] [gust-hz=0.5] [gust-len=px] [stiffness=1] [shear=0.05] [bend=0.2] [max-stretch=1.1]
    [damping=0.99] [cols=N rows=N] [rect=x,y,w,h] [name=cloth] [substeps=4] [iterations=10] [preroll=frames]
    [circle=x,y,r[@bone];...] [capsule=x1,y1,x2,y2,r[@bone];...] [from=F] [to=F]
```
Hornea una **superficie de tela** determinista sobre la malla de una capa de *imagen*, o sobre una capa *vectorial* cuando se da `rect=x,y,w,h` (la capa vectorial recibe una malla en cuadrícula sobre ese rectángulo), como canales de deformación por vértice. `pin=` elige los vértices fijados (bordes `top`, `left`..., `corners`, `row:N`, `col:N`, `v:N`, `@bone` une los anclajes a un hueso, `none`). `from`/`to` limitan el rango de fotogramas horneado, `preroll` ejecuta la simulación antes del fotograma `from` para que la tela empiece en reposo.

```
$ osatura cloth small.json cloth.json sun rect=100,60,120,120 cols=6 rows=6 pin=top gravity=900 wind=40,0
baked cloth on 'sun' (84 channels); wrote cloth.json
$ osatura cloth small.json cloth.json sun
a vector layer needs rect=x,y,w,h                    [exit 1]
$ osatura cloth small.json cloth.json nosuch
no layer at 'nosuch'                                 [exit 1]
```
Los `84 channels` son keys de `lattice_dx` / `lattice_dy` (destino `sun::cloth::<col>,<row>`) de los 42 puntos no fijados de la cuadrícula de 7x7 (la fila superior está fijada). Son canales reales del archivo, así que un horneado de tela agranda el proyecto (98 canales en lugar de 14 aquí): usa `size-report` para ver el coste. **Límites (ROADMAP §9):** la tela es plana (sin pliegues fuera del plano, por lo que el cizallamiento es dócil por defecto); rechaza las mallas libres (solo mallas en cuadrícula); sin autocolisión; horneado por fotograma entero.

### `cage`

**Sinopsis:** `osatura cage <project.json> <out.json> <layer> [rect=x,y,w,h|auto] [hull] [verts=N] [pad=P] [name=cage]`

Añade un **deformador de jaula de coordenadas de valor medio** alrededor de una capa: el contenido dentro de la jaula sigue los vértices de la jaula, el contenido fuera no cambia. `auto` (por defecto cuando no hay `rect`) usa los límites con relleno, `verts=N` es el número de vértices por lado, `hull` usa la envolvente convexa, `pad=` añade relleno a la caja, `name=` da nombre al deformador (por defecto `cage`).

```
$ osatura cage small.json cg.json sun auto verts=3
added cage 'cage' (12 vertices) to 'sun'; wrote cg.json
```
**Límites (ROADMAP §9):** no hay ajuste automático de jaula más allá de una caja con relleno o una envolvente convexa (sin trazado cóncavo); no hay una caída suave fuera de la jaula (aparece un escalón visible donde el arte cruza el borde de la jaula); las mallas libres no tienen desplazamientos con keys por vértice (usa una jaula).

### `cage-pose`

**Sinopsis:** `osatura cage-pose <project.json> <out.json> <layer> <frame> [name=cage] [move=i:dx,dy;...] [all=dx,dy] [ease=linear]`

Coloca keys de desplazamientos de vértices de la jaula en un fotograma (canales `cage_dx` / `cage_dy`). `move=` desplaza vértices individuales, `all=` desplaza todos los vértices. Los canales bloqueados se rechazan.

```
$ osatura cage-pose cg.json cp.json sun 4 all=5,0
keyed cage 'cage' on 'sun' at frame 4; wrote cp.json
$ osatura cage-pose cg.json cp.json sun 4 move=0:5,5
keyed cage 'cage' on 'sun' at frame 4; wrote cp.json
```

### `warp-pivot` y `soft-scale`

**Sinopsis:**
```
osatura warp-pivot <project.json> <out.json> <layer> <frame> [deformer=NAME] [verts=1,4-7] [region=x,y,w,h]
    [pivot=x,y|bone:NAME|centroid|center] [rotate=DEG] [scale=S|sx,sy] [move=dx,dy] [radius=R] [falloff=smooth] [ease=linear]
osatura soft-scale <project.json> <out.json> <layer> <frame> [deformer=NAME] [verts=..|region=..] scale=S|sx,sy
    [radius=R] [falloff=smooth] [pivot=..]
```
`warp-pivot` rota/escala/mueve una **selección** de puntos de control (deformadores de jaula, lattice, tela, pin, spine; una malla de imagen en cuadrícula recibe una cuadrícula de warp por vértice) alrededor de un pivote y coloca la key del resultado en el fotograma. La selección es `verts=` (índices y rangos) y/o `region=`; una selección vacía es un error (`the selection is empty (give verts= and/or region=)`). `radius=` añade una **caída suave** (`linear smooth gauss sharp constant`). `soft-scale` es un escalado proporcional (selección suave); sin `verts`/`region` toda la malla se redimensiona respecto al centro de su caja delimitadora con `scale=F`, `width=W`, `height=H` o `size=WxH`, con `aspect=keep|free`.

```
$ osatura warp-pivot cg.json wp.json sun 4 verts=0-3 rotate=15 pivot=centroid radius=30 falloff=gauss
warp-pivot on 'sun' keyed at frame 4; wrote wp.json
$ osatura soft-scale cg.json ss.json sun 4 scale=1.2 pivot=center
soft-scale on 'sun' keyed at frame 4; wrote ss.json
```

### `layer-pivot`

**Sinopsis:** `osatura layer-pivot <project.json> <out.json> <layer> pivot=x,y|bone:NAME`

Mueve el pivote (ancla) de una capa **sin cambiar la imagen**, de modo que los canales de rotación y escala giran en torno al nuevo punto.

```
$ osatura layer-pivot small.json lp.json sun pivot=100,100
moved the pivot of 'sun'; wrote lp.json
```

### `mesh-lock`

**Sinopsis:** `osatura mesh-lock add <project.json> <out.json> <name> host=LAYER[:path] follower=LAYER[:path] [verts=1,4-7] [strength=1]`
y `osatura mesh-lock list|remove <project.json> [<out.json> <name>]`

Hace que los vértices del seguidor sigan la **superficie de la malla anfitriona** en cada fotograma (transferencia de desplazamiento baricéntrica), p. ej. una camisa que sigue a un cuerpo. Se rechazan los ciclos; el anfitrión y el seguidor deben tener el mismo `parallax` (`mesh lock 'lock1': host and follower must have the same parallax`).

```
$ osatura mesh-lock add two.json ml1.json lock1 host=ra follower=rb verts=0-1 strength=0.5
added mesh lock 'lock1'; wrote ml1.json
$ osatura mesh-lock list ml1.json
lock1: rb:0 follows ra:0 vertices [0, 1] strength 0.5
1 mesh lock(s)
$ osatura mesh-lock remove ml1.json ml2.json lock1
removed mesh lock 'lock1'; wrote ml2.json
```

### `glue`

**Sinopsis:** `osatura glue add <project.json> <out.json> <name> a=LAYER[:path] b=LAYER[:path] pairs=i:j;i:j|auto=RADIUS [mode=rigid|snap|offset] [strength=1]`
y `osatura glue list|remove <project.json> [<out.json> <name>]`

Pega vértices de la capa `b` a vértices de la capa `a` (b sigue a a). `pairs=i:j;...` enumera explícitamente los pares de vértices, `auto=RADIUS` empareja cada vértice de `b` que se encuentra dentro del radio de un vértice de `a`. Modos: `rigid` mantiene el desplazamiento de reposo en el sistema de `a`, `snap` hace que coincidan, `offset` mantiene un desplazamiento constante.

```
$ osatura glue add two.json gl1.json g1 a=ra b=rb auto=5 mode=snap
added glue 'g1' (4 pair(s)); wrote gl1.json
$ osatura glue list gl1.json
g1: Snap strength 1 ra:0#0->rb:0#0 ra:0#1->rb:0#1 ra:0#2->rb:0#2 ra:0#3->rb:0#3
1 glue(s)
$ osatura glue add small.json x.json g1 a=sun b=hills auto=20 mode=snap
no vertex of 'hills' lies within 20 of a vertex of 'sun'          [exit 1]
```

### `mesh-trace`

**Sinopsis:** `osatura mesh-trace <project.json> <out.json> <layer> [density=24] [tolerance=1.5] [smooth=N] [hotspot=x,y,r;...] [edge=D/4] [grading=0.5] [lambda=0.5] [taubin=1] [smooth-boundary=0] [threshold=128] [bones=R] [hot-size=E] [skin=1]`

Traza la **silueta alfa de una capa de imagen** en una malla triangular libre de densidad variable (más densa a lo largo del contorno, alrededor de los puntos calientes y de los huesos), la suaviza y le aplica skinning a los huesos. Imprime el número de vértices/triángulos y la calidad.

```
$ osatura mesh-trace o.json mt.json top density=12 tolerance=2 bones=0
wrote mt.json: 9589 vertices (1494 on the outline), 17682 triangles, area 1227677 px2
  quality min 0.484 mean 0.890, min angle 18.3 deg, mean edge 3.0 px at the outline / 13.7 px inside
```
(`o.json` es una importación OpenRaster de 1280x960; un `density` menor produce menos triángulos, más grandes. La ejecución tardó 5 s en una compilación debug.) **Límites:** necesita una capa de imagen (una silueta alfa); una malla tan densa hace grande el archivo de proyecto: comprueba `size-report`. Las mallas libres no admiten simulación de tela.

### `skin-diffuse`

**Sinopsis:** `osatura skin-diffuse <project.json> <layer> <out.json> [--influences 4]`

Calcula **pesos de skin por difusión de calor** para una capa con malla a partir de los huesos del proyecto, conservando como máximo `--influences` (por defecto 4) huesos por vértice. Ten en cuenta el orden de los argumentos: **la capa antes de la salida**. La capa debe ser una capa de imagen con malla.

```
$ osatura skin-diffuse demo.json arm sd.json
layer 'arm' is not an image layer                          [exit 1]
$ osatura skin-diffuse o.json top sd.json
layer 'top' has no mesh                                    [exit 1]
$ osatura skin-diffuse mt.json top sd.json               # mt.json from mesh-trace
wrote sd.json
```

---

## 8. Audio, lip-sync y sonidos

Las entradas de audio de los comandos de análisis (`lipsync`, `lipsync-text`, `speech-motion`, `audio-react`, `beats`, `waveform`) deben ser **WAV** (PCM entero de 8/16/24/32 bits o flotante de 32 bits, cualquier número de canales mezclado a mono, hasta 256 MiB). MP3/OGG necesitan convertirse antes (`ffmpeg -i in.mp3 in.wav`). Las **pistas** de sonido almacenadas en un proyecto (`sound`) pueden tener cualquier formato que ffmpeg sepa leer.

### `lipsync`

**Sinopsis:** `osatura lipsync <project.json> <audio.wav> <switch_layer> <out.json> [--map 0,1,2,3,4]`

Gobierna una **capa switch** (formas de la boca) a partir de la sonoridad y el brillo espectral: una heurística de amplitud / cruces por cero que elige una de cinco formas por fotograma (`rest, small, open, round, wide`) y elimina el parpadeo. **No es reconocimiento de fonemas.** `--map` da el índice de hijo del switch para cada una de las cinco formas; el valor por defecto depende del número de hijos del switch: 1 hijo -> todas; 2 -> cerrada + abierta; 3 -> cerrada + abierta + redonda/ancha; 4 -> cerrada + pequeña + abierta + redonda/ancha; 5+ -> en el orden anterior. Reemplaza un canal `layer_switch` con keys existente de esa capa. La longitud del proyecto **no** cambia (un WAV de 2 s en un proyecto de 8 fotogramas coloca keys en 8 fotogramas).

```
$ osatura lipsync small.json speech.wav mouth lip.json
wrote lip.json (2.00s of audio → 8 frames)
$ osatura lipsync small.json speech.wav mouth lip.json --map 0,1,2,1,0
$ osatura lipsync small.json speech.wav mouth lip.json --map 0,1
--map needs exactly 5 numbers (rest,small,open,round,wide)          [exit 1]
$ osatura lipsync small.json speech.wav ink lip.json
layer 'ink' is not a switch layer                                   [exit 1]
$ osatura lipsync small.json notwav.mp3 mouth x.json
invalid WAV: Failed to read enough bytes.                           [exit 1]
```

### `lipsync-text`

**Sinopsis:** `osatura lipsync-text <project.json> <audio.wav> <transcript.txt> <switch_layer> <out.json> [--map 0,1,2,3,4] [--lang ro|en] [--words-out timing.json|timing.srt]`

Alinea una **transcripción** con el audio (energía + alineación de texto: las palabras y sílabas se colocan sobre las regiones de voz encontradas en el audio) y deduce las formas de la boca a partir de las sílabas. `--lang` (por defecto `ro`) selecciona las reglas de vocales/sílabas; solo existen `ro` y `en`. `--words-out` escribe los tiempos de las palabras como JSON (palabras con sílabas) o, para un nombre `.srt`, como subtítulos.

```
$ osatura lipsync-text small.json speech.wav transcript.txt mouth lt.json --lang en --words-out words.json
wrote lt.json (3 words, 2 exposure changes; energy + text alignment, not phoneme recognition)
$ head -c 140 words.json
{ "words": [ { "text": "hello", "start": 0.0, "end": 0.22, "syllables": [ {"text":"hel","start":0.0,"end":0.16}, {"text":"lo",...
$ osatura lipsync-text ... --words-out w.srt ; head -4 w.srt
1
00:00:00,000 --> 00:00:01,220
hello there world
```
**Límites (ROADMAP §9):** sin temporización por letra (a nivel de fonema), sin modelos de velocidad específicos del hablante, sin más idiomas que ro/en. Un tono sintético se alinea como si fuera voz: usa grabaciones reales.

### `speech-motion`

**Sinopsis:** `osatura speech-motion <project.json> <audio.wav> <transcript.txt> <out.json> [--head bone] [--brow bone_or_layer] [--nod deg] [--fall deg] [--tilt deg] [--brow-px px] [--seed n] [--lang ro|en] [--words-out timing.json|timing.srt]`

Añade cabeceos, caídas, inclinaciones y alzamientos de ceja **aditivos** sincronizados con las palabras alineadas. `--head` nombra el hueso de la cabeza, `--brow` un hueso o capa para las cejas; las opciones de ángulo van en grados, `--brow-px` en píxeles, `--seed` fija la variación.

```
$ osatura speech-motion small.json speech.wav transcript.txt sm.json --head arm_up --lang en
wrote sm.json (1 additive channel(s) from 3 words)
```

### `mouth`

**Sinopsis:** `osatura mouth <project.json> <cues.txt> <switch_layer> <out.json> [--map 0,1,2,3,4] [--rhubarb]`

Importa **señales** de boca ya hechas en lugar de analizar audio: archivos Papagayo/Moho-switch (`frame PHONEME` por línea, primera línea opcional `MohoSwitch1`) o TSV de Rhubarb (`seconds<TAB>shape`, reconocido por el punto decimal; los segundos se convierten a la frecuencia de fotogramas del proyecto). Se ignoran las líneas en blanco y los comentarios `#`. Los fonemas se asignan a las cinco clases de forma (`rest/MBP/X/A` reposo; `etc/FV/L/B/G/H` pequeña; `AI/D` abierta; `O/U/WQ/F` redonda; `E/C` ancha); las formas desconocidas cuentan como `small`. `--rhubarb` trata la `E` de Rhubarb como *redonda* (la `E` de Papagayo es ancha). El `--map` por defecto es `0,1,2,3,4`.

```
$ printf '0 rest\n2 open\n4 round\n6 wide\n' > cues.txt
$ osatura mouth small.json cues.txt mouth mo.json --rhubarb
wrote mo.json (4 cues)
```

### `audio-react`

**Sinopsis:** `osatura audio-react <project.json> <out.json> <audio.wav> <target> <prop> [gain=1] [offset=0] [floor=0.05] [attack=1] [release=4] [add=on]`

La sonoridad gobierna una propiedad: una key por fotograma que sigue el nivel suavizado (RMS relativo al fotograma más sonoro). Salida = (nivel filtrado por `floor`) x `gain` + `offset`; `attack`/`release` son los tiempos de subida/bajada en fotogramas; `add=on` hace el canal aditivo en lugar de reemplazante. Un canal bloqueado se rechaza. Las opciones son palabras `key=value`; una desconocida da `unknown option 'foo'`.

```
$ osatura audio-react small.json ar.json speech.wav ink layer_rotation gain=10
wrote ar.json
$ osatura audio-react small.json ar.json speech.wav arm_up bone_angle foo=1
unknown option 'foo'                                                [exit 1]
```

### `beats`

**Sinopsis:** `osatura beats <project.json> <out.json> <audio.wav> [sensitivity=1.5] [gap=4] [as=markers|events] [name=beat]`

Detecta saltos de sonoridad por encima del promedio reciente (`sensitivity` 1.2-3, mayor = menos pulsos) separados al menos `gap` fotogramas, y los registra como **marcadores** de la línea de tiempo (por defecto) o como **eventos** llamados `name`. Imprime los fotogramas encontrados.

```
$ osatura beats small.json bt.json speech.wav as=events name=hit sensitivity=1.2
1 beats at frames [1]; wrote bt.json
$ osatura beats small.json bt.json speech.wav
0 beats at frames []; wrote bt.json
```

### `waveform`

**Sinopsis:** `osatura waveform <audio.wav> <out.json> [buckets=800]`

Picos mínimo/máximo para dibujar una forma de onda: `{"sample_rate","duration","peaks":[[min,max],...]}`. No lee ningún proyecto.

```
$ osatura waveform speech.wav wf.json buckets=50
wrote wf.json (50 buckets)
```

### `sound`

**Sinopsis:**
```
osatura sound list   <project.json>
osatura sound add    <project.json> <out.json> <name> <file.wav|ogg|mp3|...> [start_frame] [gain] [trim/fade options]
osatura sound edit   <project.json> <out.json> <name> [--start F] [--gain G] [trim/fade options]
osatura sound remove <project.json> <out.json> <name>
osatura sound mix    <project.json> <out.wav|out.ogg|out.mp3>
    trim/fade options (seconds, or frames with an f suffix such as 36f): --trim-in T --trim-out T --fade-in T --fade-out T
```
Las pistas de sonido son **referencias** (ruta de archivo relativa al proyecto, fotograma de inicio, ganancia 0..4, ventana de recorte dentro del archivo, fundidos). No se incrusta nada en el proyecto; el archivo de audio debe viajar con él y debe estar dentro del directorio del proyecto salvo que se indique `--allow-external-assets`. `edit` cambia solo los campos indicados (`0` borra un recorte). `--trim-out 0` significa «hasta el final». `sound mix` renderiza solo el audio mezclado para la **longitud del proyecto** (mediante ffmpeg: wav = PCM de 44,1 kHz y 16 bits, ogg = Vorbis q4, mp3 = 96 kb/s).
El audio mezclado también se multiplexa en `.mp4` (AAC) y `.webm` (Opus) y se incrusta en `.html`; `.gif`/`.apng` no pueden transportarlo.

```
$ osatura sound add small.json s1.json music speech.wav 2 0.8 --fade-in 0.5 --trim-out 36f
wrote s1.json
$ osatura sound list s1.json
music: speech.wav at frame 2 gain 0.8 trim 0s..1.5s fade-in 0.5s
$ osatura sound edit s1.json s2.json music --gain 0.5 --trim-in 0.1 --fade-out 4f
$ osatura sound list s2.json
music: speech.wav at frame 2 gain 0.5 trim 0.1s..1.5s fade-in 0.5s fade-out 0.16666667s
$ osatura sound mix s2.json mix.mp3
wrote mix.mp3 (0.33 s)
$ osatura sound mix s2.json mix.flac
unknown audio format '.flac' (use .wav, .ogg or .mp3)            [exit 1]
$ osatura sound add small.json x.json x nosuch.wav              # a missing file is refused, nothing written
```
`check` falla cuando un archivo de sonido referenciado falta o es ilegible. Sin ffprobe, para los archivos que no son WAV solo se comprueba la existencia (no que el archivo sea audio). **Límites (ROADMAP §9):** sin pan/EQ/ducking por pista, solo la curva de fundido lineal, sin bucle de un clip, sin reajuste de tiempos.

---

## 9. Importación

### `import-svg`

**Sinopsis:** `osatura import-svg <file.svg> <out.json> [--split]`

Línea de uso: `osatura import-svg <file.svg> <out.json> [--split]  SVG subset -> one vector layer, or one layer per shape named by its id (warnings for the rest)`

Crea un proyecto nuevo (24 fps, 24 fotogramas; tamaño tomado de `viewBox` o de `width`/`height`, y 512x512 en su defecto) con **una capa vectorial `svg`** que contiene las formas de un subconjunto de SVG: `path` (todos los comandos), `rect` (las esquinas redondeadas se ignoran), `circle`, `ellipse`, `line`, `polyline`, `polygon`, `g`/`a` con `transform`, y `fill`/`stroke`/`stroke-width` sólidos. Todo lo demás se omite con una línea `warning:` **en stdout** (cuando el elemento está fuera de `<defs>`; un `<filter>`, `<clipPath>`, `<mask>` o `<pattern>` dentro de `<defs>` y los atributos `filter`, `clip-path` y `mask` se ignoran sin aviso): `<text>`, `<filter>`, `<clipPath>`, `<mask>`, `<pattern>`, `<linearGradient>`/`<radialGradient>`, `<image>`, `<use>`, `<foreignObject>`. Un relleno de degradado (`fill="url(#g)"`) se sustituye por gris. Límites: 8 MB de SVG, 200 000 anclas y un tope de elementos.

```
$ osatura import-svg s.svg svg.json
warning: unsupported paint 'url(#g)'; using grey
warning: <text> is not supported and was skipped
warning: <filter> is not supported and was skipped
warning: <image> is not supported and was skipped
wrote svg.json (3 paths)
$ osatura check svg.json
ok: 's' 200×100 @ 24 fps, 24 frames, 0 bones, 0 ik, 1 layers, 0 channels
```
Con `--split` cada forma se convierte en **su propia capa**, con el nombre del `id` de la forma, en el orden del archivo. Una forma sin id recibe el nombre de su etiqueta y su posición (`rect1`, `circle6`), un id repetido recibe `_1`, `_2`..., y los caracteres que un nombre de capa no puede contener (`/`, `::`, caracteres de control) se convierten en `_`; los nombres se cortan a 100 caracteres. Es la forma que conviene usar cuando el dibujo se va a preparar con huesos parte por parte; véase [TUTORIAL_SVG.md](../../TUTORIAL_SVG.md).

```
$ osatura import-svg pip.svg pip.json --split
wrote pip.json (16 paths, 16 layers)
```
Importar y luego fusionar: no existe ningún comando que fusione la capa en un proyecto existente; usa `script` o edita el JSON.

### `import-ora`

**Sinopsis:** `osatura import-ora <file.ora> <out.json> [--fps N] [--frames N]`

OpenRaster -> proyecto: una **capa de imagen por cada capa ráster** (PNG incrustado), con el orden de abajo hacia arriba conservado, las capas ocultas mantenidas pero invisibles, los grupos aplanados, la `opacity` de cada capa, `visibility`, la posición (`x`,`y`) y las operaciones de composición `svg:` habituales (multiply, screen, overlay, darken, lighten, color-dodge/burn, hard/soft-light, difference, exclusion, plus -> add). El proyecto tiene `--frames` fotogramas (1 por defecto) a `--fps` (24 por defecto). La opacidad de una pila anidada se multiplica en sus capas y su modo de fusión se hereda, pero las capas superpuestas dentro de una pila translúcida **no** se aplanan antes (se transparentan entre sí). Los nombres de capa duplicados reciben los sufijos `_1`, `_2`.

```
$ osatura import-ora t.ora ora.json
wrote ora.json (2 layers)
$ osatura import-ora t.ora ora2.json --fps 12 --frames 24
$ osatura import-ora layer.png bad.json
not a zip/ORA file: invalid Zip archive: Could not find EOCD       [exit 1]
```
Límites: 2000 entradas de archivo, 128 MiB por entrada, capas de 64 Mpíxeles.

### `import-lottie`

**Sinopsis:** `osatura import-lottie <in.json> <out.json> [fx.json]`

Importa un archivo Lottie (Bodymovin) -un **subconjunto** deliberado- como proyecto. Imprime `imported N layers; wrote <out>` y una línea `warning:` por cada característica omitida.

* **Se importa:** capas de formas con trazados `sh`, `rc`, `el`, rellenos sólidos y de degradado (los puntos finales/centro/radio animados del degradado pasan a ser canales `grad_dx/grad_dy`), trazos, grupos, transformaciones de capa (posición/escala/rotación/opacidad estáticas o con claves; las trayectorias de posición curvas se hornean por fotograma), modos de fusión, una capa de fondo sólido, capas de imagen con URI de datos PNG/JPEG incrustados, capas nulas como padres (las transformaciones del padre se hornean en los hijos), máscaras de capa (add/subtract/intersect con opacidad, inversión, formas con claves), mattes de luminancia (aproximados), precomposiciones, rangos de entrada/salida de capa (como pista de opacidad en escalón) y suavizado de claves (hold -> step; manejadores bezier -> cubic-bezier; sin manejadores -> lineal).
* **Trazados animados:** un trazado cuyas claves de forma tienen la misma topología (mismo número de anclas, abierto/cerrado) se convierte en un trazado base más un **morph target** por cada clave distinta adicional (`k1`, `k2`, ...), con canales de peso `path_morph` que transportan el suavizado. Con topología distinta se usa la primera forma y se imprime una advertencia.
* **Efectos (`ef`):** el desenfoque gaussiano, la sombra paralela, el tinte y el relleno se convierten en **efectos por capa** (`effects` en la capa, de cualquier tipo) con los números animados como canales `fx_*`; el tinte/relleno/tritono/niveles estáticos en capas de forma se hornean en los colores. Los colores de los efectos no se pueden animar (se usa el primer valor); los demás efectos, y los efectos de color posteriores a un desenfoque/sombra en capas que no son de forma, generan advertencias. De cada capa se conservan como máximo los primeros efectos.
* **Se omite con una advertencia:** texto y otros tipos de capa, recursos de audio, puntos de anclaje animados (primer valor), trayectorias de movimiento curvas en propiedades distintas de la posición (segmentos rectos), máscaras sobre una fuente de track-matte, cadenas de padres demasiado largas para hornearse, más de 2000 capas, animaciones de más de 20 000 fotogramas.
* `fx.json` se conserva por compatibilidad: se escribe un array vacío (el desenfoque y la sombra paralela son ahora efectos por capa dentro del proyecto).

```
$ osatura import-lottie out2.json li.json                   # a Lottie file made by `osatura export`
imported 11 layers; wrote li.json
$ osatura import-lottie in.json out.json                    # hand-made file: animated triangle with animated blur, plus a null and a text layer
warning: layer 'text' of type 5 skipped
imported 1 layers; wrote out.json
$ osatura check out.json
ok: 't' 200×200 @ 24 fps, 24 frames, 0 bones, 0 ik, 1 layers, 2 channels       # channels: blob::path::k1 path_morph, blob::fx0 fx_blur
$ osatura import-lottie out2.json li2.json fx.json
wrote 0 post effect(s) to fx.json
imported 11 layers; wrote li2.json
$ echo '{}' > bad.json; osatura import-lottie bad.json o.json
missing 'w'                                                        [exit 1]
```
**Trampa:** `import-lottie in.json in.json` se rechaza (`refusing to overwrite the input ...`), también para `fx.json`. **Límites:** la entrada `.lottie` (zip) no se lee (`stream did not contain valid UTF-8`): descomprime antes `a/animation.json`; el trabajo cuadrático en los morphs de forma sólo está acotado por los límites de tamaño.

### `import-bvh`

**Sinopsis:** `osatura import-bvh <project.json> <file.bvh> <out.json> --joint-map Joint=bone,.. [--axis Zrotation --tol 0.5]`

Asigna los canales de rotación de las articulaciones BVH elegidas a huesos del rig como canales `bone_angle` con claves, muestreados a los fps y al número de fotogramas del proyecto. `--joint-map` es obligatorio; `--axis` (por defecto `Zrotation`) selecciona qué canal de rotación BVH se lee (`Zrotation`, `Xrotation`, `Yrotation`); `--tol` (0,5 grados por defecto) simplifica las claves. Para la asignación automática, el movimiento de la raíz, las vistas y la escala usa `mocap-retarget` (sección 10).

```
$ osatura import-bvh demo.json m.bvh ib.json --joint-map LeftUpLeg=thigh,LeftLeg=shin --axis Zrotation --tol 0.5
wrote ib.json (+2 channels from 4 BVH frames)
```

### `import-seq`

**Sinopsis:** `osatura import-seq <project.json> <out.json> <layer_name> <folder> [hold=1] [loop=on|off] [start=0] [x=0] [y=0]`

Importa una **secuencia de PNG** (todos los PNG de la carpeta, por orden de nombre de archivo) como una capa fotograma a fotograma: una capa switch llamada `<layer_name>` con una capa de imagen por fotograma y un canal en escalón que muestra cada una durante `hold` fotogramas a partir del fotograma `start`, en la posición `(x, y)`; `loop=on` la repite. Todas las imágenes deben tener el mismo tamaño (`img.png: size 640x480 differs from the first image (160x120)`).

```
$ osatura import-seq demo.json is.json frames seq hold=2 loop=on start=4 x=10 y=20
imported 3 images; wrote is.json
```

### `import-anim`

**Sinopsis:** `osatura import-anim <project.json> <out.json> <layer_name> <file.gif|file.apng|file.png> [loop=on|off] [start=0] [x=0] [y=0]`

Importa un **GIF animado o APNG** como una capa fotograma a fotograma, usando los retardos de fotograma del propio archivo (convertidos a fotogramas del proyecto). Un PNG simple da un solo fotograma.

```
$ osatura import-anim demo.json ia.json gifl dof.gif loop=off start=0
imported 4 frames as layer 'gifl'; wrote ia.json
$ osatura import-anim demo.json ia2.json apngl dof.apng
imported 3 frames as layer 'apngl'; wrote ia2.json
```
Las imágenes de los fotogramas se convierten en PNG incrustados: un GIF largo hace que el proyecto sea grande.

### `trace`

**Sinopsis:** `osatura trace <image.png> <out.json> [--threshold 128 --tolerance 1.0 --min-area 12 --fill #000000]`

Conversión sencilla de mapa de bits a vectores para imágenes con **transparencia**: traza el **canal alfa** del PNG (marching squares con `--threshold`, de 0 a 255, con interpolación lineal de los bordes), simplifica los contornos (Ramer-Douglas-Peucker) hasta `--tolerance` píxeles, descarta las regiones por debajo de `--min-area`, detecta los huecos por contención y escribe un proyecto nuevo (llamado `traced`, con el lienzo del tamaño de la imagen) con los contornos como trazados rellenos del color `--fill`. Por tanto, una imagen totalmente opaca produce un único contorno alrededor de toda la imagen (como en el ejemplo). Imprime `wrote <out> (N contours)`. Usa `autotrace` para imágenes en color.

```
$ osatura trace img.png tr.json --threshold 128 --tolerance 1.0 --min-area 12 --fill '#112233'
wrote tr.json (1 contours)
```

### `autotrace`

**Sinopsis:** `osatura autotrace <image.png|jpg> <out.json|out.svg> [--mode fill|line] [--colors N=8] [--min-area A=8] [--tol T=1] [--smooth N=3] [--corner DEG=55] [--denoise N=0] [--quant kmeans|mediancut] [--seed S] [--fit bezier|polygon|vw] [--zorder area|luminance] [--threshold T] [--invert] [--prune N=4] [--into project.json] [--name L] [--fit-stage] [--png preview.png]`

Vectorización local de mapas de bits: **sin modelo y sin red**. Cuantización de color (`--colors`, k-means o median cut), extracción de contornos, ajuste de curvas Bezier cúbicas de Schneider (`--fit`; `polygon` y Visvalingam-Whyatt `vw` son alternativas), huecos por par-impar; `--mode line` adelgaza los trazos hasta las líneas centrales (Zhang-Suen; `--prune` elimina las espuelas cortas). La salida es un proyecto nuevo, un `.svg` nuevo o, con `--into project.json`, una copia de un proyecto existente con un nuevo grupo de capas `--name` (por defecto `trace`); `--fit-stage` (con `--into`) escala el grupo nuevo de forma uniforme para ajustarlo y centrarlo en el escenario del proyecto, y `--png` escribe una vista previa renderizada. Imprime recuentos y el **error medido respecto al original** (error absoluto medio, RMSE, PSNR, SSIM).

```
$ osatura autotrace img.png at.json --colors 4 --png prev.png
layer group 'trace' added
wrote at.json: 44 regions, 0 strokes, 44 paths, 324 anchors
fidelity vs source: MAE 13.38/255, RMSE 17.59, PSNR 23.2 dB, SSIM 0.960
$ osatura autotrace img.png at.svg --mode line --threshold 100
wrote at.svg: 0 regions, 11 strokes, 11 paths, 47 anchors
fidelity vs source: MAE 52.47/255, RMSE 58.73, PSNR 12.8 dB, SSIM 0.903
$ osatura autotrace img.png at2.json --into d.json --name traced
layer group 'traced' added
wrote at2.json: 51 regions, 0 strokes, 51 paths, 347 anchors
```
**Límites (ROADMAP §9):** es un vectorizador clásico: las fotografías se convierten en bandas de color; no hay vectorización aprendida ni semántica. Aquí `--png` es una opción con valor (nombre del archivo de vista previa), a diferencia de otras opciones `--png`.

### `autotrace-bench`

**Sinopsis:** `osatura autotrace-bench`

Vectoriza formas sintéticas, degradados y trazos incorporados e imprime una tabla de fidelidad (MAE, RMSE, PSNR, SSIM, recuentos de regiones/anclas, milisegundos). No recibe argumentos; es de sólo lectura.

```
$ osatura autotrace-bench
image          mode  colors  tol     MAE    RMSE   PSNR   SSIM regions strokes  anchors      ms
shapes         fill       6    1    2.19   12.10   26.5  0.975      11       0       33    13.6
...
```

---

## 10. Entrada en vivo, MIDI, seguimiento facial y captura de movimiento

### `live-map`

**Sinopsis:** `osatura live-map <project.json> <events.json|song.mid> <mapping.json> <out.json> [--takes lib.json] [--tol 0.01] [--allow-empty]`

Convierte una **interpretación en vivo grabada** (MIDI, ratón, mando, teclas, señales faciales) en canales con claves, eventos y takes ordinarios (no destructivo, todo o nada, respetando los bloqueos). La entrada es un archivo MIDI estándar `.mid` o un JSON de eventos `{"events":[{"source","id","value","time"}]}` (`value` normalizado de 0 a 1, `time` en segundos; `source` = `midi_cc midi_note midi_pitch_bend mouse_x mouse_y mouse_button gamepad_axis gamepad_button key face`; `id` = en MIDI `"<canal 1-16>:<número>"`, en el botón del ratón `"0"`, en el mando `"<pad>:<index>"`, en las teclas un código DOM como `KeyA`).

El archivo de mapeo tiene `params` (source/id -> `target`+`prop`, con `in_min in_max out_min out_max deadzone invert curve smooth additive`) y `triggers` (cruce de umbral -> acción `jump_to_frame`, `play_take`, `set_key`, ...); las claves desconocidas son errores. `--takes` es una biblioteca de takes para las acciones `play_take` (debe existir) y `--tol` simplifica las claves.

```
$ cat map.json
{"params":[{"source":"mouse_x","id":"0","target":"arm_up","prop":"bone_angle","out_min":-30,"out_max":30}],
 "triggers":[{"source":"key","id":"KeyA","action":{"type":"jump_to_frame","frame":3}}]}
$ osatura live-map small.json ev.json map.json out.json
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 1 events (0 after the last frame skipped); wrote out.json
$ osatura live-map small.json song.mid mapm.json outm.json              # MIDI CC 1 on channel 1 -> arm_up
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 0 events (0 after the last frame skipped); wrote outm.json
$ osatura live-map small.json ev.json map.json out.json --takes lib.json    # lib.json missing
cannot read lib.json: No such file or directory (os error 2)             [exit 1]
$ osatura live-map small.json empty.json map.json out.json
note: map 0 (arm_up.BoneAngle): no matching events, nothing keyed
0 events -> +0 channels, ...; wrote out.json
```
**Trampa:** si se leyeron eventos pero **no se produjo nada** (errata en un id, source equivocado), el comando falla y no escribe nada salvo que se use `--allow-empty`; con cero eventos de entrada simplemente escribe el proyecto sin cambios. Los disparadores de salto se registran como un evento llamado `jump` con `data.to` (el eje temporal no se reescribe).
**Límites (ROADMAP §9):** este comando no ofrece entrada en vivo de webcam ni de nivel de audio en el estudio (`audio-react` funciona sin conexión); la parte interactiva la proporciona el estudio.

### `drive`

**Sinopsis:** `osatura drive <project.json> <signals.csv> <maps.json> <out.json> [--tol 0.5]`

Interpretación -> claves a partir de un **CSV**: la fila de encabezado nombra las señales y cada fila siguiente es un fotograma (índice de fila = fotograma). `maps.json` es un array de `{input, target, prop, in_min, in_max, out_min, out_max, deadzone, invert, curve, smooth}`; `--tol` (0,5 por defecto) simplifica las claves.

```
$ printf 'jaw,speed\n0,0\n0.5,0.2\n1,0.4\n0.5,0.6\n' > sig.csv
$ echo '[{"input":"jaw","target":"arm_low","prop":"bone_angle","out_min":0,"out_max":40,"smooth":0.2}]' > maps.json
$ osatura drive demo.json sig.csv maps.json dr.json --tol 0.5
wrote dr.json (+1 channels)
```

### `face-synth`, `face-train`, `face-eval`, `face-detect`, `face-track`, `face-calibrate`, `face-bake`

Una canalización dirigida por el rostro que es **totalmente local y está entrenada únicamente con rostros sintéticos** (cabezas estilizadas/de dibujos animados, 23 puntos de referencia, disposición `osatura23`). Un modelo pequeño entrenado con datos sintéticos va incorporado en el binario; `--model` lo sustituye. **Las fotografías, las webcams y el vídeo real no están validados** (ROADMAP §9; véase también [PATENT_NOTES.md](PATENT_NOTES.md): el manejo de títeres dirigido por el rostro no ha pasado ningún análisis de patentes).

| Comando | Sinopsis | Qué hace |
|---|---|---|
| `face-synth` | `osatura face-synth <out-dir> [--frames 90] [--size 160] [--seed 7] [--fps 30]` | Animación sintética de una cabeza: fotogramas PNG `000000.png...`, `truth.ndjson` (puntos de referencia reales) y `params.json` (señales reales); `--fps` es la frecuencia de fotogramas con la que se temporizan las señales (se escribe en el encabezado del NDJSON). |
| `face-train` | `osatura face-train --samples N --seed S --out model.bin [--mine-rounds 1] [--scene N] [--reg-win N] [--reg-cell N] [--det-win N] [--det-cell N] [--ridge-lambda X] [--stages N] [--refine-win N] [--stage-patch N] [--stage-cell N] [--stage-lambda X]` | Entrena el detector propio sólo con rostros sintéticos (sin descargas). `--scene` es el tamaño de la imagen de entrenamiento (64..512); `--reg-win`, `--reg-cell`, `--det-win`, `--det-cell`, `--ridge-lambda`, `--stages`, `--refine-win`, `--stage-patch`, `--stage-cell` y `--stage-lambda` son los parámetros de ajuste del tamaño del modelo y de la regresión (los valores por defecto son los que se usaron para entrenar el modelo incorporado). |
| `face-eval` | `osatura face-eval [--model m.bin] [--eval-n 200] [--seed 9] [--scene 128] [--report r.json] [--json]` | Precisión sintética con datos reservados, falsos positivos y barridos de robustez (JSON; `--report` lo escribe, `--json` lo imprime incluso con `--report`; `--scene` es el tamaño de la imagen de prueba). |
| `face-detect` | `osatura face-detect <image> [--model m.bin] [--json] [--overlay o.png] [--threshold T] [--max-faces N] [--min-side N] [--max-side N] [--max-dim N]` | Cuadros delimitadores de rostros y 23 puntos de referencia; `--overlay` los dibuja en un PNG; `--min-side` / `--max-side` acotan el tamaño del rostro en píxeles y `--max-dim` reduce antes una imagen grande. |
| `face-track` | `osatura face-track <video-or-frame-dir> <out.ndjson> [--model m.bin] [--fps 30] [--max-dim 256]` | NDJSON de puntos de referencia para `face-bake`. Un directorio de imágenes se lee directamente; un vídeo pasa por ffmpeg. |
| `face-calibrate` | `osatura face-calibrate <landmarks.ndjson> <out.json> [--calibrate-frames N] [--map mapfile.json] [--fps F] [--layout L]` | Calcula la calibración del rostro neutro a partir de los primeros N fotogramas. |
| `face-bake` | `osatura face-bake <project.json> <landmarks.ndjson> <mapfile.json> <out.json> [--calibrate-frames N\|0] [--calib calib.json] [--fps F] [--layout L] [--tol T] [--allow-empty]` | Hornea las señales faciales (`mouth_open`, `eye_l_open`, `eye_r_open`, `brow_raise`, `yaw`, `pitch`, `roll`, ...) en canales con un archivo de mapeo al estilo de `live-map` cuyo `source` es `face`. |

```
$ osatura face-synth syn --frames 6 --size 96 --seed 3
wrote 6 frames (96x96), truth.ndjson and params.json to syn
$ osatura face-train --samples 80 --seed 1 --out m.bin --scene 64        # 47 s in a debug build
wrote m.bin (649900 bytes) in 47.0s: 80 samples, seed 1, mined negatives 9
trained on synthetic faces only: expect stylised/cartoon faces to work, photographs and webcams are NOT validated
$ osatura face-train --samples 60 --seed 1 --out m.bin --scene 2
scene size must be 64..512                                                [exit 1]
$ osatura face-detect syn/000000.png --model m.bin
syn/000000.png: face 0: box centre (48.1, 47.4) side 69.0 score 4.49
$ osatura face-track syn trk.ndjson --model m.bin --fps 15 --max-dim 96
wrote trk.ndjson: 6 frames, face found in 6
$ osatura face-calibrate syn/truth.ndjson calib.json --calibrate-frames 3 --fps 15
6 frames, neutral from the first 3; wrote calib.json
$ echo '{"params":[{"source":"face","id":"mouth_open","target":"arm_low","prop":"bone_angle","out_min":0,"out_max":60}],"triggers":[]}' > map.json
$ osatura face-bake small.json trk.ndjson map.json baked.json --calibrate-frames 3 --fps 15 --tol 0.5
6 landmark frames -> +1 channels (K keys in them), 0 trigger keys set, 0 events; wrote baked.json
$ osatura face-eval --eval-n 6 --model m.bin --report ev.json            # writes the JSON report
```
Estos comandos escriben sus archivos de modelo/informe con el escritor atómico que rechaza enlaces simbólicos (las carpetas padre se crean). `face-bake` y `live-map` informan `+N channels (K keys in them), M trigger keys set` y avisan por stderr cuando la grabación es más larga que el proyecto (los fotogramas sobrantes no se hornean; la duración del proyecto no cambia).

### `mocap-info`, `mocap-match`, `mocap-retarget`

```
osatura mocap-info     <file.bvh>
osatura mocap-match    <project.json> <file.bvh> [--map map.json]
osatura mocap-retarget <project.json> <file.bvh> <out.json> [--view front|side|top|yaw=DEG] [--map map.json] [--scale auto|N] [--fps F]
    [--smooth N] [--start F] [--end F] [--rest-frame F] [--min-conf C] [--weight W] [--mirror] [--no-root] [--in-place] [--keep-length] [--footlock BONE --ground Y]
```
* **`mocap-info`** imprime la jerarquía, los canales, la frecuencia de fotogramas y la extensión de un archivo BVH.
  ```
  $ osatura mocap-info m.bvh
  4 joints (2 end sites), 15 channels per frame
  4 frames at 0.0417 s (24.000 fps), duration 0.125 s
  Hips  offset (0.000, 0.000, 0.000)  [Xpos Ypos Zpos Zrot Xrot Yrot]
    LeftUpLeg  offset (5.000, 0.000, 0.000)  [Zrot Xrot Yrot]
  ...
  ```
* **`mocap-match`** imprime la **tabla de asignación automática articulación -> hueso** (slot, articulación BVH, hueso del rig, confianza, evidencia `via`, proporción de longitud, nota), y después las articulaciones/huesos sin asignar y las advertencias. Se usan tanto los nombres como la topología. `--map` es un objeto JSON `{"bvh joint": "rig bone" | null}` (`null` o `""` excluye una articulación; también se acepta envolverlo como `{"map": {...}}`). Siempre termina con código 0: cuando no coincide nada, sólo avisa con `no bones matched: supply a --map file`.
  ```
  $ osatura mocap-match demo.json m.bvh --map map2.json         # {"Hips":null,"LeftUpLeg":"thigh","LeftLeg":"shin"}
  slot         bvh joint                  rig bone                conf  via         len x  note
  shin.l       LeftLeg                    shin                    1.00  manual      12.00  manual
  thigh.l      LeftUpLeg                  thigh                   1.00  manual      12.00  manual
  unmatched BVH joints: Hips, RightUpLeg
  ...
  ```
* **`mocap-retarget`** hornea el movimiento como canales **añadidos**: proyecta las rotaciones 3D a la `--view` 2D (`front` por defecto, `side`, `top` o `yaw=DEG`), escala el movimiento de la raíz (`--scale auto` o un número), remuestrea a `--fps` (por defecto, el del proyecto), suaviza (`--smooth` fotogramas), toma los fotogramas BVH de `--start..--end`, usa `--rest-frame` como pose de reposo, descarta las coincidencias por debajo de `--min-conf`, mezcla con `--weight`, refleja con `--mirror`, descarta el movimiento de la raíz con `--no-root`, convierte un clip con desplazamiento en uno en el sitio con `--in-place` y fija un pie con `--footlock BONE --ground Y`. Imprime la tabla, las notas y `N bones, F frames at R fps, root scale S, +C channels; wrote <out>`. Un clip más largo que el proyecto **amplía la duración del proyecto**; una nota tras el resumen indica de qué duración a cuál. `--keep-length` conserva en cambio la duración del proyecto y trunca las claves en el último fotograma (allí se coloca una clave interpolada; una nota cuenta las claves no horneadas). Si nada supera el umbral de confianza: `no bone matches to retarget (all below the confidence threshold?)` (código de salida 1). Nota: en este comando `--scale` es `auto|N`, no la escala de renderizado habitual.
  ```
  $ osatura mocap-retarget demo.json m.bvh mr.json --map map2.json --view side --smooth 1 --in-place --no-root
  ... (table)
  2 bones, 4 frames at 24 fps, root scale 12.0000, +2 channels; wrote mr.json
  ```
**Límites (ROADMAP §9):** no se probó ningún archivo real de captura de movimiento (no se incluye ninguno: los ejemplos usan un BVH de 4 fotogramas escrito a mano); la proyección 2D descarta la profundidad (un hueso que apunta hacia la cámara mantiene su último ángulo).

---

## 11. Máquinas de estados, enlaces de datos, contenedores y regiones de desplazamiento

Estos comandos gestionan las partes de **interacción en tiempo de ejecución** de un proyecto: máquinas de estados (`machines`), enlaces de datos
(`bindings`), contenedores de maquetación (`containers`) y regiones de desplazamiento (`scrolls`). Todos guardan sus datos dentro
del archivo del proyecto; las formas `add` leen un pequeño archivo JSON auxiliar, lo validan y escriben un proyecto nuevo.
El propio modelo de runtime está documentado en [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

### `machine` (add, list, remove, validate)

**Sinopsis:**
```
osatura machine add      <project.json> <out.json> <machine.json>
osatura machine list     <project.json>
osatura machine remove   <project.json> <out.json> <name>
osatura machine validate <project.json>
```

Una **máquina** tiene `inputs` con nombre (`bool`, `number` o `trigger`) y una o más `layers`; cada capa tiene un estado `entry`,
`states` y `transitions`. Un estado reproduce una **fuente**: `{"kind":"segment","start":F,"end":F}` (fotogramas de los
canales con claves del proyecto), `{"kind":"clip","clip":NAME}`, `{"kind":"region","marker":NAME}` o un blend tree 1D
`{"kind":"blend","input":NAME,"children":[{"threshold":T,"source":{...}}]}`. Campos de un estado: `name`, `source`,
`mode` (`loop` por defecto, `once`, `ping_pong`), `speed` (1), `blend_in` (segundos). Campos de una transición: `from` (un estado o
`"*"`), `to`, `when` (una lista de `{"input":NAME,"op":eq|ne|lt|le|gt|ge,"value":V}`; todas deben cumplirse; `{"input":"go"}`
solo significa «es igual a 1»), `exit_time` (0..1), `duration` (segundos de fundido cruzado) y `ease`. Las claves desconocidas en una transición
son un **error** (una errata como `conditions` convertiría, de otro modo, una transición con condición en una incondicional).

```
$ cat mach.json
{"name":"m","inputs":[{"name":"go","kind":"bool"}],"layers":[{"name":"base","entry":"idle","states":[
 {"name":"idle","source":{"kind":"segment","start":0,"end":12}},
 {"name":"act","source":{"kind":"segment","start":12,"end":36}}],
 "transitions":[{"from":"idle","to":"act","when":[{"input":"go"}],"duration":0.1}]}]}
$ osatura machine add demo.json d1.json mach.json
added machine 'm' (1 machines); wrote d1.json
$ osatura machine list d1.json
m  inputs [go:bool]
  layer base (entry idle, mask []): 2 states, 1 transitions
$ osatura machine validate d1.json
1 machines, 0 bindings: ok
```

`machine remove` se niega a eliminar una máquina que aún controla un enlace
(`binding 'goBind' still drives machine 'm'`, código de salida 1): elimina primero el enlace.
**Errores:** una máquina no válida (estado desconocido, nombres duplicados, referencia de entrada errónea) se rechaza con un mensaje y
no se escribe nada. **Relacionado:** `machine-sim`, `bind`, `state-bake` (el formato de máquina anterior, basado en fotogramas).
**Límites:** las máquinas las evalúan el runtime y `machine-sim`; los exportadores no hornean nada de esto (Lottie, GIF
y los demás archivos contienen únicamente fotogramas, véase [EXPORT_FORMATS.md](EXPORT_FORMATS.md)).

### `machine-sim`

**Sinopsis:** `osatura machine-sim <project.json> <script.json> <out_dir|out.json> [--fps N --svg-frames N]`

Reproduce eventos de entrada temporizados sobre una máquina y registra el estado de cada tick. El script es
`{"machine": NAME?, "duration": SECONDS?, "events": [{"t": SECONDS, "set": {"input": value}, "trigger": ["name"],
"value": {"external.path": number}}]}`. `machine` toma por defecto la única máquina del proyecto, y `duration` el último
evento más un segundo; `value` alimenta los enlaces de datos (véase `bind`). El `--fps` por defecto son los fps del proyecto.
`--svg-frames N` renderiza además N fotogramas SVG repartidos a lo largo de la ejecución.

```
$ cat sc.json
{"duration":1.5,"events":[{"t":0.5,"value":{"hud.go":1}}]}
$ osatura machine-sim d3.json sc.json sim.json --fps 24 --svg-frames 0
   0.000s  base     Entered idle
   0.500s  base     Exited idle
   0.500s  base     Entered act
37 ticks at 24 fps, 3 events, 0 svg frames; wrote sim.json
$ osatura machine-sim d3.json sc.json simdir --svg-frames 2     # a name that does not end in .json is a folder
...
37 ticks at 24 fps, 3 events, 2 svg frames; wrote simdir/sim.json
$ ls simdir
frame_00000.svg  frame_00036.svg  sim.json
```
`sim.json` contiene `machine`, `fps`, `ticks[]` (`tick`, `time`, `states` como pares `[layer,state]`, `fading`) y el
registro de eventos. **Relacionado:** `bind`, `machine`.

### `state-bake`

**Sinopsis:** `osatura state-bake <project.json> <machine.json> <events.json> <out.json> [--tol 0.01]`

Hornea una máquina de estados **basada en fotogramas** (un formato distinto y más antiguo que `machine`) en canales con claves ordinarios.
`machine.json` es `{"initial":NAME,"states":[{"name","length","looped","channels":[...]}],"transitions":[{"from":NAME
or "*","to","when":COND,"fade":FRAMES,"ease"}]}` donde `COND` es uno de `{"type":"event","name"}`,
`{"type":"above"|"below"|"equals","input","value"}`, `{"type":"finished"}`, `{"type":"after","frames"}`.
`events.json` es una lista de `{"frame":N,"name":NAME,"value":V}`. El resultado se añade a los canales del proyecto
(`--tol` es la tolerancia de reducción de claves), el proyecto se compila y se escribe.

```
$ echo '[{"frame":10,"name":"go"}]' > ev.json
$ osatura state-bake demo.json sm.json ev.json baked.json --tol 0.01
wrote baked.json (+1 channels)
```
**Trampa:** el canal horneado se añade después de los existentes; los canales se ejecutan en el orden de la lista, así que elimina o silencia el
canal original del mismo destino si quieres que prevalezca el horneado. Un archivo de máquina en el formato `machine`
(con `layers`) se rechaza: `invalid JSON in mach.json: missing field 'initial'`.

### `bind` (add, list, remove) y `bind-eval`

**Sinopsis:**
```
osatura bind add    <project.json> <out.json> <name> <value.path> <target:prop|@machine:input>
                     [--expr E --remap a,b,c,d --no-clamp --smooth S --mode replace|add|multiply --weight W]
osatura bind list   <project.json>
osatura bind remove <project.json> <out.json> <name>
osatura bind-eval   <project.json> <values.json> [--data out.json]
```

Un **enlace de datos** asocia una ruta de valor externa (por ejemplo `hud.size`) con una propiedad del proyecto (`sun:layer_opacity`;
los nombres de propiedad son los snake_case de la sección 1) o con una entrada de máquina (`@m:go`). `--remap a,b,c,d` asigna el
rango de entrada `a..b` al rango `c..d` (acotado salvo con `--no-clamp`); `--expr` es una expresión sobre el valor;
`--smooth S` aplica un filtro paso bajo durante S segundos; `--mode` y `--weight` controlan cómo se combina el resultado con los canales.

```
$ osatura bind add d1.json d2.json goBind hud.go @m:go
added binding 'goBind' (1 bindings); wrote d2.json
$ osatura bind add d2.json d3.json sz hud.size sun:layer_opacity --remap 0,100,0.2,1 --smooth 0.1
added binding 'sz' (2 bindings); wrote d3.json
$ osatura bind list d3.json
goBind  hud.go -> @m:go
sz  hud.size -> sun:LayerOpacity remap 0..100 -> 0.2..1 clamp smooth 0.1s
$ echo '{"hud.go":1,"hud.size":50}' > vals.json
$ osatura bind-eval d3.json vals.json --data bv.json
goBind = 1
sz = 0.6000000238418579
$ osatura bind add d2.json x.json sz hud.size sun:scale
unknown property 'scale'                                                  [exit 1]
$ osatura bind remove d3.json d7.json sz
removed binding 'sz'
```
`bind-eval` imprime la salida de cada enlace para los valores dados (`--data` también la escribe como JSON). `list` imprime los nombres de
propiedad en CamelCase; la línea de comandos siempre admite snake_case. **Relacionado:** `machine`, `machine-sim` (su mapa `value`).

### `container` (add, list, remove, solve)

**Sinopsis:**
```
osatura container add    <project.json> <out.json> <container.json>
osatura container list   <project.json>
osatura container remove <project.json> <out.json> <id>
osatura container solve  <project.json> [--size WxH --data out.json]
```

Un **contenedor** es una caja de maquetación tipo flexbox: `id`, `members`, `axis` (`row`|`column`), `gap`, `cross_gap`, `padding`
(`[left,top,right,bottom]`), `justify` (`start|center|end|space_between|space_around`), `align`
(`start|center|end|stretch`), `wrap` (`none|wrap`) y `rect`, una regla para su propio rectángulo en el lienzo:
`{"kind":"fixed",...}`, `{"kind":"pin","left","right","top","bottom","width","height"}` (distancias a los bordes del lienzo; dos cualesquiera
por eje), `{"kind":"aspect",...}` o `{"kind":"hug"}`. Un **miembro** es exactamente uno de `{"layer":PATH}`,
`{"container":ID}` o `{"spacer":true}`, con `grow`, `shrink`, `width`, `height`, `min_w/max_w/min_h/max_h` y
una anulación de `align`; `"resize":true` permite que el solucionador escale la capa a su hueco.

```
$ cat cont.json
{"id":"row1","axis":"row","gap":10,"padding":[10,10,10,10],
 "rect":{"kind":"pin","left":10,"right":10,"top":10,"height":100},
 "members":[{"layer":"sun"},{"layer":"blush","grow":1,"resize":true}]}
$ osatura container add d3.json d4.json cont.json
added container 'row1' (1 containers); wrote d4.json
$ osatura container list d4.json
row1: Row, 2 members, gap 10, justify Start, align Start, wrap None, rect {"kind":"pin","left":10.0,...}
$ osatura container solve d4.json --size 800x480 --data solved.json
canvas 800x480
container row1: x 10.00 y 10.00 w 780.00 h 100.00
layer sun: dx -430.000 dy 0.000 scale 1.0000 x 1.0000
layer blush: dx 280.432 dy -158.242 scale 33.8889 x 1.0000
```
Sin `--size` se usa el tamaño del propio proyecto. `solve` imprime el rectángulo de cada contenedor y, por capa, el desplazamiento y la
escala que la maquetación aplica sobre la animación. **Errores:** una capa desconocida, un contenedor anidado inexistente o un
ciclo hacen que `add` lo rechace (no se escribe nada). **Relacionado:** `scroll`, `layout-preview`.
**Límites:** la maquetación la aplican el runtime y la vista previa; los archivos exportados contienen la maquetación resuelta para el tamaño propio del proyecto.

### `scroll` (add, list, remove, eval)

**Sinopsis:**
```
osatura scroll add    <project.json> <out.json> <scroll.json>
osatura scroll list   <project.json>
osatura scroll remove <project.json> <out.json> <id>
osatura scroll eval   <project.json> <id> [--offset X,Y --size WxH --velocity VX,VY --time S]
```

Una **región de desplazamiento** tiene `id`, `content` (el id de un contenedor que no es miembro de otro contenedor), `viewport`
(una regla de rectángulo como la anterior), `axis` (`x`, `y` o ambos), `offset`, `elastic` (`{"limit","stiffness","damping_ratio"}`; efecto
de goma en lugar de un tope rígido), `snap_x`/`snap_y` (desplazamientos de reposo), `fling` (segundos de velocidad de suelta proyectados para hallar
el punto de aterrizaje, 0,15 por defecto), `clip_layer` (una capa rectangular usada como máscara del contenido) e `input` (una ruta de valor externa que
controla el desplazamiento).

```
$ cat scroll.json
{"id":"sc1","content":"row1","axis":"x","viewport":{"kind":"pin","left":0,"right":0,"top":0,"height":120},
 "elastic":{"limit":60,"stiffness":200,"damping_ratio":0.8},"snap_x":[0,100,200]}
$ osatura scroll add d4.json d5.json scroll.json
added scroll 'sc1' (1 scrolls); wrote d5.json
$ osatura scroll list d5.json
sc1: content row1, axis X, offset [0.0, 0.0], elastic true, snap x [0.0, 100.0, 200.0] y [], clip -
$ osatura scroll eval d5.json sc1 --offset 400,0
viewport [0.0, 0.0, 640.0, 120.0] content [-59.92364, 0.0, 640.0, 120.0] max [0.0, 0.0]
requested offset [400.0, 0.0] -> applied [59.92364, 0.0]
$ osatura scroll eval d5.json sc1 --offset 80,0 --velocity -500,0 --time 0.5
...
requested offset [80.0, 0.0] -> applied [44.184174, 0.0]
t 0.100s offset [22.794, 0.000]
t 0.300s offset [0.051, 0.000]
t 0.500s offset [-0.197, 0.000]
```
Sin `--velocity`, `eval` muestra el desplazamiento acotado (o con efecto de goma) correspondiente al solicitado. Con `--velocity`,
simula la suelta (inercia, snap, muelle) durante `--time` segundos e imprime cada 0,1 s. `--size WxH` evalúa para otro tamaño de lienzo.
En el ejemplo el contenido es tan ancho como el viewport, de modo que todo lo que pasa de 0 es sobredesplazamiento y muestra el efecto de goma.

### `layout-preview`

**Sinopsis:** `osatura layout-preview <project.json> <out.svg> --size WxH [--frame N --scroll id=X,Y]`

Renderiza un fotograma con la maquetación **resuelta de nuevo para otro tamaño de lienzo** y los contornos de los contenedores dibujados, de modo que puedas
comprobar cómo se reorganiza un diseño en un teléfono o en una pantalla ancha. `--scroll id=X,Y` fija un desplazamiento de scroll (repítelo para varias regiones).

```
$ osatura layout-preview d5.json lp.svg --size 800x480 --scroll sc1=50,0
wrote lp.svg (800x480, 1 containers, 2 layers placed)
```

## 12. Efectos, profundidad de campo, onion skin, trayectorias y cámara

### `fx`

**Sinopsis:** `osatura fx <project.json> <frame> <chain.json> <out.png> [--scale N] [--layer-normals] [--clamp]`

Un `<frame>` fuera del proyecto es un error; `--clamp` renderiza el fotograma válido más cercano.

Renderiza un fotograma y le aplica una **cadena de postefectos**. `chain.json` es una lista de efectos, cada uno con un `type`:
`blur {radius}`, `drop_shadow {dx,dy,blur,color,opacity}`, `glow {radius,strength,color?}`, `outline {width,color}`,
`posterize {levels}`, `pixelate {size}`, `levels {in_black,in_white,gamma,out_black?,out_white?}`, `fade {amount}` y
`relight {ambient,bevel,height,lights:[{x,y,z,color?,intensity?,radius?,cast_shadow?,spot?,falloff?}],ground_shadow?,
normal_map?,ambient_color?}`. Sin un `normal_map`, el relight sombrea un bisel creado a partir del canal alfa desenfocado.

```
$ cat chain.json
[{"type":"drop_shadow","dx":6,"dy":6,"blur":3,"color":"#000000","opacity":0.5},{"type":"glow","radius":6,"strength":0.5}]
$ osatura fx demo.json 12 chain.json fx.png --scale 0.5
wrote fx.png (2 effect(s))
$ cat relight.json
[{"type":"relight","ambient":0.4,"bevel":6,"height":8,"lights":[{"x":300,"y":100,"z":120,"intensity":1.0}],"ground_shadow":0.3}]
$ osatura fx demo.json 12 relight.json lit.png --scale 0.5
wrote lit.png (1 effect(s))
```
`--layer-normals` sombrea cada capa con su propio mapa de normales (véase `layer-normal`) en lugar de un único mapa para toda la imagen.
Una luz que omite `z` se rechaza (`missing field 'z'`). **Límites:** la cadena es una canalización de imágenes fijas sólo para PNG; ninguno de
estos efectos llega a Lottie, GIF ni al reproductor HTML (el exportador de Lottie tiene su propio subconjunto de efectos, véase EXPORT_FORMATS.md).

### `color-space`

**Sinopsis:** `osatura color-space <project.json> <out.json> srgb|linear`

Establece el `color_space` del proyecto, el espacio en el que mezclan el compositor ráster (PNG, GIF, APNG, vídeo, sprite sheet, `compare`, `ci`) y el compositor de coma flotante (`render-hdr`). `srgb` (el valor por defecto, y con el que se grabó cada golden) mezcla los valores almacenados codificados en sRGB; `linear` convierte cada color (rellenos, trazos, paradas de degradado, fondo, píxeles de imagen) a luz lineal, compone allí en f32 y convierte a sRGB una sola vez al final. Una capa negra al 50 % sobre blanco da entonces 188/255 en lugar de 128/255, y los degradados se interpolan en luz lineal. Con `srgb` la clave no se escribe, de modo que los archivos y los golden existentes no cambian.

```
$ osatura color-space demo.json lin.json linear
wrote lin.json (color_space: linear)
$ osatura render lin.json 0 lin.png
wrote lin.png
$ osatura color-space lin.json x.json p3
unknown colour space 'p3' (srgb or linear)
```
**Límites:** el renderizador de 8 bits cambia al compositor de coma flotante en los proyectos lineales (rasterizador de cobertura propio, por lo que los bordes con antialiasing difieren ligeramente de la salida sRGB); los colores de los efectos de capa (sombra, glow, tinte) se usan tal como están escritos, sin convertir; `dof` y la cadena de postefectos (`fx`) siguen en sRGB; SVG, Lottie y el renderizador de GPU ignoran el ajuste; el remuestreo de imágenes se hace en sRGB antes de la conversión. Los modos de fusión se aplican a los valores lineales.

### `layer-normal`

**Sinopsis:** `osatura layer-normal <project.json> <out.json> <layer> <normal.png|none> [--strength S] [--fit stretch|tile|exact] [--green-down]`

Asocia (o, con `none`, elimina) un mapa de normales en espacio tangente a una capa de imagen para `fx ... --layer-normals`.
`--fit` indica cómo se ajusta un mapa de otro tamaño (`stretch` por defecto, `tile`, `exact`); `--green-down` invierte el canal
verde (mapas de estilo DirectX); `--strength` escala la inclinación (1 = tal como está almacenado, 0 = plano).

```
$ osatura layer-normal demo.json dn.json sun n.png --strength 1 --fit stretch
wrote dn.json
$ osatura fx dn.json 12 chain.json fxn.png --scale 0.5 --layer-normals
wrote fxn.png (2 effect(s))
$ osatura layer-normal dn.json dn2.json sun none
wrote dn2.json
```

### `onion` (dos formas)

**Sinopsis:**
```
osatura onion <project.json> <frame> <out.png> [--before 2 --after 1 --step 1 --scale N]
osatura onion <project.json> <out.svg> --frame N [--before B --after A --step S --keyed --outline --all-parts
               --layers a,b --tint-before C --tint-after C --opacity X --falloff X --ghosts-only]
```
La primera forma escribe un PNG con fantasmas atenuados de los fotogramas anteriores y posteriores. La segunda escribe una imagen fija SVG: el
fotograma con fantasmas de onion skin teñidos encima. `--keyed` usa los **fotogramas clave** más cercanos en lugar de pasos fijos;
`--outline` dibuja los fantasmas como contornos; `--all-parts` crea fantasmas de todas las partes (por defecto, las partes en movimiento), `--layers` los limita,
`--ghosts-only` omite el propio fotograma.

```
$ osatura onion demo.json 12 on.png --before 2 --after 1 --step 2 --scale 0.5
wrote on.png
$ osatura onion demo.json on.svg --frame 12 --before 2 --after 2 --step 3 --outline --opacity 0.4
wrote on.svg (4 ghost(s) around frame 12)
$ osatura onion demo.json on2.svg --frame 12 --keyed --ghosts-only
wrote on2.svg (3 ghost(s) around frame 12)
```
La forma se elige por el segundo argumento: un número indica la forma PNG; un nombre que termina en `.svg`, la forma SVG.

### `trail` (dos formas)

**Sinopsis:**
```
osatura trail <project.json> <bone> <out.svg> [--from F --to F]
osatura trail <project.json> [out.svg] --target layer:PATH|bone:NAME[:head|tip]|point:LAYER:KIND:A[:B] [--from F --to F --smooth N --json]
```
Dibuja la **trayectoria de movimiento** de la punta de un hueso (primera forma) o de cualquier destino (segunda forma: una capa, la cabeza/punta de un hueso o un
punto de deformador), con una marca por fotograma y marcadores en las claves. `--smooth N` suaviza la curva; `--json` imprime la
curva muestreada como JSON (y entonces `out.svg` es opcional).

```
$ osatura trail demo.json hand tr.svg --from 0 --to 24
wrote tr.svg (25 points)
$ osatura trail demo.json tr3.svg --target layer:sun --from 0 --to 24
wrote tr3.svg (25 ticks, 0 key marker(s))
$ osatura trail demo.json tr2.svg --target bone:hand:tip --from 0 --to 24 --smooth 2 --json
{ "curve": [ [212.8, 83.84], [213.77, 83.99], ...
```

### `dof` y `dof-export`

**Sinopsis:**
```
osatura dof        <project.json> <frame> <out.png> [--focus 1.0 --strength 12 --max-radius 24 --scale N --clamp]
osatura dof-export <project.json> <out.gif|out.apng|out.mp4|out.webm|out-folder> [--from F --to F --focus N --strength N --max-radius N --scale N --codec vp9|av1]
```
Un `<frame>` fuera del proyecto es un error para `dof` (y para `render-hdr` y `fx`); `--clamp` renderiza en su lugar el fotograma válido más cercano, como en `render`.

Profundidad de campo: cada **capa de nivel superior** se desenfoca en `|parallax - focus|` veces `strength` (píxeles por unidad de
distancia de paralaje, con tope en `--max-radius`). Sin `--focus` / `--strength`, los valores siguen los canales con claves
`camera_focus` / `camera_blur` (enfoque 1,0 por defecto). Las capas obtienen su profundidad de `parallax` (véase la sección 5).
`dof-export` renderiza el rango `--from`..`--to` (números de fotograma enteros; por defecto, de 0 al final) a un GIF animado, un
PNG animado, un vídeo `.mp4`/`.webm` (requiere `ffmpeg`; `--codec` es `vp9` por defecto; no se incluye audio) o, para un nombre sin
extensión, una carpeta de PNG numerados (`f0000.png`...). Cualquier otra extensión se rechaza (`dof-export writes .gif, .apng,
.mp4, .webm or a folder (no extension)`, no se escribe nada).

```
$ osatura dof demo.json 12 dof.png --focus 1.0 --strength 12 --max-radius 24 --scale 0.5
wrote dof.png
$ osatura dof-export demo.json dof.gif --from 0 --to 6 --scale 0.25
wrote dof.gif
$ osatura dof-export demo.json dofdir --from 0 --to 2 --scale 0.25      # dofdir/f0000.png ... f0002.png
wrote dofdir
```
**Relacionado:** `parallax`, `render --proxy`, `shot`/`camera-seq` para movimientos de cámara. **Límites:** el desenfoque es una
aproximación por capa, no un modelo de lente (sin formas de bokeh ni tratamiento de oclusiones); no se exporta a Lottie.

## 13. Nivel de detalle, informes de tamaño y export-fit

### `size-report`

**Sinopsis:** `osatura size-report <project.json> [--tol T] [--json]`

Muestra **dónde están los bytes**: el archivo del proyecto, el Lottie horneado y la página HTML, por capa, imagen y canal
(claves, y cuántas se pueden eliminar con la tolerancia `--tol`, 0,05 por defecto).

```
$ osatura size-report small.json
project file: 17959 bytes (3511 gzip)
  by layer (JSON bytes; embedded image base64 in brackets):
         2983  arm
         2053  leg
  ...
  by channel (keys, removable at tol 0.05):
          420  foot_ctrl.BoneOffsetX  keys 5 removable 1
  ...
baked Lottie: 46843 bytes (11180 gzip), by layer:
         9181  arm
```
`--json` imprime los mismos datos como JSON (`html.defs_bytes_total`, `frame_bytes_max`, ...). Úsalo antes de `export-fit` o `lod`
para ver qué capa simplificar.

### `lod`

**Sinopsis:** `osatura lod <project.json> --tolerance-px 0.5 [--scale S | --size WxH] [--levels 4] [--ratio 2] [--out-dir D] [--report r.json] [--verify] [--frames N]`

Construye **niveles de detalle** L1..Ln, cada uno con una *cota de error declarada en píxeles de salida*: simplificación de trazados, reducción
de claves, colapso de mallas y descarte. El nivel k tiene la tolerancia `--tolerance-px x ratio^(k-1)`. `--scale` o `--size` indican
a qué tamaño se muestra la salida (1 px por unidad por defecto). `--out-dir` (`.` por defecto) recibe `<name>_L1.json` ... y
`--report` un informe JSON. `--verify` renderiza `--frames N` fotogramas y **mide** la desviación real (código de salida 1 si supera
la cota).

```
$ osatura lod small.json --tolerance-px 1 --levels 3 --out-dir lv --report rep.json --verify --frames 4
level  tol_px  anchors  mesh_v  keys  layers    bytes  declared_px  path  mesh  keys  cull  measured_px  ratio
L0      0.000       74       0    37      14    17959       0.0000 ...                              -      -
L1      1.000       70       0    37      14    17651       0.3861 0.386 0.000 0.000 0.000       0.3342   1.2x
L2      2.000       69       0    37      14    17573       0.9277 ...                           0.8185   1.1x
L3      4.000       65       0    37      14    17251       1.5372 ...                           1.3367   1.1x
scale 1 px per unit; level k tolerance = 1 px x 2^(k-1); files lv/s_L1..L3.json
  L1: keys: 3 of 8 keyed channels have a documented lever, 0 reduced
  L1: removed 4 path anchor(s)
$ ls lv
s_L1.json  s_L2.json  s_L3.json
```
**Trampa:** el ahorro en una demostración pequeña y hecha a mano es mínimo (alrededor del 4 % a 4 px); los LOD compensan en obras vectorizadas o con muchas mallas.
El informe y los archivos de nivel se escriben con una escritura simple (no atómica).

### `lod-pick`

**Sinopsis:** `osatura lod-pick <project.json> --screen-px WxH [--max-px 0.5] [--tolerance-px 0.25] [--levels 4] [--out chosen.json]`

Elige el **nivel más grueso** cuya cota declarada se mantiene dentro de `--max-px` píxeles cuando la obra se muestra en una
pantalla de `--screen-px` (la escala es el tamaño de pantalla dividido por el tamaño del proyecto). `--out` escribe ese nivel.

```
$ osatura lod-pick small.json --screen-px 320x240 --out chosen.json
level  tol_px  anchors ...
screen 320x240 -> scale 0.5000 px/unit; budget 0.5 px -> level L3 (declared 0.4638 px, 69 anchors, 0 mesh vertices, 37 keys)
wrote chosen.json
$ osatura lod-pick small.json --screen-px 64x48 --max-px 2 --out chosen2.json
screen 64x48 -> scale 0.1000 px/unit; budget 2 px -> level L4 (declared 0.5310 px, 62 anchors, 0 mesh vertices, 37 keys)
```

### `update-check`

**Sinopsis:** `osatura update-check [--proxy-preset tor|tor-browser|warp | --proxy-url URL] [--list-proxy-presets] [--json]`

Pregunta a GitHub si existe una versión de OSATURA más reciente. Solo se ejecuta cuando lo escribes: nada comprueba en segundo plano, nada se descarga ni se instala, y lo único que se envía es una petición anónima de la lista pública de versiones (hecha por el `curl` del sistema, solo HTTPS, 30 s, 2 MB como máximo). La respuesta compara tu versión con la más reciente publicada según las reglas de SemVer: `0.1.0-alpha.5` es más reciente que `0.1.0-alpha.4`, y `0.1.0` es más reciente que `0.1.0-rc.1`.

`--proxy-preset` envía la petición a través de un proxy gratuito que se ejecuta en tu propio equipo: `tor` (el servicio Tor en `127.0.0.1:9050`), `tor-browser` (un Tor Browser en ejecución, `127.0.0.1:9150`) o `warp` (Cloudflare WARP en modo proxy, `127.0.0.1:40000`). Los tres usan `socks5h://`, así que el nombre `api.github.com` lo resuelve el proxy y no tu sistema. Tor es la opción recomendada (red de voluntarios, sin ánimo de lucro, código abierto); WARP es gratuito, pero es un servicio comercial con cliente cerrado. Las listas de proxies públicos abiertos no se ofrecen a propósito: suelen estar comprometidos o interceptar el tráfico. `--proxy-url` acepta tu propio proxy `http://`, `https://`, `socks5://` o `socks5h://`; las dos opciones no se pueden combinar. `--list-proxy-presets` muestra los preajustes y termina. (`--proxy` es otra opción: el nivel de proxy de detalle del estudio.)

```
$ osatura update-check --proxy-preset tor
update-check: going through socks5h://127.0.0.1:9050
You are up to date: 0.1.0-alpha.4 (newest published release: v0.1.0-alpha.4).
$ osatura update-check --json
{"current":"0.1.0-alpha.4","latest":"v0.1.0-alpha.5","prerelease":true,"update_available":true,"url":"https://github.com/chiuta/osatura/releases/tag/v0.1.0-alpha.5"}
```
**Trampa:** solo las compilaciones hechas por el flujo de publicación conocen su versión; una compilación desde el código fuente (versión `0.1.0`) muestra la última versión pero no puede decir si es más reciente, y `--json` trae entonces `"current": null` y `"update_available": null`. Código de salida 1 con un motivo de una línea cuando falta `curl` o no se puede llegar a GitHub (las salidas de Tor a veces están bloqueadas o limitadas). La dirección de descarga que se muestra es siempre un enlace `github.com/chiuta/osatura`.

### `export-fit`

**Sinopsis:** `osatura export-fit <project.json> <out> --max-size 500KB [--format lottie|html|gif|apng|webm|project] [--min-ssim X] [--max-seconds 60] [--measure raw|gzip] [--frames N] [--search ladder|descent|grid] [--report out.json] [--pareto out.csv] [--audit] [--no-baseline]`

Encuentra la **exportación de mejor calidad que cabe en un presupuesto de bytes**: busca entre los parámetros de calidad del exportador (Lottie: `decimals`,
`path_eps`, `key_tol`, `frame_step`; los demás formatos tienen los suyos) y puntúa cada candidato con el SSIM respecto al
render original. `--max-size` acepta `500KB` (1000 bytes), `1.5MB`, `64000`, `200KiB` (1024 bytes). El formato se toma de
`--format` o de la extensión de salida (`.json` es Lottie, `.html`, `.gif`, `.apng`, `.webm`); `project` escribe un JSON de proyecto
reducido. `--measure gzip` presupuesta el tamaño con gzip (para servidores que comprimen). `--min-ssim` es un mínimo de calidad.
`--max-seconds` es un **presupuesto real de tiempo de reloj** (60 por defecto; `0` = ilimitado): al agotarse se conserva el mejor resultado obtenido hasta ese momento.
`--frames N` es el número de fotogramas muestreados para el SSIM (8 por defecto). `--search` elige la estrategia (`descent` por defecto).
`--report` / `--pareto` escriben el registro de la búsqueda como JSON / CSV; `--audit` imprime todos los candidatos; `--no-baseline` omite
la comparación con los ajustes por defecto.

```
$ osatura export-fit small.json fit.json --max-size 30KB --format lottie --report fr.json
lottie -> fit.json: 25756 bytes raw, 5736 gzip (budget 30000 raw); SSIM mean 0.9699, worst 0.9308 at frame 5, max CIEDE2000 68.67; 5 candidates built
time budget exhausted (60 s, checked inside every candidate): the search stopped early and the best result so far is kept (...)
$ osatura export-fit small.json fit.html --max-size 60KB --format html --max-seconds 20 --frames 4
html -> fit.html: 59166 bytes raw, 8550 gzip (budget 60000 raw); SSIM mean 1.0000, worst 1.0000 at frame 5, max CIEDE2000 8.54; 1 candidates built
existing exporter at defaults: 66391 bytes raw, 8933 gzip
$ osatura export-fit small.json fit2.json --max-size 1KB --format lottie --max-seconds 10 --frames 3
lottie -> fit2.json: 14751 bytes raw, 2418 gzip (budget 1000 raw); ...
knobs at their floor: decimals, path_eps, key_tol, frame_step
budget not met: the smallest export found is 14751 bytes (raw), above the budget of 1000 bytes: ...          [exit 1]
$ osatura export-fit small.json fit.xyz --max-size 1KB
cannot tell the format from the output name; pass --format lottie|html|gif|apng|webm|project   [exit 1]
```
**Códigos de salida:** 0 cuando el resultado cabe en el presupuesto (aunque el presupuesto de tiempo haya cortado la búsqueda); 1 cuando el presupuesto
no se puede cumplir (aun así se escribe la exportación más pequeña) o si los argumentos son incorrectos. **Trampas:** trabaja sobre la salida horneada,
por lo que no puede eliminar más de lo que permiten los parámetros (un Lottie de 1 KB es imposible para un rig de 14 capas); cada candidato es una
exportación completa, así que los proyectos grandes necesitan un `--max-seconds` generoso (una compilación de depuración es mucho más lenta que una de release). Los archivos
de informe se escriben con una escritura simple. **Relacionado:** `size-report`, `lod`, `export` (sección 4), EXPORT_FORMATS.md (consejos de tamaño).

## 14. Control de calidad

Relacionado, documentado antes: `check` (validación, sección 3), `diff` (diff semántico, sección 3), `compare` (diff de imagen de dos
renders, sección 4).

### `lint`

**Sinopsis:** `osatura lint <project.json>`

Comprobaciones estáticas del rig y de la animación. Imprime una línea por hallazgo: `Severity CODE [target:property] message`. Código de salida 1 si algún
hallazgo tiene una severidad Error o superior; Info y Warning terminan con 0.
```
$ osatura lint small.json
Info L007 [:CameraX] key at frame 24 lies outside the 0..8 timeline
Info L007 [arm_low:BoneAngle] key at frame 12 lies outside the 0..8 timeline
```
`L007` es una clave fuera de la línea de tiempo (aquí porque `small.json` es la demostración recortada a 8 fotogramas).

### `contracts`

**Sinopsis:** `osatura contracts <project.json> <contracts.json>`

**Contratos de movimiento**: pruebas unitarias para la animación. `contracts.json` es una lista de contratos con un `type`:
`loop_seamless {tol}`, `tip_range {bone,axis,min,max}`, `max_speed {bone,px_per_frame}` (existen más tipos en el enum
`Contract` del motor). Las infracciones se imprimen en stdout, el recuento en stderr, y el código de salida es 1.
```
$ cat contracts.json
[{"type":"loop_seamless","tol":0.5},{"type":"tip_range","bone":"hand","axis":"y","min":0,"max":400},{"type":"max_speed","bone":"hand","px_per_frame":3}]
$ osatura contracts small.json contracts.json
contract 0 violated at frame 8: bone 'arm_up' ends 35.4 px away from where it started (tolerance 0.5)
...
contract 2 violated at frame 6: tip of 'hand' moves 3.3 px/frame (limit 3)
7 violation(s)                                                            [stderr, exit 1]
```

### `coach`

**Sinopsis:** `osatura coach <project.json>`

Sugerencias de timing y de principios de animación (solapamiento, anticipación, suavizado). Sólo consejos, siempre código de salida 0.
```
$ osatura coach small.json
C003 [, arm_low, arm_up, foot_ctrl, hand, mouth, target] frame 0: 7 parts start moving on frame 0; offset them by a few frames (overlap) for a more natural feel
```

### `safety`

**Sinopsis:** `osatura safety <project.json> [--scale N]`

Informe de destellos, destellos rojos y movimiento de cámara (una aproximación a WCAG 2.3.1; una ayuda de ingeniería, no una certificación
de emisión). Renderiza cada fotograma a `--scale` (usa un valor pequeño para proyectos largos).
```
$ osatura safety small.json --scale 0.25
flashes      ok    worst 0 transitions/s (frame 0)
red flashes  ok    worst 0 transitions/s (frame 0)
camera       pan 0.06 widths/s, rotation 0°/s, zoom ×1.00/s
```

### `reduced-motion`

**Sinopsis:** `osatura reduced-motion <project.json> <out.json> [--factor 0.25]`

Escribe una copia más calmada: el movimiento de cámara y el wobble se escalan por `--factor` (0,25 por defecto).
```
$ osatura reduced-motion small.json calm.json --factor 0.25
wrote calm.json
```

### `golden`

**Sinopsis:**
```
osatura golden write <project.json> <golden.json> [--scale N]
osatura golden check <project.json> <golden.json> [--tol T]
```
`write --scale N` toma N en (0, 1]: un valor mayor se **rechaza** (`golden write: --scale 2 is above 1 ...`, no se escribe nada; antes se limitaba a 1 en silencio). `check` vuelve a renderizar a la escala almacenada en el archivo golden y no tiene la opción `--scale`: `golden check ... --scale N` se rechaza como opción desconocida.


Una prueba de regresión ligera: `write` registra un hash y una miniatura de 16x16 de cada fotograma en `golden.json`; `check`
vuelve a renderizar y compara. **Exacto por defecto** (`--tol 0`): cualquier diferencia en el hash de píxeles falla. `--tol T` (T > 0) relaja
la comprobación: un hash cambiado aun así pasa si la distancia entre miniaturas es como máximo T. Un `--tol` negativo o no finito es un error.
Para golden basados en imágenes con una métrica perceptual, usa `ci`.
```
$ osatura golden write small.json g.json --scale 0.25
wrote g.json (8 frames)
$ osatura golden check small.json g.json
all 8 frames match exactly
$ osatura golden check small.json g.json --tol 0.01
all 8 frames match (within thumbnail tolerance 0.01)
```
Los fotogramas que difieren se enumeran (cambio de hash, distancia entre miniaturas, número de las 256 celdas de la miniatura que difieren, mayor
diferencia de canal sobre 255), y el código de salida es 1:
```
$ osatura golden check changed.json g.json
frame 1 differs: pixel hash changed, thumbnail distance 0.003, 12 of 256 thumbnail cells differ, max channel difference 31/255
2 frame(s) differ (exact comparison; --tol T accepts small differences)       [stderr, exit 1]
```
Un fotograma presente sólo en uno de los lados se notifica como `one side only`. **Límites:** el archivo golden conserva únicamente el hash y la
miniatura, de modo que las cifras notificadas son a nivel de miniatura, no de píxel. Un cambio que es invisible en el render (por ejemplo
un color de fondo completamente cubierto por una capa opaca, como el cielo de la demostración) deja legítimamente el hash sin cambios.

### `ci` (init, run, bless, diff)

**Sinopsis:**
```
osatura ci init <project.json> [--out osatura-ci.json] [--no-goldens]
osatura ci run  [manifest] [--update] [--bless] [--prune] [--only a,b] [--junit f.xml] [--html f.html] [--json f.json] [--artifacts dir] [--quiet]
osatura ci bless [manifest]
osatura ci diff a.png b.png [--metric ssim|de2000|de76|mse] [--tol T] [--ignore x,y,w,h;...] [--aa N] [--out heat.png] [--side s.png] [--overlay o.png] [--flicker f.gif]
```
**CI de animación**: fotogramas golden comparados con una métrica perceptual (SSIM de luma, CIEDE2000), más comprobaciones estructurales y
presupuestos. El formato del manifiesto y el flujo de trabajo se describen en [CI.md](CI.md). `exit 0` = correcto, `1` = falló una comprobación,
`2` = error (manifiesto ilegible, ...).
```
$ osatura ci init small.json --out ci.json
wrote ci.json with 8 checks:
  validate     validate
  render       frames
  render       frames-float
  determinism  determinism
  fuzz         pose-fuzz
  limits       limits
  export-size  gif-size
$ osatura ci run ci.json --junit j.xml --html r.html --json ci_r.json
[PASS   ] validate     validate           project compiles; 8 lint finding(s), none at 'error' or above
[PASS   ] render       frames             6 frame(s) match
[PASS   ] determinism  determinism        6 render(s) repeated byte-identically
[INFO   ] fuzz         pose-fuzz          98 of 100 pose samples fail (98.00%); not a gate: non-gating (gate = false)
[PASS   ] export-size  gif-size           gif export is 82881 bytes (budget 124322)
[INFO   ] perf         render-speed       median 429.67 ms/frame, budget 1633 ms (+50% tolerance = 2449.50 ms)
8 checks: 6 passed, 0 failed, 0 errors, 2 info, 0 updated, 0 blessed (14.7 s)
$ osatura ci diff a.png b.png --metric ssim --out heat.png
size        480x360
SSIM        0.90536   (luma)   channels min 0.90300   (R, G, B)
de2000      max 68.674   mean 1.9434
pixels      11124 differ (6.4375%), 31 forgiven as anti-aliasing, 0 ignored
regions     118
images differ: luma SSIM 0.90536 < 0.99; ...                                [stderr, exit 1]
$ osatura ci run nonexist.json
cannot read manifest nonexist.json: No such file or directory               [exit 2]
```
`init` escribe el manifiesto y, salvo con `--no-goldens`, los PNG golden en la carpeta `goldens/` situada junto a él. `run
--update` reescribe los golden que faltan o han cambiado, `--bless` acepta los renders nuevos como golden, `--prune` elimina los
archivos golden que ya no referencia ninguna comprobación, `--only a,b` ejecuta las comprobaciones indicadas. `bless` es `run --bless`. `diff` compara dos PNG: `--tol` es el umbral
de la métrica, `--ignore` enmascara rectángulos (`x,y,w,h;...`), `--aa N` perdona las diferencias de antialiasing, y `--out`,
`--side`, `--overlay`, `--flicker` escriben un mapa de calor, una imagen lado a lado, una superposición y un GIF de parpadeo.
**Trampa:** los informes (`--junit`, `--html`, `--json`) y los archivos golden se escriben con el escritor atómico que rechaza enlaces simbólicos (creando
las carpetas padre); una ruta de informe que sea un enlace simbólico se rechaza después de ejecutar las comprobaciones (estas son lentas, así que pasa una ruta real). Una compilación de depuración es lenta: las 8 comprobaciones anteriores tardaron 15 s en un proyecto de 8 fotogramas.

### `fuzz`

**Sinopsis:** `osatura fuzz <project.json> [--samples N] [--seed S] [--strategy random|lhs|halton|sweep|corners|pca] [--frame F] [--report out.json] [--heatmap out.svg] [--minimize] [--gate] [--max-fail-rate R] [--props angle,scale,offx,offy] [--only BONES] [--margin DEG] [--db posedb.json] [--ignore KINDS] [--no-deformers]`

**Fuzzing de poses**: muestrea poses dentro de los límites de los huesos e informa de dónde se rompe el rig (pliegues, estiramientos, picos,
autointersección, IK inalcanzable, infracciones de límites, NaN, fuera del lienzo). `--strategy` elige el muestreador (`pca` muestrea a lo largo de los ejes PCA de las poses clave del propio proyecto, o de la base de datos de poses
indicada con `--db`), `--minimize` reduce una pose que falla hasta un reproductor mínimo, `--heatmap` escribe un mapa de tensión,
`--ignore` omite tipos de fallo, `--only` restringe a ciertos huesos. Sin puerta, `fuzz` es un **informe**: imprime el resumen y termina con 0 (`(report only: exit 0; add --gate, ...)`). `--gate` falla (código 1) cuando falla cualquier muestra; `--max-fail-rate R` (una **fracción** de 0 a 1) implica aplicar la puerta con esa tasa. (La comprobación `fuzz` propia de `ci` tiene su propio ajuste `gate` y no se ve afectada.)
```
$ osatura fuzz small.json --samples 30 --seed 1 --strategy lhs --report fz.json --heatmap hm.svg --max-fail-rate 1.0
wrote fz.json
wrote hm.svg
fuzz 'small': 30 samples (lhs, seed 1, frame 0), 25 parameters, 30 failures (100.0%), max severity 54.62, baseline 0.00, 0.40 ms/sample
  ik_unreachable: 27
  off_canvas: 1
  stretch: 2
$ osatura fuzz small.json --samples 10 --max-fail-rate 0.5
fuzz: failure rate 100.00% exceeds --max-fail-rate 50.00%                   [stderr, exit 1]
```
El objetivo de IK de la demostración es alcanzable deliberadamente sólo en un rango pequeño, por lo que casi cualquier pose aleatoria es «inalcanzable»:
trata la tasa como un número de regresión, no como un criterio de aprobado/suspenso para este rig. El informe y el mapa de calor se escriben de forma atómica y una ruta que sea un enlace simbólico se rechaza (salvo con `--follow-symlinks`); la carpeta padre debe existir.

### `fuzz-grid`

**Sinopsis:** `osatura fuzz-grid <project.json> --x PARAM --y PARAM [--res N] out.svg|out.png|out.json`

Un mapa de severidad de dos parámetros (`bone:NAME:PROP`, por ejemplo `bone:arm_up:angle`) en una cuadrícula de `N x N` (32 por defecto).
El tipo de salida depende de la extensión.
```
$ osatura fuzz-grid small.json --x bone:arm_up:angle --y bone:arm_low:angle --res 8 grid.svg
wrote grid.svg (8x8 cells, 59.4% failing; x = bone:arm_up:angle, y = bone:arm_low:angle)
```
Misma salvedad de escritura simple que `fuzz`.

## 15. Control de versiones (`vcs`)

**Sinopsis:**
```
osatura vcs init <project.json> [--author A]
osatura vcs commit [-m MSG | --message MSG] [--author A] [--repo DIR]
osatura vcs log [REV] [--limit N] [--json] [--repo DIR]
osatura vcs status [--json] [--repo DIR]
osatura vcs diff [A [B]] [--json] [--repo DIR]
osatura vcs show [REV] [--json] [--repo DIR]
osatura vcs branch [NAME [AT] | -d NAME] [--repo DIR]
osatura vcs tag [NAME [AT]] [--repo DIR]
osatura vcs checkout REV [--force] [--repo DIR]
osatura vcs merge REV [--no-ff] [-m MSG | --message MSG] [--author A] [--abort] [--repo DIR]
osatura vcs cherry-pick|revert REV [--author A] [--repo DIR]
osatura vcs resolve PATH [--ours] [--theirs] [--value V] [--repo DIR]
osatura vcs fsck|gc [--repo DIR]
```
Las opciones dependen del subcomando: `--json` sólo en `log`, `status`, `diff` y `show`; `--force` sólo en `checkout`; `--ours`, `--theirs`, `--value` (exactamente una) sólo en `resolve`.

Un pequeño historial direccionable por contenido para **un único archivo de proyecto** con una **fusión semántica**: los cambios se comparan como
rutas del documento (punteros JSON), no como líneas de texto, de modo que dos personas que editan claves distintas nunca entran en conflicto. `init` crea
una carpeta `.osatura-vcs/` junto al proyecto; la rama por defecto es `main`. Las revisiones (`REV`) son nombres de rama, nombres de etiqueta
o prefijos de id de commit.

```
$ osatura vcs init p.json
initialised ./.osatura-vcs for p.json (no commits yet; run: osatura vcs commit -m "...")
$ osatura vcs commit -m first --author alexio
[5f80a63179] first
$ (edit p.json)
$ osatura vcs status
on branch main
HEAD 5f80a63179
uncommitted changes:
  project: background changed "#f6f1e7" -> "#ffffff"
$ osatura vcs commit -m "white bg"
[29c09d9368] white bg
$ osatura vcs branch alt          # branch alt at 29c09d9368
$ osatura vcs checkout alt        # checked out alt (29c09d9368)
$ osatura vcs log --limit 5
4e21f15940  unknown  30 fps
29c09d9368  unknown  white bg
5f80a63179  alexio  first
$ osatura vcs merge alt           # fast-forward or a merge commit; --no-ff forces a merge commit
merge committed as f08db3f77f
$ osatura vcs tag v1 ; osatura vcs tag
tag v1 at 4e21f15940
v1  4e21f15940
$ osatura vcs fsck
ok: 50 objects, history and refs are consistent
$ osatura vcs gc
gc: kept 50 objects, removed 0 unreachable
```

**Conflictos.** Cuando ambos lados cambiaron el mismo valor, `merge` se detiene, conserva *nuestro* lado en el archivo del proyecto, enumera los
conflictos y termina con 1:
```
$ osatura vcs merge main
merge stopped with 1 conflict(s); the project file holds our side for each:
  CONFLICT /background
      project: background: ours "#222222" vs theirs "#333333" (base "#000000")
resolve each with: osatura vcs resolve <path> --ours|--theirs|--value <json>, then: osatura vcs commit   (or: osatura vcs merge --abort)
$ osatura vcs resolve background --theirs
no conflict at 'background' (see 'vcs status')                              [exit 1]   (the path needs its leading slash)
$ osatura vcs resolve /background --theirs
resolved /background; 0 conflict(s) left - run: osatura vcs commit
$ osatura vcs commit -m merged
[bd405f446a] merged
$ osatura vcs merge --abort       # alternatively: merge aborted; project restored to HEAD
```
Mientras un merge, cherry-pick o revert esté sin resolver, `checkout`, `revert` y `commit` se rechazan con
`a merge / cherry-pick / revert is in progress: resolve the conflicts and commit, or run 'vcs merge --abort'`.
Otros datos: `diff` sin argumentos compara el archivo de trabajo con HEAD, `diff A B` dos revisiones (`--json` imprime una lista
de `{kind,path,description}`); `show` imprime un commit y su lista de cambios; `branch` sin argumentos enumera las ramas (`*` marca la actual); `branch -d NAME` se niega a eliminar la rama activa; `checkout --force` descarta los cambios sin confirmar; `revert REV` y `cherry-pick REV` crean un commit nuevo (o se detienen con conflictos como `merge`); `--repo DIR`
selecciona una carpeta de repositorio distinta de la actual; `--author` toma por defecto `unknown`; `log --json` imprime los objetos
de commit. **Límites:** un archivo por repositorio, sólo local (sin remotos, sin push/pull), sin seguimiento de renombrados más allá de las rutas JSON;
los recursos binarios a los que hace referencia el proyecto no se versionan.

## 16. Estudio, colaboración, servidor de agente MCP y registro de la interfaz

### `studio`

**Sinopsis:** `osatura studio <project.json> [--port N] [--ui-config F] [--bind ADDR] [--allow-host NAMES] [--public-origin URL] [--source-url URL] [--collab-token T (16+ characters)] [--collab-secret-file F [--collab-revoke NAMES] [--collab-host NAMES]] [--collab-rate N] [--collab-log LOG [--collab-nosync]]`

Inicia el **editor de navegador** (deshacer/rehacer, línea de tiempo, paneles en vivo y de rostro, VCS y operaciones al estilo de agente) como un servidor web
local y se bloquea hasta Ctrl-C. Valores por defecto: `--port 8080`, `--bind 127.0.0.1`. El proyecto se valida al arrancar; los guardados van
al mismo archivo (`POST /api/save`).
```
$ osatura studio demo.json --port 18931 --ui-config ui.json
AGPL-3.0 source offer: http://127.0.0.1:18931/source
OSATURA studio on http://127.0.0.1:18931  (Ctrl-C to stop; saves go to demo.json)
UI settings (layout, shortcuts) in ui.json
```
Opciones: `--ui-config F` conserva la disposición y los atajos (por defecto `~/.config/osatura/ui.json`, en Windows `%APPDATA%\OSATURA\ui.json`); `--source-url URL` establece el
enlace al código fuente publicado que se muestra en `/source` y con `--version` (AGPL-3.0, sección 13; los binarios oficiales de la versión usan por defecto el commit desde el que se compilaron); `--allow-host NAMES` y `--public-origin URL` aceptan
peticiones bajo un nombre público detrás de un proxy inverso TLS; `--collab-token T` (al menos 16 caracteres, por ejemplo
`openssl rand -hex 16`) activa la sesión multiusuario y es **obligatorio para cualquier `--bind` que no sea loopback**; `--collab-nosync` sólo es válido junto con `--collab-log` (de lo contrario, es un error); `--collab-log LOG`
persiste la sesión compartida y la reanuda tras un reinicio (`--collab-nosync` omite la sincronización al arrancar).

**Opciones de refuerzo de seguridad** (oleada 7; detalles y modelo de amenazas en [SECURITY.md](SECURITY.md)): `--collab-secret-file F` sustituye
el único bearer compartido por **tokens por participante**: F contiene un secreto aleatorio (primera línea, de al menos 16 caracteres, modo 600 en
Unix, de lo contrario se rechaza) y cada persona obtiene un token con `osatura collab-token NAME`; un token está ligado a un nombre
(al unirse se usa ese nombre, nadie puede unirse como otra persona), caduca y se comprueba recalculando su HMAC-SHA-256, de modo que el
servidor no guarda ninguna tabla. `--collab-revoke NAMES` (separados por comas, requiere `--collab-secret-file`) rechaza los tokens de esos
nombres antes de que caduquen (un servidor en marcha también puede revocar con `POST /api/collab/revoke {"names":["ana"]}`, véase más abajo). `--collab-host NAMES`
(separados por comas, requiere `--collab-secret-file`) nombra a los participantes cuyos tokens pueden **guardar el proyecto, compactar el registro de sesión y revocar
tokens**; el resto de quienes tengan un token válido pueden seguir editando, pero reciben `403` en esas tres peticiones. Sin `--collab-host` todos los participantes pueden
guardar y compactar (como antes) y solo el `--collab-token` compartido o una sesión local sin autenticar puede revocar por HTTP; el estudio indica cuál es el caso
al arrancar. `--collab-token` y `--collab-secret-file` se pueden combinar (el token compartido actúa entonces como
clave de administración); cualquiera de los dos satisface la regla de que un bind que no sea loopback necesita una credencial. `--collab-rate N` limita cada dirección
de cliente a N peticiones por segundo (ráfagas de 2N; respuesta `429` con `Retry-After`); el valor por defecto es 100 cuando se exige una credencial
y sin límite en caso contrario; 0 lo desactiva. Un bind que no sea loopback imprime advertencias (HTTP sin cifrar; todas las interfaces con
`0.0.0.0` / `::`; un token compartido no tiene nombres ni caducidad).

**Reglas de seguridad** (verificadas): el servidor sólo responde a las peticiones cuyo `Host` es uno de sus propios nombres
(`curl -H 'Host: evil.example' .../api/state` da 421); `Origin` y `Sec-Fetch-Site` de otra página dan 403; el cuerpo de un POST
debe ser `application/json` (`curl -X POST -d '{}'` da 415). Un token corto se rechaza
(`--collab-token must be at least 16 characters`), y `--bind 0.0.0.0` sin token se rechaza
(`refusing to listen on 0.0.0.0:PORT: the studio has no accounts and no TLS, ...`). El servidor no tiene cuentas ni TLS: pon
un proxy TLS delante para cualquier uso que no sea tu propia máquina; cuando `--public-origin` es una URL `https://`, la cookie de inicio de sesión recibe el atributo `Secure` para las peticiones que llegan bajo ese nombre (o con `Origin` https / `X-Forwarded-Proto: https`). La página sirve un pequeño favicon SVG en `/favicon.ico`. La API HTTP (por ejemplo `GET /api/state`, `GET /api/project`,
`POST /api/op`, `POST /api/save`, `GET /frame/N.svg`) existe para la propia página y no es una API pública estable.

### `collab-token`

**Sinopsis:** `osatura collab-token <name> [--secret-file F] [--ttl 28800]`

Imprime un token de colaboración por participante para un estudio iniciado con `--collab-secret-file F`: `ap1.<name>.<expires>.<mac>`,
donde `mac` es el HMAC-SHA-256 hexadecimal del nombre y la fecha límite bajo el secreto. `<name>` tiene de 1 a 32 caracteres entre letras,
dígitos, `_` y `-`. `--ttl` es la vida útil en segundos (28800 = 8 horas por defecto, como máximo 2592000 = 30 días). El secreto se lee del
archivo `F` (primera línea) o, sin `--secret-file`, de la variable de entorno `OSATURA_COLLAB_SECRET`; nunca se
acepta como argumento, donde otros usuarios pueden verlo. El token va a stdout y la fecha límite a stderr.

```
$ openssl rand -hex 32 > collab.secret && chmod 600 collab.secret
$ osatura studio demo.json --collab-secret-file collab.secret
$ osatura collab-token ana --secret-file collab.secret --ttl 3600
ap1.ana.1790000000.3f0c...
token for 'ana', valid until Unix time 1790000000 (in 3600 s); give it to that person only; ...
$ osatura collab-token ana --secret-file collab.secret --ttl 0
--ttl needs a whole number of seconds (1 to 2592000)
```
La persona pega el token en la página de inicio de sesión del estudio (queda entonces guardado en una cookie HttpOnly que caduca con el token) o lo envía como
`X-Collab-Token`. Un token caducado recibe `401` con la fecha límite en el mensaje; el nombre del participante se toma del
token. **Relacionado:** `studio`.

### `collab-sim`

**Sinopsis:** `osatura collab-sim <project.json> [--clients N] [--ops M] [--seed S] [--undo-pct P] [--replays K] [--require-valid] [--raw] [--json]`

Un banco de pruebas para el motor de colaboración: N réplicas editan el proyecto de forma concurrente con orden de entrega aleatorio y
duplicados, y después se verifica que todas contienen el mismo documento (código de salida 1 si no). Las ediciones por defecto son ediciones válidas del proyecto
(números, valores de claves, visibilidad, capas propias); `--raw` usa ediciones JSON arbitrarias (normalmente no es un proyecto válido);
`--require-valid` hace que un resultado que no compila también termine con 1.
```
$ osatura collab-sim small.json --clients 3 --ops 20 --seed 2 --json
{"buffered":12,"clients":3,"clocks_identical":true,"converged":true,"converged_ok":true,"deliveries":34,"duplicates_dropped":0,"edits":20,"envelopes":17,...}
```
`converged_ok` es sólo la convergencia; `ok` exige además que el documento fusionado compile.

### `mcp`

**Sinopsis:** `osatura mcp <project.json>`

Un **servidor de agente**: JSON-RPC 2.0 (el Model Context Protocol) por stdin/stdout, un mensaje JSON por línea. Carga el
proyecto y expone las herramientas `get_project`, `stats`, `lint`, `coach`, `safety`, `apply_ops`, `set_key`, `undo`, `redo`,
`render_frame` y `save`. No se escribe nada hasta que se llama a `save`. Stdout transporta únicamente mensajes del protocolo.
```
$ printf '%s\n' \
 '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}' \
 '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
 '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
 '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"lint","arguments":{}}}' | osatura mcp demo.json
{"id":1,"jsonrpc":"2.0","result":{"capabilities":{"tools":{}},"protocolVersion":"2024-11-05","serverInfo":{"name":"osatura","version":"0.1.0"}}}
{"id":2,"jsonrpc":"2.0","result":{"tools":[{"description":"Return the project JSON, or the value at a JSON pointer ...","name":"get_project",...}]}}
{"id":3,"jsonrpc":"2.0","result":{"content":[{"text":"[]","type":"text"}],"isError":false}}
```
El servidor termina (código 0) cuando se cierra stdin.

### `ui`

**Sinopsis:**
```
osatura ui commands|shortcuts [--ui-config F] [--json]
osatura ui layout presets|show <name>|validate <file.json>
osatura ui search <query...> [--ui-config F]
```
El **registro de comandos** del estudio: `commands` enumera cada comando con su categoría, título y combinaciones de teclas;
`shortcuts` enumera los atajos efectivos (los predeterminados más los de `--ui-config`) y cualquier conflicto; `layout presets` enumera los
presets de disposición de paneles, `layout show NAME` imprime uno como JSON y `layout validate FILE` comprueba un archivo de disposición;
`search` ordena los comandos como lo hace la paleta de comandos.
```
$ osatura ui commands
Edit
  edit.undo                    Undo                               global:Ctrl+Z
  edit.redo                    Redo                               global:Ctrl+Shift+Z  global:Ctrl+Y
$ osatura ui layout presets
animate  V0.62(H0.74([*stage] | [*layers selection bones live face json]) | [*timeline])  [valid]
rig      H0.66(V0.84([*stage] | [*timeline (folded)]) | V0.50([*layers bones] | [*selection json]))  hidden: live,face  [valid]
compact  V0.62([*stage] | [*timeline layers selection bones live face json])  [valid]
live     H0.58(V0.70([*stage] | [*timeline]) | [*live face bones selection])  hidden: layers,json  [valid]
$ osatura ui layout validate demo.json
demo.json: unknown field `format`, expected one of `version`, `root`, `floating`, `hidden` at line 2 column 10   [exit 1]
```

## 17. Base de datos de poses, sugerencias y otras utilidades

Estos comandos construyen una **base de datos de poses** (rotaciones de huesos en los fotogramas clave de un proyecto y de cada take de una biblioteca)
y usan su estructura aprendida (ejes PCA, red pequeña opcional) para sugerir poses. Es una ayuda estadística entrenada con
*tus* poses, no un modelo general. Sus archivos de modelo y de informe se escriben con una escritura simple.

### `pose-db` (build, info, train-mlp)

**Sinopsis:**
```
osatura pose-db build <out.json> [--project P.json] [--library takes.json] [--step N]
osatura pose-db info  <db.json>
osatura pose-db train-mlp <db.json> <out.json|out.bin> [--hidden 64,64] [--epochs N] [--lr R] [--seed S]
```
```
$ osatura pose-db build db.json --project demo.json
wrote db.json: 5 poses of 3 bones in 1 sequence(s)
$ osatura pose-db info db.json
5 poses, 3 bones: arm_up, arm_low, hand
  sequence 0: project (project, 5 poses)
PCA: 2 axes keep 99.2% of the variance (total 950.0 deg^2)
  axis 1: std 27.34 deg, cumulative 78.7%
  axis 2: std 13.95 deg, cumulative 99.2%
typical nearest-pose spacing 14.43 deg, typical step between consecutive poses 35.82 deg
$ osatura pose-db train-mlp db.json mlp.json --hidden 8 --epochs 20 --seed 1
wrote mlp.json: 6 -> [8] -> 3 (epochs 20, loss 1.0189 -> 0.4921)
trained on the library itself (no held-out data here); run pose-bench for held-out accuracy
```
`--project` añade las poses en los fotogramas clave del proyecto, `--library` los takes de una biblioteca de takes (`take-record`, sección 6),
`--step N` muestrea además cada N fotogramas. La red pequeña (SGD simple) es opcional; úsala con `--model`, más abajo.

### `pose-suggest` (between, complete, next)

**Sinopsis:**
```
osatura pose-suggest between  <project.json> <db.json|-> --a F1 --b F2 [--t 0.5] [--snap 0.5] [--top 5] [--apply RANK --out out.json] [--json]
osatura pose-suggest complete <project.json> <db.json|-> --frame F [--fix bone=angle,...] [--reach bone=x,y,...] [--free b1,b2] [--prior 1] [--model m.json] [--apply RANK --out out.json]
osatura pose-suggest next     <project.json> <db.json|-> --frame F [--k 8]
```
`-` en lugar de una base de datos aprende del propio proyecto. `between` propone una pose intermedia entre dos fotogramas que se mantiene
sobre la variedad aprendida (candidatos `linear`, `pca`, `manifold`); `complete` encuentra la pose completa más plausible dados unos
ángulos fijos (`--fix`), objetivos de punta (`--reach`) y huesos libres; `next` muestra lo que hizo la biblioteca después de poses como la del
fotograma indicado. `--apply RANK` (con `--out`) escribe el candidato elegido como claves.
```
$ osatura pose-suggest between demo.json db.json --a 0 --b 24 --t 0.5 --top 3
between: 5 library poses, 2 PCA axes (99% of the variance)
 1. linear      score 1.00  plausibility 1.00  key at frame 12.0
      linear blend of the angles
      arm_up=0.0 arm_low=-7.5 hand=-10.0
 3. manifold    score 1.00  plausibility 1.00  key at frame 12.0
      PCA blend pulled toward the library
$ osatura pose-suggest complete demo.json db.json --frame 12 --fix arm_up=-40
 1. pca-ls      score 0.96  plausibility 1.00  constraint error 0.18  key at frame 12.0
      regularised least squares in PCA space (start 1)
      arm_up=-39.8 arm_low=24.9 hand=11.2
 2. library-nn  score 0.25  plausibility 1.00  constraint error 15.00  key at frame 12.0
(scores rank candidates against this library; they are heuristics, not probabilities)
$ osatura pose-suggest next demo.json db.json --frame 12 --k 3
 1. successor   score 0.60  plausibility 1.00  key at frame 24.0
      what followed pose 1 of 'project'
```

### `pose-bench`

**Sinopsis:** `osatura pose-bench`

Imprime la precisión con datos reservados de los métodos de sugerencia sobre una **biblioteca sintética de tipo marcha** (24 ciclos de entrenamiento, 6
reservados; la compleción ve 3 de 10 huesos; también entrena la red pequeña). Cuanto menor, mejor. No lee ningún proyecto. Es
**lento en una compilación de depuración** (minutos); cifras de una compilación de release:
```
completion  mean pose                                   20.96 deg RMS
completion  nearest library pose                         1.77 deg RMS
completion  PCA least squares                            0.51 deg RMS
completion  MLP                                          1.64 deg RMS
between     linear blend                                 1.83 deg RMS
next        Markov mean                                  0.72 deg RMS
```

### `pose-match`

**Sinopsis:** `osatura pose-match <project.json> <params.json> <goals.json> <out.json> [--generations N --seed N]`

Busca (un optimizador evolutivo, 80 generaciones por defecto, semilla 1, rango válido 1..100000) valores de un conjunto de
parámetros que sitúen las puntas de los huesos en posiciones objetivo del espacio del escenario. `params.json` es una lista de
`{"target","prop","min","max","start"}`; `goals.json` es una lista de `{"bone","at":[x,y],"weight"}`. El resultado se añade como
canales; la pérdida es la suma ponderada de los errores de punta al cuadrado en px².
```
$ echo '[{"target":"arm_up","prop":"bone_angle","min":-60,"max":40,"start":0},{"target":"arm_low","prop":"bone_angle","min":-60,"max":60,"start":0}]' > params.json
$ echo '[{"bone":"hand","at":[260,200],"weight":1.0}]' > goals.json
$ osatura pose-match demo.json params.json goals.json out.json --generations 30 --seed 3
wrote out.json (loss 3756.156 px²)
```
**Trampa:** la pérdida del ejemplo es grande porque los rangos no alcanzan el objetivo: comprueba el número, no sólo el
código de salida (se escribe un resultado incluso cuando la coincidencia es mala).
## Discrepancias conocidas entre la documentación y el comportamiento

Cuando el propio texto de uso del programa (el que imprime `osatura` sin argumentos) o un documento vecino dice
una cosa y el programa hace otra, las secciones anteriores describen lo que el programa hace realmente. Las diferencias encontradas
al verificar cada ejemplo, por orden de probabilidad de causar problemas, son estas:

1. **Corregido: opciones desconocidas.** Todos los comandos con una línea de uso rechazan ahora las `--words` no documentadas (con una pista «quizá quisiste decir»),
   incluidos `check`, `stats`, `lint`, `coach`, `contracts` y `machine list`. `--json`, `--quiet` y `--allow-empty` son
   propias de cada comando y se rechazan en los demás (un cambio incompatible para los scripts que las pasaban a todos). Desde la oleada 5 la comprobación es **por subcomando** (se rechazan `ci diff ... --quiet`, `trail <bone> ... --json`, `ui layout presets --json`, `golden write --tol`) y abarca también las opciones con valor. Pruebas: `osatura-cli/tests/doc_mismatch.rs` (matriz de aceptación) y `options_are_validated_per_sub_command` en `main.rs`.
2. **Corregido: salidas con `fs::write` simple.** `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, `face-*`, los archivos de poses, las instantáneas,
   `vcs` y el registro de colaboración usan el escritor atómico que rechaza enlaces simbólicos; las salidas de ffmpeg se comprueban de antemano. Una prueba dirigida por tablas
   cubre unos 24 comandos. No cubierto de extremo a extremo: `face-train`/`face-eval` (prueba unitaria sólo de su escritor).
3. **Corregido: `golden check`** es exacto por defecto; `--tol` lo relaja; imprime el recuento de diferencias y la mayor diferencia.
   La afirmación de que «un cambio de color de fondo pasa» era un artefacto de la demostración (la capa del cielo cubre el fondo), no un fallo.
4. **Corregido: código de salida de `fuzz`.** Por defecto sólo informa (código 0); `--gate` o `--max-fail-rate` aplican la puerta. Véase `fuzz`.
5. **Corregido: `dof-export`** escribe `.mp4` y `.webm` (mediante ffmpeg, con `--codec`, sin audio); el texto de uso ahora los enumera. Un nombre
   con cualquier otra extensión se rechaza y no se escribe nada.
6. **Corregido: `export` con `.png`** escribe `<name>.atlas.json`, nunca `<name>.json`; el error de formato desconocido enumera todas las extensiones.
7. **Valores por defecto que el texto de uso no indica:** `studio` escucha en `127.0.0.1:8080`; `fuzz-grid --res` es 32; `export-fit`
   muestrea 8 fotogramas, busca con `descent` y se detiene a los 60 s (`--max-seconds 0` = ilimitado); `reduced-motion --factor`
   es 0,25; `size-report --tol` es 0,05; `pose-match` ejecuta 80 generaciones con semilla 1; `lod --out-dir` es la carpeta
   actual.
8. **`machine remove` puede negarse** (`binding 'goBind' still drives machine 'm'`); el texto de uso no menciona la
   comprobación de dependencias. `machine` y `state-bake` leen dos formatos de máquina distintos (`layers`/`inputs` frente a
   `initial`/`states`); la línea de uso de `state-bake` no tiene descripción y no indica el formato.
9. **Corregido: `stats`** ya no acepta `--json` (la salida es siempre JSON; la línea de uso lo dice).
10. **`vcs`:** la rama por defecto es `main` (`checkout master` falla con `unknown revision 'master'`); `vcs resolve`
    toma la ruta del conflicto *con* su barra inicial, tal como la imprime `merge` (`/background`), y `background` solo da
    `no conflict at 'background'`; el texto de uso escribe «see: osatura vcs», pero `osatura vcs` sin argumentos imprime un
    uso de una línea y termina con 1.
11. **Líneas de uso sin descripción.** `state-bake`, `pose-match`, `pose-bench`, `follow`, `keys`, `event`, `marker`,
    `idle`, `mouth`, `beats`, `export`, `frames`, `onion` y otros tienen una sinopsis pero ninguna explicación, o una descripción que
    está en una línea de continuación; su comportamiento en este documento procede de ejecutarlos.
12. **Nomenclatura de las propiedades.** Los mensajes y las salidas de `list` imprimen las propiedades en CamelCase (`LayerOpacity`, `BoneAngle`),
    pero todas las opciones, archivos de mapeo y archivos auxiliares aceptan los nombres en snake_case (`layer_opacity`, `bone_angle`); la forma CamelCase
    se rechaza (`bind add ... sun:LayerOpacity` falla con `unknown property 'LayerOpacity'`).
13. **`cloth`:** el texto de uso habla de «per-vertex deformation»; el resultado horneado se almacena como canales `lattice_dx` / `lattice_dy`
    en los puntos de la retícula de la tela (para la cuadrícula de 7x7 por defecto, los 42 puntos no fijados), véase la sección 7.
14. **`render-gpu`** sólo existe en las compilaciones hechas con `--features gpu`; no se pudo ejecutar en la compilación de pruebas, por lo que sus opciones
    se toman del texto de uso y no se verifican aquí.
15. **Lento en las compilaciones de depuración.** `pose-bench`, `face-train`, `ci run` y `export-fit` tardan minutos en una compilación sin optimizar.
    Los tiempos de este documento proceden del binario de depuración y son mucho menores en una compilación de release.
16. **Corregido (oleada 5): `golden write --scale`** por encima de 1 se limitaba antes a 1 en silencio; ahora se rechaza (la escala está en (0, 1]).
    `golden check` ignoraba `--scale` y usaba la almacenada; ahora rechaza la opción.
17. **Corregido (oleada 5): sinopsis.** Cada opción del texto de uso de un comando está en su sinopsis aquí y viceversa (prueba
    `cli_reference_synopses_match_usage`), y cada línea de uso sin alternativas se repite aquí palabra por palabra (prueba
    `cli_reference_repeats_usage_lines_verbatim`). `render-gpu --features gpu` y `layer-normal ... relight --layer-normals`
    en las descripciones de uso nombran una característica de compilación y una opción de otro comando, no opciones de esos comandos.
18. **Otros documentos son más antiguos que el programa en dos puntos** (véase [EXPORT_FORMATS.md](EXPORT_FORMATS.md) sección 17):
    `docs/LIMITATIONS.md` L-29 dice que la importación de Lottie no admite imágenes, efectos, precomposiciones ni mattes (el importador ahora los gestiona,
    con las advertencias enumeradas en la sección 9), y L-32 enumera OCA entre los formatos de importación mientras que la CLI sólo tiene
    `export-oca`.
## Índice alfabético de comandos

Cada comando con un resumen de una línea y la sección que lo documenta. Los subcomandos (`machine add`, `vcs commit`, ...) están en la sección de su comando padre.

| Comando | Resumen | Sección |
|---|---|---|
| `--version` | imprime la versión, la licencia, la oferta de código fuente y los enlaces de soporte | [1](#1-convenciones-comunes-a-todos-los-comandos) |
| `align` | alinea capas entre sí o con el escenario | [5](#5-rigging-skins-ik-y-utilidades-de-maquetación) |
| `audio-react` | la sonoridad controla una propiedad | [8](#8-audio-lip-sync-y-sonidos) |
| `autorig` | IK, respiración y parpadeo configurados a partir de los nombres de huesos y capas | [5](#5-rigging-skins-ik-y-utilidades-de-maquetación) |
| `autotrace` | vectorización local de mapas de bits (bandas de color, ajuste de Bezier) | [9](#9-importación) |
| `autotrace-bench` | tabla de fidelidad del vectorizador sobre formas sintéticas | [9](#9-importación) |
| `beats` | detecta saltos de sonoridad y registra marcadores o eventos | [8](#8-audio-lip-sync-y-sonidos) |
| `bind` | enlaces de datos: add, list, remove | [11](#11-máquinas-de-estados-enlaces-de-datos-contenedores-y-regiones-de-desplazamiento) |
| `bind-eval` | evalúa enlaces para unos valores dados | [11](#11-máquinas-de-estados-enlaces-de-datos-contenedores-y-regiones-de-desplazamiento) |
| `bone-vis` | muestra un hueso en las superposiciones sólo durante un rango de fotogramas | [5](#5-rigging-skins-ik-y-utilidades-de-maquetación) |
| `cage` | añade un deformador de jaula de coordenadas de valor medio | [7](#7-física-tela-y-deformadores) |
| `cage-pose` | pone claves a los desplazamientos de vértices de la jaula en un fotograma | [7](#7-física-tela-y-deformadores) |
| `camera-seq` | pone claves a la cámara a través de planos guardados | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `chan-meta` | color del canal e indicadores shy, solo y bloqueo | [5](#5-rigging-skins-ik-y-utilidades-de-maquetación) |
| `check` | valida un proyecto (código de salida 1 si no es válido) | [3](#3-creación-e-inspección-de-proyectos) |
| `ci` | CI de animación: golden, diff perceptual, comprobaciones estructurales | [14](#14-control-de-calidad) |
| `clip` | apila un take en la pila de clips | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `cloth` | hornea una simulación de tela determinista | [7](#7-física-tela-y-deformadores) |
| `coach` | sugerencias de timing y principios | [14](#14-control-de-calidad) |
| `collab-sim` | prueba de convergencia de la colaboración | [16](#16-estudio-colaboración-servidor-de-agente-mcp-y-registro-de-la-interfaz) |
| `collab-token` | genera un token de colaboración por participante (HMAC-SHA-256, caduca) | [16](#16-estudio-colaboración-servidor-de-agente-mcp-y-registro-de-la-interfaz) |
| `color-space` | elige composición en sRGB o en luz lineal para el proyecto | [12](#12-efectos-profundidad-de-campo-onion-skin-trayectorias-y-cámara) |
| `comp` | estados de capa con nombre (visibilidad, opacidad, opción del switch) | [5](#5-rigging-skins-ik-y-utilidades-de-maquetación) |
| `compare` | imagen antes / después / diferencia de dos proyectos | [4](#4-renderizado-y-exportación) |
| `container` | contenedores de maquetación: add, list, remove, solve | [11](#11-máquinas-de-estados-enlaces-de-datos-contenedores-y-regiones-de-desplazamiento) |
| `contracts` | contratos de movimiento (pruebas unitarias para la animación) | [14](#14-control-de-calidad) |
| `demo` | escribe el proyecto de demostración de rig incorporado | [3](#3-creación-e-inspección-de-proyectos) |
| `demo-fx` | escribe la demostración de líquido / booleanas / partículas | [3](#3-creación-e-inspección-de-proyectos) |
| `diff` | diferencias semánticas entre dos proyectos | [3](#3-creación-e-inspección-de-proyectos) |
| `distribute` | reparte uniformemente tres o más capas | [5](#5-rigging-skins-ik-y-utilidades-de-maquetación) |
| `dof` | render de un fotograma con profundidad de campo | [12](#12-efectos-profundidad-de-campo-onion-skin-trayectorias-y-cámara) |
| `dof-export` | profundidad de campo animada (GIF/APNG/vídeo/carpeta) | [12](#12-efectos-profundidad-de-campo-onion-skin-trayectorias-y-cámara) |
| `drive` | señales CSV -> claves mediante un archivo de mapeo | [10](#10-entrada-en-vivo-midi-seguimiento-facial-y-captura-de-movimiento) |
| `event` | añade un evento de runtime con nombre (con carga JSON opcional) | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `explain` | por qué una propiedad tiene su valor: contribuciones ordenadas | [3](#3-creación-e-inspección-de-proyectos) |
| `export` | exporta GIF, APNG, WebM/MP4, hoja PNG, HTML, Lottie, .lottie, glTF/GLB, ... | [4](#4-renderizado-y-exportación) |
| `export-fit` | exportación de mejor calidad que cabe en un presupuesto de bytes | [13](#13-nivel-de-detalle-informes-de-tamaño-y-export-fit) |
| `export-oca` | escribe una carpeta Open Cel Animation | [4](#4-renderizado-y-exportación) |
| `eye-clip` | enmascara las capas de iris/pupila con la forma del ojo | [5](#5-rigging-skins-ik-y-utilidades-de-maquetación) |
| `face-bake` | puntos de referencia -> claves | [10](#10-entrada-en-vivo-midi-seguimiento-facial-y-captura-de-movimiento) |
| `face-calibrate` | calibración del rostro neutro | [10](#10-entrada-en-vivo-midi-seguimiento-facial-y-captura-de-movimiento) |
| `face-detect` | cuadros delimitadores de rostros y 23 puntos de referencia en una imagen | [10](#10-entrada-en-vivo-midi-seguimiento-facial-y-captura-de-movimiento) |
| `face-eval` | informe de precisión del detector con datos reservados | [10](#10-entrada-en-vivo-midi-seguimiento-facial-y-captura-de-movimiento) |
| `face-synth` | animación sintética de una cabeza | [10](#10-entrada-en-vivo-midi-seguimiento-facial-y-captura-de-movimiento) |
| `face-track` | NDJSON de puntos de referencia a partir de vídeo o fotogramas | [10](#10-entrada-en-vivo-midi-seguimiento-facial-y-captura-de-movimiento) |
| `face-train` | entrena el detector de rostros con rostros sintéticos | [10](#10-entrada-en-vivo-midi-seguimiento-facial-y-captura-de-movimiento) |
| `follow` | generadores de acción solapada / follow-through | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `foot-lock` | fija un controlador durante el contacto con el suelo | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `frames` | renderiza cada fotograma en una carpeta (SVG, o PNG con --png) | [4](#4-renderizado-y-exportación) |
| `fuzz` | encuentra poses en las que se rompe el rig | [14](#14-control-de-calidad) |
| `fuzz-grid` | mapa de severidad de dos parámetros | [14](#14-control-de-calidad) |
| `fx` | renderiza un fotograma con una cadena de postefectos | [12](#12-efectos-profundidad-de-campo-onion-skin-trayectorias-y-cámara) |
| `glue` | pega vértices de malla entre sí | [7](#7-física-tela-y-deformadores) |
| `golden` | escribe/comprueba fotogramas de regresión | [14](#14-control-de-calidad) |
| `idle` | genera un canal idle procedural | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `import-anim` | GIF/APNG animado -> capa fotograma a fotograma | [9](#9-importación) |
| `import-bvh` | captura de movimiento BVH -> canales | [9](#9-importación) |
| `import-lottie` | subconjunto de Lottie -> proyecto | [9](#9-importación) |
| `import-ora` | OpenRaster -> proyecto | [9](#9-importación) |
| `import-seq` | secuencia PNG -> capa fotograma a fotograma | [9](#9-importación) |
| `import-svg` | subconjunto de SVG -> capa vectorial | [9](#9-importación) |
| `keyedit` | ediciones de dope sheet como un único lote seguro para deshacer | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `keys` | operaciones masivas de claves sobre todos los canales con claves | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `layer-normal` | asocia un mapa de normales a una capa | [12](#12-efectos-profundidad-de-campo-onion-skin-trayectorias-y-cámara) |
| `layer-pivot` | mueve el pivote de una capa sin cambiar la imagen | [7](#7-física-tela-y-deformadores) |
| `layout-preview` | renderiza con la maquetación resuelta de nuevo para otro tamaño de lienzo | [11](#11-máquinas-de-estados-enlaces-de-datos-contenedores-y-regiones-de-desplazamiento) |
| `lint` | comprobaciones estáticas del rig/animación (código de salida 1 si hay errores) | [14](#14-control-de-calidad) |
| `lipsync` | controla una capa switch de boca a partir de la sonoridad del audio (no basado en fonemas) | [8](#8-audio-lip-sync-y-sonidos) |
| `lipsync-text` | alinea una transcripción con el audio y deriva las formas de la boca | [8](#8-audio-lip-sync-y-sonidos) |
| `live-map` | entrada en vivo grabada (MIDI, ratón, mando, teclas) -> claves, takes, eventos | [10](#10-entrada-en-vivo-midi-seguimiento-facial-y-captura-de-movimiento) |
| `lod` | niveles de detalle con cotas de error en píxeles declaradas | [13](#13-nivel-de-detalle-informes-de-tamaño-y-export-fit) |
| `lod-pick` | nivel más grueso que cabe en un presupuesto de píxeles en una pantalla | [13](#13-nivel-de-detalle-informes-de-tamaño-y-export-fit) |
| `loop-solve` | hace que los canales con claves hagan bucle sin costuras | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `machine` | máquinas de estados de runtime: add, list, remove, validate | [11](#11-máquinas-de-estados-enlaces-de-datos-contenedores-y-regiones-de-desplazamiento) |
| `machine-sim` | reproduce entradas temporizadas: registro de estados más fotogramas SVG | [11](#11-máquinas-de-estados-enlaces-de-datos-contenedores-y-regiones-de-desplazamiento) |
| `magnet` | hornea una atracción por proximidad | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `make-loop` | hace que un ciclo haga bucle sin costuras (opcionalmente en el sitio) | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `marker` | añade un marcador o rango en la línea de tiempo | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `mcp` | servidor de agente (JSON-RPC sobre stdio) | [16](#16-estudio-colaboración-servidor-de-agente-mcp-y-registro-de-la-interfaz) |
| `mesh-lock` | los seguidores siguen la superficie de una malla anfitriona | [7](#7-física-tela-y-deformadores) |
| `mesh-trace` | vectoriza la silueta alfa de una imagen en una malla triangular con skin | [7](#7-física-tela-y-deformadores) |
| `mirror` | refleja todos los canales con claves izquierda<->derecha | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `mocap-info` | jerarquía y extensión de un archivo BVH | [10](#10-entrada-en-vivo-midi-seguimiento-facial-y-captura-de-movimiento) |
| `mocap-match` | asignación de articulaciones BVH a huesos del rig | [10](#10-entrada-en-vivo-midi-seguimiento-facial-y-captura-de-movimiento) |
| `mocap-retarget` | hornea la captura de movimiento como canales añadidos | [10](#10-entrada-en-vivo-midi-seguimiento-facial-y-captura-de-movimiento) |
| `mouth` | importa señales de boca (Papagayo/Rhubarb) | [8](#8-audio-lip-sync-y-sonidos) |
| `onion` | onion skin en PNG o SVG | [12](#12-efectos-profundidad-de-campo-onion-skin-trayectorias-y-cámara) |
| `parallax` | profundidad multiplano para capas de nivel superior | [5](#5-rigging-skins-ik-y-utilidades-de-maquetación) |
| `parent-switch` | hornea un cambio de padre animado sin saltos | [5](#5-rigging-skins-ik-y-utilidades-de-maquetación) |
| `physics` | hornea una simulación de cadena | [7](#7-física-tela-y-deformadores) |
| `pick` | qué hay bajo un punto del lienzo (primero lo más alto) | [3](#3-creación-e-inspección-de-proyectos) |
| `pose-bench` | precisión con datos reservados de los métodos de poses | [17](#17-base-de-datos-de-poses-sugerencias-y-otras-utilidades) |
| `pose-db` | base de datos de poses: build, info, train-mlp | [17](#17-base-de-datos-de-poses-sugerencias-y-otras-utilidades) |
| `pose-match` | ajusta parámetros para que las puntas de los huesos alcancen objetivos | [17](#17-base-de-datos-de-poses-sugerencias-y-otras-utilidades) |
| `pose-suggest` | sugerencias de intermedias, compleción y siguiente pose | [17](#17-base-de-datos-de-poses-sugerencias-y-otras-utilidades) |
| `proxy-bench` | milisegundos por fotograma en cada nivel de proxy | [4](#4-renderizado-y-exportación) |
| `reduced-motion` | escribe una copia más calmada de un proyecto | [14](#14-control-de-calidad) |
| `reference` | marca una capa como referencia de calco (nunca se exporta) | [5](#5-rigging-skins-ik-y-utilidades-de-maquetación) |
| `render` | renderiza un fotograma a PNG o SVG (proxies de scrub, huesos, escala) | [4](#4-renderizado-y-exportación) |
| `render-gpu` | render por GPU (sólo compilaciones con wgpu) | [4](#4-renderizado-y-exportación) |
| `render-hdr` | render de coma flotante a OpenEXR o PNG de 16 bits | [4](#4-renderizado-y-exportación) |
| `reparent` | cambia el padre de un hueso conservando la pose de reposo | [5](#5-rigging-skins-ik-y-utilidades-de-maquetación) |
| `retarget` | copia la animación entre rigs mediante pares de cadenas | [5](#5-rigging-skins-ik-y-utilidades-de-maquetación) |
| `safety` | informe de destellos / destellos rojos / movimiento de cámara | [14](#14-control-de-calidad) |
| `script` | ejecuta un script Rhai aislado que edita el proyecto | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `scroll` | regiones de desplazamiento: add, list, remove, eval | [11](#11-máquinas-de-estados-enlaces-de-datos-contenedores-y-regiones-de-desplazamiento) |
| `select-box` | selección por marco | [3](#3-creación-e-inspección-de-proyectos) |
| `shot` | guarda un plano de cámara | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `size-report` | dónde están los bytes de un proyecto/exportación | [13](#13-nivel-de-detalle-informes-de-tamaño-y-export-fit) |
| `skin` | skins con nombre: conjuntos de capas, huesos exclusivos del skin, restricciones, cadenas IK | [5](#5-rigging-skins-ik-y-utilidades-de-maquetación) |
| `skin-diffuse` | pesos de skin por difusión de calor | [7](#7-física-tela-y-deformadores) |
| `slider` | crea un hueso controlador que mezcla poses almacenadas | [5](#5-rigging-skins-ik-y-utilidades-de-maquetación) |
| `snapshot` | historial numerado de revisiones de un proyecto en una carpeta (save, list, restore) | [3](#3-creación-e-inspección-de-proyectos) |
| `soft-scale` | escalado proporcional de una selección | [7](#7-física-tela-y-deformadores) |
| `sound` | pistas de sonido del proyecto: list, add, edit, remove, mix | [8](#8-audio-lip-sync-y-sonidos) |
| `speech-motion` | asentimientos y alzamientos de cejas aditivos temporizados sobre las palabras alineadas | [8](#8-audio-lip-sync-y-sonidos) |
| `spring-fit` | ajusta frecuencia/amortiguación de un muelle a una curva | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `state-bake` | hornea una máquina de estados basada en fotogramas en claves | [11](#11-máquinas-de-estados-enlaces-de-datos-contenedores-y-regiones-de-desplazamiento) |
| `stats` | recuentos de huesos, capas, canales, claves (JSON) | [3](#3-creación-e-inspección-de-proyectos) |
| `stretch` | hornea un seguidor que se estira para alcanzar | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `studio` | editor de navegador (servidor web local) | [16](#16-estudio-colaboración-servidor-de-agente-mcp-y-registro-de-la-interfaz) |
| `tag` | propone nombres de rol estándar para huesos y capas | [3](#3-creación-e-inspección-de-proyectos) |
| `take-apply` | aplica un take de la biblioteca con fundidos cruzados | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `take-list` | enumera los takes de una biblioteca | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `take-record` | graba un take en una biblioteca | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `timecode` | fotograma <-> código de tiempo SMPTE (con soporte de drop-frame) | [3](#3-creación-e-inspección-de-proyectos) |
| `timeline` | pistas por canal con claves, bloqueos y colores | [3](#3-creación-e-inspección-de-proyectos) |
| `trace` | vectoriza el canal alfa de un PNG en un trazado vectorial | [9](#9-importación) |
| `trail` | trayectoria de movimiento de un hueso, capa o punto | [12](#12-efectos-profundidad-de-campo-onion-skin-trayectorias-y-cámara) |
| `trim` | hornea un trim path en una capa vectorial | [5](#5-rigging-skins-ik-y-utilidades-de-maquetación) |
| `ui` | registro de comandos del estudio, atajos, disposiciones, búsqueda | [16](#16-estudio-colaboración-servidor-de-agente-mcp-y-registro-de-la-interfaz) |
| `update-check` | pregunta a GitHub si existe una versión más reciente; proxy gratuito opcional | [13](#13-nivel-de-detalle-informes-de-tamaño-y-export-fit) |
| `vcs` | control de versiones con fusión semántica para un archivo de proyecto | [15](#15-control-de-versiones-vcs) |
| `view-swap` | controla una capa switch a partir de una propiedad de un hueso (función escalón) | [5](#5-rigging-skins-ik-y-utilidades-de-maquetación) |
| `walk` | genera un ciclo de caminar | [6](#6-animación-keys-línea-de-tiempo-y-generadores) |
| `warp-pivot` | rota/escala/mueve una selección de puntos de control | [7](#7-física-tela-y-deformadores) |
| `waveform` | picos mín./máx. para dibujar | [8](#8-audio-lip-sync-y-sonidos) |
