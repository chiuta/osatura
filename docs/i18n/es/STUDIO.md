# Estudio de OSATURA

El estudio es un editor en el navegador servido por el propio binario `osatura`: `osatura studio project.json`.
Tiene un lienzo con gizmos de selección y transformación, una lista de capas, una lista de huesos, un dope sheet y un gráfico de curvas,
onion skin y trayectorias de movimiento, una paleta de comandos con atajos reasignables, sugerencias de poses, paneles de entrada en vivo y
de rostro, botones de exportación a GIF/HTML/Lottie y una sesión de edición compartida multiusuario. No tiene cuentas, ni TLS ni
base de datos: es un pequeño servidor HTTP de biblioteca estándar delante del mismo editor que usa la línea de comandos.

Todo lo que aparece aquí se comprobó con el binario real (`osatura 0.1.0`, compilación debug, Linux) y, para la interfaz
de usuario, en Chromium sin interfaz controlado con Playwright. Las capturas de pantalla se tomaron así (1400x900, el proyecto de demostración integrado). Solo se probó **Chromium**; véase la sección 11. En Windows y macOS los archivos de la versión también contienen `osatura-studio`, el mismo estudio en una ventana nativa (compilado y autoprobado en CI, aún no usado por una persona; véanse [GETTING_STARTED.md](GETTING_STARTED.md) y [LIMITATIONS.md](LIMITATIONS.md) L-75), y en Windows un doble clic en `osatura.exe` inicia el estudio y abre el navegador (L-76).

Licencia de este documento: CC-BY-4.0. Para la línea de comandos véanse [GETTING_STARTED.md](GETTING_STARTED.md) y
[CLI_REFERENCE.md](CLI_REFERENCE.md); para la seguridad del servidor véase también [SECURITY.md](SECURITY.md).

![El estudio con el proyecto de demostración: barra de herramientas arriba, el lienzo con una capa de brazo seleccionada y sus huesos dibujados, el panel de capas a la derecha y la línea de tiempo debajo con diamantes de claves.](../../img/studio-overview.png)

## 1. Iniciar el estudio

```
osatura studio <project.json> [--port N] [--ui-config F] [--bind ADDR] [--allow-host NAMES]
                [--public-origin URL] [--source-url URL] [--collab-token T] [--collab-log LOG [--collab-nosync]]
```

```
$ osatura demo demo.json
$ osatura studio demo.json --port 8080
AGPL-3.0 source offer: http://127.0.0.1:8080/source
OSATURA studio on http://127.0.0.1:8080  (Ctrl-C to stop; saves go to demo.json)
UI settings (layout, shortcuts) in /home/you/.config/osatura/ui.json      (your own home directory)
```

Abre la dirección en un navegador. El proyecto se valida al iniciar; un archivo que no supera `check` se rechaza con el
mismo mensaje. Los campos desconocidos generan un aviso (`warning: unknown field bogus (ignored)`).

| Indicador | Valor por defecto | Significado |
|---|---|---|
| `--port N` | 8080 | Puerto TCP. Un puerto ocupado da `cannot listen on 127.0.0.1:8861: Address already in use (os error 98)`; uno incorrecto, `cannot listen on 127.0.0.1:99999: invalid port value`. |
| `--bind ADDR` | `127.0.0.1` | Interfaz en la que escuchar. Cualquier dirección que no sea de loopback **requiere `--collab-token`**. |
| `--ui-config F` | `$OSATURA_UI_CONFIG`, si no `$XDG_CONFIG_HOME/osatura/ui.json`, si no en Windows `%APPDATA%\OSATURA\ui.json` (antes de alpha.3 no había ninguna: los ajustes se conservaban solo durante la sesión), si no `~/.config/osatura/ui.json` | Dónde se guardan la disposición, los atajos, los comandos recientes y los ajustes de overlays. Es un archivo por usuario, nunca parte del proyecto. |

La aplicación de ventana y el arranque con doble clic crean el primer proyecto en `OSATURA_DOCUMENTS` si esa variable está definida y, si no, en `Documents\OSATURA` (lo usa el paquete portátil).

| `--allow-host NAMES` | ninguno | Nombres de `Host` adicionales que aceptar (separados por comas), para el acceso bajo un nombre DNS. |
| `--public-origin URL` | ninguno | El origen simple (`https://studio.example.org`, sin ruta) que presenta un proxy inverso con TLS; entonces se aceptan su `Host` y su `Origin`. |
| `--source-url URL` | ninguno; en los binarios oficiales de la versión, el commit desde el que se compilaron (`https://github.com/chiuta/osatura/tree/<commit>`) | Dónde se puede descargar el código fuente correspondiente de *tu* compilación (AGPL sección 13); se muestra en `/source` y con `--version`. Un valor explícito siempre prevalece. |
| `--collab-token T` | ninguno | Secreto compartido, **de al menos 16 caracteres** (`openssl rand -hex 16`). Obligatorio para enlaces que no sean de loopback. |
| `--collab-log LOG` | ninguno | Persiste la sesión de edición compartida en un archivo y la reanuda tras un reinicio. |
| `--collab-nosync` | desactivado | Con `--collab-log`: no hace `fsync` tras cada edición (más rápido; un corte de corriente puede perder las últimas ediciones). Sin `--collab-log` se rechaza al iniciar (`--collab-nosync needs --collab-log`). |

**Guardado.** Las ediciones viven en memoria. El archivo de proyecto en disco cambia solo cuando pulsas **Save** (`Ctrl+S`), que escribe de forma atómica
(archivo temporal y después renombrado, nunca a través de un enlace simbólico). Esto también es cierto durante una sesión compartida (sección 8).

## 2. Modelo de seguridad (léelo antes de enlazar a una red)

El estudio está pensado para una persona en su propia máquina. Lo que lo protege:

* **Lista de hosts permitidos.** Solo se responde a las peticiones cuya cabecera `Host` sea uno de los nombres propios del servidor; cualquier otra recibe **421**.
  Con `--allow-host` y `--public-origin` se aceptan más nombres.
* **Protección entre sitios.** Una petición con un `Origin` ajeno, o con `Sec-Fetch-Site: cross-site`, recibe **403**. Un POST cuyo cuerpo no sea
  `application/json` recibe **415** (los POST sin cuerpo, como `/api/undo`, no necesitan tipo).
* **Cabeceras de seguridad** en las respuestas: una `Content-Security-Policy` restrictiva (`default-src 'self'` y sin enmarcado), `X-Frame-Options: DENY`,
  `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `Cache-Control: no-store`.
* **Token para todo lo que salga de la máquina.** Con `--collab-token`, cada petición necesita el token como cabecera `X-Collab-Token` o en la
  cookie `osatura_token` establecida por `POST /api/login` (HttpOnly, SameSite=Strict). El token **nunca se acepta en la URL**. Los tokens erróneos se ralentizan
  por dirección de cliente (a partir del quinto fallo seguido, esa dirección recibe **429** durante 2, 4, ... hasta 30 segundos), pero el retardo no puede detener a un
  atacante que sigue intentándolo: usa un token aleatorio largo.
* **Secretos de participante.** Unirse a una sesión compartida devuelve un secreto aleatorio de 128 bits; cada llamada hecha como ese participante debe enviarlo como
  `X-Collab-Secret`. Conocer el número de un participante no basta.

Comprobé cada uno de ellos con `curl` contra un servidor en ejecución:

```
bad Host (evil.example):               421
allowed host (--allow-host):           200
cross-origin POST (Origin: evil.example): 403
POST from the --public-origin:         200
Sec-Fetch-Site: cross-site:            403
POST with Content-Type: text/plain:    415
no token, GET / or /api/state:         401
wrong X-Collab-Token:                  401
right X-Collab-Token:                  200
token in the URL (?token=...):         401
/source without a token:               200
unknown route:                         404   (GET /favicon.ico answers with a small SVG icon, without a token)
```

Rechazos al iniciar (código de salida 1, no se inicia nada):

```
refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (and still sends everything in clear text)
--collab-token must be at least 16 characters (e.g. `openssl rand -hex 16`)
--source-url must be an http:// or https:// URL without spaces or quotes
--public-origin must be a bare origin (scheme, host and optional port, no path), e.g. https://studio.example.org
```

**Límites honestos.** El tráfico es HTTP plano: el token y todo lo demás viajan en texto claro, así que pon el estudio detrás de un proxy inverso con TLS
(`--public-origin`) si es accesible para cualquier otra persona. El `Set-Cookie` del token recibe el **indicador `Secure`** cuando la petición llega bajo un nombre `--public-origin` con `https://` (o con un `Origin` https / `X-Forwarded-Proto: https`); el HTTP plano en loopback nunca lo recibe,
y detrás de un proxy todos los clientes parecen tener la dirección del proxy, de modo que el retardo limita a todos a la vez. `/source` está abierto sin token
(la AGPL lo exige). No hay cuentas de usuario: quien tenga el token puede cambiar el proyecto y escribir la ruta de guardado.

### La oferta de código fuente de la AGPL

El estudio es software AGPL-3.0-or-later: si lo modificas y permites que otras personas lo usen a través de una red, debes ofrecerles el código fuente de tu
versión. `/source` es esa oferta; establece `--source-url` con el lugar donde está el código fuente de tu compilación. Sin ello (una compilación que hayas hecho tú mismo con un simple `cargo build` no tiene ninguna), la página lo indica y nombra al mantenedor. Los binarios de la versión compilados por `.github/workflows/release.yml` llevan la dirección de su propio commit (`OSATURA_SOURCE_URL` en tiempo de compilación; un fork que compile con su propio flujo de trabajo apunta a sí mismo).

## 3. Un recorrido por la interfaz

### Barra de herramientas

Deshacer, Rehacer, **Save**; casillas `bones` (dibujar el esqueleto), `auto-key`, `grid` (separación en unidades del lienzo, 0 = desactivada), `snap` (a otras capas y al lienzo),
`whole groups` (un clic en parte de un grupo selecciona el grupo de nivel superior), `points` (muestra anclas, manejadores, puntos de cage/lattice/pin y vértices de malla de las
capas seleccionadas); un selector `box` para lo que selecciona un cuadro arrastrado (`touches`, `contains`, `bounds touch`; empieza en `touches`); **Lock**; y los botones **Overlays**, **Suggest**, **Layout**,
**Commands** (paleta) y **Keys** (editor de atajos). Una segunda fila tiene tu nombre y **Collab**, y los botones de exportación **GIF**, **HTML player** y **Lottie**,
que descargan lo que escribiría `osatura export` (el reproductor HTML incluye el sonido mezclado cuando hay sonido y `ffmpeg` puede crear el mp3). El botón **Simple view** oculta las casillas de edición, Overlays, Suggest y el botón de la paleta de comandos, de modo que solo quedan deshacer, guardar, paneles y exportaciones; la elección se recuerda en este navegador (o ventana) y la barra completa vuelve con **Full view**.

### Paneles y disposiciones

La ventana es un dock de paneles, cada uno con un id estable: `stage` (el lienzo), `layers`, `selection`, `bones`, `live` (entrada en vivo), `face` (rostro por webcam), `json`
(todo el proyecto como JSON editable con un botón *Apply JSON*) y `timeline`. Existen cuatro disposiciones predefinidas:

```
animate  stage over timeline on the left, all panels as tabs on the right
rig      stage with a folded timeline; layers+bones and selection+json on the right (live and face hidden)
compact  stage over a tabbed timeline and panels
live     stage and timeline on the left; live, face, bones and selection on the right (layers and json hidden)
```

Cambia de disposición desde el menú **Layout** o la paleta; arrastra las pestañas para volver a acoplar o flotar un panel; "reset layout" vuelve a la predeterminada.
La disposición se guarda en la configuración de la interfaz, por usuario. `osatura ui layout presets|show NAME|validate FILE.json` imprime y valida los mismos datos
(una disposición debe mantener accesibles todos los paneles; el servidor rechaza una incompleta con 400).

### Panel de capas

![El panel de capas con una capa bloqueada (sun), una capa oculta (hills) y brazo y pierna seleccionados a la vez.](../../img/studio-layers.png)

Cada fila tiene dos casillas: **visible** y **lock** (se guarda *con el proyecto*; los clics y los cuadros omiten las capas bloqueadas, y rechazan las ediciones).
La captura muestra `sun` bloqueada, `hills` oculta y dos capas seleccionadas con Shift+clic.

### Panel de selección

Enuncia las reglas en la propia página: *Un clic elige lo que hay más arriba (se omiten las capas bloqueadas y ocultas); Shift+clic alterna; arrastrar sobre un espacio vacío
dibuja un cuadro (mantén Alt para dibujar un lazo libre, Shift añade). Arrastra el cuadro para mover, un cuadrado para escalar (Shift mantiene las proporciones, Alt escala desde el
centro), el pomo redondo para rotar (Shift ajusta a 15 grados). Mantén Ctrl mientras arrastras para desactivar el ajuste. Las teclas de flecha desplazan 1 unidad (Shift: 10),
Delete elimina capas, Esc cancela un arrastre o borra la selección.*

### Línea de tiempo: dope sheet y gráfico

![El dope sheet con una selección en cuadro de tres claves en el fotograma 12 (las claves se dibujan rellenas, las demás huecas).](../../img/studio-dopesheet.png)

![La vista del gráfico de curvas: tres curvas bone_angle (arm_up, arm_low, hand) en colores distintos con tres claves seleccionadas en el fotograma 12 que muestran sus manejadores de tangente; la línea de estado dice "3 keys selected | frames 12".](../../img/studio-graph.png)

`Graph` (o `Alt+V`) alterna entre el dope sheet y el gráfico de curvas. Los controles de la línea de tiempo: `Play`, `|<` (primer fotograma del rango), `loop`, `range` con
`Set in` / `Set out` (reproduce solo entre la entrada y la salida), `snap` (redondea las claves movidas a fotogramas enteros), `on collision` (`replace`, `merge`, `refuse` cuando las claves movidas caen sobre
otras existentes), `Copy` / `Paste` / `Duplicate` / `Delete`, `interp` + `Apply` (establece el suavizado de las claves seleccionadas), `Auto-smooth` (hornea tangentes sin sobreoscilación en
las claves seleccionadas), `ripple` N con `Insert` / `Remove` (y `grow project` para cambiar con ellos la duración del proyecto), zoom `-` `+` `Fit`, y `shy` (muestra los canales marcados como shy).

Ratón: haz clic en una clave para seleccionarla, Shift+clic para añadir, arrastra un cuadro para seleccionar, arrastra claves para moverlas; **Alt+arrastrar o el botón central desplaza**, **Ctrl+rueda hace zoom**,
**Shift+rueda desplaza** en vertical; en el gráfico **Shift restringe el arrastre de la clave a un eje**. La tolerancia de ajuste al lienzo es por defecto de 6 unidades del lienzo.
Toda edición de claves pasa por las mismas operaciones validadas que `osatura keyedit` (véase [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)), de modo que es un único paso de deshacer y los canales bloqueados la rechazan.

### Overlays: onion skin, trayectorias, proxies

![Onion skin y trayectorias de movimiento activados para el brazo en el fotograma 14, con el panel Overlays abierto.](../../img/studio-overlays.png)

`Overlays` (`Alt+7`) contiene onion skin (`Alt+N`: fotogramas anteriores/posteriores en tonos rojo/verde, paso fijo o fotogramas clave, modo contorno, atenuación, solo capas seleccionadas),
trayectorias de movimiento (`Alt+M`: trayectos de hueso, de pivote de capa o de punto de control con marcas por fotograma) y **proxies de scrub** (`Alt+Q`: imágenes de calidad reducida mientras arrastras el
cabezal de reproducción, con una caché de imágenes y una copia opcional de nivel de detalle). El panel informa, por ejemplo, de `picture cache: 2 frames, 17 KiB, 0 hits / 7 misses, emptied 4x by edits`.
Los ajustes se guardan en la configuración de la interfaz y sobreviven a un reinicio. Los overlays nunca forman parte de las exportaciones.

### Paleta de comandos y atajos

![La paleta de comandos abierta con Ctrl+K y filtrada por la palabra "toggle", con la lista de comandos coincidentes y sus indicaciones de atajo.](../../img/studio-palette.png)

`Ctrl+K` (o `Ctrl+Shift+P`) abre la paleta: escribe para filtrar (difuso, con las letras coincidentes resaltadas), las flechas se mueven, Enter ejecuta, Esc cierra. Sin escribir nada
muestra primero los comandos recientes. Hay **84 comandos**; `osatura ui commands` los enumera y `osatura ui search onion` los ordena exactamente como lo hace la paleta.

![El editor de atajos de teclado abierto con Ctrl+coma, que enumera los comandos por categoría con sus combinaciones de teclas.](../../img/studio-shortcuts.png)

`Ctrl+,` (o **Keys**) abre el editor de atajos: haz clic en una combinación y pulsa la nueva combinación de teclas. Un conflicto nombra el otro comando y no cambia nada hasta que
confirmes "Take it over". Las combinaciones reservadas del navegador se rechazan: `Ctrl+W`, `Ctrl+T`, `Ctrl+N`, `Ctrl+Q`, `Ctrl+R`, `Ctrl+Tab`, `Ctrl+Shift+Tab`, `Ctrl+Shift+W`, `Ctrl+Shift+N`, `F5`, `F11`, `F12`,
`Alt+F4`. La reasignación se guarda en la configuración de la interfaz y sobrevive a una recarga. El servidor rechaza con 400 un mapa con conflictos, un comando desconocido, una disposición incompleta o un cuerpo que no sea un objeto.

### Suggest

![El panel Suggest que muestra tres candidatos de pose intermedia ordenados (linear, pca, manifold) entre los fotogramas 12 y 24, cada uno con una puntuación y un botón Apply.](../../img/studio-suggest.png)

`Suggest` (`Alt+8`) propone poses aprendidas de las propias poses clave del proyecto: una intermedia entre dos fotogramas, la compleción de una pose parcialmente fijada o "lo que suele
venir a continuación". En el proyecto de demostración, entre los fotogramas 12 y 24 ordenó `linear` 1,00, `pca` 1,00 y `manifold` 0,99, construidos a partir de 5 poses en 2 ejes (99 % de la varianza).
`Apply` escribe la pose como claves, en un paso de deshacer. El mismo motor es `osatura pose-suggest` y `/api/suggest`. Son métodos clásicos (mezcla lineal, PCA, una variedad
con núcleo, una pequeña red opcional que entrenas tú); con un puñado de poses, los tres coinciden.

### Paneles de entrada en vivo y de rostro por webcam

* **Live input** (`Alt+4`): el ratón sobre la imagen, las teclas, el gamepad y Web MIDI se asignan a propiedades mediante una tabla; `Record` hornea una interpretación en claves en un solo paso de
  deshacer. Web MIDI necesita un navegador que lo admita (Chromium lo hace).
* **Webcam face** (`Alt+5`): los puntos de referencia alimentan la misma tabla de asignación (fuente `face`, ids como `mouth_open`, `eye_r_open`, `head_yaw`). Los botones son
  *Start camera*, *Synthetic face*, *Synthetic face (detector)*, *Calibrate (hold neutral)* y *Face example mapping*. La ruta de la cámara necesita una función proveedora de puntos de referencia
  que proporciones tú (`window.osaturaLandmarkProvider`, véase el comentario en el código fuente de la página); el vídeo nunca sale de la página y solo los puntos de referencia van al servidor local.
  Los dos modos sintéticos funcionan de entrada. *Synthetic face (detector)* renderiza la cabeza sintética en el servidor y la vuelve a encontrar con el detector integrado,
  que fue **entrenado solo con rostros sintéticos** (cabezas estilizadas, no fotografías); véase la sección de rostro de [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).

### Demo guiada

El botón **Demo** (comando `demo.panel`) abre una demo guiada de 13 pasos que construye a la persona caminando de `osatura demo-person`: decorado, cuerpo, cabeza y rostro, brazos con cinco dedos por mano, el esqueleto, las piernas caminando, los brazos balanceándose, tronco y cabeza, un parpadeo y una sonrisa, el suelo en movimiento, una comprobación y la exportación. Cada paso carga esa etapa del proyecto en el editor con una única operación de reemplazo total, de modo que puedes reproducirla, hacer scrub, seleccionar huesos y capas y leer la línea de tiempo. Las etapas son cortes del proyecto terminado; los comandos y su salida que se muestran en el panel se ejecutaron con el binario real.

* **No se escribe nada en el disco.** Guardar (y Ctrl+S) está desactivado mientras la demo está cargada. La única vía de escritura del studio, `POST /api/save`, la demo nunca la llama.
* **Tu proyecto vuelve.** *Restore my project* devuelve el proyecto que estaba abierto antes (una única operación de reemplazo total, un paso de Deshacer). Si cierras la pestaña o recargas mientras la demo está cargada, la siguiente carga de la página detecta que el servidor aún contiene la demo, mantiene Guardar desactivado y vuelve a ofrecer *Restore my project*; el original se conserva para ello en el almacenamiento local del navegador.
* La demo se niega a iniciarse dentro de una sesión de colaboración, porque reemplaza todo el proyecto.
* Regenera las etapas con `node scripts/build-studio-tour.mjs --bin target/release/osatura` y luego recompila la CLI (la página está incrustada en el binario).

## 4. Atajos de teclado (valores por defecto)

Esta es la salida de `osatura ui shortcuts` para una configuración nueva: **72 asignaciones, 84 comandos, 0 conflictos**. Los atajos pertenecen a un **contexto**:
`global` (en todas partes), `canvas` (foco en la imagen) y `timeline` (foco en la línea de tiempo). Los campos de texto conservan sus propias teclas.

### Global

| Atajo | Comando | Atajo | Comando |
|---|---|---|---|
| `Space` | reproducir / pausar (`playback.toggle`) | `Alt+B` | alternar huesos |
| `Home` / `End` | primer / último fotograma | `Alt+G` | alternar grupos completos |
| `,` / `.` | fotograma anterior / siguiente | `Alt+K` | alternar auto-key |
| `Shift+,` / `Shift+.` | retroceder / avanzar 10 fotogramas | `Alt+M` | alternar trayectorias de movimiento |
| `[` / `]` | establecer entrada / salida del rango | `Alt+N` | alternar onion skin |
| `Alt+R` | alternar rango de reproducción | `Alt+O` | alternar reproducción en bucle |
| `Alt+1` | alternar panel de capas | `Alt+P` | alternar puntos de control |
| `Alt+2` | alternar panel de selección | `Alt+Q` | alternar proxies de scrub |
| `Alt+3` | alternar panel de huesos | `Alt+S` | alternar ajuste del lienzo |
| `Alt+4` | alternar panel de entrada en vivo | `Alt+T` | alternar panel de línea de tiempo |
| `Alt+5` | alternar panel de rostro por webcam | `Alt+V` | alternar dope sheet / gráfico |
| `Alt+6` | alternar panel JSON | `Alt+Shift+S` | alternar ajuste de la línea de tiempo |
| `Alt+7` | panel Overlays | `Alt+=` / `Alt+-` | acercar / alejar la línea de tiempo |
| `Alt+8` | panel Suggest | `Ctrl+S` | guardar |
| `Ctrl+K` / `Ctrl+Shift+P` | paleta de comandos | `Ctrl+,` | atajos de teclado |
| `Ctrl+Z` | deshacer | `Ctrl+Y` / `Ctrl+Shift+Z` | rehacer |

### Lienzo

| Atajo | Comando |
|---|---|
| `Arrow keys` | desplaza la selección 1 unidad (`Shift`: 10 unidades) |
| `Delete` / `Backspace` | elimina la selección |
| `Escape` | borra la selección |
| `Ctrl+A` | selecciona todo |
| `Alt+L` | bloquea / desbloquea las capas seleccionadas |
| `I` | añade una clave en el cabezal de reproducción (`anim.add_key`) |

### Línea de tiempo

| Atajo | Comando |
|---|---|
| `ArrowLeft` / `ArrowRight` | fotograma anterior / siguiente (`Shift`: 10) |
| `-` / `=` | alejar / acercar |
| `F` | ajustar el zoom a la vista |
| `Delete` / `Backspace` | elimina las claves seleccionadas |
| `Escape` | deselecciona las claves |
| `Ctrl+A` | selecciona todas las claves |
| `Ctrl+C` / `Ctrl+V` / `Ctrl+D` | copia / pega / duplica claves |
| `Alt+ArrowLeft` / `Alt+ArrowRight` | mueve las claves seleccionadas un fotograma antes / después |
| `Alt+ArrowUp` / `Alt+ArrowDown` | sube / baja las claves seleccionadas (su valor) |
| `I` | añade una clave en el cabezal de reproducción |

Modificadores en el lienzo: **Shift** restringe (bloqueo de eje al mover, pasos de 15 grados al rotar, proporcional al escalar, la punta de un hueso entonces solo rota,
y Shift+clic alterna la selección mientras que un cuadro añade a ella); **Alt** escala desde el centro (y Alt+arrastrar sobre un espacio vacío dibuja un lazo); **Ctrl** (Cmd en macOS)
desactiva el ajuste.

## 5. Selección y gizmos

Un clic elige lo que hay más arriba bajo el puntero: una capa, un hueso (su cápsula, algo más ancha que la línea dibujada) o, con `points` activado, un punto de control. Las capas bloqueadas y ocultas
se omiten, de modo que un clic atraviesa hasta lo que queda debajo. Un cuadro selecciona según el modo del selector `box`: `touches` (todo lo que el cuadro solape), `contains` (solo lo que queda completamente dentro) o `bounds touch` (las cajas delimitadoras se solapan); Alt+arrastrar dibuja un lazo en su lugar.

Para una capa, la caja de transformación sigue los ejes propios de la capa: 8 manejadores escalan (los bordes un eje, las esquinas ambos; arrastrar más allá del borde opuesto voltea con una escala negativa),
el pomo redondo rota alrededor del centro y arrastrar por dentro mueve. Para un hueso, el círculo de la cabeza mueve el hueso y la punta lo rota (y lo estira salvo que se mantenga Shift).
Para los puntos de control (anclas, manejadores de Bézier, puntos de deformador, vértices de malla) el mismo gizmo mueve los puntos seleccionados. El ajuste abarca una cuadrícula (desactivada por defecto), los bordes
y centros de otras capas y los bordes del lienzo, dentro de 6 unidades del lienzo. **Un arrastre es un paso de deshacer** (una operación por lotes).

## 6. Auto-key: qué se pone exactamente en clave

Que una edición en el lienzo cambie un valor *estático* o cree *claves* se decide así. Lo comprobé con la API HTTP en el proyecto de demostración (se informa del `mode` de cada edición):

| Situación | Resultado |
|---|---|
| Edición de capa, `auto-key` desactivado, la propiedad no tiene canal con claves | cambio **estático** de la transformación de la capa (no se crea ningún canal) |
| Edición de capa, `auto-key` activado | las propiedades editadas se **ponen en clave** en el fotograma actual (se crearon canales nuevos `layer_x` y `layer_y`: de 14 a 16 canales) |
| Propiedad de capa o de punto que **ya tiene un canal con claves**, `auto-key` desactivado | se **pone en clave** igualmente en el fotograma actual (una edición en el fotograma 9 añadió una segunda clave a los canales desde el fotograma 5) |
| Edición de **hueso** | siempre se **pone en clave**, sea cual sea el ajuste de `auto-key` (se crearon canales de desplazamiento de hueso: de 16 a 18) |

Así, una propiedad animada nunca puede sobrescribirse en silencio con un valor estático, y la pose de un hueso nunca se "pierde" por olvidar el conmutador. Activa `auto-key` (`Alt+K`)
cuando quieras animar una propiedad que todavía es estática. La tecla `I` (`anim.add_key`, "Add key at playhead") añade una clave en el fotograma actual.

## 7. Deshacer y rehacer

Deshacer y rehacer (`Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`) actúan sobre operaciones completas: un arrastre de gizmo, un desplazamiento, una edición de claves y un Apply son cada uno un paso. Sin sesión compartida son
lineales. **En una sesión compartida, deshacer es por usuario**: revierte solo tu última edición y deja en su sitio las ediciones de los demás (sección 8). El historial de deshacer no se guarda con el
proyecto y se pierde cuando el servidor se detiene.

## 8. Edición compartida (colaboración)

### Recorrido: dos personas, un proyecto

![Dos usuarios de navegador en una sesión: la barra de herramientas muestra dos chips de participante, Ana (tú) en el fotograma 14 y Bo en el 0, y el cuadro de selección y el cursor de Bo se dibujan en el lienzo alrededor de la capa de la pierna.](../../img/studio-collab.png)

1. Inicia el estudio. Para personas en otras máquinas debes añadir un token y enlazar a una dirección de red:
   `osatura studio demo.json --bind 0.0.0.0 --port 8080 --collab-token $(openssl rand -hex 16) --collab-log session.log`,
   preferiblemente detrás de un proxy con TLS (sección 2). En una sola máquina no hace falta ningún token.
2. En el primer navegador, escribe tu nombre junto a **Collab** ("Ana") y pulsa **Collab**. El botón pasa a ser **Leave** y aparece un chip para ti. En el segundo navegador
   escribe "Bo" y únete. Cada chip muestra el fotograma actual del participante (`Ana (you) @14`, `Bo @0`).
3. Selecciona cosas. La otra persona ve tu cuadro de selección y tu cursor en tu color con tu nombre (Bo seleccionó `leg` y movió el puntero; Ana ve un cuadro morado
   con el nombre "Bo" y un punto junto a `leg` en la lista de capas).
4. Edita a la vez. En la prueba: Ana desplazó la capa del brazo 10 unidades (`Shift+ArrowRight`); Bo vio aparecer `[10,0]` en `/api/project`. Después Bo ocultó la capa `sun`, y Ana lo vio
   de inmediato.
5. Ana pulsó `Ctrl+Z`. **Solo se deshizo el desplazamiento de Ana** (el brazo volvió a `[0,0]`); el sol oculto de Bo siguió oculto.
6. Las ediciones hechas mientras alguien trabajaba desde una vista más antigua se fusionan, no se rechazan: ejecuté la simulación propia del proyecto (`osatura collab-sim`, 6 réplicas, 600 ediciones concurrentes
   aleatorias, 12 órdenes de entrega distintos) y todas las réplicas acabaron idénticas y el proyecto fusionado compiló.
7. Pulsa **Save** (o `Ctrl+S`) en cualquiera de los navegadores para escribir el estado compartido en el archivo de proyecto; no se escribe nada en el archivo antes de eso.

Solo se probó Chromium, con dos páginas en una máquina. No probé máquinas distintas, enlaces lentos ni más de dos navegadores reales.

### Persistencia: `--collab-log`

```
$ osatura studio demo.json --collab-log session.log
...
$ # (an edit is made in a session, the server is stopped and started again)
collaboration log session.log: resumed at index 1 (1 envelopes after the checkpoint)
```

El registro es un archivo de líneas (modo `0600`, solo el propietario) que contiene un punto de control encadenado por hash de **todo el proyecto** (unos 178 KB para la demostración) más una línea por cada edición aceptada. Las adiciones se
sincronizan con `fsync` en cada petición salvo con `--collab-nosync`. Tras un reinicio se reanuda el estado de la sesión (cambié el nombre del proyecto en una sesión; tras el reinicio `/api/project` seguía teniendo el
nombre nuevo) mientras que el archivo en disco no cambió (`OSATURA demo`) hasta Save. **Los participantes y el historial de deshacer no se persisten**: las páginas se unen de nuevo con ids nuevos. Un registro dañado o un registro
de otro proyecto detiene el servidor con un error en lugar de sobrescribirse. Un fallo durante una adición pierde como máximo una última línea cortada.

**El registro contiene todo tu proyecto en texto claro; protégelo como al propio proyecto.**

## 9. Referencia de la API HTTP

La página del estudio usa estas rutas y tú también puedes usarlas (scripts, pruebas). Todos los cuerpos y respuestas son JSON salvo indicación contraria; los errores son `{"ok": false, "errors": ["..."]}` con estado 400 (o 401, 403,
404, 415, 421, 429, 503 según se describe en la sección 2). Las respuestas correctas llevan `"ok": true` donde se indica. `state` en adelante designa el documento devuelto por `GET /api/state`:
`{bones:[{name,rest_angle}], channels:[...], fps, frames, width, height, can_undo, can_redo, rev, ok}`; `rev` aumenta con cada cambio. Todas las rutas están sujetas a las comprobaciones de la sección 2.

### Lectura

| Ruta | Parámetros | Respuesta |
|---|---|---|
| `GET /` | | la página del estudio (HTML) |
| `GET /source` | | la página de oferta del código fuente AGPL; se abre sin token |
| `GET /api/state` | | `state` |
| `GET /api/project` | | el JSON completo del proyecto |
| `GET /api/layers` | | `{ok, layers:[{path, depth, kind, visible, locked, locked_eff, reference, ptr}]}` (`locked_eff` también tiene en cuenta los grupos bloqueados que están por encima) |
| `GET /frame/N.svg` | `bones=1` dibuja el esqueleto | un fotograma como SVG (N puede ser fraccionario); cabeceras `X-Cache`, `X-Render-Ms` |
| `GET /api/frame` | `frame`, `proxy` 0-4 (0 = calidad completa, 1-4 = proxies de scrub), `kind` = `vector` (por defecto) \| `raster` \| `png`, `bones=1`, `lod=1&screen=WxH&lodpx=P` (dibuja desde una copia de detalle reducido con un error de hasta P píxeles, 0-16) | la imagen; cabeceras `X-Cache` (hit/miss), `X-Render-Ms`, `X-Lod-Level`, `X-Lod-Bound-Px`, `X-Proxy-Level`. Un fotograma fuera de rango: `{"errors":["frame 999 out of range"]}`; un tipo incorrecto: `bad kind 'x' (vector, raster, png)` |
| `GET /api/cache` | | `{ok, revision, entries, bytes, hits, misses, invalidations, lod:{builds, engines, invalidations, last_build_ms, levels, unavailable}}` |
| `GET /api/overlays` | `frame`, `sel` (nombres de capa separados por comas) | `{ok, frames, settings, ...}` más los fantasmas y el SVG del fotograma cuando se indica uno |
| `GET /api/sounds` | | `{ok, total, sounds:[{name, start, gain, duration, peaks, error?}]}` picos de la forma de onda tras el recorte, los fundidos y la ganancia |
| `GET /api/vcs/log` | `limit` (máx. 1000), `rev` | `{ok, branch, commits:[{hash, author, time, message, parents}]}`; `{"ok":false,"errors":["no .osatura-vcs repository found ..."]}` sin repositorio (véase `osatura vcs`) |
| `GET /api/vcs/diff` | `a` (por defecto `HEAD`), `b` | `{ok, changes}` |
| `GET /api/timeline` | | `{ok, fps, frames, groups, rows, tracks}` el modelo del dope sheet |
| `GET /api/graph` | `ch` (índices de canal separados por comas), `f0`, `f1`, `n` (muestras, 2-2000, por defecto 200) | `{ok, channels, f0, f1, range, skipped}` muestras de curva y tiradores de tangente |
| `GET /api/live` | | `{ok, mapping:{params,triggers}, recorded, recording}` |
| `GET /api/ui` | | `{ok, layout, presets, preset_names, default_layout, shortcuts, commands, conflicts, reserved, overlays, recents, panels, persisted, path, warning, ...}` |
| `GET /api/ui/search` | `q`, `limit` (1-200, por defecto 50) | `{hits:[{id, title, category, description, bindings, score, positions}]}` la clasificación de la paleta |
| `GET /export.gif`, `/export.html`, `/export.html.nosound`, `/export.lottie.json` | | el archivo, como descarga (`animation.gif`, `player.html`, `animation.json`) |

### Edición

| Ruta | Cuerpo | Respuesta |
|---|---|---|
| `POST /api/op` | `{"op":"set","path":"/fps","value":30}` — `op` es `set`, `insert` (ruta de array, `.../-` añade al final), `remove` o `batch` (`{"op":"batch","ops":[...]}`, un solo paso de deshacer). Las rutas son JSON Pointers dentro del proyecto. | `state`. Un resultado no válido se rechaza con `{"errors":["invalid project: invalid value: integer `-3`, expected u32"]}` y no cambia nada |
| `POST /api/undo`, `POST /api/redo` | ninguno | `state` (por usuario en una sesión compartida) |
| `POST /api/key` | `{"target","prop","frame","value","ease"?}` (`ease` como `{"type":"smooth"}`; lineal por defecto) | `state` |
| `POST /api/delete_key` | `{"target","prop","frame"}` | `state` |
| `POST /api/save` | ninguno | `{"ok":true,"saved":"<path>"}`; escribe de forma atómica. Solo para el anfitrión cuando el estudio se ejecuta con `--collab-host` (si no, **403**) |
| `POST /api/suggest` | `{"kind":"between"\|"complete"\|"next","a","b","t","frame","constraints","free","top","snap","prior","apply":RANK, "library"?, "model"?}` | `{ok, suggestions}`; con `apply` también el `state` de la edición. Una petición sin `kind` da `bad request: missing field `kind`` |

### Selección y gizmos (sin estado: la página guarda la selección)

Un **target** es `{"kind":"layer","path":"arm"}`, `{"kind":"bone","name":"hand"}` o `{"kind":"point","layer":"...","point":...}`.

| Ruta | Cuerpo | Respuesta |
|---|---|---|
| `POST /api/pick` | `{"frame","x","y","options"?}` (`options`: `tolerance`, `bone_radius`, `point_radius`, `layers`, `bones`, `point_layers`, `locked`, `locked_bones`) | `{ok, hit, hits:[{distance, target}]}` el superior primero |
| `POST /api/marquee` | `{"frame","rect":[x0,y0,x1,y1]` o `"lasso":[[x,y],...]` (de 3 a 5000 puntos)`,"mode":"bbox"\|"contain"\|"intersect","options"?}` | `{ok, targets}` |
| `POST /api/select` | `{"frame","selection":[...],"targets":[...],"mode":"replace"\|"add"\|"toggle"\|"subtract"}` | `{ok, selection, bounds, gizmo:{corners, handles}, points}` |
| `POST /api/gizmo` | `{"frame","selection","handle","start":[x,y],"delta":[dx,dy],"modifiers":{"constrain","center","no_snap"},"snap":{"grid","objects","canvas","tolerance"},"auto_key","locked","pivot","commit":bool}` — `handle` es `{"type":"move"}`, `rotate`, `{"type":"scale","at":"nw"}`, `bone_head` o `bone_tip` | sin `commit`: `{ok, drag:{changed, edits, guides, matrix, summary, box_after, outline}}` (una vista previa); con `commit:true`: `state` más `drag` y `selection`, un solo paso de deshacer |
| `POST /api/nudge` | `{"frame","selection","delta":[dx,dy],"auto_key","locked"}` | `state` + `drag` (ediciones con `mode` de `static` o `key`) + `selection` |
| `POST /api/delete` | `{"frame","selection"}` | `state` + `selection`; `{"errors":["nothing selected"]}` cuando está vacía |

### Línea de tiempo

| Ruta | Cuerpo | Respuesta |
|---|---|---|
| `POST /api/timeline/select` | `{"selection":[{ch,frame}...],"mode":"replace"\|"add"\|"toggle"\|"subtract","all":true` o `"channel":N` o `"group":N` o `"box":{f0,f1,row0,row1}`, `"include_locked"}` | `{ok, selection:[{ch,frame}...]}` |
| `POST /api/timeline/op` | `{"op":{...},"preview":bool}` — `op` es una de `move_keys`, `scale_keys`, `copy`, `paste`, `duplicate`, `delete`, `set_ease`, `set_value`, `add_key`, `ripple`, `offset`, `extrapolate`, `make_loop`, `handle`, `smooth`, `flags` (un nombre desconocido las enumera) | con `preview:true`: `{ok, preview:true, plan}` sin cambiar nada; en caso contrario `state` + `plan:{changed, notes, op, clip}` |

Por ejemplo, `{"preview":true,"op":{"op":"delete","keys":[{"ch":0,"frame":12}]}}` devuelve el plan (`changed: 1`) y deja el proyecto intacto.

### Superposiciones, directo, rostro, interfaz

| Ruta | Cuerpo | Respuesta |
|---|---|---|
| `POST /api/overlays` | `{"settings":{"onion":{"enabled":true,"before":3}},"frame","selection":[names],"reset":bool}` | `{ok, frames, ghosts, settings, svg}`; los ajustes se guardan en la configuración de la interfaz |
| `POST /api/trails` | `{"targets":[{"kind":"bone","name":"hand"}],"from","to"` o `"frame","before","after"`, `"smooth","svg":bool,"hit":{x,y,radius}}` | `{ok, from, to, trails:[{curve,frames,keys,...}], svg?, hit?, path_hit?}` |
| `POST /api/live/mapping` | `{"mapping":{params,triggers},"library"?}` (mismo formato que `live-map`) | estado del directo |
| `POST /api/live/push` | `{"events":[{source,id,value,time}],"midi":[raw...]}` | estado del directo con el efecto de los eventos |
| `POST /api/live/record` | `{"on":true}` inicia; `{"on":false,"tol":0.01}` detiene y **incorpora la interpretación al proyecto** como un solo paso de deshacer | estado / `state` + resumen `live` |
| `POST /api/face/config` | `{"config"?,"calibration"?}` | `{ok}` |
| `POST /api/face/calibrate` | `{"on":true}` / `{"on":false}` | `{ok, frames, calibration}` |
| `POST /api/face/push` | `{"frames":[...landmarks...],"layout"?,"size"?}` (como máximo 10000 fotogramas por envío) | estado del directo |
| `POST /api/face/detect` | `{"image":"<base64 PNG/JPEG or data: URI>","track":bool,"push":bool,"t"}` | `{ok, found, layout:"osatura23", size, points, box, note:"detector trained on synthetic faces only"}` |
| `POST /api/face/synth` | `{"t","size":32-512,"seed"}` | `{ok, image:"data:image/png;base64,...", layout, size, points}` un fotograma de la cabeza sintética con sus puntos de referencia exactos |
| `POST /api/ui` | cualquiera de `{"layout":{...}\|null,"shortcuts":{...}\|null,"recents":[...],"overlays":{...},"reset":"layout"\|"shortcuts"\|"overlays"\|"all"}` | documento de `GET /api/ui`; los campos desconocidos (`unknown field 'bogus'`), los conflictos y los diseños incompletos se rechazan y no cambia nada |
| `POST /api/login` | `{"token":"..."}` | `{ok:true}` y `Set-Cookie: osatura_token=...; Path=/; HttpOnly; SameSite=Strict`; si es incorrecto: 401 (429 a partir del 5.º fallo); sin un token configurado `{"ok":true,"token_required":false}` |

### Colaboración

Las llamadas hechas como participante envían `X-Collab-Replica: <id>` y `X-Collab-Secret: <secret>` (ambos devueltos por join). Un id de réplica con el secreto incorrecto o ausente recibe **403**.

| Ruta | Cuerpo / consulta | Respuesta |
|---|---|---|
| `POST /api/collab/join` | `{"name":"Ana","mode":"thin"\|"replica"}` | `{ok, replica, secret, colour, name, index, mode, clock, users, presence, doc}` (`snapshot` en modo replica). La primera unión inicia la sesión a partir del documento del editor |
| `POST /api/collab/leave` | `{"replica"}` | `{ok, left}` |
| `GET /api/collab/state` | `replica`, `doc=1`, `snapshot=1` | `state` + `collab_active` (+ users, presence) |
| `GET /api/collab/log` | `since=N` | `{ok, index, entries:[{index, by, envelope}]}` |
| `POST /api/collab/ops` | `{"replica","ops":[...]` o `"op":{...}`, `"base"?}` o `{"replica","envelopes":[...]}` | `state` + `applied`, `index`. Participante desconocido: `unknown participant N: join first`; lista vacía: `no ops`; las ediciones de proyecto no válidas se rechazan |
| `POST /api/collab/undo`, `.../redo` | `{"replica"}` | `state` + `collab_can_undo`, `collab_can_redo` (solo tus propias ediciones) |
| `POST /api/collab/presence` | `{"replica","n","frame","cursor":[x,y],"selection":[names up to 64]}` | `{ok}` |
| `POST /api/collab/compact` | `{}` | resumen de la compactación (`ok`, `logged`). Solo para el anfitrión cuando el estudio se ejecuta con `--collab-host` (si no, **403**) |
| `POST /api/collab/revoke` | `{"names":["ana","bob"]}` (de 1 a 64 nombres) | `{ok, revoked, revoked_total}`: los tokens de esos nombres se rechazan a partir de ese momento y los participantes que se unieron con ellos son expulsados. Requiere tokens por participante (`--collab-secret-file`) y un token de anfitrión (`--collab-host`), o bien el `--collab-token` compartido; si no, **403**. Un flujo de eventos que ya esté abierto termina cuando su cliente se desconecta |
| `GET /api/collab/events` | `since=N` o `Last-Event-ID` | un flujo de **server-sent events**: reproducción desde `since`, y después usuarios, presencia y cada evento posterior; `resync` si el cliente va demasiado retrasado; como máximo 64 flujos (`503 too many event streams`) |

En una sesión activa, las rutas de edición ordinarias (`/api/op`, `/api/undo`, `/api/redo`, y las propias ediciones del estudio mediante nudge, key o gizmo) también se registran para los demás participantes.

## 10. Solución de problemas

| Síntoma | Causa y solución |
|---|---|
| El navegador muestra 421 o "misdirected request" | Abriste el estudio con un nombre que no conoce (un nombre DNS, una dirección de la LAN). Añádelo con `--allow-host name` o abre `http://127.0.0.1:PORT/`. |
| 403 en cada petición detrás de un proxy | El proxy modifica `Origin`/`Host`. Indica el origen público: `--public-origin https://studio.example.org`. |
| 401 | Se ha establecido `--collab-token`: abre la página e introduce el token una vez, o envía `X-Collab-Token`. No se acepta un token en la URL. |
| 429 | Cinco tokens incorrectos seguidos desde tu dirección; espera hasta 30 s. Detrás de un proxy todos los usuarios comparten una misma dirección. |
| `refusing to listen on 0.0.0.0:...` | Enlace que no es de loopback y sin token. Añade `--collab-token` (16 caracteres o más). |
| `cannot listen on 127.0.0.1:8080: Address already in use` | Otro proceso ocupa el puerto; elige otro con `--port`. La causa habitual es un estudio de una ejecución anterior que sigue activo. |
| La página carga pero la imagen está en blanco | Revisa el terminal; un proyecto que no pasa `check` se rechaza al iniciar. Recarga tras corregirlo. |
| Un mensaje de la consola del navegador `Failed to load resource ... 404` | Las compilaciones antiguas no tenían `/favicon.ico`; ahora se sirve. Un 404 que persista proviene de otra cosa. |
| El texto del panel Face | El panel ahora indica que se incluye el detector entrenado con datos sintéticos, que no está validado con fotos ni webcams, y que una webcam real requiere tu propio proveedor de landmarks. |
| El diseño está roto o faltan paneles | Menú `Layout`, restablecer el diseño, o elimina el `ui.json` que se muestra al iniciar. Un archivo de configuración que no se puede leer por completo se copia a `ui.json.bak` (después `.bak.1`, ...) antes de usar los valores por defecto, y `GET /api/ui` tiene un campo `warning` al respecto. |
| Mis ediciones desaparecieron tras reiniciar | No se escribe nada hasta pulsar **Save**. Con `--collab-log` la *sesión* sobrevive a un reinicio, pero el archivo en disco solo lo escribe Save. |
| `Ctrl+W`, `Ctrl+T` y similares no se pueden asignar | Están reservados por los navegadores (lista en la sección 3). |
| Ctrl+Z no deshace el cambio de otra persona en una sesión | Es intencionado: deshacer es por usuario. |
| Las exportaciones piden `ffmpeg` | Solo para el sonido en HTML (mp3) y para mp4/webm en la línea de comandos; GIF, Lottie y HTML sin sonido no necesitan nada. |

## 11. Accesibilidad y compatibilidad con navegadores

**Navegadores.** La interfaz se probó únicamente en **Chromium** (la compilación incluida con Playwright, ejecutada sin interfaz gráfica, en Linux). Firefox y Safari no se probaron y pueden funcionar o no.
Web MIDI necesita un navegador que lo implemente (Chromium lo hace); la cámara necesita `getUserMedia` y un contexto seguro (HTTPS o `localhost`). La página usa server-sent events para la
sesión compartida. Para arrastrar se usan eventos de puntero; las pantallas táctiles no se probaron.

**Accesibilidad, con franqueza.** No auditué el estudio con un lector de pantalla ni con una sesión solo de teclado. Lo que existe: un atributo `lang` y la etiqueta meta viewport, casillas de verificación etiquetadas, un
aviso `role="status"`, algunos atributos ARIA, **un conjunto completo de atajos de teclado y una paleta de comandos** que alcanzan todos los comandos, atajos reasignables, y la línea de comandos `osatura`, que hace
todo lo que hace el editor sin puntero. Lo que falta: el lienzo es una imagen y la mayor parte de la edición sobre el lienzo se hace con el puntero (existe el desplazamiento con las teclas de flecha), no hay tema claro y tampoco
tratamiento de `prefers-reduced-motion` ni de `prefers-color-scheme`, los contornos de foco son los predeterminados del navegador y no los comprobé, y no medí el contraste de color. Si dependes de tecnología de apoyo,
prefiere la CLI y el panel JSON, e informa al proyecto de lo que falle.

## 12. Problemas conocidos

* Corregido desde la primera redacción: el texto del panel Webcam face, el 404 de `/favicon.ico`, `--collab-nosync` sin `--collab-log`, y la falta de la marca `Secure` en la cookie detrás de un `--public-origin` https.
* `osatura render p.json 99 o.png` sobre un proyecto de 24 fotogramas ahora también es un error (añade `--clamp` para renderizar el último fotograma); el `/api/frame` del estudio rechaza un fotograma fuera de rango (`frame 999 out of range`) y ahora acepta solo los fotogramas `0..frames-1`.
* Los cambios de diseño y de atajos se guardan por usuario en la configuración de la interfaz, no en el proyecto, de modo que no viajan con el archivo.
