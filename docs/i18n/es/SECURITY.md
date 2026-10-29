<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Seguridad

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/docs/SECURITY.md) · [Română](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/ro/SECURITY.md) · [Français](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/fr/SECURITY.md) · [Italiano](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/it/SECURITY.md) · [Español](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/es/SECURITY.md) · [Português](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/pt/SECURITY.md) · [Deutsch](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/de/SECURITY.md) · [Magyar](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/hu/SECURITY.md)

*Modelo de amenazas, protecciones integradas, carencias conocidas, cómo informar de un problema y cómo desplegar el studio de forma segura. Documentación CC-BY-4.0. Derivado del código fuente (`osatura-core/src/{api,assets,fsutil,expr,script}.rs`, `osatura-cli/src/{http,studio,collab_srv}.rs`), de la sección 9 de `ROADMAP.md` (oleadas de revisión 1 a 4) y de las pruebas que se citan más abajo. **OSATURA no ha tenido ninguna auditoría de seguridad independiente.** Las oleadas de revisión las realizó el propio proyecto; considera esta página como una descripción de la intención y de lo que cubren las pruebas, no como una garantía.*

Contenido: [1 Modelo de amenazas](#1-modelo-de-amenazas) - [2 Servidor HTTP del studio](#2-servidor-http-del-studio) - [3 Autenticación y colaboración](#3-autenticación-y-colaboración) - [4 Resolución de recursos y límites de entrada](#4-resolución-de-recursos-y-límites-de-entrada) - [5 Sandbox de expresiones y scripts](#5-sandbox-de-expresiones-y-scripts) - [6 Archivos, enlaces simbólicos y datos temporales](#6-archivos-enlaces-simbólicos-y-datos-temporales) - [7 Programas externos (ffmpeg)](#7-programas-externos-ffmpeg) - [8 Bibliotecas (ABI de C, WASM, fachada)](#8-bibliotecas-abi-de-c-wasm-fachada) - [9 Carencias conocidas](#9-carencias-conocidas) - [10 Cómo informar de una vulnerabilidad](#10-cómo-informar-de-una-vulnerabilidad) - [11 Lista de refuerzo para quien despliega](#11-lista-de-refuerzo-para-quien-despliega) - [12 Proxy inverso con TLS: nginx y Caddy](#12-proxy-inverso-con-tls-nginx-y-caddy)

## 1. Modelo de amenazas

Lo que procesa OSATURA: **archivos de proyecto** (JSON), imágenes, audio, archivos de importación de Lottie / OCA / de estilo glTF, archivos de asignaciones y, en el studio, peticiones HTTP de un navegador. En el modelo que sigue, los archivos de proyecto son *datos no fiables*: un proyecto recibido de otra persona puede ser hostil, y abrirlo no debe ejecutar código, leer archivos fuera de su carpeta ni agotar la máquina.

| Actor | Puede | Objetivo supuesto | Defensas principales |
|---|---|---|---|
| Autor de un proyecto o archivo de importación hostil | darte un archivo | leer tus archivos, hacer que la herramienta falle o se cuelgue, ejecutar código, llenar el disco | análisis estricto, límites de tamaño y profundidad, rechazo de rutas de recursos, sin pánicos ante la entrada, sandbox de expresiones y scripts, salida atómica |
| Una página web que visitas mientras el studio se ejecuta en localhost | hacer que tu navegador envíe peticiones a `127.0.0.1` (CSRF, DNS rebinding) | editar tu proyecto, leerlo, escribir archivos | lista de hosts permitidos, comprobaciones de Origin / `Sec-Fetch-Site`, tipo de contenido JSON, token con cookie `SameSite=Strict`, CSP |
| Otro usuario en la misma máquina | conectarse al puerto, leer `/tmp` | leer o secuestrar una sesión o archivos temporales | enlaza a `127.0.0.1` por defecto, directorios temporales privados (0700), archivos de registro 0600, se exige token para enlazar en otra dirección |
| Alguien en tu red, cuando el studio está enlazado más allá de localhost | alcanzar el puerto | usar o adivinar el token, agotar las conexiones | token obligatorio (16+ caracteres), comparación en tiempo constante, retroceso (back-off), límites de conexiones y de tamaño; **sin TLS** |
| Un colaborador con el token | editar la sesión compartida | dañar el proyecto | historial de deshacer, el registro encadenado por hashes; se confía en los colaboradores respecto al proyecto, no hay cuentas ni roles |
| Llamador hostil de las bibliotecas | pasar argumentos incorrectos a la fachada, a la ABI de C o a la de WASM | corrupción de memoria, pánicos a través de la frontera | validación de argumentos, comprobaciones de tamaño antes de reservar memoria, pánicos capturados (donde el destino hace unwinding) |

Fuera del alcance: la protección frente a un administrador local malintencionado, una cadena de herramientas de compilación o una dependencia comprometidas (véase [THIRD_PARTY.md](THIRD_PARTY.md) para lo que se distribuye; `cargo deny` vigila licencias, prohibiciones y fuentes, no vulnerabilidades), los canales laterales y la seguridad del propio ffmpeg.

## 2. Servidor HTTP del studio

`osatura studio` sirve el editor desde su propia implementación mínima de HTTP/1.1 (`osatura-cli/src/http.rs`, solo biblioteca estándar, sin framework web de terceros). Comportamiento, con las pruebas que lo fijan (pruebas unitarias de `osatura-cli`, `tests/robust.rs`, `tests/wave4.rs`, `tests/source_offer.rs` y las pruebas de humo de Chromium):

**Límites**

| Límite | Valor | Resultado al superarlo |
|---|---|---|
| Tiempo para recibir la cabecera de la petición | 10 s | 408 |
| Tiempo para recibir el cuerpo | 20 s | 408 |
| Tiempo para escribir una respuesta | 10 s | conexión cerrada |
| Tamaño de la cabecera de la petición | 16 KB | 431 |
| Número de cabeceras | 100 | 431 |
| Tamaño del cuerpo | 32 MB | 413 |
| Conexiones simultáneas | 128 | 503 |
| Cuerpos de petición con codificación chunked | no admitidos | 411 |
| Cabecera `Host` duplicada | rechazada | 400 |
| Carácter de control (CR, NUL, DEL…) en el valor de una cabecera | rechazado (se permite el tabulador) | 400 |

Cada respuesta lleva `Connection: close` (sin estado keep-alive que agotar), `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer` y un `Content-Security-Policy`.

**Política de peticiones** (`Policy::check`, aplicada a cada petición):

1. **Lista de hosts permitidos.** La cabecera `Host` debe ser la dirección enlazada (`127.0.0.1`, `localhost`, el puerto real) o un nombre indicado con `--allow-host`. En caso contrario, 421 Misdirected Request. Esto es lo que detiene el DNS rebinding: una página en `evil.example` que resuelve a `127.0.0.1` envía `Host: evil.example`.
2. **Origin y `Sec-Fetch-Site`.** Las peticiones con un `Origin` de otro sitio, o con `Sec-Fetch-Site: cross-site` / `same-site`, se rechazan con 403. `Origin: https://<allowed host>` se acepta para los hosts de `--allow-host`; detrás de un proxy inverso que sirve la página en una URL pública, pasa `--public-origin URL` para que se acepte el origen del proxy. El origen cruzado sigue rechazándose.
3. **Tipo de contenido.** `POST` exige `Content-Type: application/json` (415 en caso contrario), de modo que un formulario HTML simple no puede enviar datos a la API.
4. **Token** (véase la sección 3), cuando está configurado; obligatorio cuando `--bind` no es una dirección de loopback.

**Oferta de código fuente.** El studio sirve `/source` (también cuando se exige token) y `--source-url` fija la dirección (los binarios oficiales de la versión usan por defecto el commit desde el que se compilaron), como exige la sección 13 de la AGPL-3.0 (véase [LICENSING.md](LICENSING.md)).

**Las rutas y métodos desconocidos o mal formados** reciben una respuesta de error simple; la página del editor está compilada en el binario (`studio.html`) y no se lee del sistema de archivos, de modo que no hay superficie de path traversal para el contenido estático.

## 3. Autenticación y colaboración

La opción `--collab-token T` (o el token exigido por un `--bind` no local) activa el control de acceso; `--collab-log LOG` hace persistente una sesión compartida.

* **Formato del token.** Al menos 16 caracteres, de `A-Za-z0-9._~+/=-`. Cualquier cosa con CR, LF, `;` u otros caracteres se rechaza al arrancar (inyección de cabeceras y de cookies).
* **Presentación.** Cabecera `X-Collab-Token: T`, o la cookie `osatura_token` (`HttpOnly`, `SameSite=Strict`, más `Secure` cuando la petición llegó bajo un nombre `--public-origin` con `https://`, un `Origin` https o `X-Forwarded-Proto: https`) establecida por `POST /api/login`. Una cookie errónea cuenta como intento fallido igual que una cabecera errónea.
* **Comparación.** En tiempo constante (ambos lados se procesan con SHA-256 y se comparan los resúmenes), de modo que el tiempo de respuesta no revela coincidencias de prefijo.
* **Retroceso (back-off).** Por dirección IP de cliente, a partir del **5.º** fallo consecutivo: 2, 4, 8, 16 y luego 30 segundos de respuestas 429; se vigilan hasta 512 direcciones de cliente. El token correcto se acepta siempre, incluso durante un retroceso (así un atacante no puede dejar fuera al propietario). El retroceso ralentiza las conjeturas; no detiene un ataque distribuido. **Detrás de un proxy inverso todos los clientes aparecen con la dirección del proxy**, por lo que comparten un único contador de retroceso: una avalancha de peticiones erróneas de un visitante retrasa el contador de fallos para todos, pero el token correcto sigue funcionando.
* **Secretos de participante.** Cada colaborador recibe un secreto aleatorio de 128 bits (`X-Collab-Secret`) que identifica su flujo; como máximo 32 participantes y 64 flujos de eventos.
* **Integridad del registro.** El registro de sesión es un archivo de texto, con una línea por operación: `<hash> <json>`, encadenado con SHA-256 (versión de formato 2). Editar, borrar o reordenar una línea rompe la cadena, lo que se notifica al cargar el registro. Cada petición se `fdatasync`ea antes de la respuesta, el archivo se crea con modo 0600 y se reescribe de forma atómica. La cadena detecta daños accidentales y manipulaciones casuales; **no está firmada**, de modo que quien pueda escribir el archivo puede reconstruir una cadena válida.
* **Tokens por participante (oleada 7, opcional).** `--collab-secret-file F` sustituye el único bearer compartido por tokens derivados de un secreto: `ap1.<name>.<expires>.<mac>`, con `mac` = HMAC-SHA-256 (RFC 2104, SHA-256 propio, vectores de prueba de la RFC 4231 en las pruebas) sobre el nombre y la fecha límite. El anfitrión los genera con `osatura collab-token NAME --secret-file F --ttl SECONDS` (por defecto 8 horas, como máximo 30 días) y entrega a cada persona el suyo. Un token está ligado a un nombre (un participante que se une recibe ese nombre sea cual sea el que envíe la página, de modo que nadie puede unirse como otra persona), caduca por sí solo, se verifica recalculándolo (sin tabla en el servidor, nada que robar de él salvo el propio secreto) y su cookie recibe un `Max-Age` igual a la vida restante. `--collab-revoke NAMES` rechaza los tokens de esos nombres antes de que caduquen; cambiar el secreto lo revoca todo. El archivo de secreto debe ser privado (modo 600 en Unix; si no, el studio se niega a arrancar), tener al menos 16 caracteres y nunca se acepta como argumento de línea de comandos. `--collab-token` y `--collab-secret-file` pueden combinarse; cualquiera de los dos satisface la regla de que un enlace no loopback necesita una credencial. La caducidad se comprueba **por petición**: un flujo de eventos que ya está abierto permanece abierto hasta que termina, y la siguiente conexión se rechaza. Un token caducado o revocado recibe un `401` con el motivo y no alimenta el retroceso por conjeturas.
* **Límite de frecuencia (oleada 7).** `--collab-rate N` permite N peticiones por segundo por dirección de cliente, con ráfagas de 2N (token bucket); lo que exceda recibe `429` con `Retry-After`. El valor por defecto es 100 por segundo cuando se exige una credencial y ningún límite en caso contrario; 0 lo desactiva. Se aplica después de la autenticación y por **dirección** (el origen de una conexión), no por conexión TCP, porque un límite por conexión se elude abriendo más conexiones; detrás de un proxy inverso todos los clientes comparten la dirección del proxy, así que limita también en el proxy (sección 12).
* **Advertencias de la dirección de enlace (oleada 7).** Un `--bind` no loopback imprime una advertencia al arrancar (HTTP sin cifrar; para `0.0.0.0` / `::`, que el studio escucha en todas las interfaces; para un token compartido, que no tiene nombres y nunca caduca).
* **Grafía del token.** El plazo de un token por participante tiene una única forma escrita (dígitos decimales, sin signo, sin ceros a la izquierda, sin `+`): cualquier otra grafía del mismo número se rechaza por mal formada, y la firma se juzga antes que el plazo, de modo que un token falsificado no aprende nada de la respuesta.
* **Anfitriones (opcional).** `--collab-host NAMES` (requiere `--collab-secret-file`) nombra a los participantes que pueden **guardar** (`/api/save`), **compactar** el registro de sesión (`/api/collab/compact`) y **revocar** (`/api/collab/revoke`, un `POST` que añade nombres a la lista de revocados, como máximo 1000). El resto de quienes tengan un token válido pueden seguir editando y observando, y reciben `403` en esas tres. Sin `--collab-host` nada cambia: todo titular de una credencial válida puede hacerlo todo, como antes. El `--collab-token` compartido no está ligado a un nombre y cuenta como anfitrión.
* **Flujo de eventos.** El flujo de eventos requiere una credencial (un token) pero no un secreto de participante; el secreto identifica el flujo propio de un participante, no es una segunda contraseña.
* **Conexiones por dirección.** Una misma dirección de cliente puede mantener como máximo 16 conexiones a la vez (además del límite global), de modo que una sola máquina no pueda agotar todos los huecos. Las direcciones de loopback no se cuentan por defecto, porque un proxy inverso en el mismo host llega al studio desde `127.0.0.1` por cada visitante; limita por visitante en el proxy (sección 12).
* **Sin TLS, sin cuentas.** El servidor habla HTTP sin cifrar. Un token que viaja por una red es legible por cualquiera que esté en la ruta. Coloca el studio detrás de un proxy inverso que termine TLS o de un túnel SSH / VPN (sección 11). No hay cuentas de usuario ni permisos por proyecto; el único rol es la lista opcional de anfitriones descrita arriba, y cualquier otro token válido concede acceso de edición.

## 4. Resolución de recursos y límites de entrada

* **Los recursos** (archivos de imagen, mapas de normales, sonidos nombrados por un proyecto) se resuelven mediante `assets::resolve`: respecto al directorio de recursos de todo el proceso (`assets::default_dir`, normalmente la carpeta del proyecto de entrada). **Las rutas absolutas y cualquier componente `..` se rechazan** salvo que se indique el flag global `--allow-external-assets`. Los archivos de imagen están limitados a 64 MB y los de audio a 256 MB.
* **Archivos de proyecto.** `load_project` lee únicamente archivos normales (ni dispositivos ni tuberías), hasta 256 MB (`--max-input-bytes N` para cambiarlo). El `MAX_JSON_BYTES` de la fachada es 256 MiB.
* **Proyectos derivados.** Los comandos que escriben un proyecto derivado validan respecto al directorio del proyecto de *entrada*; cuando la salida está en otro sitio, las rutas de recursos relativas de los archivos existentes se reescriben o una advertencia indica que la salida no los encontrará (`--allow-external-assets` los hace absolutos).
* **Renderizado.** La fachada rechaza imágenes de más de 16384 px de lado (`MAX_RENDER_SIDE`) y escalas fuera de 0.05 a 4; las capas de C y wasm rechazan salidas de más de 256 MiB antes de reservar memoria. Otros límites de la fachada: fps en (0, 1000], fotogramas de 1 a 200000, escenario hasta 32768, 4096 huesos, magnitudes dentro de 1e7, `lod_levels` de 1 a 12, muestras de `fuzz` hasta 100000, colores de `trace` de 2 a 64 ([API.md](API.md)).
* **Sin pánicos ante la entrada.** Las pruebas con entradas hostiles están en `osatura-cli/tests/robust.rs` y en las pruebas de las oleadas; pruebas de tipo fuzz cubren los analizadores. Un pánico alcanzable desde la entrada se trata como una vulnerabilidad.
* **Los cuerpos del studio** están limitados a 32 MB; las ediciones pasan por la misma validación que los archivos.

## 5. Sandbox de expresiones y scripts

* **Las expresiones** (enlaces de valores, restricciones, drivers) las analiza un analizador propio: **fuente de hasta 4096 bytes, profundidad de anidamiento 64**, aritmética, comparación, las variables fijas `frame`, `time`, `fps`, `frames`, `pi`, `t`, `get("layer","property")` para drivers, y una tabla fija de funciones integradas (`sin`, `abs`, `min`, ...); sin bucles, sin E/S y sin funciones definidas por el usuario, por lo que la evaluación siempre termina. Los resultados no finitos los rechaza la política de valores finitos.
* **Los scripts** (comando `script`, feature `scripting`, activada por defecto) se ejecutan en Rhai con estos límites: **500,000 operaciones, 32 niveles de llamada, profundidad de expresión (64 en el nivel superior, 32 en funciones), cadenas de 10,000 caracteres, arrays de 10,000 elementos, mapas de 1,000 entradas, 5,000 peticiones a la API, fuente del script de 64 KB**, y `no_module` (sin `import`, por tanto sin acceso a archivos mediante módulos). La API del host expuesta a los scripts es un conjunto fijo de funciones de edición de proyectos. Compila con `--no-default-features` para eliminar por completo el motor de scripting (también elimina la dependencia `smartstring`).
* Un script sigue siendo código que has decidido ejecutar sobre un proyecto: no puede leer ni escribir archivos por sí mismo, pero puede hacer muy grande el proyecto que edita dentro de los límites anteriores, y un bucle infinito termina en el límite de operaciones.

## 6. Archivos, enlaces simbólicos y datos temporales

* **Escrituras atómicas.** Las salidas se escriben con `fsutil::write_atomic`: un archivo temporal en el mismo directorio (nombre `.<name>.tmp-<pid>-<random>`), `fsync`, `rename`. Un fallo deja el archivo antiguo o el nuevo. **Los enlaces simbólicos indicados como salida se rechazan**, citando `--follow-symlinks` (esto cubre ahora toda salida que indica el usuario: informes y goldens de `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, `face-*`, archivos de pose, instantáneas, `vcs`, el registro de colaboración y los archivos de vídeo que escribe ffmpeg, que se comprueban previamente), de modo que un enlace plantado no puede redirigir una escritura. Si el directorio no admite escritura pero el archivo sí, la alternativa es una sobrescritura in situ con `fsync`, que **no es atómica** (documentado).
* **Datos privados.** Los registros de colaboración y otros archivos que solo debe leer el propietario se crean con modo 0600 (`write_atomic_private`). Los directorios temporales se crean de forma exclusiva con modo 0700 (`private_temp_dir`), de modo que otro usuario no puede crear de antemano un nombre predecible en un `/tmp` compartido. Los valores aleatorios (tokens para nuevos participantes, nombres temporales) proceden de `/dev/urandom` (`fsutil::random_bytes`); el generador determinista con semilla del motor nunca se usa para secretos.
* **Ajustes de la interfaz.** El `ui.json` del studio ignora las claves desconocidas con una advertencia y se copia a `ui.json.bak` (luego `.bak.1`, ...) antes del primer guardado.
* **Windows / no Unix.** Los modos de permisos anteriores se aplican en Unix. En Windows y macOS la batería de pruebas pasa en CI (las comprobaciones exclusivas de POSIX se omiten en Windows), pero las comprobaciones de enlaces simbólicos, permisos y `/dev/urandom` solo se ejecutan en Linux y macOS y nadie ha revisado a mano el comportamiento en Windows.
* **Binarios sin firmar y aplicación de ventana.** Los ejecutables de la versión no están firmados; verifica las descargas con `SHA256SUMS.txt`. `osatura-studio` inicia el mismo servidor local (solo `127.0.0.1`, puerto libre aleatorio) y lo muestra en la vista web del sistema (WebView2 / WKWebView); su vista web está tan expuesta como una pestaña de navegador en esa dirección y no ha tenido una revisión de seguridad aparte.

## 7. Programas externos (ffmpeg)

ffmpeg y ffprobe se ejecutan como procesos independientes **solo** para la exportación de vídeo, la decodificación de mp3 / ogg, la mezcla de la banda sonora y algunas herramientas; nunca se incluyen ([LICENSING.md](LICENSING.md)). Cada invocación:

* pasa los argumentos como un array (sin shell, sin concatenar cadenas en una línea de comandos);
* usa `-protocol_whitelist file` (un uso interno de tubería añade `pipe,fd`), de modo que un archivo multimedia no pueda hacer que ffmpeg abra `http://`, `concat:` u otros protocolos de red o especiales;
* pasa los nombres de archivo a través de `fsutil::file_arg`, que antepone `./` a los nombres que empiezan por `-` o contienen `:` o `|` en el primer componente de la ruta, de modo que un nombre de archivo no pueda interpretarse como una opción o una URL;
* para los archivos de sonido `.wav`, `.mp3` y `.ogg` fuerza el demuxer con `-f` (`soundtrack::forced_demuxer`), de modo que un archivo que no es lo que dice su nombre se rechaza en lugar de abrirse como otro contenedor;
* usa `-loglevel error`.

El propio ffmpeg analiza formatos multimedia complejos y ha tenido vulnerabilidades propias: mantenlo actualizado y no le des contenido multimedia de fuentes en las que no confíes en una máquina que guarde secretos. Los comandos con presupuesto de tiempo (`export-fit --max-seconds`) no pueden interrumpir una única llamada en curso a ffmpeg.

## 8. Bibliotecas (ABI de C, WASM, fachada)

* La **fachada** valida cada argumento, devuelve `ApiError` y convierte los pánicos en `ApiError::Internal` donde el destino hace unwinding. `Animation` y `Player` son `Send + Sync` (comprobado en tiempo de compilación).
* La **ABI de C** ([C_ABI.md](C_ABI.md)) comprueba punteros nulos, longitudes y UTF-8, y devuelve códigos; el llamador debe seguir pasando punteros válidos y respetar las reglas de los búferes. La cadena del último error es por hilo. La ABI se ejercitó con AddressSanitizer y UndefinedBehaviorSanitizer con los ejemplos documentados (sin problemas). No está reforzada frente a un llamador que viole los contratos documentados (cierre doble, uso tras el cierre, longitudes erróneas).
* El **módulo WASM** ([WASM.md](WASM.md)) se compila en CI y supera la prueba del envoltorio con Node, y se ejecutó una vez en Chromium sin interfaz (una entrada incorrecta da errores, sin fugas en 60 ciclos de apertura y cierre, escenarios demasiado grandes rechazados), pero **no se ha revisado como componente de navegador y no se ejecutó en Firefox ni Safari**, por lo que sus propiedades de seguridad se conocen solo por la lectura del código y esa prueba. No tiene acceso al sistema de archivos ni a la red por construcción; un pánico allí provoca un trap de la instancia.
* Ninguna de las dos capas aísla el *proyecto* en un sandbox: el renderizado de un proyecto hostil está acotado por los límites anteriores, pero un proyecto puede seguir pidiendo mucho trabajo dentro de ellos (muchas capas, imágenes grandes, líneas de tiempo largas).

## 9. Carencias conocidas

Lista honesta; véase [LIMITATIONS.md](LIMITATIONS.md) para la lista más amplia de límites funcionales.

1. **Sin TLS y sin cuentas** en el studio. Desde la oleada 7 el token puede ser por participante y con caducidad (sección 3), lo que sigue sin ser un sistema de cuentas: no hay roles, ni restablecimiento de contraseña, ni TLS propio. No se añadió TLS nativo porque no hay en `Cargo.lock` ningún crate de TLS con licencia compatible (añadir uno incorporaría una dependencia nueva y grande, y código criptográfico, a un proyecto AGPL/MPL sin auditoría); la sección 12 muestra la vía admitida, un proxy inverso que termine TLS.
2. **El retroceso es por dirección** y se comparte detrás de un proxy; ralentiza pero no impide las conjeturas en línea. Usa un token largo y aleatorio (el mínimo de 16 caracteres es un suelo, no una recomendación).
3. **El registro de colaboración está encadenado, no firmado.**
4. **No es una base de código auditada.** Sin pruebas de penetración de terceros ni campaña formal de fuzzing; las comprobaciones son las pruebas propias del proyecto y sus oleadas de revisión.
5. **Dependencias.** La CI ejecuta `cargo deny` para avisos de seguridad (vulnerabilidades, crates retirados, dependencias directas sin mantenimiento), licencias, crates prohibidos (ninguna pila de cliente TLS o HTTP) y orígenes. Los avisos de mantenimiento de dependencias transitivas se revisan a mano (véase `deny.toml`). La comprobación solo conoce los avisos publicados hasta ahora; vuelve a ejecutarla antes de desplegar.
6. **El directorio de recursos es de todo el proceso.** `merge` sobre varios directorios de entrada usa el último directorio para los recursos; solo `write_json` reescribe las rutas de recursos (los demás escritores no).
7. **Alternativa no atómica** cuando el directorio de salida no admite escritura.
8. **Las comprobaciones de rutas usan rutas canónicas.** `assets::resolve` canonicaliza los archivos existentes y rechaza un archivo cuya ubicación real queda fuera del directorio del proyecto, de modo que se rechaza un enlace simbólico dentro de una carpeta de proyecto que apunta fuera de ella (probado en `assets.rs`); los enlaces que permanecen dentro de la carpeta funcionan. No se excluye una condición de carrera entre la comprobación y la lectura posterior (time-of-check to time-of-use) por parte de alguien que pueda modificar la carpeta del proyecto de forma concurrente.
9. **ffmpeg** queda fuera de nuestro control (sección 7).
10. **WASM** no está probado en un motor de navegador (en CI solo se ejecuta con Node); no se ha examinado el modelo de amenazas de navegador de una página que lo incruste (CSP, límites de memoria).
11. **Denegación de servicio por tamaño.** Los límites acotan peticiones y archivos individuales, no el trabajo total: un proyecto válido pero muy pesado puede ocupar una CPU durante mucho tiempo (el límite de conexiones anterior limita la concurrencia, no el coste de una petición).

## 10. Cómo informar de una vulnerabilidad

* **No abras un issue público** para una vulnerabilidad. Envía un informe privado al mantenedor: **`alexio@trom.tf`** (la dirección que aparece en el aviso de `--version` del programa y en los documentos de licencia), o usa los informes privados de vulnerabilidades de GitHub en la pestaña *Security* del repositorio cuando estén activados.
* Incluye: la versión (`osatura --version`) y la plataforma, el proyecto o la petición que provoca el problema (lo mejor es un archivo mínimo), lo que esperabas y lo que ocurrió, y si quieres que se te reconozca la autoría.
* Qué esperar *(intención, no un contrato: este es un proyecto de una sola persona)*: un acuse de recibo en un plazo razonable, una corrección desarrollada en privado, una prueba de regresión añadida a la suite, una nota en [CHANGELOG.md](CHANGELOG.md) y el reconocimiento si lo deseas. Concede tiempo para una corrección antes de publicar los detalles.
* Versiones admitidas: solo la última versión preliminar (actualmente `v0.1.0-alpha.4`) y el árbol actual en `main`.
* Los informes sobre la exposición de red del studio cuando se enlaza deliberadamente a una dirección pública sin proxy inverso son, en primer lugar, cuestiones de documentación: véase la sección 11.

## 11. Lista de refuerzo para quien despliega

Para quien ejecute `osatura studio` para más de una persona o fuera de localhost.

1. [ ] Es preferible **no** exponerlo. Un solo usuario: deja el enlace por defecto (`127.0.0.1`) y usa un túnel SSH (`ssh -L 8080:127.0.0.1:8080 host`).
2. [ ] Si se expone, **termina TLS** en un proxy inverso (nginx, Caddy, ...) delante de él; nunca expongas el puerto HTTP sin cifrar a Internet.
3. [ ] Usa un **token largo y aleatorio** (por ejemplo de 32+ caracteres obtenidos con `openssl rand -base64 24`), pásalo mediante un archivo protegido o el entorno del servicio en lugar de por el historial del shell, y rótalo cuando se vaya gente. Recuerda que es acceso de edición completo.
4. [ ] Arranca con `--bind 127.0.0.1` (o la interfaz privada) más el proxy; fija `--allow-host your.domain` y `--public-origin https://your.domain` para que las comprobaciones de Host y Origin pasen para el proxy y para nada más.
5. [ ] Prefiere **tokens por participante** (`--collab-secret-file`, sección 3) con un `--ttl` corto antes que un único token compartido; mantén el archivo de secreto con modo 600 fuera de la carpeta servida, e indica con `--collab-host` quiénes pueden guardar, compactar y revocar.
6. [ ] Haz que el proxy aplique un **límite de frecuencia** y, si es posible, restrinja por IP o añada una segunda capa de autenticación (HTTP basic sobre TLS, VPN, proxy SSO), porque el retroceso integrado solo ve la dirección del proxy.
7. [ ] Ejecuta como **usuario sin privilegios** en un directorio dedicado que contenga solo los proyectos que se vayan a servir; sin secretos y sin enlaces simbólicos sensibles. Usa un sistema de archivos de solo lectura en el resto, un contenedor o el aislamiento de systemd (`ProtectSystem`, `PrivateTmp`, `NoNewPrivileges`).
8. [ ] Mantén `--allow-external-assets` y `--follow-symlinks` **desactivados**.
9. [ ] Fija `--source-url` a la dirección desde la que se pueda obtener el código fuente (posiblemente modificado) (sección 13 de la AGPL).
10. [ ] Persiste con `--collab-log` en un directorio privado, haz copia de seguridad y conserva el modo 0600. Trata el registro como sensible: contiene todo el historial de ediciones.
11. [ ] Acota los recursos a nivel del sistema operativo (memoria, tiempo de CPU, archivos abiertos, cuota de disco); los límites integrados acotan peticiones individuales, no la carga total.
12. [ ] Mantén al día ffmpeg y la cadena de herramientas de Rust utilizada para la compilación; ejecuta `cargo deny check` (o `cargo audit`) sobre `Cargo.lock` para tu publicación.
13. [ ] No le des al servidor proyectos de fuentes no fiables salvo que se ejecute en el sandbox anterior. Para despliegues sin scripting, compila con `--no-default-features`.
14. [ ] Vigila en los registros las respuestas 421, 403, 429 y 503; los picos indican sondeos o una configuración errónea.

## 12. Proxy inverso con TLS: nginx y Caddy

El studio habla HTTP sin cifrar. Para cualquier uso más allá de tu propia máquina, ejecútalo en loopback y deja que un proxy termine TLS. Estas recetas se escribieron a partir de la documentación pública de los proxies y del comportamiento del studio descrito más arriba; **no se ejecutaron en el entorno de redacción** (sin nginx ni Caddy, sin certificados, sin red): pruébalas antes de depender de ellas.

Arranca el studio en loopback y nombra su host público, de modo que las comprobaciones de Host y Origin acepten el proxy y nada más:

```
osatura studio demo.json --port 8080 --bind 127.0.0.1 \
  --allow-host studio.example.org --public-origin https://studio.example.org \
  --collab-secret-file /etc/osatura/collab.secret --collab-log /var/lib/osatura/session.log
```

Con un `--public-origin` `https://`, la cookie de inicio de sesión recibe el atributo `Secure`. El flujo de eventos (`/api/collab/events`) es Server-Sent Events: el proxy no debe almacenarlo en búfer y debe permitir lecturas largas.

**nginx** (certificados de tu CA o de `certbot`):

```
limit_req_zone $binary_remote_addr zone=osatura:10m rate=20r/s;

server {
    listen 80;
    server_name studio.example.org;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    server_name studio.example.org;
    ssl_certificate     /etc/letsencrypt/live/studio.example.org/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/studio.example.org/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    add_header Strict-Transport-Security "max-age=31536000" always;
    client_max_body_size 64m;

    location / {
        limit_req zone=osatura burst=60 nodelay;
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Proto https;
        proxy_set_header Connection "";
    }
    location /api/collab/events {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Proto https;
        proxy_set_header Connection "";
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 1h;
    }
}
```

**Caddy** (obtiene y renueva el certificado por sí mismo; `Caddyfile`):

```
studio.example.org {
    encode zstd gzip
    header Strict-Transport-Security "max-age=31536000"
    request_body {
        max_size 64MB
    }
    reverse_proxy 127.0.0.1:8080 {
        flush_interval -1
        transport http {
            read_timeout 1h
        }
    }
}
```

`flush_interval -1` envía el flujo de eventos sin búfer. Caddy establece `X-Forwarded-Proto` y pasa el `Host` original. Para un límite de frecuencia de peticiones usa el plugin `caddy-ratelimit` o coloca delante el `limit_req` de nginx.

Comprobaciones tras el arranque: `curl -I https://studio.example.org/` responde `401` (página de inicio de sesión) y nunca sirve el studio sin token; `curl -I http://127.0.0.1:8080/ -H 'Host: evil.example'` responde `421`; la cookie establecida por `/api/login` lleva `Secure; HttpOnly; SameSite=Strict`; el propio registro de la sesión del studio permanece privado (sección 6). Si el proxy se ejecuta en otra máquina, enlaza el studio a la interfaz privada que el proxy pueda alcanzar, protege el puerto con un cortafuegos y recuerda que el propio studio sigue hablando HTTP sin cifrar en ese enlace.
