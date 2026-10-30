# Guia de animação do OSATURA

Como fazer as coisas se moverem, reutilizar movimento, controlá-lo a partir de áudio, rostos, mocap e entrada ao vivo, verificá-lo e
entregá-lo em tamanho reduzido. Todos os comandos aqui foram executados com o binário real (`osatura 0.1.0`, build de depuração, Linux);
as saídas foram copiadas dessas execuções. O rigging está em [RIGGING_GUIDE.md](RIGGING_GUIDE.md); o primeiro projeto em
[GETTING_STARTED.md](GETTING_STARTED.md); todas as opções em [CLI_REFERENCE.md](CLI_REFERENCE.md).

Licença deste documento: CC-BY-4.0.

Os exemplos usam o biped de 11 ossos (`biped.json`, 400x360, 24 fps, 24 quadros) do guia de rigging: retângulos
e círculos separados fixados a ossos, uma camada switch `mouth` com cinco filhos, construído por uma única chamada
de `script`. Qualquer rig com ossos de nomes semelhantes funciona.

![Seis quadros (0, 4, 8, 12, 16, 20) do ciclo de caminhada gerado para um biped simples feito de retângulos: as pernas balançam em oposição, os braços balançam em contrafase, os quadris sobem e descem.](../../img/walk-cycle.png)

## 1. Canais e chaves

Tudo o que se move é um **canal**: um alvo (osso, camada, câmera, ponto de deformador), uma propriedade, um
modo e uma fonte.

```
{"target":"hand_ctrl","prop":"bone_offset_y","source":"keys",
 "track":{"keys":[{"frame":0,"value":0,"ease":{"type":"ease_in_out"}}, ...],"extrapolate":"hold"},
 "mode":"replace","weight":1.0}
```

* **Fontes:** `keys` (keyframes), `expr` (uma expressão), `noise` (ruído fractal de valor em `[-amp, amp]`, com `freq` em
  ciclos por segundo) e `driven` (uma curva que mapeia o valor de outra propriedade para esta). Os comandos que fazem bake
  (physics, walk, clips, lip-sync...) gravam `keys` comuns. `osatura timeline` mostra a fonte de cada trilha:
  `#1 bone_offset_y (noise)`, `#0 bone_angle (expr)`, `#0 layer_switch (driven)`, ou a lista de quadros-chave.
* **Modos:** `replace`, `add` (por cima do que estiver abaixo) e `multiply`. Os canais se combinam na ordem da lista.
* **Extrapolate** (o que uma trilha faz fora da primeira e da última chave): `hold` (padrão), `linear` (continua a
  inclinação final), `loop`, `ping_pong` e `loop_offset` (repete, somando a variação da primeira à última chave em cada ciclo, o que serve para
  um offset x de caminhada). Defina-o com `keyedit ... extrapolate --channels 0 --extrapolate loop_offset`; um nome errado lista os válidos.
* As propriedades de osso são **deltas em relação à pose de repouso**: `bone_angle`, `bone_offset_x`, `bone_offset_y`,
  `bone_scale`. As propriedades de camada incluem `layer_x`, `layer_y`, `layer_rotation`, `layer_scale_x`, `layer_scale_y`,
  `layer_opacity` e `layer_switch`. Os deformadores usam `cage_dx`... como no guia de rigging.

Crie chaves a partir de um script (`set_key(target, prop, frame, value[, ease])`), pela CLI com `keyedit add`, pelo
estúdio ou em JSON.

```
$ osatura keyedit walk.json k3.json add --target torso --prop bone_angle --at 12 --value 6 --ease ease_in_out
add: 1 keys changed, wrote k3.json
```

`osatura timeline <project> [--json]` lista todas as trilhas por osso/camada com seus quadros-chave; `osatura explain <project> <bone|layer> <frame>`
mostra como um quadro foi calculado.

## 2. Interpolação e easing

Cada chave guarda o ease do segmento que **começa** nessa chave. Os nomes de easing que não precisam de parâmetros são
`linear`, `step` (mantém o valor até a próxima chave), `ease_in`, `ease_out`, `ease_in_out`, `smooth` (uma spline Catmull-Rom
pelas chaves vizinhas: suave, sem controle de overshoot) e `bounce`. Em JSON, um ease é um objeto:

```
{"type":"ease_in_out"}
{"type":"cubic_bezier","x1":0.25,"y1":0.1,"x2":0.25,"y2":1}    // CSS-style curve through (0,0) and (1,1)
{"type":"spring","bounces":3,"decay":4}                          // damped-spring settle that lands exactly on the target
{"type":"back","overshoot":0.6}                                  // ease out with overshoot
```

`keyedit ... ease --sel SEL --ease E` aceita um nome simples, `bezier:x1,y1,x2,y2` (armazenado como `cubic_bezier`) ou um objeto JSON para
`spring` e `back`; as quatro formas foram aplicadas às 17 chaves do quadril do ciclo de caminhada sem erro. Na CLI do dope sheet, a sintaxe de seleção
`SEL` é `target:prop@first..last`, `target:prop` ou `all`.

```
$ osatura keyedit walk.json k2.json ease --sel all --ease smooth
ease: 119 keys changed, wrote k2.json
```

Acrescente `-` como saída para transformar qualquer `keyedit` em um **ensaio (dry run)** que imprime o plano como JSON sem gravar nada:

```
$ osatura keyedit walk.json - move --sel 'thigh_l:bone_angle@0..6' --dt 2
{ "op": { "op": "batch", "ops": [ { "op": "set", "path": "/channels/0/track", "value": { "extrapolate": "loop", "keys": [ ... ] } } ] }, ... }
```

**Armadilha.** O `keyedit` recusa edições que deixariam chaves fora do intervalo de quadros do projeto, a menos que você passe `--free`:
`scale: frame 27 is outside the project range 0..=24` (o mesmo comando com `--free` imprimiu `scale: 17 keys changed`, e as
chaves extras ficam além do último quadro, onde o `lint` informa `L007`). Canais bloqueados também são recusados:
`channel hand_ctrl.BoneOffsetY is locked` (veja `chan-meta ... lock=on`).

O editor de curvas do estúdio desenha as mesmas curvas, com tangentes arrastáveis (`handle`, `smooth`; veja
[STUDIO.md](STUDIO.md)). As chaves têm alças de tangente de entrada/saída opcionais (`free`, `aligned` ou `auto`).

## 3. O dope sheet pela linha de comando: `keyedit`

Tudo o que a timeline do estúdio pode fazer existe como um subcomando de `keyedit`, aplicado como um único lote seguro para desfazer:

| Subcomando | Efeito |
|---|---|
| `move --sel SEL [--dt N] [--dv N] [--collision replace\|merge\|refuse] [--no-snap] [--free]` | desloca chaves no tempo e/ou no valor |
| `scale --sel SEL --pivot F --factor X` | muda o tempo em torno de um quadro |
| `dup --sel SEL --offset N` | duplica |
| `delete --sel SEL` | exclui |
| `ease --sel SEL --ease E` / `value --sel SEL --value V` / `smooth --sel SEL [--kind monotone\|catmull_rom]` | define o ease, o valor ou a suavização automática |
| `copy --sel SEL --clip clip.json` / `paste --clip clip.json --at F [--remap a:p=b:q]` | área de transferência entre projetos |
| `add --target T --prop P --at F [--value V] [--ease E]` | insere uma chave |
| `ripple --at F --by N --mode insert\|delete [--scope 0,1] [--adjust-frames]` | insere ou remove tempo |
| `offset --channels 0,1 --dt N` | desliza canais inteiros |
| `extrapolate --channels 0,1 --extrapolate loop` / `loop --channels 0,1 [--end F]` | repete depois da última chave |
| `handle --sel SEL --side in\|out --t F --v V [--mode free\|aligned\|auto]` | alça de tangente do gráfico |
| `lock --target T [--prop P] [--value on\|off]` | bloqueia um canal |

## 4. Loops

Um projeto de N quadros reproduz os quadros de 0 a N-1 repetidamente, portanto **o quadro N deve ser a mesma pose do quadro 0 e nunca é mostrado
duas vezes**. Três comandos ajudam:

* `keyedit ... loop --channels 0,1` define `extrapolate: loop` nos canais (as chaves não são alteradas):

```
$ osatura keyedit s4.json s6.json loop --channels 0,1
loop: 2 keys changed, wrote s6.json
```

* `loop-solve <in> <out> [--window 24] [--inclusive]` faz os canais animados por chaves entrarem em loop sem emendas e informa a emenda:

```
$ osatura loop-solve 04-keys.json buddy-final.json
wrote buddy-final.json: 2 channel(s) adjusted, seam 0.000 -> 0.000
```

* `make-loop <in> <out> [targets|all] [props] [--inclusive]` **remove o deslocamento** para que o movimento funcione no lugar:

```
$ osatura make-loop t1.json t2.json
ball LayerX: removed drift 280.00
wrote t2.json
```

Essa bola se moveu 280 unidades para a direita em 23 quadros. O `make-loop` a transformou em um loop perfeito no lugar... de uma bola que não
se move mais (o lint então informa `Info L005 [ball:LayerX] all keys have the same value; the channel never changes anything`).
É isso que "remove o deslocamento" significa: use-o em ciclos de caminhada que acontecem no lugar e depois mova o personagem inteiro com um
caminho `follow` ou uma câmera (seção 9). O `loop-solve` não corrige o deslocamento: na mesma bola ele informa `seam 280.000 -> 280.000`.
Se você quiser que o deslocamento **continue** entre os ciclos, dê ao canal a extrapolação `loop_offset` (seção 1): o `explain` então informa
`keys → 121.739 (2 key(s), LoopOffset outside range)` para a bola no quadro 10.

`--inclusive` seleciona a convenção mais antiga, em que o último quadro mostrado é igual ao primeiro.

**Armadilha: o noise não faz loop.** Um canal `noise` não é periódico. Para uma oscilação em loop, use uma expressão de `t`
(0..1 ao longo do clipe), como `8 * sin(t * 2 * pi)`, ou quebre o loop deliberadamente.

## 5. Expressões, noise e drivers

Um canal de **expressão** avalia uma pequena fórmula a cada quadro. Variáveis: `frame`, `time` (segundos), `fps`, `frames`,
`pi` e **`t` = 0..1 ao longo do clipe**. Operadores `+ - * / % ^`, comparações, `&& || !`, `?:`. As funções incluem
`sin`, `abs`, `min`, `clamp(x, lo, hi)`, `lerp(a, b, t)`, `wiggle(...)` (noise; dois ou três argumentos, por exemplo `wiggle(3, 5, 7)`)
e `get("target","prop")`, que lê o valor atual de outro canal (os canais são executados na ordem da lista, e os posteriores veem os resultados dos anteriores).

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

Resultado de `explain ex1.json head <frame>`: o ângulo de mundo da cabeça é -90.0 nos quadros 0, 12 e 24, **-82.0 no quadro 6** e
-98.0 no quadro 18 (um seno completo ao longo do clipe; o ângulo de repouso da cabeça é -90 graus de mundo). O noise nos quadris
move `hips` por uma fração de unidade: a cabeça dele fica em y 169.7 no quadro 0 e 170.5 no quadro 24 (170.0 sem o noise), portanto ele não faz loop (veja acima).

Os erros são detectados antes de qualquer coisa ser gravada:

```
$ osatura script biped.json ex2.json ex2.rhai       # "8 * sin(tt * 2 * pi)"
request 1 (channel head.BoneAngle) was rejected, nothing changed: channel 0 ('head'): expression error: unknown variable 'tt'
$ osatura script biped.json ex3.json ex3.rhai       # "8 * sin("
request 1 (channel head.BoneAngle) was rejected, nothing changed: channel 0 ('head'): expression error: unexpected end of expression
```

**Armadilha.** Em uma expressão, `t` é normalizado (0 a 1), não segundos nem quadros. `sin(t * 6.2832 / 24)` parece certo
e está errado: quase não se move. Use `time` para segundos ou `frame`.

### Drivers, sliders e data bindings

* `slider` e `view-swap` (guia de rigging) criam ossos controladores mais canais `driven`.
* `bind` conecta um caminho de valor externo (de um jogo, de uma UI ou de um teste) a uma propriedade ou a uma entrada de máquina de estados, com remapeamento,
  suavização e uma expressão opcional:

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

`0.75` na faixa 0..1 mapeada para -20..20 é 10, como mostrado. `bind-eval --data out.json` grava os mesmos números como JSON.

## 6. Ciclos de caminhada e a biblioteca de movimento

`walk` gera uma caminhada procedural: ele precisa dos nomes dos ossos das pernas e (opcionalmente) dos braços e dos quadris.

```
$ osatura walk biped.json walk.json --thigh-l thigh_l --thigh-r thigh_r --shin-l shin_l --shin-r shin_r \
    --arm-l arm_upper_l --arm-r arm_upper_r --hips hips --period 24 --stride 28 --knee 40 --bob 4 --swing 25
wrote walk.json (+7 channels)
```

`--period` é a duração do ciclo em quadros, `--stride` a oscilação da coxa em graus, `--knee` a flexão do joelho, `--bob`
o sobe e desce dos quadris em pixels e `--swing` a oscilação dos braços. Os sete canais têm uma chave a cada 1,5 quadro, entram em loop
(`extrapolate: loop`) e passam no `lint` com 0 achados. Como o ciclo é gerado no lugar, não há deslocamento
a remover.

### Takes e clips

Um **take** é um trecho de animação gravado, armazenado em um arquivo de **biblioteca** que você pode reutilizar entre projetos:

```
$ osatura take-record walk.json lib.json walk 0 24 thigh,shin,arm,hips
recorded 7 tracks into lib.json
$ osatura take-list lib.json
walk (24 frames, 7 tracks)
$ osatura take-apply biped.json ta.json lib.json walk 0
applied 7 tracks; wrote ta.json
```

Os últimos argumentos de `take-record` são prefixos de nomes de alvos a incluir. `take-apply` grava o take começando em um quadro,
com renomeações opcionais `from=to`, uma `speed` e `--blend FRAMES` para fazer crossfade com a animação existente.

### A pilha de clips: camadas e pesos

Um **clip** é um take colocado na timeline com um modo, um peso e máscaras. Os clips são empilhados em ordem; os posteriores ficam por cima dos
anteriores. Exemplo: uma caminhada, mais um braço direito acenando que aparece e some gradualmente.

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

A pilha nunca toca em canais feitos à mão nem em canais bloqueados. `--include`/`--exclude` aceitam padrões de nomes,
`--track-weight arm_*=1,leg_*=0.2` pondera as trilhas individualmente, e `--mode additive|multiply` empilha em vez de substituir.

**Armadilha.** `clip bake` é o que grava os canais: depois de alterar os clips, execute-o de novo (o comando avisa).

## 7. Máquinas de estados e bindings

Uma **máquina de estados** escolhe entre clips em tempo de execução a partir de entradas (booleanos, números, triggers). Ela é descrita em JSON e
verificada no momento em que é adicionada:

```
{"name":"hero","inputs":[{"name":"greet","kind":"trigger"},{"name":"walking","kind":"bool","default":1}],
 "layers":[{"name":"base","entry":"walk",
  "states":[{"name":"walk","source":{"kind":"clip","clip":"walkclip"},"mode":"loop"},
            {"name":"wave","source":{"kind":"clip","clip":"waveclip"},"mode":"once","blend_in":0.15}],
  "transitions":[{"from":"walk","to":"wave","when":[{"input":"greet"}],"duration":0.15},
                 {"from":"wave","to":"walk","exit_time":1.0,"duration":0.2}]}]}
```

Os estados reproduzem um clip, um `segment` de quadros, uma `region` de marcadores ou um `blend` 1-D de vários; as transições disparam quando todas as
condições de `when` (`input op value`, AND) são verdadeiras, opcionalmente apenas depois de `exit_time` (0 a 1 da duração do estado); vale a primeira
transição correspondente, em ordem; `"from":"*"` significa qualquer estado.

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

`machine-sim` reproduz eventos de entrada temporizados e grava um registro de estados (`sim.json`) mais quadros SVG, de modo que as máquinas de estados são testáveis
e comparáveis (diff) sem um runtime. Uma máquina de estados é uma função pura do seu registro de entradas: o mesmo script sempre dá o mesmo
resultado.

## 8. Lip-sync e fala

Todas as entradas de áudio devem ser **WAV**; converta outros formatos com o ffmpeg antes. Áudio mais longo que o projeto é **truncado** ao
comprimento do projeto; aumente o projeto antes (`frames` no arquivo) ou você terá menos quadros do que o som tem.

```
$ osatura lipsync biped.json speech.wav mouth ls.json
wrote ls.json (3.00s of audio → 24 frames)       # the project has 24 frames = 1 s: the last 2 s are cut
```

Em uma cópia de 72 quadros do projeto (3 s):

```
$ osatura lipsync talk0.json speech.wav mouth ls.json
wrote ls.json (3.00s of audio → 72 frames)
$ osatura lipsync-text talk0.json speech.wav transcript.txt mouth lt.json --lang en --words-out words.json
wrote lt.json (7 words, 18 exposure changes; energy + text alignment, not phoneme recognition)
$ osatura speech-motion lt.json speech.wav transcript.txt sm.json --head head --nod 4 --fall 3 --tilt 2 --seed 5 --lang en
wrote sm.json (1 additive channel(s) from 7 words)
```

* `lipsync` escolhe as formas da boca apenas pela **intensidade sonora** do áudio.
* `lipsync-text` alinha a sua transcrição (`Hello there, nice to meet you today`) à energia do áudio e usa o texto para
  as formas. Ele **não é reconhecimento de fonemas**; a ferramenta avisa isso a cada execução. `--words-out` grava a temporização de palavras e sílabas
  em JSON ou SRT: `00:00:00,220 --> 00:00:02,690  Hello there, nice to meet you today`. Idiomas: `--lang ro|en`.
* `speech-motion` adiciona acenos de cabeça, quedas e inclinações cronometrados pelas palavras (aditivo).
* A ordem das formas é rest, small, open, round, wide (`--map` as remapeia para o seu switch).

### Deixas feitas à mão: `mouth`

Se você tem deixas (cues) de outra ferramenta, `mouth <project> <cues.txt> <switch_layer> <out>` aceita linhas
**numeradas por quadro** (estilo switch do Moho: `frame PHONEME`) ou em **segundos** (TSV do Rhubarb): um tempo **escrito com ponto decimal
é segundos, um inteiro simples é um quadro**.

```
printf '0.00 X\n0.20 C\n0.35 B\n0.60 D\n1.00 A\n' > cues.txt
osatura mouth talk0.json cues.txt mouth mc.json                 # wrote mc.json (5 cues)
osatura mouth talk0.json cues.txt mouth mc.json --rhubarb       # also wrote mc.json (5 cues)
```

As letras de forma são mapeadas para as cinco classes de boca: `X A MBP rest` para rest; `B G H L FV etc` (e letras desconhecidas) para small; `D AI` para open;
`F O U WQ` para round; `C E` para wide. `--rhubarb` muda uma coisa: a letra `E` passa a ser tratada como a forma arredondada do Rhubarb (round) em vez da
forma larga do Papagayo (wide). Aqui `X C B D A` deu rest, wide, small, open, rest.

**Armadilha.** `0.00 X` é o segundo 0; `0 X` seria o quadro 0. Misturar os dois no mesmo arquivo é válido e confuso.

### Reação ao áudio e batidas

```
osatura audio-react biped.json ar.json speech.wav head bone_angle gain=10 add=on   # loudness drives a property
osatura beats biped.json bt.json speech.wav as=markers name=syl                    # 2 beats at frames [5, 14]
osatura sound add biped.json s1.json voice speech.wav 0 1.0
osatura sound mix s1.json mix.wav                                                  # wrote mix.wav (1.00 s)
```

`sound mix` também trunca ao comprimento do projeto (um som de 3 s virou 1.00 s em um projeto de 1 s). `export` para `.html` incorpora o som
mixado como mp3 (a página deste projeto passou de 50 KB para 131 KB), o que exige o `ffmpeg`; sem ele, a página é gravada sem som. `.mp4`,
`.webm` e a mixagem em mp3/ogg também exigem o `ffmpeg`.

## 9. Mover coisas ao longo de caminhos, follow-through, foot-lock

* **Path follow:** desenhe uma camada de linha e depois `follow <in> <out> <layer|bone:NAME> <path_layer> <start> <end> [no-orient]` anima com chaves a posição
  e (a menos que haja `no-orient`) a rotação ao longo dela:

```
cat > path.rhai <<'EOF'
add_line("route", [40, 300, 120, 260, 200, 290, 300, 240, 360, 280], "#bbbbbb", 2, false);
add_circle("ball", 0, 0, 10, "#c1121f");
EOF
osatura script biped.json p0.json path.rhai
osatura follow p0.json p1.json ball route 0 23         # 3 channels: layer_x, layer_y, layer_rotation
```

(`add_line` precisa de três ou mais pontos: `points need an even number of coordinates: 3..1000 points`.)

* **Follow-through de cadeia:** `follow <in> <out> --target SRC --prop P --chain a,b,c [--delay F --decay X]` copia o movimento de um driver para uma
  cadeia de seguidores, cada um mais tardio e mais fraco (overlapping action, caudas, antenas):

```
$ osatura follow walk.json fo.json --target hips --prop bone_offset_y --chain torso,head --delay 2 --decay 0.6
wrote fo.json (+2 channels)
```

  As chaves dos seguidores são deslocadas para mais tarde pelo atraso, de modo que algumas caem além do último quadro (`lint`: `Info L007 [head:BoneOffsetY] key at
  frame 25 lies outside the 0..24 timeline`) e dependem do loop copiado.
* **Foot-lock:** mantém um pé plantado enquanto o corpo se move. Ele precisa de um **controlador raiz** (um osso sem pai) e
  de um goal de IK para o pé:

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

  `foot-lock <in> <out> <ctrl_bone> <ground_y> [tolerance] [min_frames]` encontra os quadros em que o controlador toca o chão
  e adiciona canais de compensação (modo `add`) para que ele não deslize. Sem um controlador sem pai, ele para:
  `foot lock needs a root controller bone (no parent)`.
* **Bakes relacionados:** `stretch <in> <out> <follower> <leader>` (o seguidor mira e se estica até o líder), `magnet <in> <out> <ctrl>
  <bone:NAME|x,y> <radius> <strength>` (atração por proximidade), `bone-vis` (visibilidade em overlay).

## 10. Rastreamento facial e seus limites

**Seja honesto sobre o que isto é.** O OSATURA contém seu próprio detector de rostos e modelo de landmarks, pequenos (23 landmarks, layout
`osatura23`). Ele está embutido no binário, roda localmente, não precisa de downloads e foi **treinado apenas com rostos gerados
pelo gerador sintético do próprio projeto**: cabeças estilizadas e de desenho animado. Ele **não foi validado em fotografias, webcams nem em vídeo
real.** Os números medidos (`osatura face-eval --eval-n 50 --seed 9`) são:

| Medida | Resultado |
|---|---|
| Rostos sintéticos reservados para teste, taxa de detecção | 1.0 (50 de 50) |
| Erro de landmarks (NME) média / mediana / percentil 90 | 0.028 / 0.026 / 0.046 |
| Falsos positivos em 50 cenas sem rosto | 2 |
| Designs de desenho animado, NME onde detectado | round-emoji 0.164, manga-oval 0.166, line-drawing 0.105 |
| Designs de desenho animado, detectados | round-emoji 6/6, manga-oval 6/6, line-drawing 6/6, **boxy-robot 0/6** |

Esses números valem para o *seu próprio estilo sintético*. Não os leia como precisão em uma pessoa diante de uma câmera. Se você
precisa de rastreamento de rostos reais, produza os landmarks com uma ferramenta em que você confie e forneça-os como NDJSON (o formato de entrada do `face-bake`
é aberto); o painel Face do estúdio espera que você forneça uma função `window.osaturaLandmarkProvider` para a entrada da câmera.

O pipeline:

```
osatura face-synth face --frames 60 --size 160 --seed 7        # synthetic head animation: PNG frames, truth.ndjson, params.json
osatura face-track face face/track.ndjson --fps 24             # wrote face/track.ndjson: 60 frames, face found in 60
osatura face-bake biped.json face/track.ndjson facemap.json fb.json --fps 24
```
```
60 landmark frames -> +2 channels (K keys in them), 0 trigger keys set, 2 events; wrote fb.json   # K = number of keys in the 2 new channels
```

com um arquivo de mapeamento (aqui `facemap.json`):

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

Ids de sinais: `head_yaw`, `head_pitch`, `head_roll`, `eye_l_open`, `eye_r_open`, `mouth_open`, `mouth_wide`, `mouth_round`,
`brow_raise`, `brow_l_raise`, `brow_r_raise`. Um id errado lista os válidos: `map 0: unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)`.
O `face-bake` calibra o rosto neutro a partir dos primeiros quadros (`--calibrate-frames`, `--calib`).

**O que você deve saber.** O bake é **truncado ao comprimento do projeto**: 60 quadros de landmarks em um projeto de 24 quadros
produziram chaves de 0 a 23 apenas, e um aviso em stderr diz quantos quadros da gravação não foram gravados (o comprimento do projeto não muda). A linha de resumo mostra `(K keys in them)` para os novos canais; `N trigger keys set` conta as chaves gravadas em canais preexistentes. `face-detect <image> --overlay o.png` mostra o que o detector encontrou em uma imagem estática.

## 11. Mocap e BVH

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

* Um rig 2D só consegue usar uma projeção do movimento 3D: `--view front|side|top|yaw=DEG` a escolhe.
* `mocap-match` imprime uma tabela de confiança (as linhas `shin` acima trazem uma nota porque o comprimento do membro está 2,1 vezes fora da mediana).
  `--map map.json` substitui a estimativa e `--min-conf C` define a confiança abaixo da qual uma articulação não é usada.
* **`mocap-retarget` aumenta o projeto** para caber o clipe (24 virou 48 quadros; uma nota avisa, e `--keep-length` trunca o clipe em vez disso). `--in-place` remove o deslocamento da raiz,
  `--footlock BONE --ground Y` fixa um pé (exige o controlador raiz sem pai, como o `foot-lock`: `foot lock needs a root controller bone (no parent)`),
  e `--mirror` espelha o movimento.
* `import-bvh` é a forma mais antiga e explícita, com `--joint-map Joint=bone,...`.

## 12. Entrada ao vivo (MIDI, mouse, gamepad, teclas)

Grave uma performance no painel Live do estúdio, ou reproduza uma gravação ou um arquivo MIDI offline:

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

A roda de modulação (CC 1 no canal 1) varreu o braço de 0 a 160 graus por meio de 24 chaves gravadas (baked), e a nota ligada em 0,5 s virou o evento do projeto
`cymbal` no quadro 12. As fontes são `midi_cc`, `midi_note`, `midi_pitch_bend`, `mouse_x`, `mouse_y`, `mouse_button`,
`gamepad_axis`, `gamepad_button`, `key` e `face`. Os ids MIDI são `"<channel>:<number>"`, com canais a partir de 1. O Web MIDI no
estúdio exige um navegador que o suporte (veja [STUDIO.md](STUDIO.md)); o comando offline só precisa do arquivo.

## 13. Contêineres de layout e rolagem

Os contêineres de layout reorganizam camadas para um tamanho de canvas diferente (animação de UI responsiva); as regiões de rolagem adicionam rolagem limitada ou
elástica. Os contêineres são JSON e o solver é determinístico:

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

`dx`/`dy` são offsets aplicados por cima da animação. Os detalhes dos contêineres e de `scroll add|list|remove|eval` (limites elásticos, pontos de snap, simulação de
fling) estão em [CLI_REFERENCE.md](CLI_REFERENCE.md#scroll-add-list-remove-eval). O layout é aplicado pelo runtime e pelas pré-visualizações;
**os arquivos exportados contêm o layout resolvido para o tamanho do próprio projeto.**

## 14. Câmera, planos, profundidade de campo

```
osatura shot walk.json sh1.json wide 200 180 1          # name x y zoom [rotation]
osatura shot sh1.json sh2.json close 200 100 2.2
osatura camera-seq sh2.json sh3.json 0:wide,12:close,23:wide
osatura parallax sh3.json px.json sky,ground 0.2 1.0
osatura dof px.json 12 dof.png --focus 1.0 --strength 10
osatura dof-export px.json dof.gif --from 0 --to 11 --scale 0.5
```

`timeline sh3.json` mostra 11 canais: os 7 da caminhada mais os da câmera. `dof` desfoca cada camada de nível superior por `|parallax - focus|`
(a intensidade e o raio são limitados por `--max-radius`, padrão 24). A profundidade de campo é um render de pós-processamento; `dof-export` o grava em
GIF, APNG ou uma pasta.

## 15. Efeitos de camada

Os efeitos são uma lista ordenada em uma camada (blur, drop shadow, tint, fill) renderizada em uma passagem isolada antes da máscara, da opacidade e do modo de mesclagem
da camada, até 16 por camada. Seus números podem ser animados com canais que têm como alvo `"<layer>::fx<index>"` e uma das propriedades `fx_blur`, `fx_dx`,
`fx_dy`, `fx_soft`, `fx_amount` (as cores não são animáveis):

```
"effects":[{"type":"drop_shadow","dx":4,"dy":6,"blur":8,"color":"#00000099","opacity":0.8}]      // on layer "head"
{"target":"head::fx0","prop":"fx_dx","source":"keys","track":{"keys":[{"frame":0,"value":-6},{"frame":24,"value":6}]}}
```

O `check` passa e a sombra desliza de -6 a +6 ao longo do loop. Uma **cadeia de pós-efeitos** (outline, posterize, relight com mapas de normais...) renderiza um
quadro a partir de uma lista JSON:

```
$ echo '[{"type":"outline","width":3,"color":"#222222"},{"type":"posterize","levels":6}]' > chain.json
$ osatura fx biped.json 0 chain.json fxc.png
wrote fxc.png (2 effect(s))
```

Verificações de segurança para flashes e movimento de câmera (um auxílio de engenharia baseado na definição pública da WCAG 2.3.1, **não uma certificação para broadcast**):

```
$ osatura safety walk.json
flashes      ok    worst 0 transitions/s (frame 0)
red flashes  ok    worst 0 transitions/s (frame 0)
camera       pan 0.00 widths/s, rotation 0°/s, zoom ×1.00/s
$ osatura reduced-motion walk.json rm.json --factor 0.25
wrote rm.json
```

## 16. Onion skin e trails (overlays)

```
osatura onion p1.json 12 onion.png --before 3 --after 2                  # ghosts of neighbouring frames, raster
osatura onion p1.json onion.svg --frame 12 --before 3 --after 2 --keyed --outline   # SVG; --keyed ghosts key frames
osatura trail p1.json trail.svg --target layer:ball --from 0 --to 23     # wrote trail.svg (24 ticks, 24 key marker(s))
```

`--keyed` desenha os ghosts nos quadros-chave vizinhos, e não em um passo fixo. Os overlays nunca alteram o projeto e não fazem parte de uma exportação.
O estúdio os mostra ao vivo (`Alt+N` onion, `Alt+M` trails, `Alt+7` painel).

## 17. Idles procedurais

```
osatura idle blink  p1.json i1.json --target eye_l --prop layer_scale_y --interval 2 --seed 4 --closed 0.1
osatura idle breath p1.json i2.json --target torso  --prop bone_scale   --amp 0.03 --period 48
osatura idle sway   p1.json i3.json --target head   --prop bone_angle   --amp 3 --freq 0.5 --seed 2
```

Com seed, portanto o mesmo comando gera a mesma animação em qualquer lugar. O `autorig` adiciona respiração e piscadas para você quando os nomes correspondem.

## 18. Garantia de qualidade: check, lint, fuzz, CI

* `check` compila o projeto exatamente como o renderizador fará. Código de saída 1 em caso de erro.
* `lint` encontra problemas estáticos (código de saída 1 em caso de erros; informações e avisos não falham): alvos de IK inalcançáveis (`L003`), canais que
  nunca mudam (`L005`), chaves fora da timeline (`L007`)...
* `explain`, `coach`, `diff a.json b.json` (`+` adicionado, `-` removido, `~` alterado) e `vcs` (controle de versão com merge semântico para arquivos de projeto).
* O **fuzz** posiciona o rig aleatoriamente e informa onde ele quebra:

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

  **O `fuzz` simples é um relatório e sai com 0; acrescente `--gate` (qualquer falha sai com 1) ou `--max-fail-rate R` (uma fração) para aplicar um critério de aprovação.** A faixa de amostragem é a faixa completa de cada parâmetro, de modo que um rig com IK
  falha em quase todas as amostras (`98.0%`, 48 de 50 `ik_unreachable`, no biped com foot-lock). Restrinja com `--only`, `--props`,
  `--margin` e `--ignore KINDS`, escolha a faixa que você considera válida e defina o limite que você aceita.
  `--report out.json`, `--heatmap out.svg` e `--minimize` (a menor pose que reproduz o problema) ajudam. Estratégias: `random`, `lhs`, `halton`, `sweep`, `corners`, `pca`.
* **Goldens e CI.** `golden write walk.json gold.json` armazena assinaturas de quadros; `golden check` compara exatamente por padrão (código de saída 1 em qualquer
  mudança; `--tol T` o afrouxa). Verificar uma animação *diferente* contra elas lista os quadros que diferem: `frame 4 differs: pixel hash changed, thumbnail distance 0.013, ...`, ...
  `ci` engloba isso e mais em um manifesto:

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

  `ci run` sai com 0 quando tudo passa, 1 quando uma verificação falha e 2 em caso de erro. `ci bless` aceita os renders atuais como os novos quadros golden,
  `ci diff a.png b.png --metric ssim|de2000|de76|mse` explica uma mudança com mapas de calor. O limite de pose-fuzz de 46 % no manifesto foi calibrado para o
  projeto "como ele está agora" (a ferramenta imprime essa observação); aperte-o deliberadamente. Os detalhes estão em [CI.md](CI.md).

## 19. Tamanho e nível de detalhe

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

`size-report` mostra quais camadas, imagens e canais custam bytes. Quando uma plataforma tem um orçamento, `export-fit` encontra a exportação de melhor qualidade que cabe nele:

```
$ osatura export-fit walk.json fit.gif --max-size 60KB --format gif
gif -> fit.gif: 44540 bytes raw, 43822 gzip (budget 60000 raw); SSIM mean 0.9271, worst 0.9057 at frame 3, max CIEDE2000 74.98; 8 candidates built
time budget exhausted (60 s, checked inside every candidate): the search stopped early and the best result so far is kept (exit code 0 if it fits the size budget, 1 if not; ...)
$ osatura export-fit walk.json fit.json --max-size 8KB --format lottie
lottie -> fit.json: 14952 bytes raw, 1429 gzip (budget 8000 raw); SSIM mean 0.9350, worst 0.9060 at frame 10, max CIEDE2000 74.98; 5 candidates built
existing exporter at defaults: 76430 bytes raw, 6654 gzip
budget not met: the smallest export found is 14952 bytes (raw), above the budget of 8000 bytes: every knob is at its floor ...
```

O segundo comando saiu com **1** e mesmo assim gravou o menor arquivo que encontrou (14952 bytes). Por padrão, os orçamentos são verificados sobre o tamanho bruto;
`--measure gzip` verifica o tamanho comprimido (o Lottie acima tem 1429 bytes com gzip). A busca é limitada por `--max-seconds` (padrão 60 s de tempo real);
no build de depuração, a busca de GIF foi interrompida por esse limite e manteve seu melhor resultado.

Para desenhos com muitas curvas e malhas, o **nível de detalhe (LOD)** produz cópias simplificadas com um limite de erro *declarado* em pixels de saída:

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

`--verify` mede o desvio real (código de saída 1 se exceder o limite); na execução acima, todo erro medido ficou abaixo do seu limite declarado. Observe como
pouco um projeto minúsculo baseado em retângulos economiza (17507 para 16080 bytes): o LOD compensa em arte detalhada, não nesta.

## 20. Limites honestos

* O lip-sync é energia mais alinhamento de texto, não reconhecimento de fonemas.
* O detector de rostos é apenas sintético (seção 10): ele não torna utilizável, por si só, uma performance de webcam.
* `make-loop` remove o deslocamento; `loop-solve` não o corrige; o noise não faz loop.
* O áudio é truncado ao comprimento do projeto; `mocap-retarget` aumenta o projeto.
* Lottie, GIF e HTML não conseguem transportar todos os recursos; os avisos de exportação e `--report` listam o que foi aproximado ([EXPORT_FORMATS.md](EXPORT_FORMATS.md)).
* Somente o Chromium foi testado para o estúdio ([STUDIO.md](STUDIO.md)).
