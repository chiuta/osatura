# Primeros pasos con OSATURA

Esta guía te lleva desde un checkout recién clonado hasta un pequeño personaje animado que has construido,
dotado de rig, animado con claves, convertido en bucle, renderizado y exportado usando únicamente la línea
de comandos. Todos los comandos siguientes se ejecutaron de verdad mientras se escribía esta página; las
salidas mostradas son las que se imprimieron.

Cómo se verificó: Linux, `rustc 1.97.0`, `ffmpeg 6.1.1` y la compilación **debug** del binario
(`cargo build -p osatura-cli`). Una compilación release ejecuta el mismo código más rápido. La batería de pruebas también pasa en macOS y Windows en GitHub Actions; los comandos de esta página solo se ejecutaron a mano en Linux. El repositorio
no declara ninguna versión mínima admitida de Rust; 1.97.0 es simplemente el compilador con el que se probó.

Licencia de la documentación: CC-BY-4.0. Véanse también [RIGGING_GUIDE.md](RIGGING_GUIDE.md),
[ANIMATION_GUIDE.md](ANIMATION_GUIDE.md), [STUDIO.md](STUDIO.md), [FAQ.md](FAQ.md) y la lista completa de
comandos en [CLI_REFERENCE.md](CLI_REFERENCE.md).

## 1. Qué necesitas

| Requisito | Necesario para | Notas |
|---|---|---|
| Toolchain de Rust (`rustc`, `cargo`) | compilar OSATURA | instálala desde <https://rustup.rs>; canal estable |
| Un navegador web | el estudio (`osatura studio`) | solo se probó Chromium, véase [STUDIO.md](STUDIO.md) |
| `ffmpeg` en el `PATH` (opcional) | `.mp4`, `.webm`, mezclar sonido en vídeo/HTML, `sound mix`, `face-track` sobre un archivo de vídeo | todo lo demás funciona sin él |

Sin `ffmpeg`, los comandos que lo necesitan se detienen con un mensaje claro y no escriben nada:

```
$ PATH=/nonexistent osatura export buddy-final.json o.mp4
cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead
```

No se requiere GPU. El renderizador GPU opcional (`render-gpu`) no forma parte de la compilación por defecto.

**¿Sin Rust? Usa un archivo precompilado.** Cuando el propietario haya publicado la versión preliminar en borrador `v0.1.0-alpha.3`, la [página de Releases](https://github.com/chiuta/osatura/releases) ofrece `osatura-windows-x86_64.zip`, `osatura-macos-arm64.tar.gz`, `osatura-linux-x86_64.tar.gz` (glibc 2.35+; las versiones posteriores añaden el `osatura-linux-x86_64-musl.tar.gz` estático para distribuciones más antiguas) y `SHA256SUMS.txt`. Comprueba el archivo con las sumas (`sha256sum -c SHA256SUMS.txt --ignore-missing`, o `Get-FileHash` en Windows), descomprímelo y ejecuta `osatura` (`osatura.exe`) desde la carpeta descomprimida; omite la compilación de la sección 2 y lee "Windows: doble clic en `osatura.exe`" y "El estudio como ventana" cerca del final de esta página. Los programas no están firmados, así que el sistema te avisará la primera vez. Todo lo que sigue se ejecutó con una compilación local del mismo código fuente.

## 2. Compílalo

Desde la raíz del repositorio:

```
cargo build --release -p osatura-cli
```

El binario es `target/release/osatura`. Ponlo en tu `PATH` o crea un alias:

```
export PATH="$PWD/target/release:$PATH"      # or: alias osatura=$PWD/target/release/osatura
osatura --version
```

`--version` imprime la versión, la licencia (AGPL-3.0-or-later para la CLI y el estudio, MPL-2.0 para las
bibliotecas del motor) y la oferta de código fuente. Ejecutar `osatura` sin argumentos imprime el uso
completo; `osatura <command> --help` imprime el uso de un solo comando.

También puedes ejecutarlo sin instalarlo: `cargo run --release -p osatura-cli -- demo demo.json`.

Compilar la herramienta de línea de comandos con `--no-default-features` elimina el comando `script` (el
motor de scripting Rhai); este tutorial usa `script`, así que conserva las funciones por defecto.

## 3. Primeros pasos con la demo integrada

`osatura demo` escribe un proyecto de ejemplo completo: un brazo con skinning, una pierna con IK, una capa
de conmutación de lip-sync, un fondo con paralaje y un movimiento de cámara.

```
$ osatura demo demo.json
wrote demo.json
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels
```

`check` compila el proyecto exactamente como lo haría el renderizador, de modo que un proyecto que supera
`check` puede renderizarse. Ahora dibuja una imagen fija, con los huesos superpuestos, y exporta una animación:

```
$ osatura render demo.json 12 frame12.png --bones
wrote frame12.png
$ osatura render demo.json 12 frame12.svg
wrote frame12.svg
$ osatura export demo.json demo.gif
wrote demo.gif
$ osatura export demo.json demo.html
wrote demo.html
$ osatura export demo.json demo.lottie
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
wrote demo.lottie
```

Lo que acabas de hacer:

* `render <project> <frame> <out.png|out.svg>` dibuja un fotograma. `--bones` superpone el esqueleto,
  `--scale N` renderiza a N veces el tamaño del lienzo (0 < N <= 16).
* `export` elige el formato según la extensión del archivo: `.gif`, `.html` (una página con reproductor
  autónomo), `.json` (Lottie), `.lottie` (dotLottie), `.apng`, `.webm`, `.mp4`, `.glb`, `.png` (hoja de sprites).
* Las líneas `warning:` son la parte honesta: Lottie no puede representar todo lo que OSATURA sí puede.
  Cada limitación se imprime en stderr, y `--report r.json` escribe la misma lista como JSON.
* Abre `demo.html` en un navegador para reproducir la animación; no interviene ningún servidor ni ninguna red.

Para explorar de forma interactiva:

```
osatura studio demo.json --port 8080
```

y abre <http://127.0.0.1:8080/>. Véase [STUDIO.md](STUDIO.md).

## 4. Un tutorial de 15 minutos: un personaje que saluda

Vas a construir a "Buddy" a partir de un proyecto vacío. El archivo de proyecto es JSON, pero nunca tendrás
que escribir más que una cabecera: el comando `script` ejecuta un pequeño script de [Rhai](https://rhai.rs)
en un sandbox a través del mismo editor validado que usa el estudio. Si se rechaza una petición de un script,
*no se aplica nada* (todo o nada), de modo que un script defectuoso nunca deja un archivo a medio editar.

Trabaja en un directorio vacío. Cada paso escribe un nuevo archivo numerado, así que siempre puedes volver atrás.

### Paso 1: un proyecto vacío (1 minuto)

```
cat > buddy.json <<'EOF'
{"format": 1, "name": "Buddy", "width": 320, "height": 240, "fps": 24, "frames": 24}
EOF
osatura check buddy.json
```
```
ok: 'Buddy' 320×240 @ 24 fps, 24 frames, 0 bones, 0 ik, 0 layers, 0 channels
```

Esas seis claves son el mínimo. 24 fotogramas a 24 fps es un segundo, que será nuestro bucle.

### Paso 2: formas (2 minutos)

Las capas se dibujan de atrás hacia delante, en el orden en que las añades. Crea `shapes.rhai`:

```
add_gradient_rect("sky", 0, 0, 320, 240, "#bfe3f5", "#fdf3d8");
add_rect("ground", 0, 200, 320, 40, "#7fa66b");
add_rect("torso", 140, 120, 40, 70, "#e07a5f");
add_circle("head", 160, 95, 24, "#f2cc8f");
add_rect("upper_arm", 160, 134, 40, 12, "#3d405b");
add_rect("forearm", 200, 134, 40, 12, "#81b29a");
```
```
$ osatura script buddy.json 01-shapes.json shapes.rhai
6 request(s) applied; wrote 01-shapes.json
```

Las coordenadas son unidades del lienzo, con el origen arriba a la izquierda y la y apuntando hacia abajo.
El brazo se dibuja en horizontal, con el hombro en (160, 140), porque es la pose en la que construiremos sus huesos.

### Paso 3: huesos, parentesco y arte asociado (3 minutos)

Un hueso tiene un nombre, un padre (vacío = raíz), una longitud, una posición de cabeza (`x`, `y`) en el
espacio de su padre (para una raíz: en el lienzo) y un ángulo de reposo en grados relativo a su padre. El
ángulo 0 apunta a lo largo de +x y los ángulos positivos giran en sentido horario en pantalla, de modo que
90 apunta hacia abajo y -90 hacia arriba.
Crea `bones.rhai`:

```
add_bone("body", "", 70, 160, 190, -90);
add_bone("neck", "body", 20, 70, 0, 0);
add_bone("arm_up", "body", 40, 50, 0, 90);
add_bone("arm_low", "arm_up", 40, 40, 0, 0);
add_bone("hand_ctrl", "", 10, 210, 165, 0);
set_layer("torso", "bone", "body");
set_layer("head", "bone", "neck");
set_layer("upper_arm", "bone", "arm_up");
set_layer("forearm", "bone", "arm_low");
```
```
$ osatura script 01-shapes.json 02-bones.json bones.rhai
9 request(s) applied; wrote 02-bones.json
$ osatura render 02-bones.json 0 rig.png --bones --scale 2
wrote rig.png
```

Lee los huesos así: `body` es un hueso raíz en (160, 190) que apunta hacia arriba (-90) y mide 70 de largo.
`neck` es su hijo; su cabeza queda a 70 unidades a lo largo de `body` (en la parte superior). `arm_up` es otro
hijo de `body`, 50 unidades por encima en la columna (el hombro), girado +90 para que apunte hacia un lado.
`arm_low` es el hijo de `arm_up`: el parentesco significa que al rotar el brazo superior se arrastra con él
el antebrazo. `hand_ctrl` es un hueso raíz independiente que usaremos como objetivo de IK; no tiene arte.

`set_layer(layer, "bone", bone)` hace que una capa acompañe a un hueso de forma *rígida*: sigue el movimiento
del hueso respecto a su pose de reposo. Es la forma más sencilla de dotar de rig a un dibujo animado hecho de
piezas separadas. (Para formas flexibles se hace skinning de anclas vectoriales a varios huesos; véase
[RIGGING_GUIDE.md](RIGGING_GUIDE.md).)

### Paso 4: cinemática inversa (1 minuto)

En lugar de poner claves a los ángulos del hombro y del codo, deja que el solver alcance `hand_ctrl`. Crea `ik.rhai`:

```
add_ik("arm_ik", ["arm_up", "arm_low"], "hand_ctrl", 1);
```
```
$ osatura script 02-bones.json 03-ik.json ik.rhai
1 request(s) applied; wrote 03-ik.json
```

El último argumento es la dirección de flexión: +1 o -1 elige hacia qué lado se pliega el codo. Una cadena de
dos huesos usa un solver analítico; las cadenas de tres o más huesos usan FABRIK. A partir de ahora, mover
`hand_ctrl` mueve todo el brazo.

### Paso 5: claves y suavizado (3 minutos)

`set_key(target, property, frame, value, ease)` crea claves. Para un hueso, `bone_angle`, `bone_scale`,
`bone_offset_x` y `bone_offset_y` son *deltas respecto a la pose de reposo*, de modo que un valor de clave de
0 significa "reposo". El nombre del suavizado se aplica al segmento que *empieza* en esa clave. Crea `keys.rhai`:

```
set_key("hand_ctrl", "bone_offset_y", 0, 0, "ease_in_out");
set_key("hand_ctrl", "bone_offset_y", 12, -75, "ease_in_out");
set_key("hand_ctrl", "bone_offset_y", 24, 0, "ease_in_out");
set_key("neck", "bone_angle", 0, 0, "smooth");
set_key("neck", "bone_angle", 16, 10, "smooth");
set_key("neck", "bone_angle", 24, 0, "smooth");
```
```
$ osatura script 03-ik.json 04-keys.json keys.rhai
6 request(s) applied; wrote 04-keys.json
$ osatura timeline 04-keys.json
timeline: 24 frames at 24 fps, 2 channels
Bones
  hand_ctrl
    #0 bone_offset_y (keys @ 0 12 24)
  body
    neck
      #1 bone_angle (keys @ 0 16 24)
```

Los nombres de suavizado que no necesitan parámetros son `linear`, `step`, `ease_in`, `ease_out`, `ease_in_out`,
`smooth` y `bounce`. `bezier:x1,y1,x2,y2` lo acepta `keyedit`, y `spring` y `back` toman parámetros
(véase [PROJECT_FORMAT.md](PROJECT_FORMAT.md)). La mano y la cabeza usan tiempos distintos a propósito: la
acción solapada da sensación de vida.

### Paso 6: conviértelo en bucle (1 minuto)

Ambos canales empiezan y terminan con el mismo valor (el fotograma 24 repite el fotograma 0), que es lo que
necesita un bucle. `loop-solve` comprueba la costura y configura los canales para que hagan bucle:

```
$ osatura loop-solve 04-keys.json buddy-final.json
wrote buddy-final.json: 2 channel(s) adjusted, seam 0.000 -> 0.000
$ osatura lint buddy-final.json
0 finding(s), 0 error(s)
```

"seam 0.000 -> 0.000" significa que la última pose ya es igual a la primera, así que no hubo que corregir nada.
Un proyecto de N fotogramas reproduce los fotogramas 0..N-1 de forma repetida, de modo que el fotograma N es la
misma pose que el fotograma 0 y nunca se muestra dos veces. Si creas un bucle de forma que el *último fotograma
mostrado* sea igual al primero, pasa `--inclusive` (la convención anterior). Si tu animación tiene un
desplazamiento neto, `make-loop` lo elimina. Véase [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).

`lint` ejecuta comprobaciones estáticas del rig y de la animación (objetivos de IK inalcanzables, canales
muertos, huesos que faltan...); termina con código 1 cuando encuentra errores, por lo que es seguro ponerlo en
un script o en un trabajo de CI.

### Paso 7: míralo, renderízalo (2 minutos)

```
$ osatura stats buddy-final.json
```
imprime un resumen en JSON (aquí: 5 huesos, 1 cadena de IK, 6 capas, 2 canales, 6 claves). Renderiza imágenes
fijas y una secuencia completa de fotogramas:

```
$ osatura render buddy-final.json 12 frame12.png --scale 2
wrote frame12.png
$ osatura render buddy-final.json 12 frame12.svg
wrote frame12.svg
$ osatura frames buddy-final.json frames --png
wrote 24 frames to frames
$ ls frames | head -3
frame_0000.png
frame_0001.png
frame_0002.png
```

![Cuatro fotogramas (0, 6, 12, 18) de la animación de Buddy, una vez normal y otra con los huesos superpuestos. El brazo derecho se levanta en un saludo con el codo hacia abajo mientras la cabeza asiente ligeramente.](../../img/tutorial-buddy.png)

Sin `--png`, `frames` escribe archivos SVG. Los renders son deterministas: el mismo proyecto y fotograma dan
los mismos bytes en cualquier máquina con la misma compilación, lo que hace posibles las comprobaciones de
fotogramas de referencia de [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).

### Paso 8: exportar (2 minutos)

```
$ osatura export buddy-final.json buddy.gif
wrote buddy.gif
$ osatura export buddy-final.json buddy.html
wrote buddy.html
$ osatura export buddy-final.json buddy.json --report buddy-report.json
wrote buddy.json
$ cat buddy-report.json
{
  "count": 0,
  "format": "lottie",
  "warnings": []
}
$ osatura export buddy-final.json buddy.mp4        # needs ffmpeg
wrote buddy.mp4
$ osatura export buddy-final.json buddy.apng
wrote buddy.apng
```

Tamaños de archivo de este pequeño proyecto: GIF 145 KB, JSON de Lottie 21 KB, reproductor HTML 50 KB,
APNG 75 KB, MP4 6 KB. `buddy.json` es Lottie; tiene una lista de avisos vacía porque Buddy solo usa funciones
que Lottie puede expresar. El archivo HTML es un reproductor autónomo (ábrelo en cualquier navegador).

Algunas reglas que conviene recordar:

* `export` y `render` **se niegan a sobrescribir el proyecto de entrada** (`refusing to overwrite the input
  project buddy-final.json with output of the same name`), y se niegan a escribir a través de un enlace
  simbólico salvo que pases `--follow-symlinks`. Los comandos que transforman un proyecto en otro
  (`script`, `loop-solve`, `walk`, `autorig`...) *no* hacen esa comprobación: escriben de forma atómica, de modo
  que `osatura script p.json p.json x.rhai` edita en el sitio. Usa nombres distintos hasta que confíes en un script.
* GIF tiene 256 colores y transparencia de 1 bit; para un alfa suave usa `.apng` o `.webm`.
* Ejecuta `osatura export-fit` (véase [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)) cuando necesites ajustarte a un presupuesto de bytes.

### Paso 9 (opcional): edítalo en el estudio

```
osatura studio buddy-final.json --port 8080
```

Arrastra las capas, selecciona una clave en el dope sheet, pulsa `Space` para reproducir. No se escribe nada
en tu archivo hasta que pulses **Save** (`Ctrl+S`).

## 5. Cómo encajan las piezas

| Concepto | En el archivo | Lo creas con |
|---|---|---|
| Capa (vectorial, imagen, grupo, conmutación...) | `layers` | `add_rect`, `add_circle`, `add_polygon`, `add_line`, `import-svg`, `autotrace`... |
| Hueso | `bones` (los padres antes que los hijos) | `add_bone`, `set_parent`, `reparent` |
| Cadena de IK / restricción / muelle | `ik`, `constraints`, `springs` | `add_ik`, `add_aim`, `add_copy_rotation`, `add_pin`; los muelles mediante JSON |
| Canal (una propiedad animada) | `channels` | `set_key`, `keys`, `keyedit`, expresiones, ruido, bakes |
| Clip, take, máquina de estados, enlace | `clips`, `machines`, `bindings` | `take-record`, `clip`, `machine`, `bind` |

Todo es JSON plano, por lo que funcionan los diffs (`osatura diff a.json b.json`), el control de versiones
(`osatura vcs`) y los scripts. Los campos desconocidos se notifican como avisos y se ignoran:

```
$ osatura check unk.json
warning: unknown field bogus (ignored)
ok: 'X' 100×100 @ 24 fps, 10 frames, 0 bones, 0 ik, 0 layers, 0 channels
```

Un proyecto solo puede referenciar archivos de imagen y de sonido **dentro de su propio directorio**. Las rutas
absolutas y `..` se rechazan salvo que pases `--allow-external-assets` (véase [FAQ.md](FAQ.md)).

## 6. Adónde ir después

* Dota de rig algo flexible (skins, pesos, lattice, cage, tela, física): [RIGGING_GUIDE.md](RIGGING_GUIDE.md).
* Ciclos de caminata, pilas de clips, lip-sync, mocap, máquinas de estados, control de calidad y presupuestos de tamaño:
  [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).
* El editor en el navegador, sus atajos y la API HTTP: [STUDIO.md](STUDIO.md).
* Preguntas y mensajes de error: [FAQ.md](FAQ.md). Limitaciones conocidas: [LIMITATIONS.md](LIMITATIONS.md).
* Todos los comandos y opciones: [CLI_REFERENCE.md](CLI_REFERENCE.md); el formato de archivo: [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

## Windows: doble clic en `osatura.exe`

`osatura.exe` es un programa de línea de comandos. Desde 0.1.0-alpha.2, abrirlo desde el Explorador (doble clic) crea `Documents\OSATURA\first-rig.json` a partir de la demo de la persona caminando (`osatura demo-person`; un archivo existente nunca se sobrescribe), inicia el estudio en `http://127.0.0.1:8080` y lo abre en tu navegador; la ventana de consola permanece abierta mientras el estudio se ejecuta, ciérrala para detenerlo. Abierto desde un terminal sin argumentos imprime el uso como antes. En 0.1.0-alpha.1 un doble clic solo imprimía el uso y la ventana se cerraba de inmediato.

Los ejecutables **no están firmados**: Windows SmartScreen muestra "Editor desconocido" (elige *Más información* y después *Ejecutar de todas formas*) y Gatekeeper de macOS te pide aprobar el programa en *Ajustes del Sistema > Privacidad y seguridad*. Comprueba antes la descarga con `SHA256SUMS.txt`. Si el puerto 8080 está ocupado, ejecuta `osatura studio first-rig.json --port 8090` desde un terminal.

## El estudio como ventana: `osatura-studio` (Windows y macOS)

Los archivos de Windows y macOS contienen un segundo programa, `osatura-studio` (`osatura-studio.exe`), una pequeña ventana nativa alrededor del estudio del navegador: inicia el motor `osatura` de la misma carpeta como `studio` en un puerto local libre, muestra la página en una ventana WebView2 (Windows 11 ya la incluye; Windows más antiguo necesita el runtime Evergreen) o WKWebView (macOS), y detiene el servidor al cerrar la ventana. No aparece ninguna ventana de consola. Mantenlo en la misma carpeta que `osatura(.exe)`.

* **Exportaciones** (GIF, reproductor HTML, Lottie) abren un cuadro nativo "Guardar como" que empieza en la carpeta del proyecto, y un mensaje nombra el archivo cuando se guarda. (alpha.2 las guardaba en silencio en tu carpeta de Descargas; alpha.3 te pregunta.)
* Haz doble clic para trabajar en `Documents/OSATURA/first-rig.json` (se crea a partir de la demo de la persona caminando, `osatura demo-person`, si no existe), o inícialo como `osatura-studio my-project.json` (por ejemplo mediante "Abrir con").
* `osatura-studio --selftest` inicia el servidor, descarga la página y lo detiene sin ventana; el código de salida es 0 si funcionó (la CI lo ejecuta en Windows y macOS).
* **Limitaciones.** La ventana se usó en un escritorio real de Windows 11 (alpha.2: el estudio y las exportaciones funcionaron); el cuadro "Guardar como" de alpha.3 está compilado y autoprobado en CI pero nadie lo ha probado todavía, y nadie ha usado la ventana en macOS. Otras funciones propias de un navegador que dependen del sistema (arrastrar y soltar desde el Explorador o el Finder, abrir archivos desde los selectores propios del estudio) son las de la vista web del sistema y no se han probado. No hay ventana para Linux (necesitaría paquetes del sistema GTK / WebKitGTK): usa `osatura studio` y un navegador. El programa no está firmado (véase arriba).
