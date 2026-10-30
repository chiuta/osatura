<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->

# Formato de proyecto de OSATURA

Esta es la referencia del archivo de proyecto de OSATURA: un único documento JSON que
contiene el escenario, el rig, el arte y la animación. Está escrita a partir de las
estructuras serde de `osatura-core/src/project.rs` y los módulos a los que hacen referencia,
y todos los ejemplos de las secciones 13 a 15 se comprobaron con el binario `osatura`
(`osatura check`, `osatura render`, `osatura explain`).

Contenido

1. [Convenciones y tipos primitivos](#1-convenciones-y-tipos-primitivos)
2. [Carga, versionado y campos desconocidos](#2-carga-versionado-y-campos-desconocidos)
3. [Campos de nivel superior](#3-campos-de-nivel-superior)
4. [Rig: bones, ik, constraints, springs](#4-rig-bones-ik-constraints-springs)
5. [Capas](#5-capas)
6. [Trazados, anclas, relleno, trazo](#6-trazados-anclas-relleno-trazo)
7. [Deformadores, efectos, mapa de normales](#7-deformadores-efectos-mapa-de-normales)
8. [Canales, pistas, easing](#8-canales-pistas-easing)
9. [Expresiones](#9-expresiones)
10. [Cámara, planos, audio, metadatos](#10-cámara-shots-audio-metadatos)
11. [Clips, skins, enlaces, interacción, lods](#11-clips-skins-enlaces-interacción-lods)
12. [Reglas de validación, límites y mensajes de error](#12-reglas-de-validación-límites-y-mensajes-de-error)
13. [Ejemplo 1: pelota que rebota](#13-ejemplo-1-pelota-que-rebota)
14. [Ejemplo 2: brazo con rig e IK](#14-ejemplo-2-brazo-con-rig-e-ik)
15. [Ejemplo 3: boca con capa switch](#15-ejemplo-3-boca-con-capa-switch)
16. [Orden de evaluación](#16-orden-de-evaluación)
17. [Orden de renderizado](#17-orden-de-renderizado)
18. [Recursos y rutas](#18-recursos-y-rutas)
19. [Política de versionado](#19-política-de-versiones)
20. [Discrepancias conocidas entre código y comentarios](#20-discrepancias-conocidas-entre-el-código-y-los-comentarios)

## 1. Convenciones y tipos primitivos

| Tipo | Forma JSON | Notas |
| --- | --- | --- |
| number | número JSON | Se almacena como `f32` salvo que el campo indique entero. Todo float debe cumplir `abs(x) <= 1e7` (sección 12). |
| `Vec2` | `[x, y]` | Dos números. Las unidades del escenario son píxeles, +x a la derecha, +y hacia abajo. |
| `Rgba` | string | `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, o uno de `black`, `white`, `red`, `green` (0,128,0), `blue`, `yellow`, `transparent`. La salida es siempre hexadecimal en minúsculas. |
| `Paint` | string u objeto | Una cadena de color es un relleno sólido. `{"from": [x,y], "to": [x,y], "stops": [...]}` es un degradado lineal. `{"center": [x,y], "radius": r, "stops": [...]}` es un degradado radial. El tipo es un enum sin etiqueta. |
| `Stop` | `{"offset": 0..1, "color": Rgba}` | Las paradas se ordenan (de forma estable) al dibujar y los desplazamientos se limitan a 0..1 en el rasterizador. Un degradado degenerado pinta la última parada. |
| ángulo | number | Grados en todo el archivo (ángulos de huesos, rotación de capa, skew, rotación de cámara, `angle_limits`). |
| tiempo | number | Fotogramas en claves y marcadores, segundos en sonidos, máquinas y emisores. Cada campo indica cuál. |
| nombres | string | Nombres de hueso, capa, trazado, deformador, ik y constraint. Las secciones 5 y 12 enumeran las restricciones de caracteres. |

Un color incorrecto dentro de un contexto de relleno produce el mensaje de serde
`data did not match any variant of untagged enum Paint`, porque `Paint` no tiene etiqueta y serde
no puede decir a qué rama se refería.

Las tablas de campos siguientes usan esta notación: **Req** significa que el campo no tiene valor por defecto y debe estar presente;
en caso contrario se muestra el valor por defecto. «Rango» es lo que imponen el cargador o el motor. Un rango escrito
«limitado» (clamped) no es un error: el motor limita el valor en silencio.

## 2. Carga, versionado y campos desconocidos

`Project::from_json(text)` es el único punto de entrada usado por la CLI, la API de C, el módulo WASM
y el Studio. Hace, por orden:

1. Analiza el JSON con serde. Serde ignora los campos que no conoce.
2. Ejecuta la comprobación de magnitud: cualquier literal float cuyo valor absoluto supere `1e7` hace fallar la carga
   con `invalid JSON in <file>: value ... is outside the supported range of +-1e7 ...`. Los literales enteros
   se detectan más tarde, en la nueva comprobación dentro de `Engine::new` (véase la sección 12).
3. Aplica la puerta de formato: un `format` mayor que el `FORMAT_VERSION` de la compilación (actualmente `1`) se
   rechaza con `project format 2 is newer than this build supports (1); update OSATURA`.

`Engine::new` (o `Engine::with_asset_dir`) valida entonces todo el proyecto y devuelve todos los
problemas que encuentra en una lista. La CLI los imprime así:

```text
<file> is not a valid project:
  - first problem
  - second problem
```

Los textos de error son para personas. No son un contrato estable; compáralos por su texto solo en pruebas.

### Campos desconocidos

Serde descarta las claves desconocidas al cargar, de modo que un error tipográfico como `fil` en lugar de `fill` se perdería en silencio. El módulo
`unknownfields` compara el JSON de entrada con el proyecto reserializado e informa de las claves que
no sobrevivieron:

```text
warning: unknown field layers[0].paths[0].fil (ignored)
```

`osatura check` imprime estas advertencias. Los valores iguales al valor por defecto no se notifican (el escritor
omite los valores por defecto, por lo que un `"visible": true` explícito no supone problema). Algunas estructuras son estrictas en lugar de
permisivas y rechazan las claves desconocidas con un error grave: `Transition` y `Condition` de las máquinas, la configuración de CI,
la configuración de entrada en directo, las superposiciones y la configuración de parámetros faciales; los archivos de configuración se leen con `parse_strict`.

### Ida y vuelta de los metadatos del editor

El proyecto se serializa con `skip_serializing_if` para las listas opcionales. Un archivo que escribas a mano puede
ser más corto que lo que `osatura` escribe de vuelta; ambos son equivalentes.

## 3. Campos de nivel superior

| Nombre JSON | Tipo | Por defecto | Rango / regla | Significado |
| --- | --- | --- | --- | --- |
| `format` | integer | `FORMAT_VERSION` (1) | debe ser `<=` la versión de la compilación | Versión del formato, sección 19. |
| `name` | string | **Req** | cualquiera | Nombre para mostrar. Lo imprime `check`. |
| `width` | number | **Req** | `> 0`, `<= 32768` | Ancho del escenario en píxeles. |
| `height` | number | **Req** | `> 0`, `<= 32768` | Alto del escenario en píxeles. |
| `fps` | number | **Req** | `(0, 1000]` | Fotogramas por segundo. |
| `frames` | integer (`u32`) | **Req** | `1..=200000` | Duración de la línea de tiempo. Los índices de fotograma son `0..frames-1`. Un literal float como `48.0` se rechaza: `invalid type: floating point `48.0`, expected u32`. |
| `background` | `Rgba` o null | null (transparente) | | Color del escenario detrás de todo. |
| `color_space` | string | `"srgb"` (no se escribe) | `srgb`, `linear` | Espacio en el que mezclan los compositores raster y float. `linear`: los colores se convierten a luz lineal, se componen en f32 y se reconvierten una sola vez (un negro al 50 % sobre blanco da 188/255, no 128/255). Se establece con `osatura color-space`. SVG, Lottie, el renderizador de GPU, `dof` y `fx` lo ignoran. |
| `camera` | object | valores por defecto | sección 10 | Cámara global. |
| `bones` | array | `[]` | como máximo 4096 | Esqueleto, sección 4. |
| `ik` | array | `[]` | | Cadenas de IK. |
| `constraints` | array | `[]` | | Aim, copy-rotation, pin. |
| `springs` | array | `[]` | | Movimiento secundario, con bake al cargar. |
| `layers` | array | `[]` | | Capas raíz, dibujadas en el orden de la lista. |
| `channels` | array | `[]` | | Canales de animación, evaluados en el orden de la lista. |
| `markers`, `comps`, `events`, `bone_vis`, `channel_meta`, `shots` | arrays | se omiten si están vacíos | no se validan | Metadatos de la línea de tiempo y del editor, sección 10. |
| `sounds` | array | se omite si está vacío | se valida | Pistas de audio, sección 10. |
| `skins`, `clips`, `clip_baked`, `lods` | arrays | se omiten si están vacíos | no se validan al cargar | Sección 11. |
| `mesh_locks`, `glues` | arrays | se omiten si están vacíos | se validan | Enlaces de malla entre capas, sección 11. |
| `machines`, `bindings`, `containers`, `scrolls` | arrays | se omiten si están vacíos | se validan | Interacción y layout, sección 11. |

Proyecto válido mínimo:

```json
{"name": "empty", "width": 100, "height": 100, "fps": 24, "frames": 1}
```

Magnitudes: todo float del archivo debe cumplir `abs(x) <= 1e7`. Esto se aplica por igual a coordenadas,
escalas, desplazamientos y valores de clave. El ancho y el alto están además limitados a 32768, y la salida
raster a 16384 por lado y 100 megapíxeles (sección 18).

## 4. Rig: bones, ik, constraints, springs

### 4.1 `bones[]`

Los huesos deben enumerarse empezando por los padres. El padre de un hueso debe aparecer antes en la lista.

| Nombre JSON | Tipo | Por defecto | Rango | Significado |
| --- | --- | --- | --- | --- |
| `name` | string | **Req** | único | Referenciado por pesos, ik, canales, `Layer.bone`. |
| `parent` | string o null | null | un hueso anterior | Hueso padre. Null significa raíz. |
| `length` | number | **Req** | finito, `>= 0` | Longitud del hueso en píxeles, a lo largo del eje +x del hueso. |
| `rest_offset` | `Vec2` | `[0,0]` | | Posición de la cabeza en el espacio del padre (espacio del escenario para las raíces). |
| `rest_angle` | number | 0 | | Rotación de reposo en grados relativa al padre. |
| `angle_limits` | `[min, max]` o null | null | `min <= max` | Límite del delta de ángulo animado, en grados. |
| `scale_limits` | `[min, max]` o null | null | `0 < min <= max` | Límite de la escala animada. |

Interacciones: los canales `bone_angle` se suman a `rest_angle`; `bone_offset_x/y` se suman a `rest_offset`;
`bone_scale` multiplica. `clamp_limits` se ejecuta después de todos los canales y antes de FK, y de nuevo después de cada
cadena de IK.

```json
{"name": "upper", "parent": null, "length": 90, "rest_offset": [60, 120],
 "rest_angle": 0, "angle_limits": [-120, 120]}
```

### 4.2 `ik[]`

| Nombre JSON | Tipo | Por defecto | Rango | Significado |
| --- | --- | --- | --- | --- |
| `name` | string | **Req** | único | Destino de los canales `ik_mix`. |
| `bones` | array de strings | **Req** | al menos 2; cada uno hijo directo del anterior | Cadena de la raíz a la punta. |
| `target` | string | **Req** | un hueso, no incluido en la cadena | Hueso cuya cabeza intenta alcanzar la punta. |
| `bend` | number | 1 | +1 o -1 | Hacia qué lado se pliega una cadena de dos huesos. |
| `mix` | number | 1 | limitado a 0..1 | Mezcla inicial de FK a IK; se anima con `ik_mix`. `<= 0` omite la cadena. |
| `iterations` | integer | 16 | | Pasadas de FABRIK para cadenas de más de dos huesos. Es un mínimo: el solver sale antes cuando la punta queda a menos de 0.01 px y, en caso contrario, continúa hasta 256 pasadas. |
| `soft` | number | 0 | `0..0.9` | Fracción del alcance en la que la extensión se suaviza. Con `stretch > 1` la zona es la última fracción del alcance estirado (`reach x stretch`). |
| `stretch` | number | 1 | `1..4` | Factor máximo de estiramiento; por encima de 1 la escala del hueso raíz crece para alcanzar el objetivo. |

Una cadena de dos huesos usa el solver analítico de la ley de cosenos; las cadenas más largas usan FABRIK. Tras resolver,
los ángulos se mezclan en la pose como `old + wrap_pi(new - old) * mix`.

```json
{"name": "reach", "bones": ["upper", "lower"], "target": "target", "bend": 1, "mix": 1}
```

### 4.3 `constraints[]`

Los tipos de restricción usan una etiqueta interna `"type"`, aplanada junto a `name`, `bone` y `weight`.

| Nombre JSON | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `name` | string | **Req** | Destino de los canales `constraint_weight`. |
| `bone` | string | **Req** | El hueso restringido. |
| `type` | `aim`, `copy_rotation`, `pin` | **Req** | Tipo. |
| `target` | string | **Req** para `aim`, `pin` | Hueso al que apuntar o sobre el que deslizarse. |
| `offset` | number | 0 | Solo `aim`: grados adicionales. |
| `source` | string | **Req** para `copy_rotation` | Hueso cuya rotación en el mundo se copia. |
| `weight` | number | 1 | Influencia, limitada a 0..1. `<= 0` omite la restricción. |

Una restricción no puede tener a sí misma como destino. Las restricciones se ejecutan en el orden de la lista después de todas las cadenas de IK.

```json
{"name": "aim", "bone": "root", "type": "aim", "target": "look", "offset": 0, "weight": 0.5}
```

### 4.4 `springs[]`

Un muelle amortiguado que hace que la rotación en el mundo de un hueso se retrase respecto a su rotación animada. Se
simula una vez al cargar y se hace bake, de modo que cualquier fotograma puede evaluarse en cualquier orden.

| Nombre JSON | Tipo | Por defecto | Rango | Significado |
| --- | --- | --- | --- | --- |
| `bone` | string | **Req** | un muelle por hueso | Hueso que se controla. |
| `frequency` | number | 3 | `(0, 30]` Hz | Rigidez. |
| `damping` | number | 0.35 | `>= 0` | Razón de amortiguamiento; por debajo de 1 oscila, 1 es crítico. |
| `warmup` | bool | true | | Precalcula una pasada para que los bucles empiecen en régimen estacionario. |

Los ángulos con bake sustituyen al ángulo del hueso después de las restricciones (paso 8 de la evaluación).

## 5. Capas

Una capa es `{"name": ..., "type": ..., <campos del tipo>, <campos comunes>}`. `type` es la etiqueta de
`LayerKind`; sus campos se aplanan dentro del objeto de la capa.

### 5.1 Campos comunes

| Nombre JSON | Tipo | Por defecto | Rango | Significado |
| --- | --- | --- | --- | --- |
| `name` | string | **Req** | no vacío, sin `/`, sin `::`; la ruta completa debe ser única | Nombre de la capa. Los hijos se direccionan como `parent/child`. |
| `type` | string | **Req** | `vector`, `group`, `switch`, `liquid`, `boolean`, `emitter`, `image` | Tipo. |
| `transform` | object | identidad | | `position` `[0,0]`, `rotation` 0, `scale` `[1,1]`, `skew` 0, `anchor` `[0,0]`. |
| `opacity` | number | 1 | limitada a 0..1 | Se multiplica en la composición de la capa. |
| `visible` | bool | true | | Las capas ocultas se omiten al dibujar pero aún pueden ser fuentes de máscara. |
| `blend` | string | `normal` | véase abajo | Modo de mezcla. |
| `mask` | `{layer, invert}` | null | sin automáscara, sin ciclos | Usa el alfa de otra capa como máscara. |
| `bone` | string | null | un hueso conocido | Se ancla a un hueso: la matriz de skin del hueso se aplica entre la matriz del escenario del padre y la transformación local. |
| `parallax` | number | 1 | | Factor de profundidad de cámara. Solo cuenta el valor de la capa de nivel superior. |
| `deformers` | array | se omite si está vacío | solo capas vector e image | Sección 7. |
| `wobble` | object | null | | `amp`, `scale` (0.03), `rate` (8), `seed` (0): desplazamiento por ruido determinista. Anima `amp` con `layer_wobble`. |
| `reference` | bool | false | | Capa de referencia: oculta salvo que el editor muestre las referencias. |
| `locked` | bool | false | | Bloqueo del editor, véase abajo. |
| `normal_map` | object | null | | Sección 7. |
| `effects` | array | se omite si está vacío | como máximo 16 | Sección 7. |

Matriz local: `T(position) * R(rotation) * Shear(skew) * S(scale) * T(-anchor)`.

Modos de mezcla: `normal`, `multiply`, `screen`, `overlay`, `darken`, `lighten`, `color_dodge`,
`color_burn`, `hard_light`, `soft_light`, `difference`, `exclusion`, `add` (plus-lighter de CSS).

`locked` bloquea únicamente las operaciones del editor: selección en el lienzo, gizmos, desplazamientos, borrados, ediciones de la línea de tiempo y
ediciones de canales (mediante `assert_unlocked`), para la capa y sus descendientes. El renderizado y la
evaluación lo ignoran, y las `Op` simples (sección «Editor ops» de ARCHITECTURE.md) se siguen aplicando.

Los campos comunes se escriben después de los campos del tipo en la salida, pero el orden es irrelevante en la entrada.

### 5.2 `vector`

| Campo | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `paths` | array de trazados | `[]` | Sección 6. |

### 5.3 `group`

| Campo | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `layers` | array de capas | `[]` | Los hijos heredan la transformación del grupo. |

### 5.4 `switch`

| Campo | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `layers` | array de capas | `[]` | Dibujos candidatos. |
| `selected` | integer | 0 | Índice inicial del hijo. Se anima con `layer_switch` (claves de paso). |

Se muestra exactamente un hijo. El valor se redondea; los negativos pasan a 0; los valores iguales o superiores al número de
hijos pasan a ser el último hijo. La validación no rechaza las claves fuera de rango. Véase la sección 20 para una
discrepancia en la redacción de lint sobre esto.

### 5.5 `liquid`

Líquido de metabolas. El campo es la suma de `r^2 / d^2` sobre las gotas; el contorno es la isocurva,
extraída por marching squares sobre una cuadrícula limitada a 600 por 600 celdas y suavizada con Catmull-Rom.

| Campo | Tipo | Por defecto | Rango | Significado |
| --- | --- | --- | --- | --- |
| `blobs` | array de `{pos, radius, bone?}` | **Req** | al menos 1; `radius > 0` | Una gota. `bone` la ancla a un hueso. |
| `threshold` | number | 1 | `> 0` | Nivel de la isocurva; 1 hace que una gota aislada sea un círculo de su radio. |
| `cell` | number | 3 | `>= 0.25` | Celda de la cuadrícula en unidades de capa. |
| `fill` | `Paint` | null | | |
| `stroke` | objeto de trazo | null | | |

### 5.6 `boolean`

| Campo | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `op` | `union`, `intersect`, `difference`, `xor` | **Req** | Operación. |
| `paths` | array de trazados | **Req**, al menos 2 | `paths[0]` es el sujeto; el recorte es la unión del resto. |

El resultado toma el relleno y el trazo del sujeto; la tolerancia de aplanado es 0.25. Los trazados resultantes se
llaman `<nombre del primer trazado><índice>`.

### 5.7 `emitter`

`{"type": "emitter", "spec": {...}}`. Las partículas son una función de forma cerrada del tiempo, por lo que cualquier fotograma
es evaluable. Los tiempos están en segundos.

| Campo de `spec` | Por defecto | Rango | Significado |
| --- | --- | --- | --- |
| `pos` | `[0,0]` | | Posición del emisor (espacio de la capa). |
| `bone` | null | | Se ancla a un hueso. |
| `rate` | 20 | `(0, 2000]` | Partículas por segundo. |
| `lifetime` | 1 | `> 0` | Segundos. |
| `lifetime_jitter` | 0 | 0..1 | |
| `speed` | 60 | | Píxeles por segundo. |
| `speed_jitter` | 0 | 0..1 | |
| `direction` | -90 | | Grados; -90 es hacia arriba. |
| `spread` | 30 | | Grados de anchura del cono. |
| `gravity` | `[0,0]` | | Aceleración, píxeles por segundo al cuadrado. |
| `size_start`, `size_end` | 3, 3 | | Radio a lo largo de la vida. |
| `color_start`, `color_end` | `#fff`, `#fff0` | | Color a lo largo de la vida. |
| `seed` | 0 | | Semilla determinista. |
| `start` | 0 | | Tiempo de inicio en segundos. |
| `duration` | null | | La emisión se detiene tras este número de segundos; null es para siempre. |
| `max_particles` | 2000 | | Tope estricto de partículas vivas. |

### 5.8 `image`

| Campo | Tipo | Por defecto | Rango | Significado |
| --- | --- | --- | --- | --- |
| `source` | `{file?, png_base64?}` | **Req** | uno de los dos | `file` es una ruta relativa al directorio del proyecto (sección 18); `png_base64` es autocontenido. Ninguno: `image has neither 'png_base64' nor 'file'`. |
| `origin` | `Vec2` | `[0,0]` | | Esquina superior izquierda en el espacio de la capa. |
| `size` | `Vec2` o null | null (tamaño en píxeles) | | Tamaño mostrado. |
| `mesh` | objeto o null | null | | Malla de deformación. |

`mesh`: `cols`, `rows` (malla de cuadrícula, cada uno al menos 1), `weights` (pesos de huesos por vértice; vacío
significa estático) y, opcionalmente, `free` = `{vertices, triangles, uvs}`. Una malla libre necesita al menos 3
vértices, al menos 1 triángulo, como máximo 200000 vértices y 400000 triángulos, coordenadas finitas,
índices de triángulo válidos y `uvs` vacío o de la misma longitud que `vertices`. Con deformadores pero
sin malla, se usa una cuadrícula de 16 por 16.

```json
{"name": "img2", "type": "image", "source": {"png_base64": "iVBOR..."},
 "origin": [100, 0], "size": [80, 80],
 "mesh": {"free": {"vertices": [[100,0],[180,0],[180,80],[100,80]],
                   "triangles": [[0,1,2],[0,2,3]],
                   "uvs": [[0,0],[8,0],[8,8],[0,8]]}, "weights": []}}
```

## 6. Trazados, anclas, relleno, trazo

### 6.1 Trazado (`paths[]`)

| Nombre JSON | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `name` | string | `""` | Un nombre vacío se sustituye por `path{index}` para el direccionamiento. No debe contener `::`. Único dentro de la capa. |
| `closed` | bool | false | |
| `anchors` | array | **Req** | Anclas Bezier. |
| `fill` | `Paint` | null | |
| `stroke` | stroke | null | |
| `morphs` | array de `{name, anchors}` | se omite si está vacío | Formas de mezcla aditivas (blend shapes). El número de anclas debe ser igual al del trazado base. El peso se controla con `path_morph`. |

### 6.2 Ancla

| Nombre JSON | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `pos` | `Vec2` | **Req** | Posición. |
| `handle_in`, `handle_out` | `Vec2` | null | Puntos de control en el mismo espacio que `pos` (absolutos, no relativos). Un manejador ausente equivale a `pos` (una esquina). |
| `weights` | array de `{bone, weight}` | se omite si está vacío | Skinning. Cada hueso debe existir y cada peso es finito y `>= 0`. Los pesos se normalizan en la evaluación; vacío significa estático. Los manejadores también reciben skinning. |

### 6.3 Trazo

| Nombre JSON | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `paint` | `Paint` | **Req** | |
| `width` | number | 1 | |
| `cap` | `butt`, `round`, `square` | `round` | |
| `join` | `miter`, `round`, `bevel` | `round` | Un valor desconocido da ``unknown variant `foo`, expected one of `miter`, `round`, `bevel` ``. |
| `dash` | array | se omite | Se usa solo si su longitud es par, al menos 2, y todas las entradas son `> 0`. |
| `dash_offset` | number | 0 | |
| `width_profile` | array de `[t, factor]` | se omite | Multiplicador de ancho lineal por tramos a lo largo de la longitud (`t` en 0..1), constante más allá de los extremos, límite de inglete 2. Un trazo de ancho variable ignora `dash` y `join`. |

Ejemplo de un relleno de degradado radial con un trazo (del ejemplo de la pelota):

```json
"fill": {"center": [-8, -10], "radius": 44,
         "stops": [{"offset": 0, "color": "#ffd9a0"}, {"offset": 1, "color": "#d6336c"}]},
"stroke": {"paint": "#3d2c2e", "width": 2}
```

El linter (`osatura lint`) advierte de problemas de degradado (L011), de claves de switch (L012) y de problemas de datos del editor (L013); véase
`osatura-core/src/lint.rs`.

## 7. Deformadores, efectos, mapa de normales

### 7.1 `deformers[]`

Permitidos solo en capas `vector` e `image`. Cada uno tiene una etiqueta `"type"` y un `name` (único dentro de
la capa, sin `::`). Se ejecutan en el orden de la lista sobre la geometría en espacio de reposo, antes del skinning y antes de
`wobble`. Los desplazamientos son estáticos; anímalos con los canales de subdestino.

| `type` | Campos | Restricciones | Se anima con (destino) |
| --- | --- | --- | --- |
| `lattice` | `cols`, `rows`, `rect` `[x,y,w,h]`, `offsets` | `cols, rows >= 1`; `w, h > 0`; `offsets` vacío o `(cols+1)*(rows+1)` en orden de filas | `lattice_dx/dy` (`layer::name::c,r`) |
| `spine` | `rest` (puntos de la curva), `offsets`, `radius` (borde suave opcional) | al menos 2 puntos | `spine_dx/dy` (`layer::name::i`) |
| `pin` | `pins`, `offsets`, `radius` | de 1 a 64 pins, `radius > 0`, pins no coincidentes | `pin_dx/dy` (`layer::name::i`) |
| `cage` | `rest` (polígono), `offsets` | de 3 a 256 vértices, polígono simple, área distinta de cero | `cage_dx/dy` (`layer::name::i`) |
| `cloth` | como `lattice` | como `lattice` | `lattice_dx/dy` (`layer::name::c,r`) |

Lattice mezcla los desplazamientos con pesos de Bezier-Bernstein (suave); cloth desplaza cada vértice de la cuadrícula
exactamente (bilineal). Pin usa una función de base radial de soporte compacto de Wendland. Cage usa
las coordenadas de valor medio de Floater: el contenido dentro del polígono lo sigue, el contenido fuera se deja
intacto.

```json
"deformers": [
  {"type": "lattice", "name": "lat", "cols": 2, "rows": 2, "rect": [0, 0, 100, 100], "offsets": []},
  {"type": "spine", "name": "sp", "rest": [[0,0],[50,0],[100,0]], "offsets": [], "radius": 30},
  {"type": "pin", "name": "pn", "pins": [[0,0],[100,100]], "offsets": [], "radius": 80},
  {"type": "cage", "name": "cg", "rest": [[0,0],[100,0],[100,100],[0,100]], "offsets": []},
  {"type": "cloth", "name": "cl", "cols": 2, "rows": 2, "rect": [0, 0, 100, 100], "offsets": []}
]
```

### 7.2 `effects[]`

Como máximo 16 por capa. Cada efecto tiene `"type"`, `enabled` (true por defecto) y los campos siguientes. El
destino de canal `fxN` indexa el array incluyendo los efectos desactivados.

| `type` | Campos | Rango |
| --- | --- | --- |
| `blur` | `radius` | 0..64 |
| `drop_shadow` | `dx`, `dy`, `blur`, `color`, `opacity` (1) | desplazamientos de como máximo 1024; blur 0..64; opacity 0..1 |
| `tint` | `black`, `white`, `amount` (1) | amount 0..1 |
| `fill` | `color`, `opacity` (1) | opacity 0..1 |

Se anima con `fx_blur`, `fx_dx`, `fx_dy`, `fx_soft`, `fx_amount` sobre el destino `layer::fxN`.

### 7.3 `normal_map`

`{"data": base64-PNG | "path": file, "fit": "stretch" | "tile" | "exact" (default stretch),
"strength": 0..8 (default 1), "green_down": bool (default false)}`. Necesita `data` o `path`. Alimenta
el posefecto `relight`; los renders normales no lo usan. Como `Layer.normal_map` se ajusta al cuadro delimitador en píxeles de esa capa. `fit` `exact` exige un mapa exactamente del tamaño del fotograma y, en caso contrario, es un error.

## 8. Canales, pistas, easing

### 8.1 Canal

```json
{"target": "ball", "prop": "layer_y", "source": "keys",
 "track": {"keys": [{"frame": 0, "value": 40}, {"frame": 24, "value": 170}]},
 "mode": "replace", "weight": 1}
```

| Nombre JSON | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `target` | string | `""` | Nombre, o forma `layer::sub`. Se ignora para las propiedades de cámara. |
| `prop` | string | **Req** | Una de las propiedades en snake_case de abajo. |
| `source` | `keys`, `expr`, `noise`, `driven` | **Req** | Etiqueta aplanada, con los campos de esa fuente a su lado. |
| `mode` | `replace`, `add`, `multiply` | `replace` | Cómo se combina el valor con el actual. |
| `weight` | number | 1 | Influencia, debe ser finita; mezcla hacia el resultado. |

Combinación: replace es `cur + (v - cur) * w`; add es `cur + v * w`; multiply es
`cur * (1 + (v - 1) * w)`. Los valores no finitos se omiten. Los canales se ejecutan en el orden de la lista, así que un canal
posterior ve los resultados de los anteriores (mediante `get(...)` y `driven`).

Fuentes:

| `source` | Campos |
| --- | --- |
| `keys` | `track`: `{keys: [{frame, value, ease?}], extrapolate?}` |
| `expr` | `expr`: string (sección 9) |
| `noise` | `amp`, `freq` (ciclos por segundo), `seed` (0), `octaves` (1). Salida en `[-amp, amp]`; basada en hash, determinista. |
| `driven` | `input` (destino), `input_prop`, `curve`: una pista cuyos «fotogramas» son valores de entrada (por ejemplo, ángulo del codo a cantidad de abultamiento). |

### 8.2 Propiedades

Hay 39 propiedades. La propiedad decide qué significa `target`.

| Grupo | Propiedades | `target` |
| --- | --- | --- |
| Hueso | `bone_angle`, `bone_scale`, `bone_offset_x`, `bone_offset_y` | nombre del hueso |
| Capa | `layer_opacity`, `layer_x`, `layer_y`, `layer_rotation`, `layer_scale_x`, `layer_scale_y`, `layer_skew`, `layer_switch`, `layer_wobble` | ruta de la capa (`parent/child`) |
| Cámara | `camera_x`, `camera_y`, `camera_zoom`, `camera_rotation`, `camera_focus`, `camera_blur` | se ignora |
| Rig | `ik_mix`, `constraint_weight` | nombre de ik o de constraint |
| Trazado | `path_morph`, `point_dx`, `point_dy`, `grad_dx`, `grad_dy` | `layer::path::morphName`, `layer::path::anchorIndex`, `layer::path::fill.from` / `fill.to` / `stroke.from` / `stroke.to` |
| Deformador | `lattice_dx/dy`, `spine_dx/dy`, `pin_dx/dy`, `cage_dx/dy` | `layer::deformer::c,r` (lattice, cloth) o `layer::deformer::i` |
| Efecto | `fx_blur`, `fx_dx`, `fx_dy`, `fx_soft`, `fx_amount` | `layer::fxN` |

Semántica de los manejadores de degradado: en los degradados lineales `.from` y `.to` mueven el punto inicial y el final en
`(dx, dy)`. En los degradados radiales `.from` mueve el centro y `.to` cambia el radio en `dx`.
Solo los trazados cuyo relleno o trazo es un degradado tienen estos destinos. `layer_switch` requiere una capa
`switch`. Una propiedad desconocida enumera todas las variantes válidas en el error.

Límites al escribir: opacity 0..1, zoom de cámara al menos 0.01, blur de cámara al menos 0, `ik_mix` y
`constraint_weight` 0..1.

### 8.3 Pista y clave

`Track` = `{"keys": [...], "extrapolate": "hold"}`. Una clave es `{"frame": number, "value": number,
"ease": {...}}`. Las claves deben ser finitas y estrictamente crecientes en fotograma:
`channel N ('target'): keys must be finite and strictly increasing in frame`. El ease de una clave
da forma al segmento que empieza en esa clave.

Extrapolación (fuera de la primera y la última clave): `hold` (por defecto), `linear`, `loop`, `ping_pong`,
`loop_offset`.

### 8.4 Easing

`ease` es un objeto con una etiqueta `type`. El valor por defecto es `{"type": "linear"}`.

| `type` | Parámetros | Curva |
| --- | --- | --- |
| `linear` | | `u` |
| `step` | | Mantiene el valor de la clave hasta la siguiente clave. |
| `ease_in` | | `u^2` |
| `ease_out` | | `1 - (1-u)^2` |
| `ease_in_out` | | Smoothstep. |
| `cubic_bezier` | `x1, y1, x2, y2` | Curva al estilo CSS a través de (0,0) y (1,1). |
| `smooth` | | Hermite Catmull-Rom no uniforme a través de las claves vecinas. |
| `spring` | `bounces`, `decay` | `1 - e^(-decay*u) * cos(2*pi*bounces*u) * window`, window = `1 - smooth01((u-0.85)/0.15)`; aterriza exactamente en el objetivo. |
| `back` | `overshoot` | Cúbica con pendiente inicial `3 + overshoot` (limitada a 0 o más). Overshoot 1 alcanza un máximo de 1.037 (u = 2/3); overshoot 2 alcanza un máximo de 1.1317 (u = 0.5556). |
| `bounce` | | Restitución 0.6, tres rebotes, duración total de 3.352 unidades de tiempo. Primer impacto en u = 0.2983; picos de rebote 0.64, 0.8704, 0.953344; termina en 1. |

`back` y `bounce` son derivaciones originales (véase `CLEANROOM.md`).

## 9. Expresiones

Un canal con `"source": "expr"` evalúa una fórmula por fotograma. El código fuente está limitado a 4096 bytes
y a una profundidad de anidamiento de 64; los errores se notifican en la validación con una posición, por ejemplo
`channel 2 ('upper'): expression error: unexpected end of expression`.

Variables: `frame`, `time` (= frame / fps), `fps`, `frames`, `pi`, `t` (= frame / frames).

Operadores (de menor a mayor precedencia): `?:`, `||`, `&&`, comparaciones, `+ -`, `* / %`, unarios `- !`,
`^` (asociativo por la derecha). La división por cero da 0; `%` es euclídeo (`rem_euclid`).

Funciones: `sin cos tan asin acos atan atan2 abs sqrt pow min max clamp lerp smoothstep floor ceil
round fract sign mod exp ln noise` (1 o 2 argumentos) `fbm` (2 o 3) `wiggle` (2 o 3) `saw tri
square deg rad`, y `get("target", "prop")`. El resultado no finito de una función pasa a ser 0.

`get("target", "prop")` lee el valor actual de cualquier propiedad, incluidos los resultados de canales
anteriores. Ambas cadenas deben resolverse en la validación (`channel 2: get("q", "bone_angle"): unknown
target`). Un canal que lee un valor debe ir después del canal que lo escribe.

Ejemplo del proyecto de la pelota:

```json
{"target": "shadow", "prop": "layer_opacity", "source": "expr",
 "expr": "clamp((get(\"ball\", \"layer_y\") - 40) / 130, 0.15, 1)"}
```

Las expresiones de binding (sección 11) usan una forma restringida: la variable es `value`, y las variables de tiempo
y `get()` están prohibidas.

## 10. Cámara, shots, audio, metadatos

### 10.1 `camera`

| Nombre JSON | Tipo | Valor por defecto | Rango | Significado |
| --- | --- | --- | --- | --- |
| `position` | `Vec2` o null | null (centro del escenario) | | Punto del mundo en el centro de la pantalla. |
| `zoom` | número | 1 | limitado a un mínimo de 0.01 | |
| `rotation` | número | 0 | grados | |
| `focus` | número | 1 | | La profundidad `parallax` de la capa que queda nítida (la usan `dof` y `dof-export`). |
| `blur` | número | 12 | finito, `>= 0` | Radio de desenfoque en píxeles por unidad de distancia de paralaje. |

Paralaje: una capa con `parallax = p` ve `zoom = 1 + (zoom - 1) * p`,
`pos = centre + (pos - centre) * p`, `rotation = -rotation * p`. Un `focus` o `blur` no finito
produce `camera focus must be finite and camera blur a finite number >= 0`.

### 10.2 Arrays de metadatos (no se validan al cargar)

Contienen datos del editor y de las herramientas. El motor los acepta como datos; las operaciones
que los usan los comprueban cuando se ejecutan.

| Array | Campos de cada elemento |
| --- | --- |
| `markers` | `name`, `frame`, `end` (opcional; una región cuando está definido) |
| `comps` | `name`, `layers`: mapa de ruta de capa a `{visible, opacity, selected?}` |
| `events` | `name`, `frame`, `data` (cualquier JSON) |
| `bone_vis` | `bone`, `from`, `to` (rango de fotogramas en el que se muestra el hueso) |
| `channel_meta` | `target`, `prop`, `color`, `shy`, `solo`, `lock` (indicadores del editor; `lock` bloquea set-key) |
| `shots` | `name`, `x`, `y`, `zoom`, `rotation` (vistas de cámara guardadas) |

```json
"markers": [{"name": "intro", "frame": 0, "end": 12}, {"name": "hit", "frame": 20}],
"events": [{"name": "step", "frame": 12, "data": {"sound": "step.wav"}}],
"shots": [{"name": "wide", "x": 200, "y": 150, "zoom": 1, "rotation": 0}]
```

### 10.3 `sounds[]`

| Nombre JSON | Tipo | Rango | Significado |
| --- | --- | --- | --- |
| `name` | cadena | único | |
| `file` | cadena | una ruta de recurso (sección 18) | WAV u otro audio admitido. |
| `start` | número | segundos | Posición en la línea de tiempo. |
| `gain` | número | 0..4 | |
| `trim_in`, `trim_out` | número | `trim <= 86400` | Segundos recortados del inicio y del final del archivo. |
| `fade_in`, `fade_out` | número | cabe en el clip recortado | Segundos. |

`check` (no `Engine::new`) informa de un archivo ausente: `sound 's': cannot read sound file ./nofile.wav: ...`.

## 11. Clips, skins, enlaces, interacción, lods

### 11.1 `clips[]` y `clip_baked`

Un clip coloca un `take` reutilizable en la línea de tiempo. El baking de clips añade canales
ordinarios (después de los hechos a mano) y guarda una instantánea en `clip_baked`, de modo que
repetir el baking es idempotente y el trabajo manual nunca se elimina.

| Campo del clip | Significado |
| --- | --- |
| `name`, `start`, `speed`, `loops` | Posición. |
| `take` | `{name, length, tracks: [{target, prop, keys}]}` |
| `mode` | `replace`, `additive`, `multiply` |
| `weight`, `weight_keys` | Influencia; pista animada opcional. |
| `include`, `exclude` | Listas de patrones que seleccionan pistas. |
| `track_weights` | `[{pattern, prop, weight}]` |
| `fade_in`, `fade_out` | Fotogramas. |
| `rename` | `[[from, to]]` cambios de nombre de los destinos. |

### 11.2 `skins[]`

`{name, group, layers, constraints, iks, bones}`. Los skins del mismo `group` son variantes
excluyentes. La escala de hueso por skin se condensa en un canal de pasos `bone_scale` de tipo
multiplicación al hacer el baking. Las reglas de los skins solo se comprueban cuando se ejecutan
operaciones de skin.

### 11.3 `mesh_locks[]` y `glues[]`

Un par de glue usa `VertexRef` = `{layer, path (por defecto 0), index}`; un mesh lock nombra rutas completas con `{layer, path}` y una lista opcional de índices de vértices seguidores (vacía significa todos los vértices).

```json
"mesh_locks": [{"name": "lock", "host": {"layer": "hostm", "path": 0},
                "follower": {"layer": "follm", "path": 0}, "vertices": [], "strength": 1}],
"glues": [{"name": "g", "mode": "rigid", "strength": 1,
           "pairs": [{"a": {"layer": "hostm", "path": 0, "index": 0},
                      "b": {"layer": "s1", "path": 0, "index": 0}}]}]
```

El `mode` de glue es `snap`, `rigid` u `offset`. Ambos se validan con `links::resolve` y se aplican
en el espacio del escenario en orden topológico (paso 12 de la evaluación).

### 11.4 `machines[]` y `bindings[]` (solo en tiempo de ejecución)

Las máquinas de estados y los data bindings los dirige el Player en tiempo de ejecución. Para la
exportación y el baking se convierten en canales `replace` constantes que se añaden al proyecto.
Cada máquina necesita al menos una capa.

```json
"machines": [{"name": "m",
  "inputs": [{"name": "go", "kind": "bool", "default": 0},
             {"name": "fire", "kind": "trigger", "hold": false}],
  "layers": [{"name": "main", "entry": "idle", "mask": [],
    "states": [
      {"name": "idle", "source": {"kind": "region", "marker": "intro"},
       "mode": "loop", "speed": 1, "blend_in": 0.1},
      {"name": "wave", "source": {"kind": "clip", "clip": "wave"},
       "mode": "once", "speed": 1, "blend_in": 0.1}],
    "transitions": [
      {"from": "idle", "to": "wave", "when": [{"input": "go", "op": "eq", "value": 1}],
       "duration": 0.2, "ease": {"type": "ease_in_out"}},
      {"from": "wave", "to": "idle", "exit_time": 1}]}]}]
```

El `kind` de una entrada es `bool`, `number` o `trigger`. El `source.kind` de un estado es `clip`,
`segment`, `region` o `blend`; `mode` es `loop`, `once` o `ping_pong`. `Transition` y `Condition`
rechazan las claves desconocidas.

```json
"bindings": [
  {"name": "hp", "path": "player.health", "expr": "value/100",
   "remap": {"in_min": 0, "in_max": 1, "out_min": 0, "out_max": 1, "clamp": true,
             "curve": {"type": "linear"}},
   "smooth": 0.1,
   "target": {"kind": "prop", "target": "sw", "prop": "layer_opacity"},
   "mode": "replace", "weight": 1},
  {"name": "go", "path": "input.go",
   "target": {"kind": "input", "machine": "m", "input": "go"}}]
```

El `weight` de un binding debe estar dentro de 0..1 (`binding 'b': weight must be within 0..1`). Los
destinos de máquinas y bindings se comprueban mediante una compilación de prueba.

### 11.5 `containers[]` y `scrolls[]`

Diseño unidimensional al estilo flexbox. Campos de un container: `id`, `members`, `axis` (`row`,
`column`), `gap`, `cross_gap`, `padding` (4 números), `justify`, `align`, `wrap`, `rect`. Un `rect`
tiene un `kind`: `fixed` (`x,y,w,h`), `pin`, `aspect` o `hug`. Un miembro tiene exactamente uno de
`layer`, `container` o `spacer`, además de `grow`, `shrink`, `width`, `height`, `align`, `resize`.
Límites: 100000 miembros, profundidad de anidamiento 64. Un scroll tiene `id`, `viewport`,
`content`, `axis`, `offset`, `elastic`, `snap_x`, `snap_y`, `fling`, `clip_layer`, `input`.

```json
"containers": [{"id": "row", "axis": "row", "gap": 4, "padding": [2,2,2,2],
  "justify": "start", "align": "start", "wrap": "none",
  "rect": {"kind": "fixed", "x": 0, "y": 0, "w": 200, "h": 60},
  "members": [{"layer": "c1"}, {"spacer": true, "grow": 1},
              {"layer": "c2", "grow": 0, "shrink": 1, "width": 30, "height": 30,
               "align": "center", "resize": true}]}],
"scrolls": [{"id": "sc", "viewport": {"kind": "fixed", "x": 0, "y": 100, "w": 100, "h": 80},
  "content": "list", "axis": "y", "offset": [0, 10],
  "elastic": {"limit": 40, "stiffness": 180, "damping_ratio": 0.9},
  "snap_x": [], "snap_y": [0, 40], "fling": 0.15, "clip_layer": "mask_rect", "input": "scroll.y"}]
```

Los desplazamientos de los containers se suman a la posición de la capa y se multiplican en la
escala (paso 7 de la evaluación). `apply_clip_masks` añade referencias de máscara al proyecto que
recibe, por lo que modifica la copia del motor.

### 11.6 `lods[]`

Registro de una pasada de nivel de detalle escrita por `osatura lod`: `level`, `tolerance_px`,
`scale`, `declared_px`, `bound_stage`, `path_px`, `mesh_px`, `keys_px`, `cull_px`, `stats`
(`path_anchors`, `mesh_vertices`, `keys`, `layers`, `bytes`) y `notes`. Es documentación de lo que
hizo una pasada; el renderizador nunca lo lee.


## 12. Reglas de validación, límites y mensajes de error

`Engine::new` valida en una sola pasada y devuelve todos los problemas. La tabla enumera lo que se
aplica; los mensajes entre comillas fueron producidos por `osatura check` con archivos rotos a
propósito.

### 12.1 Límites globales

| Regla | Límite | Mensaje |
| --- | --- | --- |
| fps | `(0, 1000]` | `fps must be within (0, 1000]` |
| frames | `1..=200000` | `frames must be within 1..=200000` |
| width, height | `> 0` | `width and height must be > 0` |
| width, height | `<= 32768` | `width and height must be at most 32768` |
| cualquier float | `abs <= 1e7` | `value 100000000.0 at /bones/0/length is outside the supported range of +-1e7 (coordinates, scales, offsets and key values must stay below it)` |
| bones | como máximo 4096 | `at most 4096 bones` |
| format | `<= 1` | `project format 2 is newer than this build supports (1); update OSATURA` |
| tamaño del JSON | CLI `--max-input-bytes`, por defecto 256 MiB; API `MAX_JSON_BYTES` 256 MiB | |

El límite de 1e7 se aplica dos veces. Al cargar, un literal float que lo supera hace fallar el
análisis y el mensaje se envuelve como `invalid JSON in <file>: value ...`. Un literal sin punto
decimal (un entero) supera el análisis y lo detecta la nueva comprobación de `Engine::new`, donde
aparece como un elemento de lista ordinario. El límite existe para que una multiplicación posterior
no desborde `f32` hasta el infinito.

### 12.2 Rig

| Regla | Mensaje |
| --- | --- |
| el padre del hueso debe existir antes | `bone 'lower' has parent 'nope' which is missing or listed after it` |
| la longitud del hueso es finita, `>= 0` | `bone 'lower' has invalid length` |
| `angle_limits` `min <= max`; `scale_limits` `0 < min <= max` | |
| ik necesita 2 huesos | `ik 'reach': needs at least two bones` |
| la cadena ik está enlazada por el padre | `ik 'reach': each bone must be a direct child of the previous one` |
| el destino de ik no está en la cadena | |
| soft, stretch | `ik 'reach': soft must be within 0..0.9 and stretch within 1..4` |
| una restricción no puede tener a sí misma como destino | |
| spring | `spring on 'upper': frequency must be in (0, 30] Hz and damping >= 0`; uno por hueso |

### 12.3 Capas, rutas, geometría

| Regla | Mensaje |
| --- | --- |
| el nombre de capa no está vacío, sin `/`, sin `::` | `layer name 'a/b' must be non-empty and contain neither '/' nor '::'` |
| ruta duplicada | `duplicate layer path 'arm'` (y líneas para los destinos animables duplicados) |
| los nombres de rutas y deformadores no contienen `::` y son únicos dentro de la capa | |
| el número de anclajes del morph es igual al de la ruta base | |
| los pesos de skin nombran huesos conocidos; peso finito, `>= 0` | |
| free mesh | al menos 3 vértices, al menos 1 triángulo, como máximo 200000 / 400000, finito, índices válidos, `uvs` vacío o con el mismo número |
| grid mesh | `cols`, `rows >= 1` |
| máscara | sin auto-máscara; `mask cycle involving layer 'arm'` |
| origen de imagen | `layer 'im': image has neither 'png_base64' nor 'file'` |
| liquid | al menos 1 blob; `threshold > 0`; `cell >= 0.25`; `radius > 0` del blob |
| boolean | al menos 2 rutas |
| emitter | `rate` en `(0, 2000]`; `lifetime > 0`; jitters 0..1 |
| deformadores | solo en capas vectoriales y de imagen; véase la sección 7.1 para cada tipo |
| efectos | como máximo 16; blur y softness 0..64; desplazamientos como máximo 1024; cantidades 0..1 |
| normal map | `strength` 0..8; necesita `data` o `path` |
| cámara | `camera focus must be finite and camera blur a finite number >= 0` |

### 12.4 Canales

| Regla | Mensaje |
| --- | --- |
| el destino se resuelve | `channel 0: unknown target 'zzz' for BoneOffsetX` |
| `layer_switch` en una capa switch | `channel 2: layer_switch needs a switch layer, 'arm' is not one` |
| peso finito | |
| claves finitas y estrictamente crecientes | `channel 0 ('target'): keys must be finite and strictly increasing in frame` |
| la expresión se analiza correctamente | `channel 2 ('upper'): expression error: unexpected end of expression` |
| los destinos y props de `get()` se resuelven | `channel 2: get("q", "bone_angle"): unknown target` |
| la entrada dirigida se resuelve | |

Los nombres de `prop` o `ease` desconocidos producen errores de serde que enumeran todas las
variantes válidas, por ejemplo ``unknown variant `foo`, expected one of ...``. Los tipos enteros
incorrectos producen ``invalid type: floating point `48.0`, expected u32``. Un campo obligatorio
ausente produce ``missing field `frames` ``.

### 12.5 Otras secciones validadas

- Sonidos: `gain` 0..4, `trim` como máximo 86400 s, los fundidos caben dentro del clip recortado,
  nombres únicos. La existencia del archivo la comprueba `check`, no `Engine::new`.
- Máquinas: al menos una capa (`machine 'm': needs at least one layer`); las transiciones y las
  condiciones rechazan las claves desconocidas. Bindings: `binding 'b': weight must be within 0..1`.
  Sus destinos se comprueban compilándolos una vez.
- Containers y scrolls: miembros únicos, como máximo 100000, profundidad de anidamiento 64, reglas
  de rect.
- Mesh locks y glues: referencias resueltas por `links::resolve`.
- Archivos de imagen y audio: sección 18.

### 12.6 Datos del editor: advertencias, y errores solo para números no finitos

`Engine::new` y `check` examinan `skins`, `comps`, `markers`, `events`, `shots`, `channel_meta`, `bone_vis`, `clips` y
`lods` (módulo `editordata`). Para que todo proyecto guardado anteriormente siga cargándose, **solo los valores que no pueden existir en un archivo guardado
son errores**: NaN o infinito en markers, events, shots, `bone_vis`, opacidad de comps, lods y opacidad de capa. Todo lo demás es
una **advertencia** que imprime `check` (y que `lint` notifica como L013): zoom de cámara `<= 0` (el motor lo limita a 0.01), `opacity`
de capa fuera de 0..1 (limitada), `ik_mix` y `constraint_weight` fuera de 0..1 (limitados), valores con clave de `layer_opacity` y
`camera_zoom` fuera de su rango, referencias desconocidas a capas/huesos/canales, nombres duplicados, tiempos fuera de la
animación, skins no válidos (`skins::validate`), clips (`clipstack::validate`, y destinos de pistas de take que no se resuelven,
respetando `rename`), `channel_meta` y `lods`. Los fixtures de `out/` se cargan sin advertencias.

`osatura check` imprime las advertencias de lint de este tipo; el linter completo (`osatura lint`) tiene más reglas (L001 en adelante).

## 13. Ejemplo 1: pelota que rebota

Verificado con `osatura check` (`ok: 'bouncing ball' 320×240 @ 24 fps, 48 frames, 0 bones, 0 ik,
3 layers, 5 channels`) y `osatura render ball.json 7 out.png`. Lo que muestra:

- `layer_y` con claves y easing `bounce` en el primer segmento: la pelota cae desde y=40 hasta el
  suelo en y=170 y rebota tres veces. El primer impacto cae hacia el fotograma 7.2 (24 / 3.352).
  El segmento de 24 a 47 vuelve a subir con `ease_out`.
- Una sombra que sigue a la pelota: `layer_x` y `layer_opacity` son expresiones que leen la pelota
  con `get()`. Se enumeran después de los canales de la propia pelota, porque los canales se
  ejecutan en orden.
- Squash en el contacto: una expresión devuelve 0.8 para `layer_scale_y` cuando la pelota está por
  debajo de y=160.
- Círculo a partir de cuatro anclajes Bézier con longitud de tirador 16.57 (0.5523 veces el radio 30).

```json
{
  "format": 1,
  "name": "bouncing ball",
  "width": 320,
  "height": 240,
  "fps": 24,
  "frames": 48,
  "background": "#f4efe6",
  "layers": [
    {
      "name": "floor",
      "type": "vector",
      "paths": [
        {
          "name": "line",
          "closed": false,
          "anchors": [{"pos": [0, 200]}, {"pos": [320, 200]}],
          "stroke": {"paint": "#3d2c2e", "width": 3}
        }
      ]
    },
    {
      "name": "shadow",
      "type": "vector",
      "paths": [
        {
          "name": "blob",
          "closed": true,
          "anchors": [
            {"pos": [-26, 0], "handle_out": [-26, -5], "handle_in": [-26, 5]},
            {"pos": [0, -6], "handle_in": [-14, -6], "handle_out": [14, -6]},
            {"pos": [26, 0], "handle_in": [26, -5], "handle_out": [26, 5]},
            {"pos": [0, 6], "handle_in": [14, 6], "handle_out": [-14, 6]}
          ],
          "fill": "#00000055"
        }
      ],
      "transform": {"position": [160, 200]}
    },
    {
      "name": "ball",
      "type": "vector",
      "paths": [
        {
          "name": "disc",
          "closed": true,
          "anchors": [
            {"pos": [30, 0], "handle_in": [30, -16.57], "handle_out": [30, 16.57]},
            {"pos": [0, 30], "handle_in": [16.57, 30], "handle_out": [-16.57, 30]},
            {
              "pos": [-30, 0],
              "handle_in": [-30, 16.57],
              "handle_out": [-30, -16.57]
            },
            {
              "pos": [0, -30],
              "handle_in": [-16.57, -30],
              "handle_out": [16.57, -30]
            }
          ],
          "fill": {
            "center": [-8, -10],
            "radius": 44,
            "stops": [
              {"offset": 0, "color": "#ffd9a0"},
              {"offset": 1, "color": "#d6336c"}
            ]
          },
          "stroke": {"paint": "#3d2c2e", "width": 2}
        }
      ],
      "transform": {"position": [160, 40]}
    }
  ],
  "channels": [
    {
      "target": "ball",
      "prop": "layer_y",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 40, "ease": {"type": "bounce"}},
          {"frame": 24, "value": 170, "ease": {"type": "ease_out"}},
          {"frame": 47, "value": 40}
        ]
      }
    },
    {
      "target": "ball",
      "prop": "layer_x",
      "source": "keys",
      "track": {"keys": [{"frame": 0, "value": 60}, {"frame": 47, "value": 260}]}
    },
    {
      "target": "shadow",
      "prop": "layer_x",
      "source": "expr",
      "expr": "get(\"ball\", \"layer_x\")"
    },
    {
      "target": "shadow",
      "prop": "layer_opacity",
      "source": "expr",
      "expr": "clamp((get(\"ball\", \"layer_y\") - 40) / 130, 0.15, 1)"
    },
    {
      "target": "ball",
      "prop": "layer_scale_y",
      "source": "expr",
      "expr": "get(\"ball\", \"layer_y\") > 160 ? 0.8 : 1",
      "mode": "replace"
    }
  ]
}
```

## 14. Ejemplo 2: brazo con rig e IK

Verificado con `osatura check` (3 bones, 1 ik, 3 layers, 2 channels). `osatura explain arm.json
24` informa de que la punta está en (170, 190), igual que el hueso `target`, y `osatura render arm.json 24
out.png --bones` dibuja el esqueleto. Lo que muestra:

- `upper` es una raíz con un límite de ángulo; `lower` es su hijo (`rest_offset` `[90, 0]` es igual a
  la longitud del padre); `target` es un hueso raíz libre al que se dirige la cadena IK.
- `reach` es una cadena IK de dos huesos, por lo que se usa el solucionador analítico.
- `sleeve` tiene skinning: cada anclaje enumera pesos de huesos. `hand` y `goal` se unen con
  `bone`.
- Al mover el hueso `target` con los canales `bone_offset_x` y `bone_offset_y`, el brazo se estira
  hacia él.

```json
{
  "format": 1,
  "name": "rigged arm",
  "width": 320,
  "height": 240,
  "fps": 24,
  "frames": 48,
  "background": "#f4efe6",
  "bones": [
    {
      "name": "upper",
      "parent": null,
      "length": 90,
      "rest_offset": [60, 120],
      "rest_angle": 0,
      "angle_limits": [-120, 120]
    },
    {
      "name": "lower",
      "parent": "upper",
      "length": 80,
      "rest_offset": [90, 0],
      "rest_angle": 0
    },
    {
      "name": "target",
      "parent": null,
      "length": 10,
      "rest_offset": [230, 120],
      "rest_angle": 0
    }
  ],
  "ik": [
    {
      "name": "reach",
      "bones": ["upper", "lower"],
      "target": "target",
      "bend": 1,
      "mix": 1
    }
  ],
  "layers": [
    {
      "name": "arm",
      "type": "vector",
      "paths": [
        {
          "name": "sleeve",
          "closed": true,
          "anchors": [
            {"pos": [60, 106], "weights": [{"bone": "upper", "weight": 1}]},
            {
              "pos": [150, 104],
              "weights": [
                {"bone": "upper", "weight": 0.5},
                {"bone": "lower", "weight": 0.5}
              ]
            },
            {"pos": [228, 110], "weights": [{"bone": "lower", "weight": 1}]},
            {"pos": [228, 130], "weights": [{"bone": "lower", "weight": 1}]},
            {
              "pos": [150, 136],
              "weights": [
                {"bone": "upper", "weight": 0.5},
                {"bone": "lower", "weight": 0.5}
              ]
            },
            {"pos": [60, 134], "weights": [{"bone": "upper", "weight": 1}]}
          ],
          "fill": "#ffb347",
          "stroke": {"paint": "#3d2c2e", "width": 3, "join": "round"}
        }
      ]
    },
    {
      "name": "hand",
      "type": "vector",
      "bone": "lower",
      "paths": [
        {
          "name": "palm",
          "closed": true,
          "anchors": [
            {"pos": [246, 120], "handle_in": [246, 111], "handle_out": [246, 129]},
            {"pos": [232, 134], "handle_in": [241, 134], "handle_out": [223, 134]},
            {"pos": [218, 120], "handle_in": [218, 129], "handle_out": [218, 111]},
            {"pos": [232, 106], "handle_in": [223, 106], "handle_out": [241, 106]}
          ],
          "fill": "#ffd9a0",
          "stroke": {"paint": "#3d2c2e", "width": 3}
        }
      ]
    },
    {
      "name": "goal",
      "type": "vector",
      "bone": "target",
      "paths": [
        {
          "name": "ring",
          "closed": false,
          "anchors": [{"pos": [222, 120]}, {"pos": [238, 120]}],
          "stroke": {"paint": "#d6336c", "width": 3, "dash": [4, 4]}
        }
      ]
    }
  ],
  "channels": [
    {
      "target": "target",
      "prop": "bone_offset_x",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 0, "ease": {"type": "ease_in_out"}},
          {"frame": 24, "value": -60, "ease": {"type": "ease_in_out"}},
          {"frame": 47, "value": 0}
        ]
      }
    },
    {
      "target": "target",
      "prop": "bone_offset_y",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 0, "ease": {"type": "ease_in_out"}},
          {"frame": 24, "value": 70, "ease": {"type": "ease_in_out"}},
          {"frame": 47, "value": 0}
        ]
      }
    }
  ]
}
```

## 15. Ejemplo 3: boca con capa switch

Verificado con `osatura check` y renderizando los fotogramas 0, 5 y 9, que muestran tres bocas
distintas. Las claves fuera de rango se limitan: un valor de clave de 7 muestra el hijo 2 y -3
muestra el hijo 0.

```json
{
  "format": 1,
  "name": "switch mouth",
  "width": 200,
  "height": 160,
  "fps": 12,
  "frames": 25,
  "background": "#f4efe6",
  "layers": [
    {
      "name": "face",
      "type": "vector",
      "paths": [
        {
          "name": "head",
          "closed": true,
          "anchors": [
            {"pos": [180, 80], "handle_in": [180, 35], "handle_out": [180, 125]},
            {"pos": [100, 150], "handle_in": [145, 150], "handle_out": [55, 150]},
            {"pos": [20, 80], "handle_in": [20, 125], "handle_out": [20, 35]},
            {"pos": [100, 10], "handle_in": [55, 10], "handle_out": [145, 10]}
          ],
          "fill": "#ffd9a0",
          "stroke": {"paint": "#3d2c2e", "width": 3}
        }
      ]
    },
    {
      "name": "mouth",
      "type": "switch",
      "selected": 0,
      "layers": [
        {
          "name": "closed",
          "type": "vector",
          "paths": [
            {
              "name": "line",
              "closed": false,
              "anchors": [{"pos": [75, 110]}, {"pos": [125, 110]}],
              "stroke": {"paint": "#3d2c2e", "width": 4, "cap": "round"}
            }
          ]
        },
        {
          "name": "open",
          "type": "vector",
          "paths": [
            {
              "name": "oval",
              "closed": true,
              "anchors": [
                {"pos": [75, 106]},
                {"pos": [125, 106]},
                {
                  "pos": [112, 132],
                  "handle_in": [122, 132],
                  "handle_out": [102, 134]
                },
                {"pos": [88, 132], "handle_in": [98, 134], "handle_out": [78, 132]}
              ],
              "fill": "#7a1f3d",
              "stroke": {"paint": "#3d2c2e", "width": 3, "join": "round"}
            }
          ]
        },
        {
          "name": "round",
          "type": "vector",
          "paths": [
            {
              "name": "o",
              "closed": true,
              "anchors": [
                {
                  "pos": [112, 118],
                  "handle_in": [112, 110],
                  "handle_out": [112, 126]
                },
                {
                  "pos": [100, 130],
                  "handle_in": [106, 130],
                  "handle_out": [94, 130]
                },
                {"pos": [88, 118], "handle_in": [88, 126], "handle_out": [88, 110]},
                {
                  "pos": [100, 106],
                  "handle_in": [94, 106],
                  "handle_out": [106, 106]
                }
              ],
              "fill": "#7a1f3d",
              "stroke": {"paint": "#3d2c2e", "width": 3}
            }
          ]
        }
      ]
    }
  ],
  "channels": [
    {
      "target": "mouth",
      "prop": "layer_switch",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 0, "ease": {"type": "step"}},
          {"frame": 4, "value": 1, "ease": {"type": "step"}},
          {"frame": 8, "value": 2, "ease": {"type": "step"}},
          {"frame": 12, "value": 1, "ease": {"type": "step"}},
          {"frame": 16, "value": 0, "ease": {"type": "step"}}
        ]
      }
    }
  ]
}
```

## 16. Orden de evaluación

`Engine::evaluate(frame)` es una función pura del número de fotograma y del proyecto. Los springs se
simulan una sola vez al construir el motor; nada más conserva estado entre fotogramas. Los pasos:

1. **Estado inicial** (`initial_state`): transformaciones estáticas de las capas, opacidad limitada a 0..1, `selected`
   de switch, amplitud de wobble, cámara, mezcla de `ik`, pesos de las restricciones y desplazamientos iniciales de los
   slots de parámetros (desplazamientos de huesos, desplazamientos de deformadores, pesos de morph, etc.).
2. **Canales** (`apply_channels`) en el orden de la lista, cada uno combinándose con el valor actual según su
   `mode` y `weight`. Los resultados no finitos se omiten. Las escrituras se limitan como se indica en la sección 8.2.
3. **Limitación de límites** (`clamp_limits`): `angle_limits` y `scale_limits` por hueso.
4. **Cinemática directa**: matrices de mundo de todos los huesos, primero los padres.
5. **Cadenas IK** en el orden de la lista, omitiendo las cadenas con `mix <= 0`. Tras cada cadena se limita la pose
   y se recalcula la FK, de modo que la siguiente cadena ve el resultado.
6. **Restricciones** en el orden de la lista, omitiendo `weight <= 0`: aim, copy_rotation, pin.
7. **Containers de diseño**: los desplazamientos se suman a x e y de la capa y se multiplican en la escala.
8. **Springs baked**: los ángulos baked sustituyen al ángulo del hueso y después se recalcula la FK.
9. **Matrices de skin**: `skin_mat = world * inverse(rest world)` por hueso.
10. **Cámara**: una matriz de cámara por cada capa raíz, usando el `parallax` de la capa raíz (sección 10.1).
11. **Nodos** en preorden. La matriz de escenario de un nodo es `parentStage * boneAttach(skin_mat) * local`.
    La geometría vectorial se construye en este orden:
    1. morphs (blend shapes aditivos);
    2. desplazamientos `point_dx/dy` por anclaje;
    3. desplazamientos de los tiradores de degradado (`grad_dx/dy`);
    4. deformadores en el orden de la lista, y después `wobble`;
    5. skinning con pesos normalizados (pesos vacíos significan estático; los tiradores de degradado
       también reciben skinning).
    La geometría con skinning vive en coordenadas de reposo del escenario, y la transformación de la capa se aplica después.
    Un switch muestra un hijo (redondeado, limitado). La geometría de liquid, boolean y emitter se
    sintetiza en cada fotograma. Una imagen recibe una malla grid o libre; con deformadores pero sin malla recibe una
    cuadrícula de 16 por 16.
12. **Mesh locks y glues** en el espacio del escenario, en orden topológico.
13. **Reparación de valores no finitos** (`repair_nonfinite`): un nodo cuya geometría contiene NaN o infinito
    se oculta y se activa `Frame.nonfinite`. `evaluate_checked` y `check_all_frames_finite` lo convierten
    en un error.

Consecuencias en las que puedes confiar:

- Un canal solo ve los canales anteriores a él en la lista (mediante `get` y `driven`).
- IK sustituye a FK para los huesos de la cadena, en proporción a `mix` (la mezcla es
  `old + wrap_pi(new - old) * mix`). Con `mix` 0 la cadena es FK pura.
- Las restricciones se ejecutan después de IK, así que una restricción aim sobre un hueso IK prevalece.
- Los springs actúan en último lugar sobre el ángulo del hueso, después de IK y de las restricciones.

## 17. Orden de renderizado

- Las capas raíz se dibujan en el orden de la lista, y los hijos en el orden de la lista dentro de su padre.
- Un nodo que no es `visible` se omite (aun así puede ser una fuente de máscara).
- Un nodo se renderiza en una capa fuera de pantalla cuando tiene `opacity < 1`, efectos, un modo de mezcla distinto
  de `normal`, o una máscara. Pasos: contenido; efectos de capa en el orden de la lista (sobre un lienzo con relleno); máscara
  (la capa de máscara se dibuja con su propia transformación; su alfa, incluida su opacidad, multiplica;
  `invert` lo invierte); composición con opacidad y modo de mezcla.
- La profundidad de campo la aplican únicamente `dof` y `dof-export`: radio = `abs(parallax - focus) *
  blur`, como máximo 24 píxeles por defecto.
- La superposición de huesos (`--bones`) se dibuja en último lugar.

## 18. Recursos y rutas

Los proyectos pueden hacer referencia a archivos: `image.source.file`, `normal_map.path`, `sounds[].file`.

- Las rutas deben ser relativas, no deben contener `..` y, tras seguir los enlaces simbólicos, deben permanecer dentro del
  directorio del proyecto: `layer 'im': asset '/etc/passwd': absolute paths are not allowed (a project
  may only reference files inside its own directory; use --allow-external-assets to override)`, y
  `layer 'im': asset '../x.png': '..' is not allowed (...)`.
- `--allow-external-assets` levanta esa restricción para un solo comando. Solo se abren archivos regulares.
- Límites de tamaño: imágenes 64 MB en disco, audio 256 MB. Imágenes decodificadas: cada lado como máximo 16384 y
  como máximo 64 megapíxeles.
- Límites de salida: ráster como máximo 16384 por lado y 100 megapíxeles; secuencias como máximo 2 GiB; presupuesto
  de exportación de fotogramas por anclajes como máximo 40 millones.
- Cuando la CLI escribe un proyecto, lo vuelve a validar mediante `Engine::new` y reescribe las rutas de los recursos
  relativas a la ubicación de salida.

Prefiere `png_base64` para un archivo autocontenido.

## 19. Política de versiones

- `format` es un entero, actualmente `1` (`FORMAT_VERSION`).
- Un archivo con un `format` mayor que el que admite la compilación se rechaza con el mensaje de la sección 12.1;
  el motor no adivina.
- Añadir un campo opcional con un valor por defecto no es un cambio de versión. Las compilaciones anteriores lo ignoran (y
  `check` lo advierte como campo desconocido).
- Eliminar o reinterpretar un campo exige un incremento de versión, y ese incremento exige un paso de migración en
  `Project::from_json` que actualice los documentos antiguos antes de la validación.
- La API de la biblioteca Rust tiene su propia versión, `API_VERSION` 1.4.0, protegida por el test de instantánea
  `api_surface.txt`; la ABI de C tiene `ABI_VERSION` 1. Véase `docs/API.md`.

## 20. Discrepancias conocidas entre el código y los comentarios

Se encontraron al escribir esta referencia.

1. **Corregido.** La regla de claves de switch (`lint::switch_key_warnings`, L012) ahora dice lo que hace el motor: una clave menor que 0 muestra el hijo 0,
   una clave igual o superior al número de hijos muestra el último hijo (un switch vacío no muestra nada). Prueba: tests unitarios de `lint`, verificados
   contra el motor.
2. El comentario de `project.rs` dice que los archivos sin `format` son la versión 1, pero el valor por defecto de serde es
   `FORMAT_VERSION`. Hoy coinciden solo porque la constante vale 1. Cuando se aumente la constante, un
   archivo sin `format` se leería como la versión más reciente.
3. **Corregido.** `unknownfields` ya no informa de una estructura vacía escrita explícitamente (`clips[].weight_keys: {"keys": []}`);
   `default_like` es recursivo, y un test fija el conjunto de estructuras opcionales que el escritor omite cuando están vacías.
4. **Corregido (como advertencias).** Los campos del editor se validan; véase la sección 12.6.
5. **Corregido en la oleada 7 (L-71).** La geometría de una capa con skinning está en el espacio de reposo del escenario y la transformación del padre se aplica después
   de ella. `Layer.bone` ahora se une con `world = bone_skin * parent_stage * local`: la matriz de skin (mundo del hueso x reposo inverso) es
   un movimiento en el espacio del escenario, por lo que se aplica después de toda la cadena de padres. En reposo la matriz de skin es la identidad, así que nada
   se mueve; cuando el hueso está **en pose**, un grupo padre con escala 2 ya no duplica el recorrido de una capa unida. Una capa
   sin hueso propio sigue al hueso de su ancestro más cercano unido a un hueso; el hueso propio de una capa sustituye al del ancestro
   (los dos movimientos no se apilan). Pruebas: `bone_attach_under_a_transformed_group_composes_bone_after_the_group`,
   `bone_attach_is_inherited_and_a_child_bone_replaces_the_parents`. Los proyectos que dependían del antiguo recorrido duplicado (un hueso
   en pose bajo un grupo escalado o rotado) se renderizan de forma diferente; los proyectos sin esa combinación no cambian.
