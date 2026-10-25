<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# Formatos de exportación e importación de OSATURA

Qué conserva, aproxima y descarta cada formato de archivo que OSATURA escribe o lee, qué comando lo produce y cómo
comprobar el resultado. Este texto se publica bajo licencia CC-BY-4.0. Versión documentada: **osatura 0.1.0**.

Cada línea de comandos de abajo se ejecutó sobre el proyecto de demostración integrado (`osatura demo demo.json`) o sobre una copia de 8 fotogramas
(`small.json`, el mismo proyecto con `"frames": 8`). Los tamaños citados proceden de ese proyecto de 8 fotogramas y 640x480 y son solo
orientativos: escalan con los fotogramas, los anclajes y las capas, no con nada fijo. Los detalles de los comandos (todas las opciones,
todos los errores) están en [CLI_REFERENCE.md](CLI_REFERENCE.md); el formato de archivo del propio proyecto está en
[PROJECT_FORMAT.md](PROJECT_FORMAT.md); el estado de códecs y patentes está en [PATENT_NOTES.md](PATENT_NOTES.md) (no es asesoramiento jurídico).

## Contenido

1. [Qué formato para cada tarea](#1-qué-formato-para-cada-tarea)
2. [Formatos de salida de un vistazo](#2-formatos-de-salida-de-un-vistazo)
3. [Imágenes fijas: SVG, PNG, PNG de 16 bits, OpenEXR](#3-imágenes-fijas)
4. [Animación ráster: hoja de sprites PNG, GIF, APNG, WebM, MP4](#4-animación-ráster)
5. [Reproductor HTML](#5-reproductor-html)
6. [Lottie (.json) y dotLottie (.lottie)](#6-lottie-json-y-dotlottie-lottie)
7. [glTF binario (.glb)](#7-gltf-binario-glb)
8. [Carpeta Open Cel Animation (OCA)](#8-carpeta-open-cel-animation-oca)
9. [Importación: SVG, OpenRaster, Lottie, imágenes, GIF/APNG](#9-importación-svg-openraster-lottie-imágenes-gifapng)
10. [Importación: captura de movimiento BVH, MIDI y otras entradas en directo](#10-importación-captura-de-movimiento-bvh-midi-y-otras-entradas-en-directo)
11. [Formatos de audio](#11-formatos-de-audio)
12. [Matriz de compatibilidad de funciones](#12-matriz-de-compatibilidad-de-funciones)
13. [Cómo reducir el tamaño de los archivos: export-fit, lod, size-report](#13-cómo-reducir-el-tamaño-de-los-archivos)
14. [Requisitos de ffmpeg](#14-requisitos-de-ffmpeg)
15. [Referencias de patentes y licencias](#15-referencias-de-patentes-y-licencias)
16. [Validar lo que has escrito](#16-validar-lo-que-has-escrito)
17. [Diferencias con otros documentos](#17-diferencias-con-otros-documentos)

---

## 1. Qué formato para cada tarea

| Quieres... | Usa | Por qué |
|---|---|---|
| El aspecto exacto, compartido como un solo archivo, que se reproduce en cualquier navegador y permite desplazarse por la animación | `.html` | Fotogramas SVG en línea dibujados por el propio renderizador de OSATURA; no se aproxima nada. |
| Una animación vectorial pequeña para una página web o una aplicación con un reproductor Lottie | `.json` o `.lottie` | Vectorial, independiente de la resolución, pequeño; pero consulta la lista de descartes (imágenes, trazos de anchura variable, audio...). |
| Una vista previa en bucle para un chat o un README | `.gif` | Funciona en todas partes; 256 colores por fotograma, transparencia de 1 bit, sin sonido. |
| Un sustituto de GIF con color completo y alfa | `.apng` | RGBA de 8 bits sin pérdida; más grande que GIF. |
| Un vídeo con sonido | `.mp4` (H.264 + AAC) o `.webm` (VP9/AV1 + Opus) | Requiere que instales `ffmpeg`. `.webm` conserva la transparencia. |
| Hoja de sprites para un motor de juegos | `.png` | Un PNG más un atlas JSON con los rectángulos de las teselas. |
| Un esqueleto y un movimiento para una herramienta 3D | `.glb` | Huesos, animación y formas planas como mallas de triángulos. |
| Fotogramas para herramientas de composición o pintura | `export-oca` (Open Cel Animation) o `frames --png` | Cels como PNG, en una carpeta. |
| Imágenes fijas en luz lineal o de alta profundidad de bits | `render-hdr` (`.exr` o `.png` de 16 bits) | Renderizador de coma flotante, OpenEXR de 16 bits o half/float. |
| Un solo fotograma | `render` (`.svg` o `.png`) | Vectorial o ráster de 8 bits. |

## 2. Formatos de salida de un vistazo

| Formato | Comando | Extensión | Requiere ffmpeg | Alfa | Audio | Tamaño para la demo de 8 fotogramas* |
|---|---|---|---|---|---|---|
| SVG (un fotograma) | `render` | `.svg` | no | sí | no | unos pocos KB |
| Secuencia SVG/PNG | `frames` | carpeta | no | sí | no | un archivo por fotograma |
| PNG (un fotograma) | `render` | `.png` | no | sí (8 bits) | no | |
| PNG de 16 bits | `render-hdr` | `.png` | no | sí | no | |
| OpenEXR (half/float) | `render-hdr` | `.exr` | no | sí | no | 466 KB half, 923 KB float (640x480) |
| Hoja de sprites PNG + JSON | `export` | `.png` (+ `.atlas.json`) | no | sí | no | |
| GIF | `export` | `.gif` | no | 1 bit | no | 83 KB |
| APNG | `export` | `.apng` | no | sí | no | 131 KB |
| WebM | `export` | `.webm` | sí | sí | sí (Opus) | 10.5 KB |
| MP4 | `export` | `.mp4` | sí | no | sí (AAC) | 8.9 KB |
| Reproductor HTML | `export` | `.html`, `.htm` | solo para el audio | sí | mp3 incrustado | 66 KB |
| Lottie | `export` | `.json` | no | sí | no | 47 KB |
| dotLottie | `export` | `.lottie` | no | sí | no | 11.5 KB |
| glTF binario | `export` | `.glb` | no | sí | no | 115 KB |
| Open Cel Animation | `export-oca` | carpeta | no | sí | no | un PNG por cel |

\* GIF, APNG, WebM y MP4 con `--scale 0.5` (320x240); formatos vectoriales a tamaño completo. «Requiere ffmpeg» significa un
`ffmpeg` que funcione en `PATH` (sección 14). Los nombres de salida se eligen por la extensión, sin distinguir mayúsculas de minúsculas; una extensión desconocida se rechaza y
no se escribe nada (`unknown export format '.xyz' (use .gif .html .json .mp4 .apng .webm .png .glb)`, y también se
acepta `.lottie`).

Todos los escritores se niegan a sobrescribir el proyecto de entrada con la salida, escriben a través de un archivo temporal y lo renombran, y rechazan los
enlaces simbólicos salvo que se indique `--follow-symlinks` (CLI_REFERENCE sección 1.4). La protección de tamaño de los exportadores vectoriales
(Lottie, HTML, GLB) rechaza una exportación de más de unos 40 millones de valores (fotogramas x anclajes, más huesos y capas por fotograma)
con `export would write about N million values ...`.

## 3. Imágenes fijas

### SVG (`render ... out.svg`, `frames`)

```
$ osatura render demo.json 12 f12.svg --bones
wrote f12.svg
$ osatura frames demo.json fr            # fr/frame_0000.svg ... frame_0047.svg
```
La salida vectorial del propio renderizador de OSATURA: rutas, degradados, trazos con perfiles de anchura, guiones, modos de mezcla, máscaras y efectos de
capa se escriben como SVG. `--bones` dibuja la superposición del esqueleto. Para `.svg`, `--scale` deja sin cambios el viewBox del documento. **Descartes y límites conocidos:** el renderizador no informa de ninguno; cómo trate un visor SVG concreto los modos de mezcla y los filtros depende
de ese visor. `--proxy N` escribe un proxy de desplazamiento simplificado que descarta los trazos finos, los degradados y los modos de mezcla (no para
entregables). **Validación:** ábrelo en un navegador, o `python3 -c "import xml.dom.minidom,sys; xml.dom.minidom.parse(sys.argv[1])" f12.svg`.

### PNG (`render ... out.png`, `frames --png`)

RGBA de 8 bits, sRGB, renderizado por el rasterizador propio del motor. `--scale N` en (0, 16] multiplica el tamaño en píxeles. Un proyecto con
`"background": null` se renderiza transparente donde no se dibuja nada. Por defecto no se gestiona el color: los valores se mezclan en el espacio codificado en sRGB.
Desde la oleada 7 un proyecto puede optar por la composición en luz lineal (`osatura color-space p.json out.json linear`, campo `color_space`):
las salidas ráster, GIF, APNG, vídeo y `render-hdr` se componen entonces en luz lineal f32 y se convierten una sola vez a sRGB; SVG y
Lottie no cambian (LIMITATIONS L-22).

### PNG de 16 bits y OpenEXR (`render-hdr`)

```
$ osatura render-hdr small.json 3 f.exr                           # HALF, 640x480, 465,611 bytes
$ osatura render-hdr small.json 3 f2.exr linear --precision float # FLOAT, 923,211 bytes
$ osatura render-hdr small.json 3 f4.png                          # 16-bit/colour RGBA, sRGB-encoded
```
* **OpenEXR:** una sola parte, por líneas de barrido, **sin comprimir**, canales `A B G R`, luz lineal, alfa premultiplicado. `HALF` por defecto,
  `FLOAT` con `--precision float` (o `--float32`). La palabra `linear` no tiene efecto en EXR (siempre es lineal).
* **PNG de 16 bits:** codificado en sRGB salvo que se indique la palabra `linear`.
* **Fidelidad:** el renderizador de coma flotante tiene más precisión que el de 8 bits, pero la ilustración es color sRGB ordinario, de modo que el archivo
  contiene valores del rango de visualización con mayor precisión, no luz HDR referida a la escena. El fondo del proyecto no se dibuja.
* **Descartes y límites conocidos:** un fotograma por archivo; sin compresión, capas ni AOV; sin perfil ICC; `--8bit` recurre al rasterizador de 8 bits
  como fuente.
* **Validación:** `ffprobe f.exr` informa `exr 640x480 gbrapf32le`; cualquier visor de EXR lo abre.

## 4. Animación ráster

Todos estos formatos renderizan primero cada fotograma con el motor ráster. Un proyecto con `"background": null` se renderiza con píxeles
transparentes donde no se dibuja nada; la demo tiene un color de fondo (`#f6f1e7`) y por tanto es opaca. `--scale N` en (0, 16]
se aplica a `.gif .apng .webm .mp4 .png`; en otros formatos se rechaza (`unknown option --scale for 'export .json'`).

### Hoja de sprites PNG (`export ... out.png`)

```
$ osatura export small.json out3.png --columns 4 --scale 0.25
wrote out3.atlas.json
wrote out3.png                    # 4 columns x 2 rows of 160x120 tiles
```
Un PNG con todos los fotogramas como teselas (`--columns N`, por defecto una cuadrícula casi cuadrada: ceil(sqrt(frames)) columnas) y un atlas
`out3.atlas.json` (el nombre de salida con la extensión sustituida por `.atlas.json`) que enumera `{index, x, y, w, h}` por fotograma. Se conserva el alfa.
El atlas se llamaba antes `out3.json` y podía sobrescribir en silencio un archivo Lottie con el mismo nombre base; el nuevo nombre lo evita, y la ruta del atlas pasa por las mismas protecciones de entrada y enlaces simbólicos.

### GIF (`export ... out.gif`)

```
$ osatura export small.json out.gif --scale 0.5
wrote out.gif                      # 8 frames, 83 KB
```
* Una imagen por fotograma de origen, **en bucle infinito**, una paleta local de hasta 256 colores por fotograma (cuantizada, sin tramado).
* **Temporización:** los retardos de GIF son centésimas de segundo enteras, y los visores tratan cualquier valor inferior a 2 cs como 10 cs, así que 50 fps es el máximo real.
  Hasta 50 fps los retardos se difunden con corrección de error (29.97 fps alterna 3 y 4 cs), de modo que la duración total queda a 1 cs de
  `frames / fps`. Por encima de 50 fps la secuencia se **diezma a 50 fps**, se mantiene la duración y se imprime una advertencia
  (`GIF cannot hold 60 fps ... use APNG, WebM or MP4 for the full rate`).
* **Transparencia:** de 1 bit (se activa el indicador de color transparente); los píxeles semitransparentes no se conservan.
* **Descartes conocidos:** sin audio (`note: GIF cannot carry audio; ...` cuando el proyecto tiene sonidos); sin alfa parcial; las imágenes de más de 65535 píxeles
  de ancho o de alto se rechazan (`image too large for GIF`).
* **`--report r.json`:** escribe la lista de cosas que el formato no pudo transportar (aquí: el diezmado de la frecuencia de fotogramas, si lo hubo).
* **Consejos de tamaño:** un `--scale` menor, menos fotogramas, o `export-fit --format gif` (sección 13). GIF es el menos eficiente de los formatos
  ráster para ilustraciones fotográficas o con muchos degradados.
* **Validación:** `ffprobe -v error -count_frames -select_streams v -show_entries stream=nb_read_frames -of csv=p=0 out.gif` imprime el
  número de fotogramas (8).

### APNG (`export ... out.apng`)

PNG animado, RGBA de 8 bits, sin pérdida, un fotograma por fotograma del proyecto a los fps del proyecto (0.0417 s por fotograma a 24 fps). Se conserva el alfa completo. Sin audio. Más grande
que GIF para ilustraciones planas (131 KB frente a 83 KB en el ejemplo) porque no hay paleta. La mayoría de los navegadores reproducen APNG; algunas herramientas no,
y en ese caso usa WebM o GIF. **Validación:** `ffprobe` con `-count_frames` (8 fotogramas para el ejemplo).

### WebM (`export ... out.webm`) y MP4 (`export ... out.mp4`)

```
$ osatura export small.json out.mp4  --scale 0.5     # h264, yuv420p, 320x240, 8 frames
$ osatura export small.json out.webm --scale 0.5     # vp9, yuv420p + alpha_mode=1
$ osatura export small.json out.webm --codec av1     # needs an ffmpeg with libaom-av1
```
* **MP4:** H.264 (`libx264`), `yuv420p`, **sin alfa**. Audio AAC cuando el proyecto tiene pistas de sonido.
* **WebM:** VP9 (`libvpx-vp9`, por defecto) o AV1 (`libaom-av1`, `--codec av1`); se conserva el canal alfa (`yuva420p`, etiqueta de contenedor
  `alpha_mode=1`); audio Opus (`libopus`) cuando el proyecto tiene sonidos. Cualquier otro valor de `--codec`: `unknown --codec 'foo' (vp9 or av1)`.
* **Sonido:** las pistas de sonido del proyecto se mezclan (inicio, ganancia, recorte, fundidos; cortadas a la duración del proyecto) y se multiplexan.
* **Requiere ffmpeg:** los fotogramas se renderizan a PNG en un directorio temporal privado, se pasan a `ffmpeg` y después se eliminan. Si
  falta `ffmpeg`: `cannot run ffmpeg (...); install it or export a GIF/HTML instead` (salida 1, no se escribe nada). Si le falta
  el codificador: `ffmpeg failed`.
* **Límites conocidos:** solo 8 bits; el tamaño del vídeo sigue el tamaño del proyecto y `--scale` (comprueba el resultado con `ffprobe`).
  `dof-export` también escribe `.mp4`/`.webm`, sin audio.
* **Validación:** `ffprobe -v error -show_entries stream=codec_name,width,height,pix_fmt,nb_frames,r_frame_rate -of csv=p=0 out.mp4`
  imprimió `h264,320,240,yuv420p,24/1,8`.
* **Patentes:** véase la sección 15; H.264 y AAC están cubiertos por consorcios de licencias, VP9/AV1 no se han evaluado.

## 5. Reproductor HTML

```
$ osatura export small.json out.html
wrote out.html                     # 66 KB for 8 frames
```
Un archivo HTML autocontenido: **cada fotograma como SVG en línea**, un botón de reproducir/pausar, un control deslizante (entrada de rango) y una etiqueta de fotograma,
gobernado por un único `<script>` en línea. **No realiza ninguna petición de red**: la única URL del archivo es el espacio de nombres SVG. Para abrirlo
basta un navegador. La imagen es exactamente la que produce el renderizador SVG, así que es el más fiel de los formatos «reproducibles»
(máscaras, modos de mezcla, efectos, huesos, IK, deformadores y partículas están todos incorporados en los fotogramas).

* **Audio:** si el proyecto tiene sonidos, el audio mezclado se incrusta como un URI de datos mp3 (requiere `ffmpeg`; advertencia por encima de 3 MiB, y el
  audio se descarta con una advertencia por encima de 16 MiB).
* **Tamaño:** crece linealmente con los fotogramas (unos 7.7 KB por fotograma aquí) y con el detalle vectorial; el archivo no está comprimido. Consejos
  habituales: sírvelo comprimido con gzip (el ejemplo de 66 KB pesa unos 9 KB con gzip), reduce el número de fotogramas o el detalle de las rutas con `lod`, y usa
  `export-fit --format html`.
* **Límites:** `--report` se rechaza (`unknown option --report for 'export .html'`);
  se aplica la protección de tamaño de la exportación vectorial; el reproductor solo ofrece reproducir/pausar y un control deslizante.
* **Validación:** ábrelo; `grep -c '<svg' out.html` cuenta los fotogramas; `grep -o 'http[^"<]*' out.html` debería enumerar solo el espacio de nombres SVG.

## 6. Lottie (.json) y dotLottie (.lottie)

```
$ osatura export small.json out.json --report rep.json
wrote out.json
warning: [variable-width-stroke] ink: variable-width stroke profiles are not supported by Lottie; the constant base width is used
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
$ osatura export small.json out.lottie       # a zip: manifest.json + a/animation.json
```

### Cómo se construye

El archivo es Bodymovin (`"v": "5.7.0"`) con `fr` = fps del proyecto, `ip` = 0, `op` = número de fotogramas y `w`/`h` = el tamaño del escenario. **Cada
capa vectorial se convierte en una capa de formas cuya geometría de rutas se hornea por fotograma en el espacio del mundo (fotogramas clave de retención)**, de modo que huesos, IK,
skinning, deformadores, springs, partículas, la cámara (incluido el paralaje) y las transformaciones de los padres están todos *dentro* de la geometría. El resultado
se reproduce como el original, pero no es un rig editable: no hay huesos, ni canales, ni expresiones. Se escribe una capa de fondo sólido
cuando el proyecto tiene un color de fondo. En consecuencia, el archivo crece con `frames x anchors`; es mucho más grande que un
Lottie hecho a mano con animación de transformaciones (47 KB para la demo de 8 fotogramas, 11.5 KB dentro de un `.lottie`).

### Se exporta

Rellenos y trazos sólidos y con degradado (paradas de degradado con alfa; los extremos del degradado siguen la transformación animada de la capa), patrones de guiones, remates, uniones y límite de inglete, anchuras de trazo escaladas por la escala de la capa, modos de mezcla, opacidad de capa, mattes de pista de alfa y de alfa invertido
para las máscaras de capa (una copia de la fuente de la máscara actúa como matte), efectos de capa (desenfoque, sombra paralela, tinte, relleno) en
capas de una sola hoja, y **opacidad de grupo como precomposición** para que los hijos superpuestos no se transparenten entre sí. Los números se
escriben con su representación decimal más corta.

### Se advierte (nunca se descarta en silencio)

Cada elemento siguiente se imprime en stderr como `warning: [code] layer: message` y se escribe en `--report FILE` como
`{"format":"lottie","count":N,"warnings":[{"code","layer","message"}]}`:

| Código | Significado |
|---|---|
| `image-layer` | Las capas de imagen no se exportan (la salida Lottie contiene solo capas vectoriales). |
| `variable-width-stroke` | Los perfiles de anchura no se admiten; se usa la anchura base constante. |
| `stroke-anisotropic` | Escala no uniforme o inclinación en la capa; los trazos de Lottie son uniformes, así que la anchura se escala por la media geométrica. |
| `gradient-anisotropic` | Un degradado en una capa así se aproxima (los degradados de Lottie no se deforman por inclinación). |
| `blend-add` | El modo de mezcla `add` no tiene equivalente en Lottie; se usa `screen`. |
| `group-blend` | Un modo de mezcla en un grupo no se exporta. |
| `group-mask` | Una máscara en un grupo no se exporta (las máscaras en capas individuales sí). |
| `group-effect` | Los efectos de capa en un grupo (de varias capas) no se exportan. |
| `mask-source-not-vector` | La fuente de la máscara no es una capa vectorial, por lo que la máscara no se exporta. |
| `depth-of-field` | Los canales `camera_focus`/`camera_blur` no se aplican; usa `dof-export`. |
| `audio` | Lottie no puede transportar audio; N pistas de sonido no se incluyen. |

### Se descarta sin equivalente en Lottie

Huesos, IK, canales, máquinas de estados, data bindings, containers, regiones de scroll, markers/events, sonidos, cadenas de postefectos (`fx`),
normal maps y la estructura editable del rig. Los colores de efecto por capa no se pueden animar (se usa el primer valor) y
los tamaños están en unidades del escenario (LIMITATIONS L-23).

### dotLottie (`.lottie`)

Un zip con `manifest.json` (`{"animations":[{"id":"animation"}],"generator":"OSATURA","version":"2"}`) y
`a/animation.json` (el mismo Lottie de arriba, comprimido con deflate: 46,843 bytes pasaron a ser un archivo de 11,515 bytes). `--report` funciona de la misma
manera. **Importar** un `.lottie` no es posible (`import-lottie` necesita el JSON; descomprime antes `a/animation.json`).

### Validación y salvedades

* `python3 -m json.tool out.json > /dev/null` para comprobar que está bien formado; `python3 -c "import json;d=json.load(open('out.json'));print(d['v'],d['fr'],d['op'],len(d['layers']))"`.
* `unzip -l out.lottie` debe mostrar exactamente las dos entradas.
* Ida y vuelta: `osatura import-lottie out.json back.json` seguido de `osatura check back.json` verifica que OSATURA puede leer su propia
  salida (la ida y vuelta tiene pérdidas: el rig desaparece).
* **No había ningún reproductor Lottie independiente disponible** cuando se escribió esto (LIMITATIONS L-28): la salida se comprobó con la disposición de campos de Lottie
  y con el propio importador de OSATURA. Pruébalo en tu reproductor de destino (lottie-web, reproductor dotLottie, Skottie, ...) y lee las
  advertencias; si la exactitud importa, usa `.html` o vídeo.
* **Consejos de tamaño:** `export-fit --format lottie` busca entre cuatro parámetros (`decimals`, `path_eps`, `key_tol`, `frame_step`) según un presupuesto de bytes;
  `lod` reduce los anclajes antes de exportar; `size-report` muestra el tamaño de Lottie horneado por capa (sección 13).
* **Patentes:** Lottie y dotLottie son formatos abiertos; nada en PATENT_NOTES.md se refiere a ellos.

## 7. glTF binario (.glb)

```
$ osatura export small.json out.glb
wrote out.glb
warning: [glb-limit] blend mode Screen of 'sun' is not exported (glTF materials only alpha-blend); it is drawn as normal
warning: [glb-limit] animated opacity of 'target' is not exported (frozen at frame 0)
```
glTF 2.0 binario (número mágico `glTF`, versión 2, `"generator": "osatura"`), solo funciones básicas. El proyecto debe tener huesos
(`the project has no bones to export` en caso contrario).

* **Exacto:** la jerarquía de huesos y su animación (muestreada en cada fotograma, interpolación lineal, sin cizallamiento).
* **Los rellenos** se convierten en mallas planas de triángulos (ear clipping). Las formas son rígidas (una capa con un hueso sigue el nodo de ese hueso) o tienen **skinning**
  (JOINTS_0/WEIGHTS_0, 4 influencias por vértice) cuando todos los anclajes llevan pesos de huesos.
* **Los trazos** se trianguan en mallas propias (anchura, perfil de anchura, remates plano/redondo/cuadrado, uniones de inglete/redondas/biseladas, guiones);
  los trazos translúcidos pueden verse más oscuros donde se superponen los triángulos.
* **Los degradados** (lineales y radiales) se convierten en un pequeño PNG generado que se usa como textura de color base; extensión pad (clamp).
* **Las máscaras** se hornean recortando triángulos contra el relleno de la capa de máscara, solo para una fuente de máscara vectorial con rellenos opacos y sin
  trazos, y solo para la forma de máscara del fotograma 0. **Las capas de imagen** se convierten en cuadriláteros texturizados o mallas de deformación con PNG incrustado.
* **Los morphs de ruta** se convierten en morph targets con pesos animados cuando la comprobación por fotograma reproduce la geometría del motor; en caso contrario
  la ruta queda como una forma estática del fotograma 0 y se informa del motivo.
* **No se exporta (se informa con advertencias `[glb-limit]`):** modos de mezcla distintos de normal (se dibujan como normal), opacidad de capa animada (congelada en el
  fotograma 0), animación de transformación de capa que no sea de huesos, efectos de capa, deformadores más allá de su forma del fotograma 0, rutas animadas sin un
  morph válido, formas de máscara animadas, sonido.
* **Coordenadas:** y se niega (glTF tiene y hacia arriba); las formas se apilan con un pequeño desplazamiento en z (0.01 por forma) para el orden de dibujo.
* **Validación:** la cabecera es `glTF`, versión 2, y el campo de longitud es igual al tamaño del archivo (instrucción de `python3` de una línea en la sección 16). Usa un validador
  o visor de glTF de tu elección para la comprobación completa; no se incluye ninguno ni se ejecutó ninguno para este documento. El ejemplo de arriba tiene 22 nodos,
  15 mallas, 1 skin, 1 animación y 7 imágenes incrustadas.
* **Consejos de tamaño:** menos anclajes vectoriales (`lod`), sin degradados (cada uno es una textura), menos fotogramas (la animación se muestrea por fotograma).
* **Patentes:** formato abierto; sin referencia en PATENT_NOTES.md.

## 8. Carpeta Open Cel Animation (OCA)

```
$ osatura export-oca small.json oca1                 # one layer "render" with all composited frames
$ osatura export-oca small.json oca2 --layers        # one OCA layer per top-level layer
wrote oca2/small.oca
```
Escribe `<out_dir>/<project name>.oca` (JSON, `ocaVersion` 1.1) más cels PNG `layerN/fNNNN.png`. Sin `--layers` hay una capa
`render` con los fotogramas compuestos; con `--layers` cada capa de nivel superior se renderiza sola sobre transparencia (11 capas para la demo).
`--scale N` multiplica el tamaño de los cels. **Descartes conocidos:** los cels son rásteres (no sobreviven vectores, huesos ni canales); la opacidad de capa y la mezcla
no se escriben (cada capa es una `paintlayer`, con opacidad 1); los sonidos no se exportan; los archivos dentro de una carpeta existente se sobrescriben.
**Importación:** no hay ningún comando `import-oca` en la CLI. **Validación:** el archivo `.oca` es JSON (`python3 -m json.tool`); todos los cels enumerados
deben existir como PNG.

## 9. Importación: SVG, OpenRaster, Lottie, imágenes, GIF/APNG

Las importaciones crean un proyecto **nuevo** (no se fusionan con uno existente) salvo que se indique lo contrario; usa `script` o edita el JSON para combinarlos.

### SVG (`import-svg`)

```
$ osatura import-svg art.svg svg.json
warning: unsupported paint 'url(#g)'; using grey
warning: <text> is not supported and was skipped
wrote svg.json (3 paths)
```
Subconjunto: `path` (todos los comandos), `rect` (se ignoran las esquinas redondeadas), `circle`, `ellipse`, `line`, `polyline`, `polygon`, `g`/`a` con `transform`,
`fill` sólido, `stroke`, `stroke-width`. Resultado: 24 fps, 24 fotogramas, tamaño tomado de `viewBox` o de `width`/`height` (si no, 512x512), **una capa
vectorial `svg`**. Se omiten con una línea `warning:` (impresa en **stdout**): `<text>`, `<filter>`, `<clipPath>`, `<mask>`, `<pattern>`,
`<linearGradient>`/`<radialGradient>` (una pintura de degradado recurre al gris), `<image>`, `<use>`, `<foreignObject>`. Límites: 8 MB de SVG,
200,000 anclajes. Para fotografías y rásteres usa `trace` / `autotrace` (CLI_REFERENCE sección 9).

### OpenRaster (`import-ora`)

```
$ osatura import-ora art.ora ora.json --fps 12 --frames 24
```
Una **capa de imagen por cada capa ráster** (PNG incrustado), orden de abajo arriba, las capas ocultas se conservan pero invisibles, los grupos se aplanan, se mantienen la opacidad,
la visibilidad y la posición de cada capa, y las operaciones de composición `svg:` habituales (multiply, screen, overlay, darken, lighten, color-dodge,
color-burn, hard-light, soft-light, difference, exclusion, plus asignado a add). La opacidad de una pila anidada se multiplica en sus capas y su
modo de mezcla se hereda, pero las capas superpuestas dentro de una pila translúcida **no** se aplanan primero. Los nombres duplicados reciben `_1`, `_2`.
Límites: 2000 entradas de archivo, 128 MiB por entrada, capas de 64 Mpíxeles. Un archivo que no es un zip: `not a zip/ORA file: ...` (salida 1). No hay exportación
a ORA. La importación de PSD no está implementada (LIMITATIONS L-32).

### Lottie (`import-lottie`)

```
$ osatura import-lottie in.json out.json [fx.json]
warning: layer 'text' of type 5 skipped
imported 1 layers; wrote out.json
```
Un **subconjunto** deliberado; cada función omitida es una línea `warning:`.
* **Se importa:** capas de formas con rutas `sh`, `rc`, `el`; rellenos sólidos y con degradado (los extremos, el centro y el radio animados del degradado se convierten en
  canales `grad_dx`/`grad_dy`); trazos; grupos; transformaciones de capa (posición, escala, rotación y opacidad estáticas o con fotogramas clave; las rutas de posición curvas
  se hornean por fotograma); modos de mezcla; una capa de fondo sólido; capas de imagen con URI de datos PNG/JPEG incrustados; capas nulas como padres
  (sus transformaciones se hornean en los hijos); máscaras de capa (add/subtract/intersect con opacidad, inversión, formas con claves); mattes de luminancia
  (aproximados); precomposiciones; rangos de entrada/salida de capa (como pista de pasos de opacidad); easing de fotogramas clave (hold se convierte en step, los tiradores bezier se convierten en
  cubic-bezier, ninguno se convierte en lineal).
* **Rutas animadas:** las claves de forma con la misma topología (mismo número de anclajes y abierto/cerrado) se convierten en una ruta base más un **morph target**
  por cada clave distinta adicional (`k1`, `k2`, ...) con canales de peso `path_morph` que llevan el easing. Una topología distinta usa la primera forma
  e imprime una advertencia.
* **Efectos (`ef`):** desenfoque gaussiano, sombra paralela, tinte y relleno se convierten en **efectos por capa** con números animados como canales `fx_*`;
  el tinte/relleno/tritono/niveles estáticos en capas de formas se hornean en los colores. Los colores de efecto no se pueden animar (se usa el primer valor); los demás
  efectos son advertencias.
* **Se omite con una advertencia:** texto y otros tipos de capa, recursos de audio, puntos de anclaje animados (se usa el primer valor), rutas de movimiento curvas en
  propiedades distintas de la posición, máscaras en una fuente de track matte, cadenas de padres demasiado largas para hornear, más de 2000 capas, animación de más
  de 20,000 fotogramas.
* `fx.json` se conserva por compatibilidad: se escribe un array vacío.
* **No se admite:** entrada zip `.lottie`. **Ojo:** no hay protección contra la sobrescritura de la entrada: `import-lottie in.json in.json` sustituye el
  Lottie por el proyecto.

### Imágenes, GIF y APNG como capas fotograma a fotograma

```
$ osatura import-seq demo.json is.json frames seq hold=2 loop=on start=4 x=10 y=20      # folder of PNGs
$ osatura import-anim demo.json ia.json gifl dof.gif loop=off start=0                    # animated GIF/APNG
```
`import-seq` toma todos los PNG de una carpeta por orden de nombre de archivo (todos del mismo tamaño); `import-anim` lee un GIF o APNG animado usando los
retardos del propio archivo (un PNG simple da un fotograma). Ambos añaden una capa switch con una imagen incrustada por fotograma más un canal de pasos; funcionan sobre un
proyecto **existente**. Un GIF largo hace grande el proyecto (cada fotograma es un PNG incrustado). `trace` y `autotrace` convierten mapas de bits en rutas
vectoriales (`trace` lee el canal alfa; `autotrace` cuantiza los colores y ajusta curvas Bézier).

## 10. Importación: captura de movimiento BVH, MIDI y otras entradas en directo

### BVH (`import-bvh`, `mocap-info`, `mocap-match`, `mocap-retarget`)

```
$ osatura mocap-info walk.bvh
$ osatura import-bvh demo.json walk.bvh ib.json --joint-map LeftUpLeg=thigh,LeftLeg=shin --axis Zrotation --tol 0.5
wrote ib.json (+2 channels from 4 BVH frames)
```
`import-bvh` asigna los canales de rotación de las articulaciones elegidas a huesos como canales `bone_angle` con claves, muestreados a los fps y al número de fotogramas del proyecto
(`--axis` selecciona `Zrotation`, `Xrotation` o `Yrotation`; `--tol` simplifica las claves). `mocap-match` propone una correspondencia articulación-hueso con una
tabla de confianza y `mocap-retarget` hornea canales añadidos con movimiento de raíz, una vista (`front|side|top|yaw=DEG`) y una escala. El BVH
es solo el subconjunto de texto: sin formatos binarios de mocap, sin FBX (por diseño). **Límites (LIMITATIONS L-45):** no se probó ningún archivo real de captura de movimiento; la proyección
2D descarta la profundidad; se ignoran la torsión sobre el eje de un hueso, los dedos y la cara; el rig conserva las longitudes de sus huesos, por lo que el deslizamiento de los pies solo
se reduce.

### MIDI y otras entradas en directo (`live-map`)

```
$ osatura live-map small.json song.mid mapping.json out.json
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 0 events (0 after the last frame skipped); wrote out.json
```
Un **Standard MIDI File** (`.mid`; mensajes MIDI 1.0, running status, metaeventos de tempo, división en ticks o SMPTE) o un JSON de eventos
(`{"events":[{"source","id","value","time"}]}`, valor normalizado 0..1, tiempo en segundos) se convierte en canales con claves, eventos y takes mediante un
archivo de mapeo. Fuentes: `midi_cc midi_note midi_pitch_bend mouse_x mouse_y mouse_button gamepad_axis gamepad_button key face`. Como máximo
3,000,000 eventos; tiempos dentro de 7 días. Nada se comunica con un dispositivo MIDI desde la CLI; el studio usa Web MIDI del navegador. MIDI es **solo
importación** (no hay exportación MIDI). Los archivos mal formados devuelven un error, nunca un fallo.

## 11. Formatos de audio

| Uso | Formatos | Notas |
|---|---|---|
| Análisis (`lipsync`, `lipsync-text`, `speech-motion`, `audio-react`, `beats`, `waveform`) | solo **WAV** | PCM entero de 8/16/24/32 bits o float de 32 bits, cualquier número de canales mezclado a mono, hasta 256 MiB. Convierte antes MP3/OGG (`ffmpeg -i in.mp3 out.wav`). |
| Pistas de sonido del proyecto (`sound add`) | WAV, OGG, MP3, cualquier cosa que acepte `ffprobe` | Se guarda como una **referencia** (ruta relativa, fotograma de inicio, ganancia 0..4, ventana de recorte, fundidos); no se incrusta nada; el archivo debe permanecer dentro de la carpeta del proyecto salvo con `--allow-external-assets`. |
| Salida mezclada (`sound mix`) | `.wav` (PCM de 44.1 kHz y 16 bits), `.ogg` (Vorbis q4), `.mp3` (96 kb/s) | Mezclada mediante ffmpeg para la **duración del proyecto**; `.flac` y otros: `unknown audio format '.flac' (use .wav, .ogg or .mp3)`. |
| Dentro de las exportaciones | `.mp4` AAC, `.webm` Opus, `.html` mp3 incrustado | `.gif`/`.apng` quedan en silencio con una nota; Lottie advierte `[audio]`; GLB y OCA descartan el sonido. |
| Indicaciones de boca (`mouth`) | texto Papagayo/Moho-switch, TSV de Rhubarb | No es audio: archivos de indicaciones. |

Codificadores usados: `libmp3lame` (mp3), `libvorbis` (ogg), `aac`, `libopus`. Sin panorámica, EQ, ducking ni bucle por pista (ROADMAP sección 9).

## 12. Matriz de compatibilidad de funciones

Las filas son cosas que puede contener un proyecto, las columnas los formatos de salida. **sí** = se transporta como en el proyecto, **aprox.** = se transporta con una
diferencia visible, **horneado** = presente solo como resultados por fotograma, **no** = se descarta. Las celdas de los formatos que no pude comprobar en un visor independiente
describen lo que escribe el exportador.

| Función | SVG / PNG | Hoja de sprites / APNG | GIF | WebM / MP4 | HTML | Lottie / .lottie | GLB | OCA |
|---|---|---|---|---|---|---|---|---|
| Formas vectoriales, rellenos | sí | horneado (ráster) | horneado (ráster) | horneado (ráster) | sí (SVG) | sí | aprox. (triángulos) | horneado (ráster) |
| Degradados | sí | horneado | horneado | horneado | sí | sí (aprox. si la capa está inclinada) | sí (textura) | horneado |
| Trazos con perfil de anchura | sí | horneado | horneado | horneado | sí | aprox. (anchura constante) | sí | horneado |
| Guiones, remates, uniones | sí | horneado | horneado | horneado | sí | sí | sí (triangulado) | horneado |
| Capas de imagen (mapa de bits) | sí | horneado | horneado | horneado | sí | no (`image-layer`) | sí | horneado |
| Huesos y animación | horneado | horneado | horneado | horneado | horneado | horneado | **sí (exacto)** | horneado |
| Skinning / deformadores | horneado | horneado | horneado | horneado | horneado | horneado | con skinning; deformadores en el fotograma 0 | horneado |
| Cámara y paralaje | horneado | horneado | horneado | horneado | horneado | horneado (los movimientos de cámara están en la geometría) | no (las formas proceden del fotograma 0) | horneado |
| Modos de mezcla | sí | horneado | horneado | horneado | sí | aprox. (`add` pasa a `screen`; ninguno en grupos) | no (normal) | horneado |
| Máscaras de capa | sí | horneado | horneado | horneado | sí | solo capas individuales | forma del fotograma 0, restringida | horneado |
| Efectos de capa (desenfoque, sombra...) | sí | horneado | horneado | horneado | sí | capas de una sola hoja | no | horneado |
| Opacidad animada | horneado | horneado | horneado | horneado | horneado | sí | no (congelada en el fotograma 0) | horneado |
| Morphs de ruta | horneado | horneado | horneado | horneado | horneado | horneado | morph targets (cuando son válidos) | horneado |
| Profundidad de campo | mediante `dof-export` | mediante `dof-export` | mediante `dof-export` | mediante `dof-export` | no | no (con advertencia) | no | no |
| Cadena de postefectos (`fx`) | solo PNG | no | no | no | no | no | no | no |
| Transparencia | sí | sí (alfa) | 1 bit | WebM sí, MP4 no | sí | sí | sí | sí |
| Audio | no | no | no | **sí** | sí (mp3) | no (con advertencia) | no | no |
| Bucle | n/a | hoja: n/a, APNG: sí | sí | depende del reproductor | reproductor | reproductor (dotLottie 2.0 deja el bucle al reproductor o a una máquina de estados) | n/a | n/a |
| Rig/canales editables después | no | no | no | no | no | no | solo huesos | no |
| Máquinas de estados, bindings, containers, scroll | no | no | no | no | no | no | no | no |
| Independiente de la resolución | sí (SVG) | no | no | no | sí | sí | sí | no |
| Requiere ffmpeg | no | no | no | **sí** | solo audio | no | no | no |

«Horneado» significa que el efecto de la función está en los píxeles o en la geometría por fotograma, pero la función en sí no puede editarse después.

## 13. Cómo reducir el tamaño de los archivos

1. **Ver dónde están los bytes:** `osatura size-report project.json` enumera el archivo del proyecto, el Lottie horneado y la página HTML por capa,
   imagen y canal (`--json` para scripts).
2. **Ajustarse a un presupuesto automáticamente:** `osatura export-fit project.json out.json --max-size 50KB --format lottie` busca entre los parámetros del exportador y
   se queda con el mejor SSIM que quepa. Formatos: `lottie`, `html`, `gif`, `apng`, `webm`, `project` (un JSON de proyecto aligerado). `--measure gzip` aplica el presupuesto al
   tamaño con gzip, `--min-ssim X` fija un mínimo de calidad, `--max-seconds N` es un límite real de tiempo de reloj (se conserva el mejor resultado hasta el momento). Sale con 1 y el
   resultado más pequeño cuando no se puede cumplir el presupuesto.
3. **Reducir primero la ilustración:** `osatura lod project.json --tolerance-px 1 --levels 3 --out-dir lv --verify` escribe proyectos de nivel de detalle con un
   error declarado (y, con `--verify`, medido) en píxeles de salida; `lod-pick --screen-px WxH` elige el nivel más grueso para una pantalla dada.
4. **Por formato:**
   * Lottie/HTML/GLB crecen con `frames x anchors`: menos fotogramas, menos anclajes, menos fps, rangos de claves recortados (herramientas `keys`).
   * GIF/APNG crecen con píxeles x fotogramas: `--scale 0.5` es una reducción de 4x; GIF encaja con ilustración plana, APNG con alfa, WebM/MP4 con cualquier cosa con movimiento.
   * `.lottie` es el mismo Lottie comprimido con deflate (unas 4 veces más pequeño en el ejemplo); sirve `.html`/`.json` comprimidos con gzip.
   * Las imágenes de un proyecto se incrustan como PNG en base64: reduce su tamaño antes de importarlas.
5. Límites: un Lottie de un rig de 14 capas no se puede hacer arbitrariamente pequeño (un presupuesto de 1 KB en el ejemplo informa «every knob is at its floor»); la
   búsqueda necesita tiempo en proyectos grandes.

## 14. Requisitos de ffmpeg

OSATURA nunca incluye ni enlaza ffmpeg; ejecuta el `ffmpeg` (y `ffprobe`) que encuentra en `PATH` como un **subproceso externo** de tu propia instalación.

| Necesario para | Codificador/decodificador | Comprobado en |
|---|---|---|
| `export .mp4` | `libx264` (`yuv420p`), `aac` | ffmpeg 6.1.1 |
| `export .webm` (por defecto) | `libvpx-vp9`, `libopus` | ffmpeg 6.1.1 |
| `export .webm --codec av1` | `libaom-av1`, `libopus` | ffmpeg 6.1.1 |
| `dof-export .mp4/.webm` | igual que arriba, `--codec` | ffmpeg 6.1.1 |
| `sound mix .ogg` | `libvorbis` | ffmpeg 6.1.1 |
| `sound mix .mp3`, audio de `.html` | `libmp3lame` | ffmpeg 6.1.1 |
| `face-track` a partir de un vídeo | solo decodificadores | |
| comprobación de contenido de `sound add` | `ffprobe` (sin él, solo se comprueba la existencia en archivos que no son WAV) | |

Comprueba el tuyo con `ffmpeg -hide_banner -encoders | grep -E 'libx264|aac|libvpx-vp9|libaom-av1|libopus|libvorbis|libmp3lame'`. Todo lo demás
(GIF, APNG, HTML sin audio, Lottie, dotLottie, GLB, OCA, PNG, SVG, EXR) funciona sin ffmpeg.
Si falta ffmpeg, el comando falla con `cannot run ffmpeg (...)` y no escribe nada; si falta un codificador, da `ffmpeg failed`.

## 15. Referencias de patentes y licencias

* [PATENT_NOTES.md](PATENT_NOTES.md) registra lo que **no** se ha examinado. Para la exportación, la fila relevante es *Video and audio codecs through ffmpeg
  (`export` .mp4 / .webm, `soundtrack`)*: H.264 y AAC están cubiertos por consorcios de licencias; VP9 y AV1 se comercializan como libres de regalías, pero se han anunciado
  reclamaciones de consorcios e individuales para ambos; **estado: no evaluado, no consideres VP9/AV1 como despejados.** Como ffmpeg es la instalación del propio usuario, cada
  elección de códec es responsabilidad del usuario, y OSATURA no debe incluir ffmpeg.
* Los escritores deterministas de ráster, GIF, APNG, PNG, SVG, OpenEXR, OCA, ORA, glTF y Lottie son implementaciones propias de OSATURA a partir de descripciones
  públicas de los formatos (clean-room, véase CLEANROOM.md). Su estado de patentes no se evalúa más allá de eso: PATENT_NOTES.md enumera solo las áreas citadas
  (marionetas controladas por el rostro, máquinas de estados y data binding, FABRIK/MVC/pesos, relleno stencil-then-cover, códecs).
* GIF: las patentes de LZW caducaron hace mucho en todas las jurisdicciones; es conocimiento general, no una autorización registrada en el repositorio.
* Licencias de los archivos producidos: los archivos que exportas son tuyos; el motor es MPL-2.0 y la CLI AGPL-3.0-or-later, lo cual se aplica a los programas, no a los
  archivos que escriben. El studio ofrece su código fuente correspondiente en `/source` (AGPL sección 13).
* Esto no es asesoramiento jurídico.

## 16. Validar lo que has escrito

| Formato | Comprobación | Resultado esperado |
|---|---|---|
| cualquier proyecto | `osatura check out.json` | `ok: ...` (también después de `import-*`) |
| GIF | `ffprobe -v error -count_frames -select_streams v -show_entries stream=nb_read_frames -of csv=p=0 out.gif` | número de fotogramas (8) |
| APNG | el mismo comando sobre `out.apng` | número de fotogramas (8) |
| MP4 | `ffprobe -v error -show_entries stream=codec_name,width,height,pix_fmt,nb_frames,r_frame_rate -of csv=p=0 out.mp4` | `h264,320,240,yuv420p,24/1,8` |
| WebM | lo mismo sobre `out.webm` con `:stream_tags=alpha_mode` | `vp9,...,yuv420p,...,1` (alpha_mode 1) |
| EXR | `ffprobe f.exr` | `exr 640x480 gbrapf32le` |
| Lottie | `python3 -c "import json;d=json.load(open('out.json'));print(d['v'],d['fr'],d['op'],d['w'],d['h'],len(d['layers']))"` | `5.7.0 24.0 8 640.0 480.0 12` |
| Advertencias de Lottie | `osatura export p.json out.json --report rep.json` | `rep.json` enumera cada aproximación |
| dotLottie | `unzip -l out.lottie` | solo `manifest.json` y `a/animation.json` |
| Ida y vuelta de Lottie | `osatura import-lottie out.json back.json && osatura check back.json` | `ok:` |
| GLB | `python3 -c "import struct;b=open('out.glb','rb').read();print(b[:4],struct.unpack('<II',b[4:12]),len(b))"` | `b'glTF' (2, N)` con N igual a la longitud |
| HTML | `grep -c '<svg' out.html`; `grep -o 'http[^"<]*' out.html` | un `<svg` por fotograma; solo la URL del espacio de nombres SVG |
| SVG | abrir en un navegador o analizar como XML | bien formado |
| Hoja de sprites | `python3 -m json.tool out.atlas.json` | `{index,x,y,w,h}` por fotograma |
| OCA | `python3 -m json.tool <name>.oca` | todos los cels referenciados existen |
| Regresión visual | `osatura ci diff a.png b.png --metric ssim` | salida 0 cuando son iguales; véase [CI.md](CI.md) |

Para comprobaciones visuales de una exportación con pérdidas frente al original, `osatura export-fit` ya informa de SSIM y CIEDE2000 respecto al renderizado original, y
`osatura compare` escribe una imagen de antes/después/diferencia de dos proyectos.

## 17. Diferencias con otros documentos

* `docs/LIMITATIONS.md` L-29 dice que la importación de Lottie no admite recursos de imagen, efectos, precomps ni mattes; el importador actual sí importa imágenes
  con URI de datos incrustados, precomposiciones, efectos por capa (desenfoque, sombra paralela, tinte, relleno) y aproxima los mattes de luminancia (sección 9). La lista de
  funciones omitidas de la sección 9 es de lo que el comando realmente advierte.
* `docs/LIMITATIONS.md` L-32 enumera OCA entre los formatos de importación; la CLI tiene una **exportación** a OCA (`export-oca`) y ningún comando `import-oca`.
* El texto de uso de `export` nombra solo `.gif .html .json .mp4` más «los formatos abiertos»; la lista completa de extensiones es la de la sección 2.
* Otras discrepancias entre el texto de uso y el comportamiento se recogen al final de [CLI_REFERENCE.md](CLI_REFERENCE.md).
