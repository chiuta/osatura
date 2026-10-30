<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Preguntas frecuentes

*Respuestas breves, cada una tomada de algo que se ejecutó o se leyó en el árbol de código fuente y en los demás documentos. Documentación CC-BY-4.0. Cuando una respuesta depende de una limitación, lo indica y remite a [LIMITATIONS.md](LIMITATIONS.md). Las cadenas de error de la última sección se capturaron de una compilación real; se citan exactamente, de modo que puedes buscar en esta página el texto que veas.*

Contenido: [Instalación y configuración](#instalación-y-configuración) - [ffmpeg](#ffmpeg) - [Rendimiento](#rendimiento) - [Determinismo](#determinismo) - [Licencias y tu salida](#licencias-y-tu-salida) - [Seguridad de los archivos y rutas de recursos](#seguridad-de-los-archivos-y-rutas-de-recursos) - [Por qué la salida debe diferir de la entrada](#por-qué-la-salida-debe-diferir-de-la-entrada) - [Rigging y animación](#rigging-y-animación) - [Estudio](#estudio) - [Rostro, voz y captura de movimiento](#rostro-voz-y-captura-de-movimiento) - [Errores, seguridad y contribuciones](#errores-seguridad-y-contribuciones) - [Limitaciones](#limitaciones) - [Comparaciones con otras herramientas](#comparaciones-con-otras-herramientas) - [Tabla de solución de problemas](#tabla-de-solución-de-problemas)

---

## Instalación y configuración

**1. ¿Cómo instalo OSATURA?**
Los archivos precompilados (Windows x86-64, macOS arm64, Linux x86-64 y el módulo WebAssembly) se preparan como la versión preliminar en borrador `v0.1.0-alpha.3` en <https://github.com/chiuta/osatura/releases>; se podrán descargar cuando el propietario publique el borrador, y no están firmados (SmartScreen muestra "Editor desconocido"; macOS pide aprobación). No hay instalador: descomprime el archivo y ejecuta `osatura`. O compílalo desde el código fuente con una toolchain estable reciente de Rust: `cargo build -p osatura-cli` produce `target/debug/osatura`; añade `--release` para obtener `target/release/osatura`. Véase [DEVELOPMENT.md](DEVELOPMENT.md).

**2. ¿Qué plataformas funcionan?**
Linux x86-64 es donde se desarrolló y ejercitó el proyecto. En GitHub Actions la batería de pruebas y clippy también pasan en macOS (arm64) y Windows (x86-64), y allí se compilan los binarios de la versión para los tres; pero ninguna persona ha usado todavía los ejecutables de Windows o macOS en un escritorio real, así que están "probados por CI", no "admitidos" ([LIMITATIONS.md](LIMITATIONS.md), sección 1).

**3. ¿Hay un instalador gráfico o una aplicación?**
No hay instalador. La línea de comandos es el producto. El estudio es una página web local servida por `osatura studio`; en Windows, al hacer doble clic en `osatura.exe` se inicia y se abre tu navegador. Los archivos de Windows y macOS también contienen `osatura-studio`, una pequeña ventana nativa alrededor del mismo estudio (solo se ha compilado y autoprobado en CI); véanse [STUDIO.md](STUDIO.md) y [GETTING_STARTED.md](GETTING_STARTED.md).

**4. ¿Cómo empiezo rápidamente?**
Sigue [GETTING_STARTED.md](GETTING_STARTED.md): construye un personaje animado a partir de un proyecto vacío, con cada paso ejecutado, y termina con un GIF.

**5. ¿Puedo compilar sin el motor de scripting?**
Sí: `cargo build -p osatura-cli --no-default-features` deja fuera Rhai, de modo que el comando `script` no existe (véase [DEVELOPMENT.md](DEVELOPMENT.md)). La función `scripting` es el único lugar por donde entra un crate con licencia de copyleft débil (smartstring); véase [THIRD_PARTY.md](THIRD_PARTY.md).

**6. ¿Necesita conexión de red o una cuenta?**
No. El programa no necesita ninguna cuenta ni red para funcionar (el estudio no tiene cuentas). Por defecto el estudio escucha solo en tu propia máquina (véase la pregunta 51).

**7. ¿Dónde guarda el estudio su configuración?**
En `$OSATURA_UI_CONFIG` si está definida; si no, en `$XDG_CONFIG_HOME/osatura/ui.json`; si no, en Windows `%APPDATA%\OSATURA\ui.json`; si no, en `~/.config/osatura/ui.json`. Tus proyectos nunca los toca.

---

## ffmpeg

**8. ¿Necesito ffmpeg?**
Solo para exportar MP4 y WebM, para `sound mix`, para audio mp3/ogg y para `face-track` sobre un archivo de vídeo. La exportación a GIF, APNG, HTML, secuencias PNG, SVG, glTF y JSON no necesita nada más. La lista completa está en [CLI_REFERENCE.md](CLI_REFERENCE.md), sección 1.6.

**9. ¿Se incluye ffmpeg en el paquete?**
Nunca. OSATURA inicia `ffmpeg` desde tu `PATH` como un programa independiente y no se ejecuta nada más. La situación de licencias y patentes de tu compilación de ffmpeg te corresponde comprobarla a ti; véase [PATENT_NOTES.md](PATENT_NOTES.md).

**10. ¿Qué dice cuando falta ffmpeg?**
`cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead` (para WebM, la sugerencia termina en `or export an APNG instead`). No se escribe nada.

**11. ¿Qué codificadores debe tener mi ffmpeg?**
`libx264` para mp4, `libvpx-vp9` o `libaom-av1` para webm, `aac` y `libopus` para el sonido en ellos, `libvorbis` y `libmp3lame` para `sound mix`.

---

## Rendimiento

**12. ¿Por qué va tan lento?**
Probablemente estás ejecutando una compilación debug. Las cifras de los documentos proceden de una compilación debug y son entre 10 y 30 veces más lentas que una compilación release ([CLI_REFERENCE.md](CLI_REFERENCE.md), sección 1.7). En la máquina de pruebas de 2 núcleos, una compilación debug necesitó 46 s para escribir como PNG los 48 fotogramas del proyecto de demostración y 57 s para exportarlo como GIF. Una compilación release nueva hizo los mismos dos trabajos en 1,5 s y 2,2 s. Compila con `cargo build --release` y usa `target/release/osatura`.

**13. ¿Usa la GPU?**
El renderizador por defecto es la CPU. Existen un crate `gpu` y un comando `render-gpu` tras un indicador de función, pero no había ninguna GPU real disponible durante la compilación: en una GPU por software (llvmpipe), un fotograma de 640x480 tardó entre 253 y 295 ms, frente a 13,8 ms en la ruta de CPU. Considera el renderizado en GPU como no probado en hardware ([LIMITATIONS.md](LIMITATIONS.md)).

**14. ¿Cuánto cuesta un rig por fotograma?**
En una compilación debug, el ejemplo `biped` tuvo una mediana de 64,5 ms por fotograma en la comprobación de rendimiento de `ci`. Un bake de tela de una bandera pequeña tardó 0,43 s. Mide tu propio proyecto con `osatura stats` y `osatura ci`; los rigs difieren mucho.

**15. ¿Cómo puedo aligerar un proyecto pesado?**
Usa `lod` para crear variantes más baratas, `export-fit` para encontrar los ajustes de exportación que cumplan un presupuesto de bytes y `proxy-bench` para comparar proxies. Véanse [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) y [EXPORT_FORMATS.md](EXPORT_FORMATS.md). `export-fit` puede detenerse antes con `--max-seconds` y entonces informa de que se le agotó el tiempo.

**16. Un script con miles de llamadas falla. ¿Por qué?**
`osatura script` acepta como máximo 5000 peticiones por ejecución y es todo o nada: si se rechaza cualquier petición, no se escribe nada. Divide un trabajo grande en varios scripts que lean la salida anterior.

---

## Determinismo

**17. ¿Es reproducible la salida?**
El renderizado está construido para serlo: el mismo proyecto y la misma seed dan los mismos píxeles, y la física, las partículas y el ruido usan seed. Esto es lo que permite que `golden` y `ci` comparen renders. Solo se ha comprobado en un tipo de máquina (Linux x86-64); no se promete una salida idéntica bit a bit en otras CPU o compiladores.

**18. ¿Por qué un bake de física da un resultado distinto después de cambiar un solo número?**
El bake es determinista para una entrada dada, pero es una simulación: cualquier cambio en la masa, la velocidad de fotogramas, el colisionador o la seed cambia toda la ejecución. Haz el bake con los mismos ajustes y la misma seed para comparar. El bake se fusiona con la animación existente por defecto (`merge=keep`: se conservan los canales existentes y el comando dice `kept existing animation on 'b2' (use merge=replace to overwrite)`); usa `merge=replace` para sobrescribir.

**19. ¿El ruido hace bucle?**
No. Un canal `noise` no se repite por sí solo, así que un bucle mostrará un salto en la costura. Usa en su lugar claves o una expresión como `8 * sin(t * 2 * pi)`; en las expresiones `t` va de 0 a 1 a lo largo del clip, no en fotogramas ([ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)).

**20. ¿Puedo comprobar que una animación hace un bucle limpio?**
Ejecuta `lint`. `make-loop` elimina por completo el desplazamiento: en una prueba, una pelota que rodaba quedó estática y `lint` avisó (L005). `loop-solve` cierra la costura en el movimiento cíclico, pero dejó un objeto con desplazamiento con un salto visible (de 280 a 280 en su canal de desplazamiento). Para un desplazamiento continuo, usa el modo de extrapolación `loop_offset`.

---

## Licencias y tu salida

*Este es un resumen en lenguaje llano de [LICENSING.md](LICENSING.md), no asesoramiento jurídico. Sigue pendiente la revisión por un abogado de la configuración de licencias.*

**21. ¿Qué licencias se aplican?**
MPL-2.0 para la biblioteca principal, la ABI de C y los crates de WASM y GPU; AGPL-3.0-or-later para el programa de línea de comandos y el estudio (`osatura-cli`); CC-BY-4.0 para la documentación y para los recursos y modelos de ejemplo sintéticos.

**22. ¿De quién son las animaciones y los renders que hago?**
Tuyos. La declaración del proyecto es que los proyectos, animaciones y renders que crees con OSATURA te pertenecen por completo; las licencias se aplican al software, no a su salida. Exportar un GIF o ejecutar un comando no genera ninguna obligación.

**23. ¿Puedo usar la salida comercialmente?**
Sí, según la declaración anterior. La excepción es todo lo que copies de los recursos de ejemplo: son CC-BY-4.0 y requieren atribución.

**24. ¿Tengo que publicar mis proyectos si uso las partes AGPL?**
No. La AGPL cubre el programa. Si ejecutas un `osatura-cli` o un estudio modificados de modo que otras personas lo usen a través de una red, debes ofrecerles su código fuente. El estudio lo hace con una página `/source` y un indicador `--source-url` que apunta a tu copia.

**25. ¿Puedo enlazar la biblioteca principal en una aplicación de código cerrado?**
La biblioteca principal es MPL-2.0, que es un copyleft a nivel de archivo: puedes usarla desde código cerrado y debes compartir los cambios en los propios archivos cubiertos por la MPL. La CLI y el estudio son AGPL y son otro asunto.

**26. ¿Puedo decir que mi herramienta "funciona con OSATURA"?**
Sí: "compatible con OSATURA" y "hecho con OSATURA" están permitidos. Una compilación modificada no debe distribuirse con el mismo nombre ([TRADEMARK.md](TRADEMARK.md)).

**27. ¿Puedo licenciar mi contribución con doble licencia?**
No hay ningún acuerdo de licencia de colaborador. Las contribuciones se aceptan bajo la licencia del proyecto con una firma DCO (`git commit -s`), por lo que no existe un propietario que pudiera relicenciarlas. Detalles en [CONTRIBUTING.md](CONTRIBUTING.md).

**28. ¿Qué pasa con el código de terceros?**
Unos 360 crates de terceros (contados en todas las plataformas; la aplicación de ventana añade el toolkit de Windows / macOS), todos con licencias permisivas salvo seis con MPL-2.0: smartstring (MPL-2.0 o posterior, solo con la función `scripting`) y cinco que solo la aplicación de ventana puede incorporar. Las listas están en [THIRD_PARTY.md](THIRD_PARTY.md) y [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md).

**29. ¿Hay preocupaciones sobre patentes?**
Véase [PATENT_NOTES.md](PATENT_NOTES.md). Son notas de ingeniería, no un dictamen jurídico.

---

## Seguridad de los archivos y rutas de recursos

**30. ¿Puede un comando dañar mi proyecto?**
Las escrituras son atómicas: el resultado va a un archivo temporal en el mismo directorio, se sincroniza y se renombra sobre el destino, de modo que un fallo deja el archivo antiguo. Los comandos de edición vuelven a validar el proyecto que están a punto de escribir y se niegan a escribir uno no válido (`output would be invalid: ...`). El comando `script` no aplica nada si falla cualquier petición.

**31. ¿Es seguro abrir un proyecto de otra persona?**
Razonablemente, pero no de forma incondicional. Un proyecto son datos, no código; el único programa externo que OSATURA inicia es ffmpeg, para los formatos enumerados en la pregunta 8. El cargador rechaza dispositivos, tuberías y directorios, limita el tamaño de la entrada y rechaza las rutas de recursos absolutas o con `..`. Los scripts de Rhai se ejecutan en un sandbox. Aun así, lee [SECURITY.md](SECURITY.md) antes de procesar archivos no fiables en un servicio.

**32. ¿Dónde busca un proyecto los sonidos y las imágenes?**
Relativos a la carpeta del archivo de proyecto. Las rutas absolutas y cualquier ruta con `..` se rechazan salvo que pases `--allow-external-assets`, y debes pasarlo de nuevo en cada comando posterior que cargue ese proyecto. Ejemplo de la referencia:
`sound add: asset '/etc/passwd': absolute paths are not allowed (a project may only reference files inside its own directory; use --allow-external-assets to override) (nothing was added)`.

**33. ¿Qué tamaño de proyecto leerá?**
Hasta 256 MiB por defecto. Cámbialo con `--max-input-bytes N`. Los archivos más grandes fallan con `cannot read demo.json: 51450 bytes exceed the input limit of 100 bytes (raise it with --max-input-bytes N)` (las cifras son las del ejemplo).

**34. ¿Por qué se niega a escribir en un enlace simbólico?**
Para que un enlace colocado junto a tu salida no pueda redirigir una escritura a otro lugar. El mensaje es `refusing to write through a symbolic link; pass --follow-symlinks to write to the link's target, or name the target file yourself`.

**35. ¿Por qué no puedo leer desde `<(...)` o una tubería?**
Las entradas deben ser archivos regulares: `cannot read /dev/fd/63: not a regular file (devices, pipes and directories are refused)`. Escribe antes los datos en un archivo.

**36. ¿Un campo desconocido en un proyecto lo rompe?**
No: se imprime `warning: unknown field bogus (ignored)` y el estado de salida es 0. Los archivos auxiliares que escribes para un comando (contratos, mapeos, máquinas) son más estrictos: una clave desconocida ahí es un error y no se aplica nada.

---

## Por qué la salida debe diferir de la entrada

**37. ¿Por qué `render in.png in.png` dice "refusing to overwrite"?**
En los comandos que leen un proyecto y escriben una imagen, un vídeo u otro archivo que no es un proyecto, el nombre de salida no puede ser el nombre de entrada. El mensaje es `refusing to overwrite the input project X with output of the same name`. Evita el clásico descuido de exportar encima de tu única copia del proyecto.

**38. Entonces, ¿por qué `script` y `autorig` pueden escribir sobre su propia entrada?**
La protección cubre las salidas que no son proyectos. Los comandos que leen un proyecto y escriben un proyecto (`script`, `autorig`, `loop-solve` y otros) pueden nombrar el mismo archivo como entrada y salida; la escritura es atómica y, si el comando falla, no cambia nada. Si quieres conservar el original, dale un nombre nuevo o usa `osatura vcs` para mantener el historial ([CI.md](CI.md) y [CLI_REFERENCE.md](CLI_REFERENCE.md)).

**39. ¿Por qué `render` rechaza `out.jpg`?**
`render` escribe solo `.png` o `.svg`: `render writes .png or .svg; 'out.jpg' has another extension (nothing was written)`. Para otros formatos usa `export`; la lista está en [EXPORT_FORMATS.md](EXPORT_FORMATS.md).

---

## Rigging y animación

**40. ¿Hacia dónde está el cero grados de un hueso?**
El ángulo 90 de un hueso apunta hacia abajo; los ángulos no se miden desde el eje x como lo hace un libro de texto de matemáticas. Los canales son deltas respecto a la pose de reposo, y `rest_offset` está en el sistema del padre ([RIGGING_GUIDE.md](RIGGING_GUIDE.md)).

**41. Mi cadena de IK no se estira o da un tirón. ¿Qué son `soft` y `stretch`?**
`soft` (0 a 0,9) suaviza la cadena hasta la extensión completa y `stretch` (1 a 4) le permite alargarse. Ambos existen solo en JSON, no como opciones de Rhai o de la CLI. Con `stretch` por encima de 1, la zona suave es el final del alcance estirado. Las cadenas de dos huesos se resuelven de forma analítica; las de tres o más usan FABRIK.

**42. Mis restricciones dan resultados raros.**
Las restricciones se ejecutan en el orden de la lista, y el orden importa. Un aim con peso 0,5 da la mitad del ángulo (un ángulo de -74,7 pasa a -37,4). Reordénalas en el JSON si una restricción posterior debe ver el resultado de una anterior.

**43. ¿Hay un skinner automático para arte vectorial?**
No en la línea de comandos. `Project::auto_skin` existe solo en la biblioteca. Para arte vectorial se hace el rig con huesos asignados como padres de capas, con lattices, spines o pins (solo JSON), o con mallas trazadas mediante `mesh-trace`.

**44. `skin-diffuse` y `cloth` se quejan de la malla.**
Ambos necesitan una malla, y cloth necesita una malla de **cuadrícula**: `cloth needs a grid mesh; the layer has a free-form (traced) mesh`. Una capa sin ninguna malla da `layer 'flag' has no mesh`.

**45. `foot-lock` dice que necesita una raíz.**
`foot lock needs a root controller bone (no parent)`. El hueso controlador raíz no debe tener hueso padre.

**46. `scale` rechaza mis fotogramas clave.**
`scale: frame 27 is outside the project range 0..=24` significa que una clave resultante quedaría fuera del proyecto; `keyedit scale` se rechaza cuando está fuera de rango salvo que añadas `--free`. Una ejecución en seco usa `-` como nombre de salida.

**47. ¿Por qué mi proyecto se hizo más largo después de `mocap-retarget`?**
`mocap-retarget` amplía la duración del proyecto para ajustarla al movimiento capturado. `follow --chain` puede desplazar claves más allá del último fotograma, lo que `lint` notifica (L007).

**48. ¿Cómo animo una capa sin hueso?**
Edita los canales `layer_x` y `layer_y` de la capa. En el estudio, activa la clave automática; con ella desactivada, una edición de capa permanece estática salvo que esa propiedad ya tenga un canal con claves. Las ediciones de huesos siempre crean claves. Véase [STUDIO.md](STUDIO.md).

---

## Estudio

**49. ¿Cómo abro el estudio?**
`osatura studio project.json` y abre la dirección que imprime, normalmente `http://127.0.0.1:<port>`. Es una aplicación de una sola página servida por el pequeño servidor HTTP propio de OSATURA (solo biblioteca estándar); no necesita internet.

**50. ¿El estudio cambia mi archivo mientras trabajo?**
No. El archivo en disco cambia solo cuando pulsas Save. Cerrar la página sin guardar pierde las ediciones (el registro de colaboración, si está activado, conserva todo el proyecto, pero no el historial de deshacer).

**51. ¿Pueden conectarse otras personas?**
Solo si enlazas una dirección que no sea de loopback, y entonces se exige un token: `refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (and still sends everything in clear text)`. El token debe tener al menos 16 caracteres, por ejemplo obtenido con `openssl rand -hex 16`. No hay cifrado: usa un túnel SSH o un proxy inverso con TLS.

**52. ¿Dónde pongo el token?**
En la cabecera `X-Collab-Token` o en la cookie `osatura_token` (HttpOnly, SameSite=Strict y Secure cuando la petición llega bajo un `--public-origin` https). Nunca se acepta en la URL. A partir del quinto intento fallido, esa dirección de cliente recibe HTTP 429 con un retardo creciente (2, 4 ... hasta 30 s).

**53. ¿Qué significan los errores HTTP 403, 415 y 421?**
421: la cabecera Host no es una que el estudio espere. 403: un `Origin` o `Sec-Fetch-Site` ajeno. 415: un POST que no es JSON. Son protecciones contra páginas de tu navegador que llaman al servidor local; véanse [STUDIO.md](STUDIO.md) y [SECURITY.md](SECURITY.md).

**54. ¿Funciona en cualquier navegador?**
Solo se probó en Chromium sin interfaz. Los demás no se han probado. No hay un comando de zoom del lienzo, y existe un panel dentro del estudio para el seguimiento facial con webcam, pero nunca hubo una cámara real disponible para probarlo ([LIMITATIONS.md](LIMITATIONS.md)).

---

## Rostro, voz y captura de movimiento

**55. ¿OSATURA rastrea rostros?**
Hay un pequeño detector integrado, entrenado solo con rostros sintéticos. En 50 escenas de prueba sintéticas (seed 9) encontró todos los rostros, con un error medio de puntos de referencia de 0,028 del tamaño del rostro, y dio 2 falsas alarmas en 50 escenas sin rostro. En dibujos de estilo de dibujos animados funcionó mal: no se encontró ninguno de los 6 rostros de robot cuadrado, y los casos de emoji redondo y de óvalo manga tuvieron errores de 0,164 y 0,166. Nunca se ha ejecutado sobre fotografías reales ni con una webcam real. Ten en cuenta que [LIMITATIONS.md](LIMITATIONS.md) L-13 y el texto del panel de webcam del estudio dicen que no se incluye ningún detector de rostros; eso está desactualizado respecto a este detector sintético.

**56. ¿Cómo funciona el lip-sync?**
Las señales de boca eligen entre cinco formas (reposo, pequeña, abierta, redonda, ancha). Un tiempo de señal con punto decimal son segundos; un entero sin más es un fotograma. `--rhubarb` solo cambia cómo se clasifica la letra E (redonda en lugar de ancha). El audio más largo que el proyecto se trunca.

**57. ¿Puedo animar a partir de un vídeo?**
`face-track` lee vídeo mediante ffmpeg. `face-bake` escribe las señales como canales con claves (el resumen dice "+N channels (K keys in them), M trigger keys set") y trunca a la duración del proyecto, con un aviso en stderr que cuenta los fotogramas no horneados. Una señal desconocida da `unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)`.

---

## Errores, seguridad y contribuciones

**58. ¿Cómo informo de un error?**
Abre una incidencia con la línea de comandos, la salida exacta (también stderr), el estado de salida, la compilación (`osatura --version`) y, si puedes, un proyecto mínimo. Ejecuta antes `osatura check project.json` y `osatura lint project.json`: muchos problemas se notifican ahí. Véase [CONTRIBUTING.md](CONTRIBUTING.md).

**59. ¿Cómo informo de un problema de seguridad?**
En privado, no en una incidencia pública; véase [SECURITY.md](SECURITY.md). La dirección del repositorio es `alexio@trom.tf`; es la misma dirección que el contacto del mantenedor y no se ha sustituido por un buzón de un equipo de seguridad.

**60. ¿Hay una lista de correo, un chat o un contrato de soporte?**
No. Los documentos nombran un contacto del mantenedor; la primera versión sigue siendo una versión preliminar en borrador.

**61. ¿Cómo contribuyo?**
Firma cada commit (`git commit -s`), mantén las pruebas en verde (`cargo test`, y `osatura ci` para el trabajo de animación) y lee [DEVELOPMENT.md](DEVELOPMENT.md). La documentación es CC-BY-4.0 y agradece correcciones, sobre todo donde una página y el programa discrepan.

---

## Limitaciones

**62. ¿Cuál es la limitación más importante?**
Ha sido construido y probado por un único proyecto en un único entorno. Todo lo que no figure como ejecutado está "sin probar". Lee [LIMITATIONS.md](LIMITATIONS.md); tiene ids que puedes citar en las incidencias.

**63. ¿Qué formatos de exportación requieren cuidado?**
El vídeo necesita ffmpeg. Lottie y glTF cubren un subconjunto de lo que OSATURA puede dibujar y animar, así que comprueba qué se pierde en [EXPORT_FORMATS.md](EXPORT_FORMATS.md). El error de `export` para una extensión desconocida enumera `.gif .html .json .mp4 .apng .webm .png .glb`, lo que omite `.lottie`; `.lottie` (dotLottie) es un destino válido, véase [EXPORT_FORMATS.md](EXPORT_FORMATS.md), sección 6.

**64. ¿Es utilizable la compilación WASM?**
Se compila para `wasm32-unknown-unknown` en GitHub Actions y la prueba del envoltorio pasa contra el módulo real con Node 20, y se adjunta al borrador de la versión como `osatura-web.tar.gz`. No se ha ejecutado en un navegador y no se han medido su velocidad ni su memoria ([WASM.md](WASM.md), [LIMITATIONS.md](LIMITATIONS.md)). Considérala verificada en parte.

**65. ¿Importa archivos de proyecto de otras herramientas?**
Importa SVG, OpenRaster, JSON de Lottie, imágenes, GIF/APNG, captura de movimiento BVH y MIDI ([EXPORT_FORMATS.md](EXPORT_FORMATS.md), secciones 9 y 10). No importa PSD, archivos zip `.lottie` ni carpetas OCA. Las importaciones crean un proyecto nuevo. Ten en cuenta que `import-lottie in.json in.json` no tiene protección contra la sobrescritura y reemplaza su entrada. No esperes que un archivo importado coincida con el original píxel a píxel.

**66. ¿Es estable el JSON del proyecto?**
Es el "formato 1". Un archivo más nuevo se rechaza: `project format 2 is newer than this build supports (1); update OSATURA`. Como solo existe una versión preliminar alfa, el propio formato 1 aún podría cambiar antes de una versión estable; véase [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

---

## Comparaciones con otras herramientas

**67. ¿En qué se diferencia de un editor de animación de escritorio basado en línea de tiempo?**
Esos son programas interactivos centrados en el dibujo. OSATURA se basa primero en comandos: cada operación es un comando o un script, el proyecto es un JSON comparable con diff y los renders son deterministas, por lo que pueden probarse en CI. El estudio es una vista web ligera sobre el mismo motor, y está mucho menos pulido que un editor maduro.

**68. ¿En qué se diferencia de una herramienta de animación esquelética 2D para juegos?**
Esas herramientas se centran en mallas, skins y bibliotecas de runtime para motores de juego concretos. OSATURA también tiene huesos, IK, restricciones y mallas, pero una oferta de runtime más reducida: su propio reproductor en HTML, una ABI de C y un crate WASM sin compilar. Elige según el runtime al que debas apuntar.

**69. ¿En qué se diferencia de un compositor de motion graphics?**
Un compositor se construye en torno a capas con efectos y expresiones sobre metraje de vídeo. OSATURA tiene efectos de capa y expresiones, pero su centro es el rigging de personajes y el arte vectorial, y no hace composición de vídeo, 3D ni corrección de color de metraje.

**70. ¿Cuándo es OSATURA la opción equivocada?**
Cuando necesitas hoy rendimiento en tiempo real en una GPU, un flujo de trabajo pulido con tableta gráfica, animación dibujada a mano fotograma a fotograma, un producto respaldado por un proveedor con versión estable y soporte, o un comportamiento confirmado fuera de Linux x86-64.

---

## Tabla de solución de problemas

Cada mensaje siguiente lo produjo una compilación real y termina con estado de salida 1 salvo que se indique lo contrario. La redacción de un mensaje puede cambiar antes de una versión publicada.

| Mensaje | Causa | Solución |
|---|---|---|
| `cannot read nope.json: No such file or directory (os error 2)` | Ruta o nombre incorrectos | Comprueba la ruta y el directorio actual |
| `invalid JSON in broken.json: EOF while parsing a value at line 2 column 0` | El archivo no es un JSON completo | Corrige la sintaxis en la línea indicada; restaura desde una copia de seguridad o con `osatura vcs` |
| `badparent.json is not a valid project:` then `bone 'a' has parent 'ghost' which is missing or listed after it` | Falta un hueso padre o figura después de su hijo | Define los padres antes que los hijos; corrige el nombre |
| `invalid JSON in fmt2.json: project format 2 is newer than this build supports (1); update OSATURA` | Archivo de una compilación más nueva | Usa una compilación más nueva |
| `warning: unknown field bogus (ignored)` (exit 0) | Una clave que esta compilación no conoce | Elimina o corrige la clave; busca una errata |
| `refusing to overwrite the input project X with output of the same name` | El nombre del archivo de salida es igual al de entrada | Elige un nombre de salida distinto |
| `render writes .png or .svg; 'out.jpg' has another extension (nothing was written)` | Extensión incorrecta para `render` | Usa `.png`/`.svg`, o `export` para otros formatos |
| `unknown option --bone for 'render' (did you mean --bones?); see: osatura (no arguments) for the usage` | Opción mal escrita | Usa la opción sugerida |
| `unknown export format '.bmp' (use .gif .html .json .mp4 .apng .webm .png .glb)` | Extensión no exportable | Elige un formato de la lista (`.lottie` también es válido) |
| `--scale needs a number in (0, 16]` | Escala fuera de rango | Usa un valor mayor que 0 y hasta 16 |
| `bad frame number 'x'` | Un argumento de fotograma no es un número entero | Pasa un entero |
| `cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead` | ffmpeg no está en el `PATH` | Instala ffmpeg, o exporta GIF, HTML o APNG |
| `the project has no sound tracks (see: sound add)` | Comando de sonido en un proyecto sin sonido | Añade una pista con `sound add` |
| `channel hand_ctrl.BoneOffsetY is locked` | El canal está bloqueado | Desbloquéalo, o edita otro canal |
| `request 1 (hand_ctrl.BoneOffsetY at 0) was rejected, nothing changed: channel hand_ctrl.BoneOffsetY is locked` | Lo mismo, desde `script` | Desbloquea primero; no se aplicó nada |
| `request 1 (channel head.BoneAngle) was rejected, nothing changed: ... expression error: unknown variable 'tt'` | Errata en una expresión | Las variables son frame, time, fps, frames, pi, t |
| `... unexpected end of expression` | Expresión sin terminar | Cierra los paréntesis, termina la fórmula |
| `points need an even number of coordinates: 3..1000 points` | Número impar de coordenadas o muy pocos puntos | Da pares x,y: de 3 a 1000 puntos |
| `cannot read /dev/fd/63: not a regular file (devices, pipes and directories are refused)` | Sustitución de procesos o tubería como entrada | Guarda antes en un archivo regular |
| `sound add: asset '/etc/passwd': absolute paths are not allowed (...)` | Recurso fuera de la carpeta del proyecto | Cópialo junto al proyecto, o pasa `--allow-external-assets` |
| `cannot read demo.json: N bytes exceed the input limit of M bytes (raise it with --max-input-bytes N)` | Archivo por encima del límite de entrada | Aumenta `--max-input-bytes` si confías en el archivo |
| `refusing to write through a symbolic link; pass --follow-symlinks ...` | La salida es un enlace simbólico | Nombra el destino, o pasa `--follow-symlinks` |
| `cloth needs a grid mesh; the layer has a free-form (traced) mesh` | Cloth sobre una malla trazada | Construye una malla de cuadrícula para la capa |
| `layer 'flag' has no mesh` | Un comando de malla en una capa sin malla | Crea primero una malla |
| `no bone 'b9'` | El nombre del hueso no existe | Comprueba los nombres con `stats` o `explain` |
| `unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)` | Nombre de señal incorrecto | Usa un nombre de la lista |
| `foot lock needs a root controller bone (no parent)` | La raíz elegida tiene un padre | Usa un controlador sin padre |
| `scale: frame 27 is outside the project range 0..=24` | Las claves escaladas quedan fuera del proyecto | Alarga el proyecto, escala menos o usa `--free` |
| `--extrapolate must be hold, linear, loop, ping_pong or loop_offset` | Modo incorrecto | Usa uno de los cinco |
| `cannot listen on 127.0.0.1:8861: Address already in use (os error 98)` | Puerto ocupado | Elige otro `--port` o detén el otro proceso |
| `cannot listen on 127.0.0.1:99999: invalid port value` | Puerto por encima de 65535 | Usa de 1 a 65535 |
| `refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (...)` | Enlace público sin token | Añade `--collab-token`, o permanece en loopback |
| `--collab-token must be at least 16 characters (e.g. \`openssl rand -hex 16\`)` | Token débil | Genera uno más largo |
| `--source-url must be an http:// or https:// URL without spaces or quotes` | URL mal formada | Usa una URL simple |
| `--public-origin must be a bare origin (scheme, host and optional port, no path), e.g. https://studio.example.org` | El origen tiene una ruta | Da solo esquema, host y puerto |
| `output would be invalid: ...` | La edición dejaría un proyecto no válido | Lee los motivos tras los dos puntos; no se escribió nada |
| `budget not met: the smallest export found is 14952 bytes (raw), above the budget of 8000 bytes ...` | `export-fit` no puede alcanzar el presupuesto | Aumenta el presupuesto o simplifica el proyecto (el archivo más pequeño se escribe igualmente) |
| `time budget exhausted (60 s ...)` | `export-fit` se quedó sin tiempo | Aumenta `--max-seconds` |
| `fuzz: failure rate 98.00% exceeds --max-fail-rate 50.00%` | Las poses rompieron el rig y pediste una barrera (`--gate` o `--max-fail-rate`) | Mira los casos que fallan; aumenta la tasa solo si es lo que quieres; `fuzz` a secas solo informa (exit 0) |
| `no .osatura-vcs repository found from ... (run: osatura vcs init <project.json>)` | Todavía no hay historial | Ejecuta `vcs init` |
| `frame 999 out of range` (studio API) | Fotograma más allá del proyecto | Usa un fotograma entre 0 y el último fotograma |
| `bad kind 'x' (vector, raster, png)` (studio API) | Tipo incorrecto | Usa un tipo de la lista |
| `nothing selected` (studio API) | Un comando de selección sin selección | Selecciona primero capas o huesos |

Los códigos de salida de `osatura ci` son 0 correcto, 1 fallo, 2 error.

---

*Documentación CC-BY-4.0. Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio). Puedes compartirla y adaptarla con atribución. Véanse también: [INDEX.md](INDEX.md), [GLOSSARY.md](GLOSSARY.md).*
