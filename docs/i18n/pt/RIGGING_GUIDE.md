# Guia de rigging do OSATURA

Este guia cobre tudo o que faz um desenho se mover: ossos, restrições, cinemática inversa,
skinning, deformadores, física, auto-rigging, espelhamento, troca de vistas, rigs de rosto e trabalho de câmera.
Todos os exemplos foram executados com o binário real (`osatura 0.1.0`, build de depuração, Linux) e os números
citados são os números que ele imprimiu. Se você ainda não criou um primeiro projeto, comece por
[GETTING_STARTED.md](GETTING_STARTED.md). As opções de cada comando estão em
[CLI_REFERENCE.md](CLI_REFERENCE.md); o formato de arquivo está em [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

Licença deste documento: CC-BY-4.0.

![Oito pequenos renders: uma faixa com skinning dobrada por um osso, três variantes de IK (alvo fora do alcance, stretch 2, soft 0.5) e quatro deformadores vetoriais (cage, lattice, spine, pin), cada um movido no quadro 12.](../../img/rigging-sheet.png)

*Folha de contatos dos exemplos abaixo: pesos de skin com FK, FABRIK com alvo inalcançável,
`stretch = 2`, `soft = 0.5`, deformadores cage, lattice, spine e pin.*

## 1. Duas maneiras de editar um rig

* **Comandos e `script`.** A maioria das edições de rig tem um comando (`autorig`, `cage`, `physics`, `skin`...) ou
  uma função Rhai em `osatura script` (`add_bone`, `add_ik`, `add_aim`, `add_copy_rotation`, `add_pin`,
  `set_layer`, `set_bone`, `set_parent`, `delete_bone`...). Um script é tudo ou nada: se qualquer requisição for
  rejeitada, nada é gravado. Um script pode conter no máximo 5000 requisições.
* **JSON.** Algumas coisas não têm comando nem setter de script: `soft` e `stretch` do IK, molas, deformadores
  vetoriais (lattice, spine, pin) e pesos de skin por âncora. Edite o JSON do projeto (em qualquer editor, com um
  script Python ou no painel JSON do estúdio) e execute `osatura check` em seguida.

`check`, `lint`, `explain` e `render --bones` são os quatro comandos que você usará o tempo todo ao fazer rigging.
`explain` é o mais valioso: ele lista, em ordem, cada valor de repouso, canal, restrição e contribuição de IK que
produziu a transformação final de um osso, de modo que você nunca precisa adivinhar por que algo está onde está.

```
$ osatura explain con1.json b3 0
- bone 'b3' (length 80), parent 'b2'
- ancestry: b3 ← b2 ← b1
- rest pose: offset (80.00, 0.00), angle 0.00°
- no channels animate this bone directly
- constraint 'b3_aims' (Aim { target: "target", offset: 0.0 }) at weight 1.00
- result at frame 0: head (203.6, 101.4), tip (173.5, 27.3), world angle -112.1°
```

## 2. Ossos e hierarquias

Um osso tem um `name`, um `parent` (nenhum = raiz), um `length`, um `rest_offset` (onde fica sua cabeça, no
referencial do **pai**; para um osso raiz, é o canvas) e um `rest_angle` em graus relativo ao
pai. Convenções:

* y aponta para baixo; o ângulo 0 aponta ao longo de +x; ângulos positivos giram no sentido horário na tela; **90 é para baixo, -90 é para cima**.
* `rest_offset` é girado pelo ângulo de mundo do pai. Um filho de um osso que aponta para cima e tem
  `rest_offset` (70, 0) fica 70 unidades *acima ao longo do pai*, e não 70 unidades à direita do canvas.
* Os ossos devem ser listados começando pelos pais. Qualquer outra ordem é rejeitada:
  `bone 'a' has parent 'ghost' which is missing or listed after it`.

Os canais de animação de um osso são **deltas em relação à pose de repouso**: `bone_angle`, `bone_offset_x`,
`bone_offset_y`, `bone_scale`. Um valor de chave 0 significa "repouso".

Uma cadeia mínima de 3 ossos usada pela maioria dos exemplos abaixo (`lab.json`; ela é gerada, veja a seção 4):

```
b1  root, length 80, head at (60,150), angle 0
b2  child of b1, rest_offset (80,0)
b3  child of b2, rest_offset (80,0)
```

Comandos úteis: `reparent <in> <out> <bone> <new_parent|none>` mantém a pose de repouso (verifiquei: `b3`
permanece com a cabeça em (220,150) quando é reparentado para none), `set_parent` em um script é a forma de mais baixo nível, e
`bone-vis <in> <out> <bone> <from> <to>` mostra um osso apenas nas sobreposições (overlays) durante um intervalo de quadros.

### Fixação rígida: o rig mais simples

Uma camada pode acompanhar um osso rigidamente com `set_layer("layer", "bone", "bone_name")`. É assim que o
personagem do tutorial e o biped abaixo são feitos: peças separadas, cada uma fixada a um osso. Não precisa de
pesos e sobrevive a todas as exportações. Escolha esta opção quando sua arte já estiver cortada em peças.

**Armadilha.** Fixe as camadas enquanto os ossos estão na pose de repouso e desenhe cada peça de modo que sua articulação fique
na cabeça do seu osso. Uma peça desenhada com alguns pontos de diferença vai girar em torno do ponto errado.

**Armadilha.** Não defina o ângulo de repouso dos ossos dos braços como ±100 "para obter uma T-pose" com um offset de repouso de 0: os
dois braços partem do mesmo ponto. Dê a cada ombro um `rest_offset` lateral e um ângulo de 180 ou
0, como no biped (`add_bone("arm_upper_l", "torso", 50, 62, 22, 180)`).

## 3. Restrições

As restrições são listadas no array `constraints` do projeto e são executadas **na ordem da lista** depois dos
canais. Crie-as com `add_aim`, `add_copy_rotation` e `add_pin` (script) ou em JSON.

| Restrição | Script | Efeito |
|---|---|---|
| Aim | `add_aim(name, bone, target[, offset, weight])` | gira `bone` para que ele aponte para a cabeça do osso `target` |
| Copy rotation | `add_copy_rotation(name, bone, source, weight)` | `bone` assume (uma parte ponderada d)a rotação de mundo de `source` |
| Pin | `add_pin(name, bone, target, weight)` | move a cabeça de `bone` para a cabeça de `target` |

Verificado com a cadeia de 3 ossos, sendo `target` um osso em (250,40):

```
add_aim("b3_aims", "b3", "target", 0, 1);        // weight 1   -> world angle -74.7 deg
add_aim("b3_aims", "b3", "target", 0, 0.5);      // weight 0.5 -> world angle -37.4 deg
```

O peso mistura linearmente o ângulo animado e o ângulo restrito, de modo que 0.5 reduz a rotação pela metade.

Os limites de ângulo são propriedades do osso (`set_bone(name, "angle_min", -20); set_bone(name, "angle_max", 20)`).
Eles limitam a rotação **animada**: se você animar `b1` até 60 graus, ele é mantido em 20:

```
$ osatura explain l2.json b1 12
- channel #0 BoneAngle: keys → 60.000 (2 key(s), Hold outside range); mode replace, weight 1.00
- angle limits: animated rotation is clamped to [-20, 20]°
- result at frame 12: head (60.0, 150.0), tip (135.2, 177.4), world angle 20.0°
```

**Armadilha: a ordem importa.** Uma copy-rotation colocada no *pai* de um osso com aim interage com o aim.
Se um osso acabar apontando para um lugar estranho, execute `explain` nele e no pai dele e leia a lista de contribuições
de cima para baixo; depois reordene o array `constraints`.

## 4. Cinemática inversa

```
add_bone("goal", "", 10, 200, 60, 0);
add_ik("chain", ["b1","b2","b3"], "goal", 1);   // name, bones root->tip, target bone, bend
```

* Dois ossos: um solver analítico. `bend` (+1 ou -1) escolhe para que lado a articulação dobra.
* Três ou mais ossos: FABRIK. `iterations` é um piso; o solver para antes assim que a ponta fica a menos de
  0.01 px e, caso contrário, roda até um limite rígido.
* `mix` é a mistura FK/IK (0 = a pose FK animada por chaves, 1 = IK total). Ele pode ser animado com a propriedade
  `ik_mix`, que é como um braço alterna entre FK e IK. `autorig` adiciona cadeias com `mix 0`, para que a animação
  FK existente continue funcionando.
* O alvo é a **cabeça** de um osso; não dê arte a ele e use um comprimento pequeno (`goal`, `hand_ctrl`).

### Alvos inalcançáveis: reto, soft, stretch

A cadeia `b1..b3` tem alcance de 240 a partir da raiz (60,150). Movendo o goal para (330,60), a 285 de distância:

| Configuração | Resultado (posição da ponta) |
|---|---|
| padrão | totalmente estendida em direção ao goal, ponta (287.7, 74.1), exatamente a 240 da raiz |
| `"stretch": 2` | a cadeia se alonga, a ponta alcança (330.0, 60.0) |
| `"soft": 0.5` (3 ossos) | a cadeia permanece dobrada, ponta (258.8, 83.7); ela não se endireita |

`soft` (0 a 0.9) e `stretch` (1 a 4) são campos da cadeia de IK no JSON do projeto; não existe
comando nem setter de script:

```
python3 - <<'EOF'
import json
d = json.load(open('03-ik.json'))        # the Buddy project of GETTING_STARTED.md
d['ik'][0]['soft'] = 0.5                 # or: d['ik'][0]['stretch'] = 1.5
json.dump(d, open('ik_soft.json', 'w'))
EOF
osatura check ik_soft.json
```

No braço do Buddy (alcance 80 a partir do ombro em (160,140)) com o goal em (250,140), a 90 de distância:

```
soft 0,   stretch 1    tip (240.0, 140.0)    straight, 10 short of the goal
soft 0.5, stretch 1    tip (228.5, 140.0)    held back, elbow still bent
soft 0,   stretch 1.5  tip (250.0, 140.0)    arm lengthened, goal reached
```

O soft IK em um membro de dois ossos se aproxima da extensão total de forma assintótica: à medida que o goal se afasta para 100, 120,
135 e 150 de distância, o x da ponta se estabiliza em 231.1, 234.6, 236.3 e 237.4 e nunca chega a 240. Esse é o comportamento
pretendido de "sem estalo no joelho". Com `stretch` também acima de 1, a zona soft é a última fração `soft` do
alcance esticado (`reach x stretch`): a cadeia se alonga continuamente e suaviza até o seu comprimento máximo (antes da wave 7,
`soft` era ignorado além do alcance total e a cadeia se endireitava de repente nesse ponto).

**Armadilha.** O `lint` informa alvos inalcançáveis como informação, não como erros:
`Info L003 [leg_ik_l] target 'foot_goal_l' is out of reach at frame 12 (148 px > 144 px); the chain will be fully stretched`.
Leia essas linhas; uma perna que "flutua" quase sempre é uma delas.

**Armadilha.** O `fuzz` (veja [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)) amostra poses aleatórias, incluindo posições de
controladores muito além do alcance, de modo que um rig com IK mostra uma contagem alta de `ik_unreachable`. Isso é uma afirmação
sobre a faixa amostrada, não um bug.

## 5. Skinning

O OSATURA tem quatro mecanismos de skinning; escolha de acordo com o tipo de arte que você tem.

### 5.1 Âncoras vetoriais com pesos (JSON)

As âncoras de um caminho vetorial podem carregar `weights`, uma lista de `{bone, weight}`. A
âncora então segue uma mistura linear das transformações dos ossos. Esta é uma faixa de 12 âncoras com skinning distribuído entre os
três ossos; os `weights` vão de `b1` a `b2` e a `b3`:

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

No quadro 12, o ângulo de mundo de `b3` é 50.0 graus e a cabeça dele está em (191.4, 211.3) (segundo o `explain`). A faixa dobra
suavemente entre as articulações porque os pesos se sobrepõem ali.

**Armadilha.** Não existe comando que calcule pesos vetoriais para você: a API do projeto tem um skinner vetorial
automático (`Project::auto_skin`), mas ele é uma função de biblioteca, usada pela demo, e não é exposto como comando da
CLI. Para arte vetorial, use a fixação rígida (seção 2) ou escreva os pesos como acima.

**Armadilha.** Poucas âncoras, poucos lugares para dobrar. Um retângulo simples tem quatro âncoras, então só pode dobrar
nos cantos. Subdivida a forma (adicione âncoras ao longo das arestas) antes de aplicar skinning ou um deformador.

### 5.2 Malhas de imagem: `mesh-trace` e `skin-diffuse`

Para arte raster, trace a silhueta do alfa em uma malha de triângulos e depois calcule os pesos por difusão de calor:

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

`skin-diffuse <project> <layer> <out> [--influences N]` atribui a cada vértice da malha até N pesos de ossos. Ele
precisa de uma malha: em uma camada de imagem sem malha, ele para com `layer 'flag' has no mesh`. Com um único osso,
todo vértice recebe peso 1.0 nele (a bandeira tem um osso, `mast`), o que é correto e pouco
interessante; use pelo menos dois ossos para ver uma mistura. `mesh-trace` também tem `hotspot=x,y,r` para tornar a malha mais
densa onde você espera dobras, e opções para aplicar skinning durante o traçado (`bones=R`, `skin=1`; veja
[CLI_REFERENCE.md](CLI_REFERENCE.md)).

### 5.3 Skin sets (roupas, expressões, estados de dano)

O comando `skin` é uma ideia diferente: um **skin set** troca grupos inteiros de camadas (e, opcionalmente,
ossos, restrições e cadeias de IK) em conjunto.

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

`apply` mostra um skin e oculta os outros do mesmo grupo. `schedule` anima a escolha com **chaves em degrau na opacidade da camada** (o projeto não tem um indicador "visible" animável), portanto use-o em camadas cuja opacidade você não anime de outra forma.
Os ossos por skin inativos não podem ser excluídos do rig sem quebrar os índices, então eles são **colapsados**:
um canal `multiply` em `bone_scale` com chaves em degrau 0 e 1 faz a escala do osso ser 0, e tudo o que o acompanha
ou tem peso nele encolhe até virar um ponto:

```
{"target":"tail","prop":"bone_scale", ... "keys":[{"frame":0,"value":1,"ease":{"type":"step"}},
                                                  {"frame":12,"value":0,"ease":{"type":"step"}}], "mode":"multiply"}
```

Regras impostas pela validação: um osso pertence a no máximo um skin; todo descendente de um osso de skin pertence ao mesmo skin; os ossos de skin
não devem ter `scale_limits`. Use `skin rig <group> <name> <constraint,...|-> <ik,...|->` para restrições e cadeias de IK que
devem estar ativas apenas com um skin.

### 5.4 Peças rígidas mais uma camada switch

Para coisas que mudam de forma de maneira discreta (bocas, mãos, olhos), coloque as alternativas em uma camada `switch`
e anime `layer_switch` com chaves. Veja a seção 11.

## 6. Deformadores

Os deformadores movem os pontos de uma camada. Eles são listados em `deformers` da camada, são animados por meio de
canais com um alvo em três partes `"layer::deformer::point"` e agem sobre as **âncoras** de um caminho ou sobre os vértices de uma
malha de imagem. **Se uma forma tiver poucas âncoras, ela mal vai se deformar: subdivida-a primeiro.**

### Cage (coordenadas de valor médio)

```
$ osatura cage lab0.json c1.json strip auto verts=3 pad=10
added cage 'cage' (12 vertices) to 'strip'; wrote c1.json
$ osatura cage-pose c1.json c2.json strip 12 move=1:0,-40 ease=ease_in_out
keyed cage 'cage' on 'strip' at frame 12; wrote c2.json
```

`cage` envolve uma camada em um polígono (`auto` = limites com margem, `hull` = envoltória convexa, `verts=N` = segmentos por
lado); o conteúdo interno segue os vértices do polígono, e o conteúdo externo não é afetado. `cage-pose` anima com chaves os offsets
dos vértices (canais `cage_dx`, `cage_dy` com o alvo `strip::cage::1`). `timeline` os lista em
"Deformers and paths". Um canal bloqueado é recusado.

### Lattice, spine, pin (JSON)

Esses três não têm comando; adicione-os aos `deformers` de uma camada e anime-os com canais:

```
"deformers": [{"type":"lattice","name":"ffd","cols":4,"rows":1,"rect":[60,138,240,24]}]
"channels":  [{"target":"strip::ffd::2,0","prop":"lattice_dy","source":"keys",
               "track":{"keys":[{"frame":0,"value":0},{"frame":12,"value":-40}]}}]

"deformers": [{"type":"spine","name":"curve","rest":[[60,150],[180,150],[300,150]],"radius":40}]
"channels":  [{"target":"strip::curve::1","prop":"spine_dy", ...}]

"deformers": [{"type":"pin","name":"pins","pins":[[60,150],[180,150],[300,150]],"radius":90}]
"channels":  [{"target":"strip::pins::2","prop":"pin_dy", ...}]
```

* **Lattice**: uma grade de forma livre. O id do ponto é `col,row`. Bom para squash e stretch.
* **Spine**: uma polilinha com um `radius` de influência; mover um ponto da spine dobra a arte próxima (caudas, tentáculos).
* **Pin**: deformação por pinos, estilo puppet; cada pino puxa os pontos dentro de `radius`.

Cada um deles foi renderizado corretamente na folha de contatos acima quando aplicado a uma faixa sem pesos de ossos. Um caminho
que **também** tenha skinning de ossos será deformado pelos dois; remova os pesos da faixa de teste se quiser ver um
deformador sozinho.

### Deformar uma seleção: `warp-pivot`, `soft-scale`, `layer-pivot`

```
osatura warp-pivot c1.json w1.json strip 12 deformer=cage verts=1-2 rotate=25 pivot=centroid radius=60 falloff=smooth ease=ease_in_out
osatura soft-scale c1.json w2.json strip 12 deformer=cage scale=1.5
osatura soft-scale c1.json w3.json strip 12 height=60 aspect=free
osatura layer-pivot lab0.json w4.json strip pivot=60,150
```

`warp-pivot` gira, escala ou move uma seleção de pontos de controle (cage, lattice, cloth, pin, spine ou uma
malha de imagem em grade) em torno de um pivô (`x,y`, `bone:NAME`, `centroid`, `center`) e anima o resultado com chaves. `radius` adiciona uma
atenuação de seleção suave (`linear`, `smooth`, `gauss`, `sharp`, `constant`); o exemplo acima selecionou os vértices 1-2 e
a atenuação acrescentou 7 e 8, de modo que 8 canais receberam chaves. `soft-scale` faz redimensionamento proporcional (`aspect=free` permite que
os dois eixos difiram). `layer-pivot` move o pivô de uma camada sem mover a arte dela.

### Mesh locks e glue

* `mesh-lock add <in> <out> <name> host=LAYER follower=LAYER [verts=...] [strength=1]`: os vértices do follower
  seguem a superfície do host por deslocamento baricêntrico. Use-o para manter uma tatuagem, um remendo ou um botão em um
  tecido que se deforma. Ciclos são recusados. `mesh-lock list` imprimiu `lock: patch:0 follows host:0 vertices all strength 1`.
* `glue add <in> <out> <name> a=LAYER b=LAYER auto=RADIUS|pairs=i:j;i:j [mode=rigid|snap|offset]`: pares de vértices
  de duas camadas são soldados para que `b` siga `a`. `glue list` imprimiu
  `weld: Rigid strength 1 host:0#1->edge:0#0 host:0#2->edge:0#3`.

### Cloth

O cloth precisa de uma malha em **grade**. Uma malha traçada de forma livre é recusada:
`cloth needs a grid mesh; the layer has a free-form (traced) mesh`. Peça uma grade com `cols=` e `rows=`:

```
$ osatura cloth flag.json flag_c.json flag pin=top gravity=900 wind=140,0 gust=0.5 cols=8 rows=10 from=0 to=35
baked cloth on 'flag' (180 channels); wrote flag_c.json
```

Esse bake levou 0,43 s no build de depuração e produziu 180 canais com chaves (6480 chaves), de modo que o resultado é totalmente
editável, é exportado como qualquer outra animação e é determinístico. `pin=` aceita `top`, `left`, `corners`, `none`,
`row:N`, `col:N`, `v:N` ou `...@bone` para fixar a um osso. `cols`/`rows` definem a grade de simulação; a imagem renderizada é deformada por ela,
e você pode ver pequenas emendas de triângulos em um render PNG com zoom alto.

## 7. Física e molas

### Física baked: `physics`

`physics` simula uma cadeia de ossos (gravidade, vento, colisões) e grava o resultado em chaves comuns de ângulo de osso.

```
osatura physics lab0.json ph1.json b2,b3 preset=tail gravity=600 --allow-empty
osatura physics lab0.json ph2.json b2,b3 preset=hair circle=200,230,30
osatura physics lab0.json ph3.json b2,b3 gravity=600 capsule=150,200,250,220,15 wind=40,0 gust=0.5
osatura physics lab.json  ph4.json b2,b3 gravity=300 drive=b1:0.5 vortex=b1:0.8:0.2:120
```

* `preset=` é um de `hair`, `tail`, `rope`, `antenna`, `cape`.
* `circle=x,y,r[@bone];...` e `capsule=x1,y1,x2,y2,r[@bone];...` são colisores; `@bone` faz um colisor seguir um osso.
* `drive=bone:gain` faz o movimento de outro osso empurrar a cadeia; `vortex=bone|x,y:swirl[:radial[:radius]]`
  adiciona um campo de força em redemoinho; `follow=bone` e `lateral=` moldam como a cadeia segue um pai.
* `merge=keep` (o padrão) **não sobrescreve** um osso que já tenha animação:
  `kept existing animation on 'b2' (use merge=replace to overwrite)`. Use `merge=replace` quando for essa a sua intenção.
* Cadeias no estilo `b2,b3/b1` e um osso desconhecido (`no bone 'b9'`) são validados.

### Molas ao vivo: `springs`

Um `Spring` adiciona um follow-through amortecido a um osso no momento da avaliação, sem nada gravado (baked). Ele existe apenas em JSON:

```
"springs": [{"bone":"b3","frequency":4.0,"damping":0.3}]
```

`frequency` é em Hz (quanto maior, mais rígida), `damping` é uma razão (abaixo de 1 oscila, 1 é criticamente amortecido) e `warmup`
(ativado por padrão) faz uma passagem prévia para que os loops comecem em regime estacionário. O `explain` diz:
`spring: world rotation lags behind via a 4.0 Hz, damping 0.30 oscillator (pre-rolled for loops)`.

Para reproduzir uma sensação que você desenhou, ajuste-a:

```
$ osatura spring-fit --start 0 --samples 0.3,0.7,1.1,1.2,1.05,0.95,1.0,1.02 --fps 24
frequency 3.36 Hz, damping 0.52 (rms error 0.055 of the step size)
add to the project: {"bone": "<bone>", "frequency": 3.36, "damping": 0.52}
```

**Armadilha.** Molas não fazem parte do Lottie. Faça o bake delas (`physics`) se exportar para um formato que não tenha
equivalente; `export --report` lista o que foi aproximado.

## 8. Auto-rig, tagging e `coach`

Dê aos seus ossos e camadas nomes com palavras padrão (`arm_upper_l`, `thigh_r`, `eye_l`, `mouth`) e o OSATURA pode
reconhecer os papéis deles:

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

`autorig <in> <out> [seed]` então adiciona automação não destrutiva a partir desses papéis:

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

O projeto passou de 11 ossos e 0 cadeias de IK para 15 ossos e 4 cadeias de IK, com 3 novos canais. Tudo o que foi
adicionado são dados comuns: exclua o que você não quiser. As partes ausentes são ignoradas com um motivo
(`- arm_l: upper/lower bone not found`, `- breathing: no torso bone`, `- blink: no eye layers`), então leia o relatório.

**Armadilha.** `autorig` é um comando de projeto para projeto e a saída pode ser a própria entrada. Em um arquivo que lhe importa,
grave com um novo nome: `osatura autorig buddy.json buddy.json` substitui `buddy.json` (a gravação é atômica, mas
não há desfazer).

`coach <project>` lê a animação e imprime sugestões de timing e de princípios; `lint` verifica o rig
estaticamente. Ambos são apenas consultivos.

## 9. Espelhamento e simetria

`mirror <in> <out>` espelha todos os canais animados por chaves em relação ao eixo vertical: os alvos trocam de lado (prefixos `L_`/`R_`,
sufixos `_L`/`_R` e `_l`/`_r`, as palavras left/right) e as rotações e translações horizontais
mudam de sinal. Os canais espelhados **substituem** os canais do projeto. No biped caminhando, as chaves da coxa esquerda
`0.0, 10.7, 19.8, 25.9` se tornaram `-0.0, -10.7, -19.8, -25.9` na coxa direita e vice-versa, e `hips` (sem lado)
manteve seus valores. `diff` lista os seis canais alterados:

```
$ osatura mirror walk.json mir.json
wrote mir.json
$ osatura diff walk.json mir.json
~ channel 'arm_upper_l:BoneAngle' changed (track)
~ channel 'arm_upper_r:BoneAngle' changed (track)
...
6 difference(s)
```

**Armadilha.** Em um projeto sem nomes de esquerda/direita, `mirror` grava silenciosamente uma cópia inalterada e diz
`wrote`. Sempre use `diff` no resultado. Nomeie os lados de forma consistente (`_l`/`_r`) antes de animar.

## 10. Troca de vistas e troca de pai

### Turnarounds: `view-swap`

Um turnaround (frente, três quartos, perfil) é uma camada switch com um filho por vista, controlada pela propriedade de um osso
controlador:

```
cat > ctl.rhai <<'EOF'
add_bone("view_ctrl", "", 10, 20, 20, 0);
EOF
osatura script biped.json v0.json ctl.rhai
osatura view-swap v0.json v1.json mouth view_ctrl.bone_offset_x "-40:0,0:2,40:4"
```

Canal de saída: um canal `driven` `layer_switch` ← `view_ctrl.bone_offset_x` com uma curva em degrau (filho 0 abaixo de 0,
filho 2 a partir de 0, filho 4 a partir de 40). Anime o `bone_offset_x` de `view_ctrl` para trocar de vista. Os índices dos filhos devem existir e a camada
deve ser um switch (`child index 5 is out of range (the layer has 3)`).

`slider <in> <out> <name> "at:target.prop=val,...;at:..."` cria um **pose slider**, um osso controlador `ctrl_<name>` cujo
`offset_x` mistura poses armazenadas:

```
$ osatura slider biped.json sl.json look "0:head.bone_angle=0;24:head.bone_angle=15"
controller bone: ctrl_look (animate its offset_x)
```

### Reparentear ao longo do tempo: `parent-switch`

Para passar um objeto de uma mão para a outra (ou para pegar um adereço) sem estalo:

```
$ osatura parent-switch biped.json ps.json arm_upper_r 0:torso,12:head
wrote ps.json
```

Ele grava (bake) canais de offset e ângulo do osso (aqui 3 canais, com uma chave em cada quadro) para que o osso mantenha sua posição
de mundo na troca. Não funciona para ossos com IK, restrições ou molas.

## 11. Rigs de rosto

### Formas de boca

Desenhe as formas de boca como filhos de uma **camada switch** nesta ordem, que todo comando de lip-sync pressupõe:
**rest, small, open, round, wide**. A boca do biped é `add_switch("mouth", ["m_rest","m_small","m_open","m_round","m_wide"])`.
Se o seu switch tiver menos filhos, `--map 0,1,2,3,4` (cinco números, um por forma) escolhe qual filho cada forma usa; o padrão
segue a contagem de filhos (1 filho: todas; 2: fechada+aberta; 3: fechada+aberta+redonda/larga; 4: fechada+pequena+aberta+redonda/larga; 5+: nessa ordem).

Fixe o switch ao osso da cabeça (`set_layer("mouth", "bone", "head")`) para que ele gire com a cabeça. Animá-lo é
o assunto da [seção de lip-sync do guia de animação](ANIMATION_GUIDE.md#8-lip-sync-e-fala).

### Olhos e clipping

`eye-clip <in> <out>` mascara as camadas de íris e pupila pela forma do olho do mesmo lado, de modo que uma pupila não possa sair do
olho. `autorig` o executa. Em um rig sem camadas de íris/pupila, ele grava o arquivo sem alterações. As piscadas são um
`layer_scale_y` animado por chaves ou procedural na camada do olho:

```
$ osatura idle blink p1.json i1.json --target eye_l --prop layer_scale_y --interval 2 --seed 4 --closed 0.1
wrote i1.json
```

`idle blink|breath|sway` geram canais procedurais com seed (`--interval`, `--amp`, `--period`, `--freq`,
`--seed`, `--closed`): a mesma seed produz os mesmos tempos de piscada em qualquer máquina.

### Controlar um rosto a partir de uma performance

`face-synth`, `face-track`, `face-bake` e o painel Face do estúdio transformam um fluxo de landmarks em canais. Eles são
tratados, com números de precisão honestos, na [seção de rastreamento facial do guia de animação](ANIMATION_GUIDE.md#10-rastreamento-facial-e-seus-limites).

## 12. Câmera, planos e parallax

* `shot <in> <out> <name> <x> <y> <zoom> [rotation]` salva um enquadramento de câmera nomeado (centro da câmera e zoom);
  `camera-seq <in> <out> <frame:shot[:cut],...>` anima a câmera com chaves através de planos salvos; `timeline` mostra os
  canais de câmera resultantes.
* `parallax <in> <out> <far_layer,...,near_layer> <far_value> <near_value>` distribui um fator `parallax` pelas
  camadas de nível superior (as mais distantes se movem menos quando a câmera faz pan).
* `dof <in> <frame> <out.png> [--focus 1.0 --strength 12 --max-radius 24]` desfoca cada camada de nível superior por
  `|parallax - focus|`; `dof-export` renderiza um intervalo para GIF, APNG ou uma pasta. O foco e a intensidade seguem os canais
  `camera_focus` / `camera_blur`, a menos que você os fixe.

Verificado no biped caminhando:

```
osatura shot walk.json sh1.json wide 200 180 1          # name x y zoom
osatura shot sh1.json sh2.json close 200 100 2.2
osatura camera-seq sh2.json sh3.json 0:wide,12:close,23:wide
osatura parallax sh3.json px.json sky,ground 0.2 1.0
osatura dof px.json 12 dof.png --focus 1.0 --strength 10
osatura dof-export px.json dof.gif --from 0 --to 11 --scale 0.5     # 80 KB
```

Os efeitos de camada (blur, drop shadow, tint, fill, outline, posterize...) podem ser por camada ou aplicados como uma cadeia: veja o
[guia de animação](ANIMATION_GUIDE.md#15-efeitos-de-camada).

## 13. Uma checklist de rigging

1. Use `check` após cada alteração estrutural; use `lint` antes de animar.
2. Mantenha a pose de repouso simples e simétrica; anime deltas a partir dela.
3. Coloque os controladores (goals de IK, `view_ctrl`, `root_ctrl`) no rig como ossos sem arte.
4. Prefira peças rígidas para arte de desenho animado, aplique skinning apenas ao que precisa dobrar e subdivida antes de deformar.
5. Use `explain` quando algo estiver errado; é mais rápido do que adivinhar.
6. Faça o bake de física e cloth se a animação precisar sobreviver em um formato sem esses recursos, e leia `export --report`.
7. Grave em novos nomes de arquivo até confiar em um comando; comandos de projeto para projeto podem sobrescrever a própria entrada.

## 14. Limites que você deve conhecer

* O skinning de arte vetorial é **linear blend** (não dual quaternion), com pesos fornecidos por você; não existe skinner
  vetorial automático na CLI.
* Lattice, spine, pin, `soft`/`stretch` do IK e molas existem apenas em JSON.
* Física e cloth são bakes: se você alterar o rig, faça o bake de novo. `physics` não sobrescreve a animação existente, a menos que isso seja pedido.
* O cloth precisa de uma malha em grade; malhas traçadas de forma livre servem para skinning e mesh locks.
* Lottie e GIF não conseguem representar tudo (traços de largura variável, gradientes distorcidos, molas...). Os avisos são
  impressos na exportação; veja [EXPORT_FORMATS.md](EXPORT_FORMATS.md) e [LIMITATIONS.md](LIMITATIONS.md).
* O detector de rostos é treinado apenas com rostos sintéticos (seção 11 e o guia de animação).
