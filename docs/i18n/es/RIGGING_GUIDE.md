# Guía de rigging de OSATURA

Esta guía cubre todo lo que hace que un dibujo se mueva: huesos, restricciones, cinemática inversa,
skinning, deformadores, física, auto-rigging, simetría, cambio de vista, rigs faciales y trabajo de cámara.
Cada ejemplo se ejecutó con el binario real (`osatura 0.1.0`, compilación de depuración, Linux) y las cifras
citadas son las que imprimió. Si aún no has construido un primer proyecto, empieza por
[GETTING_STARTED.md](GETTING_STARTED.md). Las opciones de cada comando están en
[CLI_REFERENCE.md](CLI_REFERENCE.md); el formato de archivo está en [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

Licencia de este documento: CC-BY-4.0.

![Ocho renders pequeños: una tira con skinning doblada por un hueso, tres variantes de IK (objetivo fuera de alcance, stretch 2, soft 0.5) y cuatro deformadores vectoriales (cage, lattice, spine, pin), cada uno movido en el fotograma 12.](../../img/rigging-sheet.png)

*Hoja de contacto de los ejemplos siguientes: pesos de skin con FK, FABRIK con un objetivo inalcanzable,
`stretch = 2`, `soft = 0.5`, y los deformadores cage, lattice, spine y pin.*

## 1. Dos formas de editar un rig

* **Comandos y `script`.** La mayoría de las ediciones de rig tienen un comando (`autorig`, `cage`, `physics`, `skin`...) o
  una función Rhai en `osatura script` (`add_bone`, `add_ik`, `add_aim`, `add_copy_rotation`, `add_pin`,
  `set_layer`, `set_bone`, `set_parent`, `delete_bone`...). Un script es de todo o nada: si se rechaza cualquier
  petición, no se escribe nada. Un script puede contener como máximo 5000 peticiones.
* **JSON.** Algunas cosas no tienen comando ni función de script para modificarlas: el `soft` y el `stretch` de la IK, los muelles,
  los deformadores vectoriales (lattice, spine, pin) y los pesos de skin por ancla. Edita el JSON del proyecto (con cualquier editor, un
  script de Python o el panel JSON del estudio) y ejecuta `osatura check` después.

`check`, `lint`, `explain` y `render --bones` son los cuatro comandos que usarás constantemente mientras haces rigging.
`explain` es el más valioso: enumera, en orden, cada valor de reposo, canal, restricción y contribución de IK
que produjo la transformación final de un hueso, de modo que nunca tienes que adivinar por qué algo está donde está.

```
$ osatura explain con1.json b3 0
- bone 'b3' (length 80), parent 'b2'
- ancestry: b3 ← b2 ← b1
- rest pose: offset (80.00, 0.00), angle 0.00°
- no channels animate this bone directly
- constraint 'b3_aims' (Aim { target: "target", offset: 0.0 }) at weight 1.00
- result at frame 0: head (203.6, 101.4), tip (173.5, 27.3), world angle -112.1°
```

## 2. Huesos y jerarquías

Un hueso tiene un `name`, un `parent` (ninguno = raíz), una `length`, un `rest_offset` (dónde está su cabeza, en el
sistema de referencia **del padre**; para un hueso raíz es el lienzo) y un `rest_angle` en grados relativo al
padre. Convenciones:

* y apunta hacia abajo; el ángulo 0 apunta a lo largo de +x; los ángulos positivos giran en sentido horario en pantalla; **90 es abajo, -90 es arriba**.
* `rest_offset` se rota según el ángulo en el mundo del padre. Un hijo de un hueso que apunta hacia arriba y tiene
  `rest_offset` (70, 0) se sitúa 70 unidades *a lo largo del padre hacia arriba*, no 70 unidades a la derecha del lienzo.
* Los huesos deben enumerarse empezando por los padres. Cualquier otra cosa se rechaza:
  `bone 'a' has parent 'ghost' which is missing or listed after it`.

Los canales de animación de un hueso son **deltas respecto a la pose de reposo**: `bone_angle`, `bone_offset_x`,
`bone_offset_y`, `bone_scale`. Un valor de clave de 0 significa «reposo».

Una cadena mínima de 3 huesos usada por la mayoría de los ejemplos siguientes (`lab.json`; se genera, véase la sección 4):

```
b1  root, length 80, head at (60,150), angle 0
b2  child of b1, rest_offset (80,0)
b3  child of b2, rest_offset (80,0)
```

Comandos útiles: `reparent <in> <out> <bone> <new_parent|none>` conserva la pose de reposo (lo comprobé: `b3`
permanece con la cabeza en (220,150) al reasignarlo a none), `set_parent` en un script es la forma de más bajo nivel, y
`bone-vis <in> <out> <bone> <from> <to>` muestra un hueso en las superposiciones solo durante un rango de fotogramas.

### Anclaje rígido: el rig más sencillo

Una capa puede viajar sobre un hueso de forma rígida con `set_layer("layer", "bone", "bone_name")`. Así están hechos el
personaje del tutorial y el bípedo de más abajo: piezas separadas, cada una anclada a un hueso. No necesita
pesos y sobrevive a cualquier exportación. Elígelo cuando tu arte ya esté cortado en piezas.

**Trampa.** Ancla las capas mientras los huesos están en su pose de reposo y dibuja cada pieza de modo que su articulación quede
sobre la cabeza de su hueso. Una pieza dibujada unas unidades desplazada oscilará alrededor del punto equivocado.

**Trampa.** No pongas el ángulo de reposo de los huesos de los brazos en ±100 «para obtener una pose en T» con un desplazamiento de reposo de 0: los
dos brazos parten de un mismo punto. Da a cada hombro un `rest_offset` lateral y un ángulo de 180 o
0, como en el bípedo (`add_bone("arm_upper_l", "torso", 50, 62, 22, 180)`).

## 3. Restricciones

Las restricciones se enumeran en el array `constraints` del proyecto y se ejecutan **en el orden de la lista** después de los
canales. Créalas con `add_aim`, `add_copy_rotation` y `add_pin` (script) o en JSON.

| Restricción | Script | Efecto |
|---|---|---|
| Aim | `add_aim(name, bone, target[, offset, weight])` | rota `bone` para que apunte a la cabeza del hueso `target` |
| Copy rotation | `add_copy_rotation(name, bone, source, weight)` | `bone` toma (una parte ponderada de) la rotación en el mundo de `source` |
| Pin | `add_pin(name, bone, target, weight)` | mueve la cabeza de `bone` sobre la cabeza de `target` |

Verificado con la cadena de 3 huesos, siendo `target` un hueso en (250,40):

```
add_aim("b3_aims", "b3", "target", 0, 1);        // weight 1   -> world angle -74.7 deg
add_aim("b3_aims", "b3", "target", 0, 0.5);      // weight 0.5 -> world angle -37.4 deg
```

El peso mezcla linealmente entre el ángulo animado y el ángulo restringido, de modo que 0.5 reduce el giro a la mitad.

Los límites de ángulo son propiedades del hueso (`set_bone(name, "angle_min", -20); set_bone(name, "angle_max", 20)`).
Acotan la rotación **animada**: da a `b1` una clave de 60 grados y se queda en 20:

```
$ osatura explain l2.json b1 12
- channel #0 BoneAngle: keys → 60.000 (2 key(s), Hold outside range); mode replace, weight 1.00
- angle limits: animated rotation is clamped to [-20, 20]°
- result at frame 12: head (60.0, 150.0), tip (135.2, 177.4), world angle 20.0°
```

**Trampa: el orden importa.** Una copy-rotation colocada en el *padre* de un hueso con aim interactúa con el aim.
Si un hueso acaba apuntando a un sitio extraño, ejecuta `explain` sobre él y sobre su padre, lee la lista de contribuciones
de arriba abajo y luego reordena el array `constraints`.

## 4. Cinemática inversa

```
add_bone("goal", "", 10, 200, 60, 0);
add_ik("chain", ["b1","b2","b3"], "goal", 1);   // name, bones root->tip, target bone, bend
```

* Dos huesos: un solver analítico. `bend` (+1 o -1) elige hacia qué lado se pliega la articulación.
* Tres o más huesos: FABRIK. `iterations` es un mínimo; el solver se detiene antes si la punta queda a menos de
  0.01 px y, en caso contrario, se ejecuta hasta un tope estricto.
* `mix` es la mezcla FK/IK (0 = la pose FK con claves, 1 = IK completa). Se puede animar con la propiedad
  `ik_mix`, que es como un brazo cambia entre FK e IK. `autorig` añade cadenas con `mix 0` para que la animación
  FK existente siga funcionando.
* El objetivo es la **cabeza** de un hueso; no le pongas arte y dale una longitud pequeña (`goal`, `hand_ctrl`).

### Objetivos fuera de alcance: recto, soft, stretch

La cadena `b1..b3` tiene un alcance de 240 desde su raíz (60,150). Al mover el goal a (330,60), a 285 de distancia:

| Ajuste | Resultado (posición de la punta) |
|---|---|
| por defecto | totalmente extendida hacia el goal, punta (287.7, 74.1), exactamente a 240 de la raíz |
| `"stretch": 2` | la cadena se alarga, la punta llega a (330.0, 60.0) |
| `"soft": 0.5` (3 huesos) | la cadena permanece doblada, punta (258.8, 83.7); no se endereza |

`soft` (de 0 a 0.9) y `stretch` (de 1 a 4) son campos de la cadena de IK en el JSON del proyecto; no hay
comando ni función de script para modificarlos:

```
python3 - <<'EOF'
import json
d = json.load(open('03-ik.json'))        # the Buddy project of GETTING_STARTED.md
d['ik'][0]['soft'] = 0.5                 # or: d['ik'][0]['stretch'] = 1.5
json.dump(d, open('ik_soft.json', 'w'))
EOF
osatura check ik_soft.json
```

En el brazo de Buddy (alcance 80 desde el hombro en (160,140)) con el goal en (250,140), a 90 de distancia:

```
soft 0,   stretch 1    tip (240.0, 140.0)    straight, 10 short of the goal
soft 0.5, stretch 1    tip (228.5, 140.0)    held back, elbow still bent
soft 0,   stretch 1.5  tip (250.0, 140.0)    arm lengthened, goal reached
```

La IK soft en una extremidad de dos huesos se aproxima a la extensión completa de forma asintótica: a medida que el goal se aleja a 100, 120,
135 y 150 de distancia, la x de la punta se estabiliza en 231.1, 234.6, 236.3 y 237.4 y nunca llega a 240. Este es el
comportamiento deseado de «sin salto de rodilla». Con `stretch` también por encima de 1, la zona soft es la última fracción `soft` del
alcance estirado (`reach x stretch`): la cadena se alarga de forma continua y se suaviza hacia su longitud máxima (antes de la wave 7
`soft` se ignoraba más allá del alcance completo y la cadena saltaba a recta en ese punto).

**Trampa.** `lint` informa de los objetivos inalcanzables como información, no como errores:
`Info L003 [leg_ik_l] target 'foot_goal_l' is out of reach at frame 12 (148 px > 144 px); the chain will be fully stretched`.
Lee esas líneas; una pierna que «flota» es casi siempre una de ellas.

**Trampa.** `fuzz` (véase [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)) muestrea poses aleatorias, incluidas posiciones de controladores
muy fuera de alcance, por lo que un rig con IK muestra un recuento alto de `ik_unreachable`. Eso es una afirmación
sobre el rango muestreado, no un error.

## 5. Skinning

OSATURA tiene cuatro mecanismos de skinning; elige según el tipo de arte que tengas.

### 5.1 Anclas vectoriales con pesos (JSON)

Las anclas de un trazado vectorial pueden llevar `weights`, una lista de `{bone, weight}`. El
ancla sigue entonces una mezcla lineal de las transformaciones de los huesos. Esta es una tira de 12 anclas con skinning repartido entre los
tres huesos; los `weights` pasan de `b1` a `b2` y a `b3`:

```
python3 - <<'EOF'
import json
d = {"format":1,"name":"Lab","width":400,"height":300,"fps":24,"frames":24,"background":"#f6f1e7"}
d["bones"] = [{"name":"b1","parent":None,"length":80,"rest_offset":[60,150],"rest_angle":0},
              {"name":"b2","parent":"b1","length":80,"rest_offset":[80,0],"rest_angle":0},
              {"name":"b3","parent":"b2","length":80,"rest_offset":[80,0],"rest_angle":0}]
def w(x):
    t = (x - 60) / 80.0
    if t <= 0.5: return [{"bone":"b1","weight":1.0}]
    if t < 1:    a = (t-0.5)/0.5; return [{"bone":"b1","weight":1-a},{"bone":"b2","weight":a}]
    if t < 1.5:  return [{"bone":"b2","weight":1.0}]
    if t < 2:    a = (t-1.5)/0.5; return [{"bone":"b2","weight":1-a},{"bone":"b3","weight":a}]
    return [{"bone":"b3","weight":1.0}]
xs  = list(range(60, 301, 20))
top = [{"pos":[x,138],"weights":w(x)} for x in xs]
bot = [{"pos":[x,162],"weights":w(x)} for x in reversed(xs)]
d["layers"] = [{"name":"strip","type":"vector","paths":[{"name":"strip","closed":True,"anchors":top+bot,"fill":"#e07a5f"}]}]
ease = {"type":"ease_in_out"}
d["channels"] = [{"target":"b2","prop":"bone_angle","source":"keys","track":{"keys":[
    {"frame":0,"value":0,"ease":ease},{"frame":12,"value":50,"ease":ease},{"frame":24,"value":0,"ease":ease}]}}]
json.dump(d, open('lab.json','w'), indent=1)
EOF
osatura check lab.json && osatura render lab.json 12 lab12.png --bones
```
```
ok: 'Lab' 400×300 @ 24 fps, 24 frames, 3 bones, 0 ik, 1 layers, 1 channels
wrote lab12.png
```

En el fotograma 12, el ángulo en el mundo de `b3` es de 50.0 grados y su cabeza está en (191.4, 211.3) (según `explain`). La tira se dobla
suavemente entre las articulaciones porque los pesos se solapan ahí.

**Trampa.** No existe ningún comando que calcule los pesos vectoriales por ti: la API del proyecto tiene un skinner
vectorial automático (`Project::auto_skin`), pero es una función de biblioteca, usada por la demo, y no está expuesta como
comando de la CLI. Para arte vectorial usa el anclaje rígido (sección 2) o escribe los pesos como se muestra arriba.

**Trampa.** Pocas anclas, pocos sitios por donde doblar. Un rectángulo simple tiene cuatro anclas, así que solo puede doblarse por
sus esquinas. Subdivide la forma (añade anclas a lo largo de los bordes) antes de aplicar skinning o un deformador.

### 5.2 Mallas de imagen: `mesh-trace` y `skin-diffuse`

Para arte raster, traza la silueta alfa en una malla de triángulos y luego calcula los pesos por difusión de calor:

```
osatura mesh-trace flag.json flag_m.json flag density=12
```
```
wrote flag_m.json: 369 vertices (178 on the outline), 558 triangles, area 17237 px2
  quality min 0.480 mean 0.867, min angle 19.5 deg, mean edge 3.0 px at the outline / 10.4 px inside
ok: ...
```
```
osatura skin-diffuse flag_m.json flag flag_s.json --influences 4
```

`skin-diffuse <project> <layer> <out> [--influences N]` asigna a cada vértice de la malla hasta N pesos de huesos. Necesita
una malla: en una capa de imagen que no la tiene se detiene con `layer 'flag' has no mesh`. Con un solo hueso
cada vértice recibe peso 1.0 sobre él (la bandera tiene un hueso, `mast`), lo cual es correcto y poco
interesante; usa al menos dos huesos para ver una mezcla. `mesh-trace` también tiene `hotspot=x,y,r` para hacer la malla más
densa donde esperas que haya flexión, y opciones para aplicar skinning mientras traza (`bones=R`, `skin=1`; véase
[CLI_REFERENCE.md](CLI_REFERENCE.md)).

### 5.3 Conjuntos de skin (atuendos, expresiones, estados de daño)

El comando `skin` es una idea distinta: un **conjunto de skin** intercambia grupos enteros de capas (y, opcionalmente,
huesos, restricciones y cadenas de IK) a la vez.

```
cat > hats.rhai <<'EOF'
add_rect("hat_red", 138, 62, 44, 14, "#c1121f");
add_rect("hat_blue", 138, 62, 44, 14, "#1d4e89");
add_bone("tail", "body", 40, 0, 0, 180);
add_rect("tail_shape", 100, 185, 60, 8, "#3d405b");
set_layer("hat_red", "bone", "neck");
set_layer("hat_blue", "bone", "neck");
set_layer("tail_shape", "bone", "tail");
EOF
osatura script blob.json k0.json hats.rhai
osatura skin k0.json k1.json add hat red hat_red
osatura skin k1.json k2.json add hat blue hat_blue
osatura skin k2.json k3.json bones hat red tail        # the tail exists only with the red hat
osatura skin k3.json k4.json apply hat blue            # show blue, hide red
osatura skin k4.json k5.json schedule hat 0:red,12:blue
osatura check k5.json
```

`apply` muestra un skin y oculta los demás del mismo grupo. `schedule` anima la elección con **claves de paso (step) sobre la opacidad de la capa** (el proyecto no tiene ningún indicador «visible» animable), así que úsalo en capas cuya opacidad no animes de otro modo.
Los huesos por skin inactivos no se pueden eliminar del rig sin romper los índices, así que se **colapsan**:
un canal `multiply` sobre `bone_scale` con claves de paso 0 y 1 hace que la escala del hueso sea 0, y todo lo que viaja
sobre él o está ponderado a él se encoge hasta un punto:

```
{"target":"tail","prop":"bone_scale", ... "keys":[{"frame":0,"value":1,"ease":{"type":"step"}},
                                                  {"frame":12,"value":0,"ease":{"type":"step"}}], "mode":"multiply"}
```

Reglas impuestas por la validación: un hueso pertenece como máximo a un skin; todo descendiente de un hueso de skin pertenece al mismo skin; los huesos de skin
no deben tener `scale_limits`. Usa `skin rig <group> <name> <constraint,...|-> <ik,...|->` para las restricciones y cadenas de IK que
solo deban estar activas con un skin.

### 5.4 Piezas rígidas más una capa switch

Para lo que cambia de forma de manera discreta (bocas, manos, ojos), pon las alternativas en una capa `switch`
y anima `layer_switch` con claves. Véase la sección 11.

## 6. Deformadores

Los deformadores mueven los puntos de una capa. Se enumeran en los `deformers` de la capa, se animan mediante
canales con un destino de tres partes `"layer::deformer::point"` y actúan sobre las **anclas** de un trazado o los vértices de
una malla de imagen. **Si una forma tiene muy pocas anclas, apenas se deformará: subdivídela primero.**

### Cage (coordenadas de valor medio)

```
$ osatura cage lab0.json c1.json strip auto verts=3 pad=10
added cage 'cage' (12 vertices) to 'strip'; wrote c1.json
$ osatura cage-pose c1.json c2.json strip 12 move=1:0,-40 ease=ease_in_out
keyed cage 'cage' on 'strip' at frame 12; wrote c2.json
```

`cage` envuelve una capa en un polígono (`auto` = límites con relleno, `hull` = envolvente convexa, `verts=N` = segmentos por
lado); el contenido interior sigue los vértices del polígono, el exterior no se toca. `cage-pose` anima con claves los desplazamientos
de vértices (canales `cage_dx`, `cage_dy` con el destino `strip::cage::1`). `timeline` los enumera bajo
«Deformers and paths». Un canal bloqueado se rechaza.

### Lattice, spine, pin (JSON)

Estos tres no tienen comando; añádelos a los `deformers` de una capa y anímalos con canales:

```
"deformers": [{"type":"lattice","name":"ffd","cols":4,"rows":1,"rect":[60,138,240,24]}]
"channels":  [{"target":"strip::ffd::2,0","prop":"lattice_dy","source":"keys",
               "track":{"keys":[{"frame":0,"value":0},{"frame":12,"value":-40}]}}]

"deformers": [{"type":"spine","name":"curve","rest":[[60,150],[180,150],[300,150]],"radius":40}]
"channels":  [{"target":"strip::curve::1","prop":"spine_dy", ...}]

"deformers": [{"type":"pin","name":"pins","pins":[[60,150],[180,150],[300,150]],"radius":90}]
"channels":  [{"target":"strip::pins::2","prop":"pin_dy", ...}]
```

* **Lattice**: una cuadrícula de forma libre. El id del punto es `col,row`. Bueno para squash y stretch.
* **Spine**: una polilínea con un `radius` de influencia; mover un punto de la spine dobla el arte cercano (colas, tentáculos).
* **Pin**: deformación tipo puppet-pin; cada pin tira de los puntos dentro de su `radius`.

Cada uno de ellos se renderizó correctamente en la hoja de contacto de arriba al aplicarse a una tira sin pesos de huesos. Un trazado
que *además* tenga skinning de huesos será deformado por ambos; quita los pesos de la tira de prueba si quieres ver un
deformador por sí solo.

### Deformar una selección: `warp-pivot`, `soft-scale`, `layer-pivot`

```
osatura warp-pivot c1.json w1.json strip 12 deformer=cage verts=1-2 rotate=25 pivot=centroid radius=60 falloff=smooth ease=ease_in_out
osatura soft-scale c1.json w2.json strip 12 deformer=cage scale=1.5
osatura soft-scale c1.json w3.json strip 12 height=60 aspect=free
osatura layer-pivot lab0.json w4.json strip pivot=60,150
```

`warp-pivot` rota, escala o mueve una selección de puntos de control (cage, lattice, cloth, pin, spine o una
malla de imagen en cuadrícula) alrededor de un pivote (`x,y`, `bone:NAME`, `centroid`, `center`) y anima con claves el resultado. `radius` añade una
atenuación de selección suave (`linear`, `smooth`, `gauss`, `sharp`, `constant`); el ejemplo anterior seleccionó los vértices 1-2 y
la atenuación añadió el 7 y el 8, así que se animaron con claves 8 canales. `soft-scale` hace un redimensionado proporcional (`aspect=free` permite que los
dos ejes difieran). `layer-pivot` mueve el pivote de una capa sin mover su arte.

### Bloqueos de malla y glue

* `mesh-lock add <in> <out> <name> host=LAYER follower=LAYER [verts=...] [strength=1]`: los vértices del follower
  siguen la superficie del host mediante desplazamiento baricéntrico. Úsalo para mantener un tatuaje, un parche o un botón sobre una
  tela que se deforma. Los ciclos se rechazan. `mesh-lock list` imprimió `lock: patch:0 follows host:0 vertices all strength 1`.
* `glue add <in> <out> <name> a=LAYER b=LAYER auto=RADIUS|pairs=i:j;i:j [mode=rigid|snap|offset]`: se sueldan pares de vértices
  de dos capas para que `b` siga a `a`. `glue list` imprimió
  `weld: Rigid strength 1 host:0#1->edge:0#0 host:0#2->edge:0#3`.

### Cloth

Cloth necesita una malla de **cuadrícula**. Una malla de forma libre trazada se rechaza:
`cloth needs a grid mesh; the layer has a free-form (traced) mesh`. Pide una cuadrícula con `cols=` y `rows=`:

```
$ osatura cloth flag.json flag_c.json flag pin=top gravity=900 wind=140,0 gust=0.5 cols=8 rows=10 from=0 to=35
baked cloth on 'flag' (180 channels); wrote flag_c.json
```

Ese bake tardó 0.43 s en la compilación de depuración y produjo 180 canales con claves (6480 claves), de modo que el resultado es totalmente
editable, se exporta como cualquier otra animación y es determinista. `pin=` acepta `top`, `left`, `corners`, `none`,
`row:N`, `col:N`, `v:N` o `...@bone` para fijarlo a un hueso. `cols`/`rows` fijan la cuadrícula de simulación; la imagen renderizada se deforma con ella
y puede que veas finas costuras triangulares en un render PNG con mucho zoom.

## 7. Física y muelles

### Física con bake: `physics`

`physics` simula una cadena de huesos (gravedad, viento, colisiones) y hace el bake del resultado en claves de ángulo de hueso normales.

```
osatura physics lab0.json ph1.json b2,b3 preset=tail gravity=600 --allow-empty
osatura physics lab0.json ph2.json b2,b3 preset=hair circle=200,230,30
osatura physics lab0.json ph3.json b2,b3 gravity=600 capsule=150,200,250,220,15 wind=40,0 gust=0.5
osatura physics lab.json  ph4.json b2,b3 gravity=300 drive=b1:0.5 vortex=b1:0.8:0.2:120
```

* `preset=` es uno de `hair`, `tail`, `rope`, `antenna`, `cape`.
* `circle=x,y,r[@bone];...` y `capsule=x1,y1,x2,y2,r[@bone];...` son colisionadores; `@bone` hace que un colisionador siga a un hueso.
* `drive=bone:gain` hace que el movimiento de otro hueso empuje la cadena; `vortex=bone|x,y:swirl[:radial[:radius]]`
  añade un campo de fuerza en remolino; `follow=bone` y `lateral=` dan forma a cómo la cadena sigue a un padre.
* `merge=keep` (por defecto) **no sobrescribe** un hueso que ya tiene animación:
  `kept existing animation on 'b2' (use merge=replace to overwrite)`. Di `merge=replace` cuando lo quieras de verdad.
* Las cadenas del estilo `b2,b3/b1` y un hueso desconocido (`no bone 'b9'`) se validan.

### Muelles en vivo: `springs`

Un `Spring` añade un seguimiento amortiguado (follow-through) a un hueso en tiempo de evaluación, sin bake. Es solo JSON:

```
"springs": [{"bone":"b3","frequency":4.0,"damping":0.3}]
```

`frequency` está en Hz (más alta es más rígida), `damping` es una razón (por debajo de 1 oscila, 1 es amortiguamiento crítico) y `warmup`
(activado por defecto) precalcula una pasada para que los bucles empiecen en régimen estacionario. `explain` dice:
`spring: world rotation lags behind via a 4.0 Hz, damping 0.30 oscillator (pre-rolled for loops)`.

Para igualar una sensación que hayas dibujado, ajústala:

```
$ osatura spring-fit --start 0 --samples 0.3,0.7,1.1,1.2,1.05,0.95,1.0,1.02 --fps 24
frequency 3.36 Hz, damping 0.52 (rms error 0.055 of the step size)
add to the project: {"bone": "<bone>", "frequency": 3.36, "damping": 0.52}
```

**Trampa.** Los muelles no forman parte de Lottie. Haz su bake (`physics`) si exportas a un formato que no tenga
equivalente; `export --report` enumera lo que se aproximó.

## 8. Auto-rig, etiquetado y `coach`

Nombra tus huesos y capas con palabras estándar (`arm_upper_l`, `thigh_r`, `eye_l`, `mouth`) y OSATURA puede
reconocer sus roles:

```
$ osatura tag biped.json | head -8
bone  hips                     -> hips           (90%)
bone  torso                    -> torso          (80%)
bone  head                     -> head           (90%)
bone  arm_upper_l              -> arm.upper.l    (90%)
bone  arm_lower_l              -> arm.lower.l    (90%)
...
no bone found for: neck, face.eye.l, face.eye.r, face.brow.l, ...
```

`autorig <in> <out> [seed]` añade entonces automatización no destructiva a partir de esos roles:

```
$ osatura autorig biped.json ar.json 7
+ IK arm_l -> arm_l_ctrl (mix 0, bend +1)
+ IK arm_r -> arm_r_ctrl (mix 0, bend +1)
+ IK leg_l -> leg_l_ctrl (mix 0, bend -1)
+ IK leg_r -> leg_r_ctrl (mix 0, bend -1)
+ breathing on torso
+ blink on eye_l
+ blink on eye_r
wrote ar.json
```

El proyecto pasó de 11 huesos y 0 cadenas de IK a 15 huesos y 4 cadenas de IK con 3 canales nuevos. Todo lo
añadido son datos normales: elimina lo que no quieras. Las partes que faltan se omiten con un motivo
(`- arm_l: upper/lower bone not found`, `- breathing: no torso bone`, `- blink: no eye layers`), así que lee el informe.

**Trampa.** `autorig` es un comando de proyecto a proyecto y la salida puede ser la entrada. En un archivo que te importe,
escribe en un nombre nuevo: `osatura autorig buddy.json buddy.json` reemplaza `buddy.json` (la escritura es atómica, pero
no hay deshacer).

`coach <project>` lee la animación e imprime sugerencias de timing y de principios; `lint` comprueba el rig
de forma estática. Ambos son orientativos.

## 9. Simetría y espejo

`mirror <in> <out>` refleja todos los canales con claves a través del eje vertical: los destinos intercambian lados (prefijos `L_`/`R_`,
sufijos `_L`/`_R` y `_l`/`_r`, las palabras left/right) y las rotaciones y traslaciones horizontales
cambian de signo. Los canales reflejados **reemplazan** los canales del proyecto. En el bípedo caminando, las claves del muslo izquierdo
`0.0, 10.7, 19.8, 25.9` pasaron a ser las del muslo derecho `-0.0, -10.7, -19.8, -25.9` y viceversa, y `hips` (sin lado)
conservó sus valores. `diff` enumera los seis canales modificados:

```
$ osatura mirror walk.json mir.json
wrote mir.json
$ osatura diff walk.json mir.json
~ channel 'arm_upper_l:BoneAngle' changed (track)
~ channel 'arm_upper_r:BoneAngle' changed (track)
...
6 difference(s)
```

**Trampa.** En un proyecto sin nombres de izquierda/derecha, `mirror` escribe en silencio una copia sin cambios y dice
`wrote`. Haz siempre un `diff` del resultado. Nombra los lados de forma coherente (`_l`/`_r`) antes de animar.

## 10. Cambio de vista y cambio de padre

### Giros de personaje: `view-swap`

Un turnaround (frontal, tres cuartos, perfil) es una capa switch con un hijo por vista, controlada por una propiedad de un hueso
controlador:

```
cat > ctl.rhai <<'EOF'
add_bone("view_ctrl", "", 10, 20, 20, 0);
EOF
osatura script biped.json v0.json ctl.rhai
osatura view-swap v0.json v1.json mouth view_ctrl.bone_offset_x "-40:0,0:2,40:4"
```

Canal de salida: un canal `driven` `layer_switch` ← `view_ctrl.bone_offset_x` con una curva de paso (hijo 0 por debajo de 0,
hijo 2 desde 0, hijo 4 desde 40). Anima el `bone_offset_x` de `view_ctrl` para cambiar de vista. Los índices de hijo deben existir y la capa
debe ser un switch (`child index 5 is out of range (the layer has 3)`).

`slider <in> <out> <name> "at:target.prop=val,...;at:..."` crea un **slider de poses**, un hueso controlador `ctrl_<name>` cuyo
`offset_x` mezcla entre poses almacenadas:

```
$ osatura slider biped.json sl.json look "0:head.bone_angle=0;24:head.bone_angle=15"
controller bone: ctrl_look (animate its offset_x)
```

### Cambiar de padre con el tiempo: `parent-switch`

Para pasar un objeto de una mano a otra (o para recoger un accesorio) sin saltos:

```
$ osatura parent-switch biped.json ps.json arm_upper_r 0:torso,12:head
wrote ps.json
```

Hace el bake de los canales de desplazamiento y ángulo del hueso (aquí 3 canales, una clave en cada fotograma) para que el hueso conserve su posición
en el mundo en el cambio. No funciona con huesos que tengan IK, restricciones o muelles.

## 11. Rigs faciales

### Formas de la boca

Dibuja las formas de la boca como hijos de una **capa switch** en este orden, que es el que asumen todos los comandos de lip-sync:
**rest, small, open, round, wide**. La boca del bípedo es `add_switch("mouth", ["m_rest","m_small","m_open","m_round","m_wide"])`.
Si tu switch tiene menos hijos, `--map 0,1,2,3,4` (cinco números, uno por forma) elige qué hijo usa cada forma; el valor por defecto
sigue el número de hijos (1 hijo: todas; 2: cerrada+abierta; 3: cerrada+abierta+redonda/ancha; 4: cerrada+pequeña+abierta+redonda/ancha; 5+: en ese orden).

Ancla el switch al hueso de la cabeza (`set_layer("mouth", "bone", "head")`) para que gire con la cabeza. Animarlo es
el tema de la [sección de lip-sync de la guía de animación](ANIMATION_GUIDE.md#8-lip-sync-y-habla).

### Ojos y recorte

`eye-clip <in> <out>` enmascara las capas de iris y pupila con la forma del ojo del mismo lado, de modo que una pupila no pueda salir del
ojo. `autorig` lo ejecuta. En un rig sin capas de iris/pupila escribe el archivo sin cambios. Los parpadeos son un
`layer_scale_y` con claves o procedural en la capa del ojo:

```
$ osatura idle blink p1.json i1.json --target eye_l --prop layer_scale_y --interval 2 --seed 4 --closed 0.1
wrote i1.json
```

`idle blink|breath|sway` generan canales procedurales con semilla (`--interval`, `--amp`, `--period`, `--freq`,
`--seed`, `--closed`): la misma semilla da los mismos instantes de parpadeo en cualquier máquina.

### Controlar una cara a partir de una interpretación

`face-synth`, `face-track`, `face-bake` y el panel Face del estudio convierten un flujo de puntos de referencia en canales. Se
tratan, con cifras de precisión honestas, en la [sección de seguimiento facial de la guía de animación](ANIMATION_GUIDE.md#10-seguimiento-facial-y-sus-límites).

## 12. Cámara, planos y parallax

* `shot <in> <out> <name> <x> <y> <zoom> [rotation]` guarda un encuadre de cámara con nombre (centro de cámara y zoom);
  `camera-seq <in> <out> <frame:shot[:cut],...>` anima la cámara con claves a través de los planos guardados; `timeline` muestra los
  canales de cámara resultantes.
* `parallax <in> <out> <far_layer,...,near_layer> <far_value> <near_value>` reparte un factor `parallax` entre las
  capas de nivel superior (las lejanas se mueven menos cuando la cámara hace un paneo).
* `dof <in> <frame> <out.png> [--focus 1.0 --strength 12 --max-radius 24]` desenfoca cada capa de nivel superior en función de
  `|parallax - focus|`; `dof-export` renderiza un rango a un GIF, APNG o carpeta. El foco y la intensidad siguen los canales
  `camera_focus` / `camera_blur` a menos que los fijes.

Verificado en el bípedo caminando:

```
osatura shot walk.json sh1.json wide 200 180 1          # name x y zoom
osatura shot sh1.json sh2.json close 200 100 2.2
osatura camera-seq sh2.json sh3.json 0:wide,12:close,23:wide
osatura parallax sh3.json px.json sky,ground 0.2 1.0
osatura dof px.json 12 dof.png --focus 1.0 --strength 10
osatura dof-export px.json dof.gif --from 0 --to 11 --scale 0.5     # 80 KB
```

Los efectos de capa (blur, drop shadow, tint, fill, outline, posterize...) pueden aplicarse por capa o como una cadena: véase la
[guía de animación](ANIMATION_GUIDE.md#15-efectos-de-capa).

## 13. Una lista de comprobación de rigging

1. `check` después de cada cambio estructural; `lint` antes de animar.
2. Mantén la pose de reposo aburrida y simétrica; anima deltas respecto a ella.
3. Pon los controladores (goals de IK, `view_ctrl`, `root_ctrl`) en el rig como huesos sin arte.
4. Prefiere piezas rígidas para el arte de dibujos animados, aplica skinning solo a lo que deba doblarse y subdivide antes de deformar.
5. Usa `explain` cuando algo vaya mal; es más rápido que adivinar.
6. Haz el bake de la física y de cloth si la animación debe sobrevivir en un formato sin esas funciones, y lee `export --report`.
7. Escribe en nombres de archivo nuevos hasta que confíes en un comando; los comandos de proyecto a proyecto pueden sobrescribir su entrada.

## 14. Límites que conviene conocer

* El skinning del arte vectorial es de **mezcla lineal** (sin cuaterniones duales), con pesos que proporcionas tú; no existe ningún skinner
  vectorial automático en la CLI.
* Lattice, spine, pin, el `soft`/`stretch` de la IK y los muelles son solo JSON.
* La física y cloth son bakes: si cambias el rig, vuelve a hacer el bake. `physics` no sobrescribe la animación existente a menos que se le indique.
* Cloth necesita una malla de cuadrícula; las mallas de forma libre trazadas sirven para skinning y bloqueos de malla.
* Lottie y GIF no pueden representarlo todo (trazos de ancho variable, degradados sesgados, muelles...). Las advertencias se
  imprimen al exportar; véanse [EXPORT_FORMATS.md](EXPORT_FORMATS.md) y [LIMITATIONS.md](LIMITATIONS.md).
* El detector facial está entrenado solo con caras sintéticas (sección 11 y la guía de animación).
