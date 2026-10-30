# Guía de animación de OSATURA

Cómo hacer que las cosas se muevan, reutilizar el movimiento, controlarlo a partir de audio, caras, mocap y entrada en directo, comprobarlo y
distribuirlo con poco peso. Todos los comandos de aquí se ejecutaron con el binario real (`osatura 0.1.0`, compilación de depuración, Linux);
las salidas están copiadas de esas ejecuciones. El rigging está en [RIGGING_GUIDE.md](RIGGING_GUIDE.md); el primer proyecto en
[GETTING_STARTED.md](GETTING_STARTED.md); todas las opciones en [CLI_REFERENCE.md](CLI_REFERENCE.md).

Licencia de este documento: CC-BY-4.0.

Los ejemplos usan el bípedo de 11 huesos (`biped.json`, 400x360, 24 fps, 24 fotogramas) de la guía de rigging: rectángulos
y círculos sueltos anclados a huesos, una capa switch `mouth` con cinco hijos, construido con una sola llamada a `script`.
Sirve cualquier rig con huesos de nombres similares.

![Seis fotogramas (0, 4, 8, 12, 16, 20) del ciclo de caminar generado de un bípedo sencillo hecho de rectángulos: las piernas oscilan en oposición, los brazos contraoscilan, las caderas suben y bajan.](../../img/walk-cycle.png)

## 1. Canales y claves

Todo lo que se mueve es un **canal**: un destino (hueso, capa, cámara, punto de deformador), una propiedad, un
modo y una fuente.

```
{"target":"hand_ctrl","prop":"bone_offset_y","source":"keys",
 "track":{"keys":[{"frame":0,"value":0,"ease":{"type":"ease_in_out"}}, ...],"extrapolate":"hold"},
 "mode":"replace","weight":1.0}
```

* **Fuentes:** `keys` (fotogramas clave), `expr` (una expresión), `noise` (ruido de valores fractal en `[-amp, amp]`, `freq` en
  ciclos por segundo) y `driven` (una curva que asigna el valor de otra propiedad a esta). Los comandos que hacen bake
  (physics, walk, clips, lip-sync...) escriben `keys` normales. `osatura timeline` muestra la fuente de cada pista:
  `#1 bone_offset_y (noise)`, `#0 bone_angle (expr)`, `#0 layer_switch (driven)`, o la lista de fotogramas clave.
* **Modos:** `replace`, `add` (encima de lo que haya debajo) y `multiply`. Los canales se combinan en el orden de la lista.
* **Extrapolate** (lo que hace una pista fuera de su primera y su última clave): `hold` (por defecto), `linear` (continúa la
  pendiente final), `loop`, `ping_pong` y `loop_offset` (repite, sumando en cada ciclo el cambio entre la primera y la última, lo que conviene a un
  desplazamiento x de caminata). Se establece con `keyedit ... extrapolate --channels 0 --extrapolate loop_offset`; un nombre incorrecto enumera los válidos.
* Las propiedades de hueso son **deltas respecto a la pose de reposo**: `bone_angle`, `bone_offset_x`, `bone_offset_y`,
  `bone_scale`. Las propiedades de capa incluyen `layer_x`, `layer_y`, `layer_rotation`, `layer_scale_x`, `layer_scale_y`,
  `layer_opacity` y `layer_switch`. Los deformadores usan `cage_dx`... como en la guía de rigging.

Crea claves desde un script (`set_key(target, prop, frame, value[, ease])`), desde la CLI con `keyedit add`, desde el
estudio o en JSON.

```
$ osatura keyedit walk.json k3.json add --target torso --prop bone_angle --at 12 --value 6 --ease ease_in_out
add: 1 keys changed, wrote k3.json
```

`osatura timeline <project> [--json]` enumera cada pista por hueso/capa con sus fotogramas clave; `osatura explain <project> <bone|layer> <frame>`
muestra cómo se calculó un fotograma.

## 2. Interpolación y easing

Cada clave guarda el ease del segmento que **empieza** en esa clave. Los nombres de easing que no necesitan parámetros son
`linear`, `step` (mantiene el valor hasta la siguiente clave), `ease_in`, `ease_out`, `ease_in_out`, `smooth` (una spline
Catmull-Rom a través de las claves vecinas: suave, sin control del overshoot) y `bounce`. En JSON un ease es un objeto:

```
{"type":"ease_in_out"}
{"type":"cubic_bezier","x1":0.25,"y1":0.1,"x2":0.25,"y2":1}    // CSS-style curve through (0,0) and (1,1)
{"type":"spring","bounces":3,"decay":4}                          // damped-spring settle that lands exactly on the target
{"type":"back","overshoot":0.6}                                  // ease out with overshoot
```

`keyedit ... ease --sel SEL --ease E` acepta un nombre simple, `bezier:x1,y1,x2,y2` (guardado como `cubic_bezier`) o un objeto JSON para
`spring` y `back`; las cuatro formas se aplicaron a las 17 claves de cadera del ciclo de caminar sin error. En la CLI de la dope sheet, la sintaxis
de selección `SEL` es `target:prop@first..last`, `target:prop` o `all`.

```
$ osatura keyedit walk.json k2.json ease --sel all --ease smooth
ease: 119 keys changed, wrote k2.json
```

Añade `-` como salida para convertir cualquier `keyedit` en una **ejecución en seco** que imprime el plan como JSON sin escribir nada:

```
$ osatura keyedit walk.json - move --sel 'thigh_l:bone_angle@0..6' --dt 2
{ "op": { "op": "batch", "ops": [ { "op": "set", "path": "/channels/0/track", "value": { "extrapolate": "loop", "keys": [ ... ] } } ] }, ... }
```

**Trampa.** `keyedit` rechaza las ediciones que dejarían claves fuera del rango de fotogramas del proyecto a menos que pases `--free`:
`scale: frame 27 is outside the project range 0..=24` (el mismo comando con `--free` imprimió `scale: 17 keys changed`, y las
claves extra quedan más allá del último fotograma, donde `lint` informa `L007`). Los canales bloqueados también se rechazan:
`channel hand_ctrl.BoneOffsetY is locked` (véase `chan-meta ... lock=on`).

El editor de curvas del estudio dibuja las mismas curvas, con tangentes arrastrables (`handle`, `smooth`; véase
[STUDIO.md](STUDIO.md)). Las claves tienen manejadores de tangente de entrada/salida opcionales (`free`, `aligned` o `auto`).

## 3. La dope sheet desde la línea de comandos: `keyedit`

Todo lo que puede hacer la línea de tiempo del estudio existe como subcomando de `keyedit`, aplicado como un único lote seguro para deshacer:

| Subcomando | Efecto |
|---|---|
| `move --sel SEL [--dt N] [--dv N] [--collision replace\|merge\|refuse] [--no-snap] [--free]` | desplaza claves en el tiempo y/o en el valor |
| `scale --sel SEL --pivot F --factor X` | cambia el timing alrededor de un fotograma |
| `dup --sel SEL --offset N` | duplica |
| `delete --sel SEL` | elimina |
| `ease --sel SEL --ease E` / `value --sel SEL --value V` / `smooth --sel SEL [--kind monotone\|catmull_rom]` | fija el ease, el valor o el suavizado automático |
| `copy --sel SEL --clip clip.json` / `paste --clip clip.json --at F [--remap a:p=b:q]` | portapapeles entre proyectos |
| `add --target T --prop P --at F [--value V] [--ease E]` | inserta una clave |
| `ripple --at F --by N --mode insert\|delete [--scope 0,1] [--adjust-frames]` | inserta o elimina tiempo |
| `offset --channels 0,1 --dt N` | desliza canales enteros |
| `extrapolate --channels 0,1 --extrapolate loop` / `loop --channels 0,1 [--end F]` | repite en bucle tras la última clave |
| `handle --sel SEL --side in\|out --t F --v V [--mode free\|aligned\|auto]` | manejador de tangente del gráfico |
| `lock --target T [--prop P] [--value on\|off]` | bloquea un canal |

## 4. Bucles

Un proyecto de N fotogramas reproduce los fotogramas de 0 a N-1 repetidamente, así que **el fotograma N debe ser la misma pose que el fotograma 0 y nunca se muestra
dos veces**. Tres comandos ayudan:

* `keyedit ... loop --channels 0,1` establece `extrapolate: loop` en los canales (las claves no se tocan):

```
$ osatura keyedit s4.json s6.json loop --channels 0,1
loop: 2 keys changed, wrote s6.json
```

* `loop-solve <in> <out> [--window 24] [--inclusive]` hace que los canales con claves hagan bucle sin costuras e informa de la costura:

```
$ osatura loop-solve 04-keys.json buddy-final.json
wrote buddy-final.json: 2 channel(s) adjusted, seam 0.000 -> 0.000
```

* `make-loop <in> <out> [targets|all] [props] [--inclusive]` **elimina el desplazamiento** para que el movimiento funcione en el sitio:

```
$ osatura make-loop t1.json t2.json
ball LayerX: removed drift 280.00
wrote t2.json
```

Esa pelota se movió 280 unidades a la derecha en 23 fotogramas. `make-loop` la convirtió en un bucle perfecto en el sitio... de una pelota que ya no
se mueve (lint informa entonces `Info L005 [ball:LayerX] all keys have the same value; the channel never changes anything`).
Eso es lo que significa «elimina el desplazamiento»: úsalo para ciclos de caminar que se mueven en el sitio y luego mueve todo el personaje con una
ruta `follow` o una cámara (sección 9). `loop-solve` no corrige el desplazamiento: sobre esa misma pelota informa `seam 280.000 -> 280.000`.
Si quieres que el desplazamiento **continúe** a lo largo de los ciclos, da al canal la extrapolación `loop_offset` (sección 1): `explain` informa entonces
`keys → 121.739 (2 key(s), LoopOffset outside range)` para la pelota en el fotograma 10.

`--inclusive` selecciona la convención antigua en la que el último fotograma mostrado es igual al primero.

**Trampa: el ruido no hace bucle.** Un canal `noise` no es periódico. Para un balanceo que haga bucle usa una expresión de `t`
(0..1 a lo largo del clip) como `8 * sin(t * 2 * pi)`, o rompe el bucle deliberadamente.

## 5. Expresiones, ruido y drivers

Un canal de **expresión** evalúa una fórmula pequeña en cada fotograma. Variables: `frame`, `time` (segundos), `fps`, `frames`,
`pi` y **`t` = 0..1 a lo largo del clip**. Operadores `+ - * / % ^`, comparaciones, `&& || !`, `?:`. Las funciones incluyen
`sin`, `abs`, `min`, `clamp(x, lo, hi)`, `lerp(a, b, t)`, `wiggle(...)` (ruido; dos o tres argumentos, por ejemplo `wiggle(3, 5, 7)`)
y `get("target","prop")`, que lee el valor actual de otro canal (los canales se ejecutan en el orden de la lista, y los posteriores ven los resultados de los anteriores).

```
cat > ex.rhai <<'EOF'
add_channel_expr("head", "bone_angle", "8 * sin(t * 2 * pi)", "add");
add_noise("hips", "bone_offset_y", 1.5, 2.0, 7, "add");
EOF
osatura script biped.json ex1.json ex.rhai
```
```
2 request(s) applied; wrote ex1.json
```

Resultado de `explain ex1.json head <frame>`: el ángulo en el mundo de la cabeza es -90.0 en los fotogramas 0, 12 y 24, **-82.0 en el fotograma 6** y
-98.0 en el fotograma 18 (un seno completo a lo largo del clip; el ángulo de reposo de la cabeza es de -90 grados en el mundo). El ruido en las caderas
mueve `hips` una fracción de unidad: su cabeza está en y 169.7 en el fotograma 0 y 170.5 en el fotograma 24 (170.0 sin el ruido), así que no hace bucle (véase arriba).

Los errores se detectan antes de escribir nada:

```
$ osatura script biped.json ex2.json ex2.rhai       # "8 * sin(tt * 2 * pi)"
request 1 (channel head.BoneAngle) was rejected, nothing changed: channel 0 ('head'): expression error: unknown variable 'tt'
$ osatura script biped.json ex3.json ex3.rhai       # "8 * sin("
request 1 (channel head.BoneAngle) was rejected, nothing changed: channel 0 ('head'): expression error: unexpected end of expression
```

**Trampa.** En una expresión `t` está normalizada (de 0 a 1), no son segundos ni fotogramas. `sin(t * 6.2832 / 24)` parece correcta
y es incorrecta: apenas se mueve. Usa `time` para segundos o `frame`.

### Drivers, sliders y vinculaciones de datos

* `slider` y `view-swap` (guía de rigging) crean huesos controladores más canales `driven`.
* `bind` conecta una ruta de valor externa (de un juego, una IU o una prueba) con una propiedad o una entrada de máquina de estados, con remapeo,
  suavizado y una expresión opcional:

```
$ osatura bind add m1.json b1.json look player.look head:bone_angle --remap 0,1,-20,20 --smooth 0.1
added binding 'look' (1 bindings); wrote b1.json
$ osatura bind add b1.json b2.json run player.running @hero:walking
$ osatura bind list b2.json
look  player.look -> head:BoneAngle remap 0..1 -> -20..20 clamp smooth 0.1s
run  player.running -> @hero:walking
$ echo '{"player.look":0.75,"player.running":0}' > vals.json
$ osatura bind-eval b2.json vals.json
look = 10
run = 0
```

`0.75` en el rango 0..1 asignado a -20..20 es 10, como se muestra. `bind-eval --data out.json` escribe las mismas cifras como JSON.

## 6. Ciclos de caminar y la biblioteca de movimiento

`walk` genera una caminata procedural: necesita nombres de hueso para las piernas y (opcionalmente) los brazos y las caderas.

```
$ osatura walk biped.json walk.json --thigh-l thigh_l --thigh-r thigh_r --shin-l shin_l --shin-r shin_r \
    --arm-l arm_upper_l --arm-r arm_upper_r --hips hips --period 24 --stride 28 --knee 40 --bob 4 --swing 25
wrote walk.json (+7 channels)
```

`--period` es la duración del ciclo en fotogramas, `--stride` el balanceo del muslo en grados, `--knee` la flexión de la rodilla, `--bob`
el vaivén de la cadera en píxeles y `--swing` el balanceo del brazo. Los siete canales tienen una clave cada 1.5 fotogramas, hacen bucle
(`extrapolate: loop`) y pasan `lint` con 0 hallazgos. Como el ciclo se genera en el sitio, no hay desplazamiento
que eliminar.

### Takes y clips

Un **take** es un fragmento de animación grabado y almacenado en un archivo de **biblioteca** que puedes reutilizar entre proyectos:

```
$ osatura take-record walk.json lib.json walk 0 24 thigh,shin,arm,hips
recorded 7 tracks into lib.json
$ osatura take-list lib.json
walk (24 frames, 7 tracks)
$ osatura take-apply biped.json ta.json lib.json walk 0
applied 7 tracks; wrote ta.json
```

Los últimos argumentos de `take-record` son prefijos de nombres de destino que se incluyen. `take-apply` escribe el take empezando en un fotograma,
con renombrados opcionales `from=to`, una `speed` y `--blend FRAMES` para hacer un fundido cruzado con la animación existente.

### La pila de clips: capas y pesos

Un **clip** es un take colocado en la línea de tiempo con un modo, un peso y máscaras. Los clips se apilan en orden; los posteriores van sobre
los anteriores. Ejemplo: una caminata, más un brazo derecho que saluda y que aparece y desaparece gradualmente.

```
$ osatura clip add biped.json c1.json lib.json walk walkclip --at 0 --loops 1 --mode replace --weight 1
added clip 'walkclip' (1 clips); run clip bake to generate channels
$ osatura clip add c1.json c2.json lib.json wave waveclip --at 0 --mode replace --weight 1 \
    --include 'arm_*_r' --fade-in 4 --fade-out 4
$ osatura clip list c2.json
1. walkclip take 'walk' frames 0..24 x1 speed 1 Replace weight 1
2. waveclip take 'wave' frames 0..24 x1 speed 1 Replace weight 1 include arm_*_r
$ osatura clip bake c2.json c3.json
baked 2 clips into 18 channels (0 replaced); wrote c3.json
$ osatura clip set-weight c3.json c4.json waveclip --keys 0:0,6:1,18:1,24:0
updated clip 'waveclip'; run clip bake to update channels
```

La pila nunca toca los canales hechos a mano ni los canales bloqueados. `--include`/`--exclude` aceptan patrones de nombres,
`--track-weight arm_*=1,leg_*=0.2` pondera las pistas individualmente, `--mode additive|multiply` apila en lugar de reemplazar.

**Trampa.** `clip bake` es lo que escribe los canales: tras cambiar clips, ejecútalo de nuevo (el comando lo indica).

## 7. Máquinas de estados y vinculaciones

Una **máquina de estados** elige entre clips en tiempo de ejecución a partir de entradas (booleanos, números, triggers). Se describe en JSON y
se comprueba al añadirla:

```
{"name":"hero","inputs":[{"name":"greet","kind":"trigger"},{"name":"walking","kind":"bool","default":1}],
 "layers":[{"name":"base","entry":"walk",
  "states":[{"name":"walk","source":{"kind":"clip","clip":"walkclip"},"mode":"loop"},
            {"name":"wave","source":{"kind":"clip","clip":"waveclip"},"mode":"once","blend_in":0.15}],
  "transitions":[{"from":"walk","to":"wave","when":[{"input":"greet"}],"duration":0.15},
                 {"from":"wave","to":"walk","exit_time":1.0,"duration":0.2}]}]}
```

Los estados reproducen un clip, un `segment` de fotogramas, una `region` de marcadores o una mezcla 1-D (`blend`) de varios; las transiciones se disparan cuando se cumplen todas las
condiciones `when` (`input op value`, AND), opcionalmente solo después de `exit_time` (de 0 a 1 de la duración del estado); gana la primera
transición coincidente en orden; `"from":"*"` significa cualquier estado.

```
$ osatura machine add c2.json m1.json machine.json
added machine 'hero' (1 machines); wrote m1.json
$ osatura machine validate m1.json
1 machines, 0 bindings: ok
$ echo '{"duration":3.0,"events":[{"t":1.0,"trigger":["greet"]}]}' > script.json
$ osatura machine-sim m1.json script.json simout --fps 12
   0.000s  base     Entered walk
   1.000s  base     Exited walk
   1.000s  base     Entered wave
   2.000s  base     Exited wave
   2.000s  base     Entered walk
37 ticks at 12 fps, 5 events, 12 svg frames; wrote simout/sim.json
```

`machine-sim` reproduce eventos de entrada temporizados y escribe un registro de estados (`sim.json`) más fotogramas SVG, de modo que las máquinas de estados se pueden probar
y comparar con diff sin un runtime. Una máquina de estados es una función pura de su registro de entradas: el mismo script siempre da el mismo
resultado.

## 8. Lip-sync y habla

Todas las entradas de audio deben ser **WAV**; convierte antes otros formatos con ffmpeg. El audio más largo que el proyecto se **trunca** a la
duración del proyecto; alarga primero el proyecto (`frames` en el archivo) o tendrás menos fotogramas de los que tiene el sonido.

```
$ osatura lipsync biped.json speech.wav mouth ls.json
wrote ls.json (3.00s of audio → 24 frames)       # the project has 24 frames = 1 s: the last 2 s are cut
```

En una copia del proyecto de 72 fotogramas (3 s):

```
$ osatura lipsync talk0.json speech.wav mouth ls.json
wrote ls.json (3.00s of audio → 72 frames)
$ osatura lipsync-text talk0.json speech.wav transcript.txt mouth lt.json --lang en --words-out words.json
wrote lt.json (7 words, 18 exposure changes; energy + text alignment, not phoneme recognition)
$ osatura speech-motion lt.json speech.wav transcript.txt sm.json --head head --nod 4 --fall 3 --tilt 2 --seed 5 --lang en
wrote sm.json (1 additive channel(s) from 7 words)
```

* `lipsync` elige las formas de la boca solo a partir de la **sonoridad** del audio.
* `lipsync-text` alinea tu transcripción (`Hello there, nice to meet you today`) con la energía del audio y usa el texto para
  las formas. **No es reconocimiento de fonemas**; la herramienta lo dice cada vez. `--words-out` escribe el timing de palabras y sílabas
  como JSON o SRT: `00:00:00,220 --> 00:00:02,690  Hello there, nice to meet you today`. Idiomas: `--lang ro|en`.
* `speech-motion` añade asentimientos, caídas e inclinaciones de cabeza sincronizados con las palabras (aditivo).
* El orden de las formas es rest, small, open, round, wide (`--map` las reasigna a tu switch).

### Señales hechas a mano: `mouth`

Si tienes señales de otra herramienta, `mouth <project> <cues.txt> <switch_layer> <out>` acepta líneas
**numeradas por fotograma** (estilo switch de Moho: `frame PHONEME`) o en **segundos** (TSV de Rhubarb): un tiempo **escrito con punto decimal
son segundos, un entero sin más es un fotograma**.

```
printf '0.00 X\n0.20 C\n0.35 B\n0.60 D\n1.00 A\n' > cues.txt
osatura mouth talk0.json cues.txt mouth mc.json                 # wrote mc.json (5 cues)
osatura mouth talk0.json cues.txt mouth mc.json --rhubarb       # also wrote mc.json (5 cues)
```

Las letras de forma se asignan a las cinco clases de boca: `X A MBP rest` a rest; `B G H L FV etc` (y las letras desconocidas) a small; `D AI` a open;
`F O U WQ` a round; `C E` a wide. `--rhubarb` cambia una sola cosa: la letra `E` se trata entonces como la forma redondeada de Rhubarb (round) en lugar de la ancha de
Papagayo. Aquí `X C B D A` dio rest, wide, small, open, rest.

**Trampa.** `0.00 X` son los segundos 0; `0 X` sería el fotograma 0. Mezclarlos en un mismo archivo es legal y confuso.

### Reactivo al audio y beats

```
osatura audio-react biped.json ar.json speech.wav head bone_angle gain=10 add=on   # loudness drives a property
osatura beats biped.json bt.json speech.wav as=markers name=syl                    # 2 beats at frames [5, 14]
osatura sound add biped.json s1.json voice speech.wav 0 1.0
osatura sound mix s1.json mix.wav                                                  # wrote mix.wav (1.00 s)
```

`sound mix` también trunca a la duración del proyecto (un sonido de 3 s pasó a 1.00 s en un proyecto de 1 s). `export` a `.html` incrusta el sonido mezclado
como mp3 (la página de este proyecto pasó de 50 KB a 131 KB), lo que requiere `ffmpeg`; sin él, la página se escribe sin sonido. `.mp4`,
`.webm` y la mezcla a mp3/ogg también requieren `ffmpeg`.

## 9. Mover cosas a lo largo de trazados, follow-through, foot-lock

* **Path follow:** dibuja una capa de línea y luego `follow <in> <out> <layer|bone:NAME> <path_layer> <start> <end> [no-orient]` anima con claves la posición
  y (salvo `no-orient`) la rotación a lo largo de ella:

```
cat > path.rhai <<'EOF'
add_line("route", [40, 300, 120, 260, 200, 290, 300, 240, 360, 280], "#bbbbbb", 2, false);
add_circle("ball", 0, 0, 10, "#c1121f");
EOF
osatura script biped.json p0.json path.rhai
osatura follow p0.json p1.json ball route 0 23         # 3 channels: layer_x, layer_y, layer_rotation
```

(`add_line` necesita tres o más puntos: `points need an even number of coordinates: 3..1000 points`.)

* **Follow-through de cadena:** `follow <in> <out> --target SRC --prop P --chain a,b,c [--delay F --decay X]` copia el movimiento de un driver a una
  cadena de seguidores, cada uno más tarde y más débil (acción solapada, colas, antenas):

```
$ osatura follow walk.json fo.json --target hips --prop bone_offset_y --chain torso,head --delay 2 --decay 0.6
wrote fo.json (+2 channels)
```

  Las claves de los seguidores se desplazan hacia más tarde según el retardo, así que algunas caen más allá del último fotograma (`lint`: `Info L007 [head:BoneOffsetY] key at
  frame 25 lies outside the 0..24 timeline`) y dependen del bucle copiado.
* **Foot-lock:** mantiene un pie plantado mientras el cuerpo se mueve. Necesita un **controlador raíz** (un hueso sin padre) y
  un goal de IK para el pie:

```
cat > ctrl.rhai <<'EOF'
add_bone("root_ctrl", "", 10, 200, 312, 0);
add_bone("foot_goal_l", "", 10, 212, 312, 0);
add_ik("leg_ik_l", ["thigh_l", "shin_l"], "foot_goal_l", -1);
EOF
osatura script biped.json fl0.json ctrl.rhai
cat > fl1.rhai <<'EOF'
set_key("foot_goal_l", "bone_offset_x", 0, 0);
set_key("foot_goal_l", "bone_offset_x", 12, -40);
set_key("foot_goal_l", "bone_offset_x", 24, 0);
set_key("foot_goal_l", "bone_offset_y", 0, 0);
set_key("foot_goal_l", "bone_offset_y", 6, -15);
set_key("foot_goal_l", "bone_offset_y", 12, 0);
set_key("foot_goal_l", "bone_offset_y", 24, 0);
EOF
osatura script fl0.json fl2.json fl1.rhai
$ osatura foot-lock fl2.json fl3.json foot_goal_l 312 1 2
contact frames 12..23 pinned at (172.0, 312.0)
wrote fl3.json
```

  `foot-lock <in> <out> <ctrl_bone> <ground_y> [tolerance] [min_frames]` encuentra los fotogramas en los que el controlador toca el suelo
  y añade canales compensatorios (modo `add`) para que no se deslice. Sin un controlador sin padre se detiene:
  `foot lock needs a root controller bone (no parent)`.
* **Bakes relacionados:** `stretch <in> <out> <follower> <leader>` (el follower apunta y se estira hacia el leader), `magnet <in> <out> <ctrl>
  <bone:NAME|x,y> <radius> <strength>` (atracción por proximidad), `bone-vis` (visibilidad en superposición).

## 10. Seguimiento facial y sus límites

**Sé honesto sobre lo que es esto.** OSATURA contiene su propio detector facial pequeño y un modelo de puntos de referencia (23 puntos, disposición
`osatura23`). Está integrado en el binario, se ejecuta en local, no necesita descargas y fue **entrenado únicamente con caras generadas
por el propio generador sintético de este proyecto**: cabezas estilizadas y de dibujos animados. **No está validado con fotografías, webcams ni vídeo
real.** Las cifras medidas (`osatura face-eval --eval-n 50 --seed 9`) son:

| Medida | Resultado |
|---|---|
| Caras sintéticas reservadas, tasa de detección | 1.0 (50 de 50) |
| Error de puntos de referencia (NME) media / mediana / percentil 90 | 0.028 / 0.026 / 0.046 |
| Falsos positivos en 50 escenas sin cara | 2 |
| Diseños de dibujos animados, NME donde se detecta | round-emoji 0.164, manga-oval 0.166, line-drawing 0.105 |
| Diseños de dibujos animados, detectados | round-emoji 6/6, manga-oval 6/6, line-drawing 6/6, **boxy-robot 0/6** |

Esas cifras corresponden a *su propio estilo sintético*. No las interpretes como precisión sobre una persona frente a una cámara. Si
necesitas seguimiento de caras reales, produce los puntos de referencia con una herramienta de tu confianza e introdúcelos como NDJSON (el formato de entrada de `face-bake`
es abierto); el panel Face del estudio espera que proporciones una función `window.osaturaLandmarkProvider` para la entrada de cámara.

El flujo de trabajo:

```
osatura face-synth face --frames 60 --size 160 --seed 7        # synthetic head animation: PNG frames, truth.ndjson, params.json
osatura face-track face face/track.ndjson --fps 24             # wrote face/track.ndjson: 60 frames, face found in 60
osatura face-bake biped.json face/track.ndjson facemap.json fb.json --fps 24
```
```
60 landmark frames -> +2 channels (K keys in them), 0 trigger keys set, 2 events; wrote fb.json   # K = number of keys in the 2 new channels
```

con un archivo de mapeo (aquí `facemap.json`):

```
{"face": {"mincutoff": 3.0},
 "params": [
  {"source":"face","id":"head_yaw","target":"head","prop":"bone_angle","out_min":-20,"out_max":20},
  {"source":"face","id":"mouth_open","target":"mouth","prop":"layer_scale_y","out_min":0.6,"out_max":1.8}
 ],
 "triggers": [
  {"source":"face","id":"eye_r_open","edge":"release","threshold":0.5,"action":{"type":"fire_event","name":"blink"}}
 ]}
```

Ids de señal: `head_yaw`, `head_pitch`, `head_roll`, `eye_l_open`, `eye_r_open`, `mouth_open`, `mouth_wide`, `mouth_round`,
`brow_raise`, `brow_l_raise`, `brow_r_raise`. Un id incorrecto enumera los válidos: `map 0: unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)`.
`face-bake` calibra la cara neutra a partir de los primeros fotogramas (`--calibrate-frames`, `--calib`).

**Cosas que conviene saber.** El bake se **trunca a la duración del proyecto**: 60 fotogramas de puntos de referencia sobre un proyecto de 24 fotogramas
produjeron claves solo de 0 a 23, y una advertencia por stderr indica cuántos fotogramas de la grabación no se hicieron bake (la duración del proyecto no cambia). La línea de resumen muestra `(K keys in them)` para los canales nuevos; `N trigger keys set` cuenta las claves escritas en canales preexistentes. `face-detect <image> --overlay o.png` muestra lo que el detector encontró en una imagen fija.

## 11. Mocap y BVH

```
$ osatura mocap-info walk.bvh
11 joints (5 end sites), 36 channels per frame
48 frames at 0.0417 s (24.000 fps), duration 1.958 s
Hips  offset (0.000, 95.000, 0.000)  [Xpos Ypos Zpos Zrot Xrot Yrot]
...
$ osatura mocap-match biped.json walk.bvh
slot         bvh joint                  rig bone                conf  via         len x  note
hips         Hips                       hips                    1.00  name         2.00
head         Head                       head                    1.00  name         4.00
shin.l       LeftLeg                    shin_l                  0.63  name         7.20  limb length ratio is 2.1x off the median
...
$ osatura mocap-retarget biped.json walk.bvh mo.json --view side --smooth 2 --weight 1
note: project length extended from 24 to 48 frames
11 bones, 48 frames at 24 fps, root scale 2.7692, +13 channels; wrote mo.json
```

* Un rig 2D solo puede usar una proyección del movimiento 3D: `--view front|side|top|yaw=DEG` la elige.
* `mocap-match` imprime una tabla de confianza (las filas `shin` de arriba llevan una nota porque la longitud de su extremidad está 2.1 veces fuera de la mediana).
  `--map map.json` anula la suposición y `--min-conf C` fija la confianza por debajo de la cual no se usa una articulación.
* **`mocap-retarget` alarga el proyecto** para que quepa el clip (24 pasó a 48 fotogramas; una nota lo indica, y `--keep-length` trunca el clip en su lugar). `--in-place` elimina el desplazamiento de la raíz,
  `--footlock BONE --ground Y` fija un pie (necesita el controlador raíz sin padre, como `foot-lock`: `foot lock needs a root controller bone (no parent)`),
  y `--mirror` refleja el movimiento.
* `import-bvh` es la forma explícita más antigua, con `--joint-map Joint=bone,...`.

## 12. Entrada en directo (MIDI, ratón, mando, teclas)

Graba una interpretación en el panel Live del estudio, o reproduce una grabación o un archivo MIDI sin conexión:

```
cat > lmap.json <<'EOF'
{"params":[{"source":"midi_cc","id":"1:1","target":"arm_upper_r","prop":"bone_angle","in_min":0,"in_max":1,"out_min":0,"out_max":160,"smooth":0.2}],
 "triggers":[{"source":"midi_note","id":"1:60","edge":"press","threshold":0.5,"action":{"type":"fire_event","name":"cymbal"}}]}
EOF
osatura live-map biped.json wheel.mid lmap.json lm.json
```
```
27 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 1 events (0 after the last frame skipped); wrote lm.json
```

La rueda de modulación (CC 1 en el canal 1) llevó el brazo de 0 a 160 grados mediante 24 claves con bake, y la nota activada a los 0.5 s pasó a ser el evento del proyecto
`cymbal` en el fotograma 12. Las fuentes son `midi_cc`, `midi_note`, `midi_pitch_bend`, `mouse_x`, `mouse_y`, `mouse_button`,
`gamepad_axis`, `gamepad_button`, `key` y `face`. Los ids MIDI son `"<channel>:<number>"` con canales desde 1. Web MIDI en el
estudio necesita un navegador que lo admita (véase [STUDIO.md](STUDIO.md)); el comando sin conexión solo necesita el archivo.

## 13. Contenedores de layout y scroll

Los contenedores de layout reorganizan las capas para un tamaño de lienzo distinto (animación de IU adaptable); las regiones de scroll añaden un desplazamiento
acotado o elástico. Los contenedores son JSON y el solver es determinista:

```
$ cat cont.json
{"id":"faces","axis":"row","gap":12,"padding":[10,10,10,10],
 "rect":{"kind":"pin","left":10,"right":10,"top":10,"height":100},
 "members":[{"layer":"eye_l"},{"layer":"eye_r","grow":1}]}
$ osatura container add biped.json c1.json cont.json
added container 'faces' (1 containers); wrote c1.json
$ osatura container solve c1.json --size 800x360
canvas 800x360
container faces: x 10.00 y 10.00 w 780.00 h 100.00
layer eye_l: dx -186.000 dy -44.000 scale 1.0000 x 1.0000
layer eye_r: dx -146.000 dy -44.000 scale 1.0000 x 1.0000
$ osatura layout-preview c1.json lp.svg --size 800x360
wrote lp.svg (800x360, 1 containers, 2 layers placed)
```

`dx`/`dy` son desplazamientos aplicados sobre la animación. Los detalles de los contenedores y de `scroll add|list|remove|eval` (límites elásticos, puntos de ajuste, simulación
de lanzamiento) están en [CLI_REFERENCE.md](CLI_REFERENCE.md#scroll-add-list-remove-eval). El layout lo aplican el runtime y las vistas previas;
**los archivos exportados contienen el layout resuelto para el tamaño propio del proyecto.**

## 14. Cámara, planos, profundidad de campo

```
osatura shot walk.json sh1.json wide 200 180 1          # name x y zoom [rotation]
osatura shot sh1.json sh2.json close 200 100 2.2
osatura camera-seq sh2.json sh3.json 0:wide,12:close,23:wide
osatura parallax sh3.json px.json sky,ground 0.2 1.0
osatura dof px.json 12 dof.png --focus 1.0 --strength 10
osatura dof-export px.json dof.gif --from 0 --to 11 --scale 0.5
```

`timeline sh3.json` muestra 11 canales: los 7 de la caminata más los de la cámara. `dof` desenfoca cada capa de nivel superior en función de `|parallax - focus|`
(la intensidad y el radio están limitados por `--max-radius`, 24 por defecto). La profundidad de campo es un render de posprocesado; `dof-export` lo escribe en un
GIF, APNG o carpeta.

## 15. Efectos de capa

Los efectos son una lista ordenada en una capa (blur, drop shadow, tint, fill) que se renderiza en una pasada aislada antes de la máscara, la opacidad y el modo de mezcla de la capa, con un máximo de 16 por capa. Sus números se pueden animar con canales con destino `"<layer>::fx<index>"` y una de las propiedades `fx_blur`, `fx_dx`,
`fx_dy`, `fx_soft`, `fx_amount` (los colores no se pueden animar):

```
"effects":[{"type":"drop_shadow","dx":4,"dy":6,"blur":8,"color":"#00000099","opacity":0.8}]      // on layer "head"
{"target":"head::fx0","prop":"fx_dx","source":"keys","track":{"keys":[{"frame":0,"value":-6},{"frame":24,"value":6}]}}
```

`check` pasa y la sombra se desliza de -6 a +6 a lo largo del bucle. Una **cadena de posefectos** (outline, posterize, relight con mapas de normales...) renderiza un
fotograma a partir de una lista JSON:

```
$ echo '[{"type":"outline","width":3,"color":"#222222"},{"type":"posterize","levels":6}]' > chain.json
$ osatura fx biped.json 0 chain.json fxc.png
wrote fxc.png (2 effect(s))
```

Comprobaciones de seguridad para destellos y movimiento de cámara (una ayuda de ingeniería basada en la definición pública WCAG 2.3.1, **no una certificación de emisión**):

```
$ osatura safety walk.json
flashes      ok    worst 0 transitions/s (frame 0)
red flashes  ok    worst 0 transitions/s (frame 0)
camera       pan 0.00 widths/s, rotation 0°/s, zoom ×1.00/s
$ osatura reduced-motion walk.json rm.json --factor 0.25
wrote rm.json
```

## 16. Onion skin y trails (superposiciones)

```
osatura onion p1.json 12 onion.png --before 3 --after 2                  # ghosts of neighbouring frames, raster
osatura onion p1.json onion.svg --frame 12 --before 3 --after 2 --keyed --outline   # SVG; --keyed ghosts key frames
osatura trail p1.json trail.svg --target layer:ball --from 0 --to 23     # wrote trail.svg (24 ticks, 24 key marker(s))
```

`--keyed` dibuja fantasmas en los fotogramas *clave* vecinos en lugar de a un paso fijo. Las superposiciones nunca modifican el proyecto y no forman parte de una exportación.
El estudio las muestra en directo (`Alt+N` onion, `Alt+M` trails, `Alt+7` panel).

## 17. Idles procedurales

```
osatura idle blink  p1.json i1.json --target eye_l --prop layer_scale_y --interval 2 --seed 4 --closed 0.1
osatura idle breath p1.json i2.json --target torso  --prop bone_scale   --amp 0.03 --period 48
osatura idle sway   p1.json i3.json --target head   --prop bone_angle   --amp 3 --freq 0.5 --seed 2
```

Con semilla, de modo que el mismo comando produce la misma animación en cualquier sitio. `autorig` añade por ti la respiración y el parpadeo cuando los nombres coinciden.

## 18. Control de calidad: check, lint, fuzz, CI

* `check` compila el proyecto exactamente como lo hará el renderizador. Sale con código 1 si hay error.
* `lint` encuentra problemas estáticos (sale con 1 si hay errores; la información y las advertencias no fallan): objetivos de IK inalcanzables (`L003`), canales que
  nunca cambian (`L005`), claves fuera de la línea de tiempo (`L007`)...
* `explain`, `coach`, `diff a.json b.json` (`+` añadido, `-` eliminado, `~` modificado) y `vcs` (control de versiones con merge semántico para archivos de proyecto).
* **Fuzz** coloca el rig en poses aleatorias e informa de dónde se rompe:

```
$ osatura fuzz walk.json --samples 200 --seed 3 --strategy lhs --only thigh_l,shin_l,arm_upper_l
fuzz 'Biped': 200 samples (lhs, seed 3, frame 0), 12 parameters, 22 failures (11.0%), max severity 1.00, baseline 0.00, 0.78 ms/sample
  off_canvas: 22
  extreme-range failure excess  arm_upper_l              28%
  extreme-range failure excess  thigh_l                  9%
  extreme-range failure excess  shin_l                   2%
  strongest interaction: bone:arm_upper_l:scale + bone:arm_upper_l:offx (cell failure rate 53% over 17 samples)
fuzz: failure rate 11.00% exceeds --max-fail-rate 5.00%
```

  **`fuzz` a secas es un informe y sale con 0; añade `--gate` (cualquier fallo sale con 1) o `--max-fail-rate R` (una fracción) para usarlo como puerta.** El rango de muestreo es el rango completo de cada parámetro, así que un rig con IK
  falla en casi todas las muestras (`98.0%`, 48 de 50 `ik_unreachable`, en el bípedo con foot-lock). Restringe con `--only`, `--props`,
  `--margin` e `--ignore KINDS`, elige el rango que consideres legal y fija el umbral que aceptes.
  `--report out.json`, `--heatmap out.svg` y `--minimize` (la pose reproducible más pequeña) ayudan. Estrategias: `random`, `lhs`, `halton`, `sweep`, `corners`, `pca`.
* **Goldens y CI.** `golden write walk.json gold.json` guarda firmas de fotogramas; `golden check` compara de forma exacta por defecto (sale con 1 ante cualquier
  cambio; `--tol T` lo relaja). Comprobar una animación *distinta* contra ellas enumera los fotogramas que difieren: `frame 4 differs: pixel hash changed, thumbnail distance 0.013, ...`, ...
  `ci` envuelve esto y más en un manifiesto:

```
$ osatura ci init walk.json --out osatura-ci.json
wrote osatura-ci.json with 9 checks:
  validate / render frames / render frames-float / determinism / pose-fuzz / limits / gif-size / loop-seam / render-speed
$ osatura ci run osatura-ci.json
[PASS   ] validate     validate           project compiles; 0 lint finding(s), none at 'error' or above
[PASS   ] render       frames             6 frame(s) match
[PASS   ] render       frames-float       2 frame(s) match
[PASS   ] determinism  determinism        6 render(s) repeated byte-identically
[PASS   ] fuzz         pose-fuzz          44 of 100 pose samples fail (44.00% <= 46.00%)
[PASS   ] limits       limits             within all limits
[PASS   ] export-size  gif-size           gif export is 106945 bytes (budget 160418)
[PASS   ] loop         loop-seam          seam 1.728% vs mean step 1.178%
[INFO   ] perf         render-speed       median 64.52 ms/frame, budget 534 ms (+50% tolerance = 801.00 ms)
9 checks: 8 passed, 0 failed, 0 errors, 1 info, 0 updated, 0 blessed (4.9 s)
```

  `ci run` sale con 0 cuando todo pasa, con 1 cuando falla una comprobación y con 2 ante un error. `ci bless` acepta los renders actuales como los nuevos fotogramas golden,
  `ci diff a.png b.png --metric ssim|de2000|de76|mse` explica un cambio con mapas de calor. El umbral de pose-fuzz del 46 % del manifiesto se calibró con el
  proyecto «tal como está ahora» (la herramienta imprime esa nota); endurécelo de forma deliberada. Los detalles están en [CI.md](CI.md).

## 19. Tamaño y nivel de detalle

```
$ osatura size-report walk.json | head -9
project file: 17507 bytes (2185 gzip)
  by layer (JSON bytes; embedded image base64 in brackets):
         2658  mouth
          643  mouth/m_small
           ...
  by channel (keys, removable at tol 0.05000000074505806):
         1116  thigh_r.BoneAngle  keys 17 removable 1
```

`size-report` muestra qué capas, imágenes y canales cuestan bytes. Cuando una plataforma tiene un presupuesto, `export-fit` encuentra la exportación de mejor calidad que cabe:

```
$ osatura export-fit walk.json fit.gif --max-size 60KB --format gif
gif -> fit.gif: 44540 bytes raw, 43822 gzip (budget 60000 raw); SSIM mean 0.9271, worst 0.9057 at frame 3, max CIEDE2000 74.98; 8 candidates built
time budget exhausted (60 s, checked inside every candidate): the search stopped early and the best result so far is kept (exit code 0 if it fits the size budget, 1 if not; ...)
$ osatura export-fit walk.json fit.json --max-size 8KB --format lottie
lottie -> fit.json: 14952 bytes raw, 1429 gzip (budget 8000 raw); SSIM mean 0.9350, worst 0.9060 at frame 10, max CIEDE2000 74.98; 5 candidates built
existing exporter at defaults: 76430 bytes raw, 6654 gzip
budget not met: the smallest export found is 14952 bytes (raw), above the budget of 8000 bytes: every knob is at its floor ...
```

El segundo comando salió con **1** y aun así escribió el archivo más pequeño que encontró (14952 bytes). Los presupuestos se comprueban por defecto sobre el tamaño en bruto;
`--measure gzip` comprueba el tamaño comprimido (el Lottie de arriba tiene 1429 bytes comprimido con gzip). La búsqueda está limitada por `--max-seconds` (60 s de tiempo real por defecto);
en la compilación de depuración la búsqueda de GIF se detuvo por ese límite y conservó su mejor resultado.

Para dibujos con muchas curvas y mallas, el **nivel de detalle** produce copias simplificadas con una cota de error *declarada* en píxeles de salida:

```
$ osatura lod walk.json --tolerance-px 0.5 --levels 3 --out-dir lods --verify
level  tol_px  anchors  mesh_v  keys  layers    bytes  declared_px  ...  measured_px  ratio
L0      0.000       75       0   119      20    17507       0.0000  ...            -      -
L1      0.500       72       0   100      20    16193       0.1153  ...       0.0895   1.3x
L2      1.000       70       0   100      20    16040       0.3969  ...       0.3597   1.1x
L3      2.000       69       0   100      20    16080       0.5570  ...       0.4997   1.1x
$ osatura lod-pick walk.json --screen-px 200x180 --max-px 0.5
screen 200x180 -> scale 0.5000 px/unit; budget 0.5 px -> level L3 (declared 0.2785 px, 69 anchors, 0 mesh vertices, 100 keys)
```

`--verify` mide la desviación real (sale con 1 si supera la cota); en la ejecución anterior todos los errores medidos quedaron por debajo de su cota declarada. Observa lo
poco que ahorra un proyecto diminuto basado en rectángulos (de 17507 a 16080 bytes): el LOD compensa con arte detallado, no con este.

## 20. Límites honestos

* El lip-sync es energía más alineación de texto, no reconocimiento de fonemas.
* El detector facial es solo sintético (sección 10): por sí solo no hace utilizable una interpretación de webcam.
* `make-loop` elimina el desplazamiento; `loop-solve` no lo corrige; el ruido no hace bucle.
* El audio se trunca a la duración del proyecto; `mocap-retarget` alarga el proyecto.
* Lottie, GIF y HTML no pueden transportar todas las funciones; las advertencias de exportación y `--report` enumeran lo que se aproximó ([EXPORT_FORMATS.md](EXPORT_FORMATS.md)).
* Solo se probó Chromium para el estudio ([STUDIO.md](STUDIO.md)).
