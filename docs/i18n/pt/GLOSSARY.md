<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->

# Glossário do OSATURA

Termos de domínio usados no código e na documentação, na ordem alfabética do original em inglês. "Veja" aponta para o
módulo (em `osatura-core/src`, salvo indicação em contrário) ou para o documento em que o termo é implementado ou
especificado. Documentos: [PROJECT_FORMAT.md](PROJECT_FORMAT.md), [ARCHITECTURE.md](ARCHITECTURE.md).
As fontes das técnicas estão listadas em `CLEANROOM.md`.

**Restrição aim.** Uma restrição que gira um osso de modo que seu eixo +x aponte para a cabeça de outro osso,
mais um deslocamento. Veja `rig`; PROJECT_FORMAT seção 4.3.

**Âncora.** Um ponto de um caminho de Bézier, com uma posição e alças de entrada e de saída opcionais; pode também
carregar pesos de ossos. Veja `path`; PROJECT_FORMAT seção 6.

**Limites de ângulo.** Mínimo e máximo por osso da rotação animada, em graus, aplicados depois dos
canais e depois de cada cadeia de IK. Veja `rig`, `engine`.

**Versão da API.** A versão da superfície estável em Rust (`api` e `prelude`), atualmente 1.4.0,
verificada por um teste de snapshot. Não é o mesmo que o `format` do arquivo nem o `ABI_VERSION` da C. Veja `api`.

**Recurso (asset).** Um arquivo referenciado por um projeto (imagem, normal map, som). Os caminhos devem ser relativos e permanecer
dentro do diretório do projeto, a menos que `--allow-external-assets` seja informado. Veja `assets`; PROJECT_FORMAT
seção 18.

**Bake.** Calcular valores ao longo do tempo e armazená-los como canais ou chaves comuns, de modo que deixem de
precisar da ferramenta que os calculou. Usado por física, clipes, magnet, footlock, máquinas de estados e mais. Veja
ARCHITECTURE seção 4.

**Dobra (bend).** O sinal (+1 ou -1) que escolhe para que lado uma cadeia de IK de dois ossos dobra, como cotovelo para dentro ou para fora.
Veja `rig`.

**Curva de Bézier.** Uma curva cúbica paramétrica definida por duas âncoras e suas alças. Achatada em segmentos
de reta (subdivisão de de Casteljau) para preenchimento e medição. Veja `path`.

**Binding.** Uma regra de runtime que mapeia um valor externo (uma variável de jogo, um slider) por meio de uma
expressão, um remap e uma suavização para uma propriedade ou uma entrada de máquina de estados. Veja `binding`.

**Modo de mesclagem.** A forma como a cor de uma camada se combina com o que está abaixo dela (multiply, screen, overlay e
assim por diante). As fórmulas seguem o W3C Compositing and Blending Level 1. Veja `raster`, `fraster`.

**Blend tree.** Uma mistura 1-D de vários clipes controlada por uma entrada numérica, usada dentro de uma máquina de
estados. Veja `machine`.

**Blob.** Um círculo de uma camada líquida (metaball). Veja `fx`.

**Osso.** Um segmento nomeado do esqueleto com posição da cabeça, comprimento, ângulo de repouso e pai
opcional. Os ossos controlam a geometria com skinning e as camadas anexadas. Veja `rig`; PROJECT_FORMAT seção 4.

**Camada booleana.** Uma camada que combina caminhos vetoriais por união, interseção, diferença ou xor. O
primeiro caminho é o sujeito e os demais são unidos como o recorte. Veja `fx`.

**Bounce (easing).** Uma curva de easing de uma bola que cai e quica com restituição 0,6 e três
quiques. Uma derivação original a partir da queda livre. Veja `anim`.

**Caixa delimitadora.** O menor retângulo alinhado aos eixos em torno de uma geometria; usado para alinhamento, seleção e
ajuste de normal maps. Veja `layout`, `pick`.

**Deformador cage.** Um deformador em que o conteúdo dentro de um polígono fechado segue os vértices do polígono
usando coordenadas de valor médio. Veja `cage`; veja também MVC.

**Spline de Catmull-Rom.** Uma spline cúbica que passa por seus pontos de controle. O easing `smooth` usa
uma versão não uniforme através das chaves vizinhas. Veja `anim`, suavização de `liquid` em `fx`.

**Canal.** Um valor animado: um alvo, uma propriedade, uma fonte (chaves, expressão, noise ou controlado) e
um modo de mesclagem e um peso. Os canais são executados na ordem da lista. Veja PROJECT_FORMAT seção 8.

**CIE76.** A diferença de cor original, a distância euclidiana em CIE L*a*b*. Barata, menos precisa
que a CIEDE2000. Veja `percep`.

**CIEDE2000.** A fórmula de diferença de cor CIE 2000 (CIE TR 142-2001), implementada como impressa em
Sharma, Wu e Dalal (2005) e verificada com seus pares de teste publicados. Usada pela CI de animação para
avaliar a mudança visível. Veja `percep`, `ci`; `docs/CI.md`.

**Clean room.** A regra de desenvolvimento segundo a qual o código é escrito a partir de padrões públicos e métodos de livros didáticos,
sem código copiado e sem reprodução de formatos ou nomes de outras ferramentas. Cada técnica é registrada em
`CLEANROOM.md`.

**Clipe.** Um take reutilizável de trilhas de animação que pode ser colocado na timeline com velocidade, loops,
peso, fades e um modo de mesclagem. Veja `clipstack`; PROJECT_FORMAT seção 11.1.

**Cloth (tecido).** Um deformador de grade cujos vértices se movem exatamente pelos seus próprios deslocamentos. Também é a simulação de
dinâmica baseada em posição que faz o bake desses deslocamentos. Veja `cloth`.

**Restrição.** Uma regra aplicada depois do IK que limita um osso: aim, copy rotation ou pin. Veja `rig`.

**Contêiner (layout).** Um grupo de layout unidimensional no estilo flexbox, com espaçamento, preenchimento, justificação,
alinhamento e quebra de linha, que posiciona camadas e outros contêineres. Veja `containers`.

**Restrição copy-rotation.** Uma restrição que faz um osso acompanhar a rotação no mundo de outro osso. Veja
`rig`.

**CRDT.** Tipo de dado replicado livre de conflitos (conflict-free replicated data type): uma estrutura que réplicas podem editar de forma independente e
mesclar com o mesmo resultado, sem coordenação. O OSATURA usa um CRDT baseado em operações. Veja `collab`;
ARCHITECTURE seção 10. Relacionados: registrador LWW, RGA, relógio de Lamport, relógio vetorial, tombstone.

**Easing de Bézier cúbica.** Uma curva de easing no estilo CSS definida por dois pontos de controle entre (0,0) e
(1,1). Veja `anim`.

**Deformador.** Um modificador de forma aplicado a uma camada vetorial ou de imagem no espaço de repouso: lattice, spine, pin,
cage ou cloth. Veja PROJECT_FORMAT seção 7.1.

**Triangulação de Delaunay.** Uma triangulação de pontos em que nenhum ponto fica dentro do círculo circunscrito de qualquer triângulo,
o que evita triângulos finos. Construída com o algoritmo de Bowyer-Watson para malhas de imagem.
Veja `meshtrace`.

**Profundidade de campo (DoF).** Desfoque que aumenta com a distância do `parallax` de uma camada em relação ao `focus` da
câmera. Aplicado apenas pelos comandos `dof` e `dof-export`. Veja `dof`.

**Determinismo.** A propriedade de que a mesma entrada produz a mesma saída. A avaliação de quadros é pura;
a renderização é idêntica byte a byte em uma máquina e um build, não verificada entre plataformas. Veja
ARCHITECTURE seção 6.

**Dope sheet.** Uma visão da timeline que mostra as chaves como marcas por canal e as edita no tempo. Veja
`timeline`, `keyops`.

**Chave controlada (driven key).** Um canal cuja entrada é o valor de outra propriedade em vez do tempo, por exemplo ângulo do cotovelo
para tamanho do volume. Veja `drivers`; PROJECT_FORMAT seção 8.1.

**Easing.** A função de modelagem aplicada ao segmento que começa em uma chave. Veja `anim`; PROJECT_FORMAT
seção 8.4.

**Editor.** O objeto que é dono de um projeto e de seu histórico de desfazer e que aplica ops. Veja `edit`;
ARCHITECTURE seção 5.

**Emissor.** Uma camada que produz partículas como uma função em forma fechada do tempo. Veja `fx`.

**Engine (motor).** A forma validada, com nomes resolvidos, de um projeto que avalia quadros. Veja `engine`.

**Envelope (collab).** A mensagem que transporta ops entre réplicas: id da réplica, número de sequência,
tempo de Lamport, dependências e operações. Veja `collab`.

**Avaliação.** Calcular a pose, a geometria e a câmera de um quadro a partir de um projeto. Uma função pura
do quadro. Veja PROJECT_FORMAT seção 16.

**FABRIK.** Forward And Backward Reaching Inverse Kinematics: um solver de IK iterativo que alternadamente
puxa a cadeia em direção ao alvo a partir da ponta e depois a reancora na raiz. Usado para cadeias de
mais de dois ossos. Veja `rig`.

**FK.** Cinemática direta (forward kinematics): calcular as transformações no mundo dos ossos a partir das transformações de seus pais e
de seus próprios ângulos e deslocamentos. Veja `rig`, `engine`.

**Versão do formato.** O campo inteiro `format` de um projeto, atualmente 1. Arquivos mais novos são rejeitados.
Veja PROJECT_FORMAT seção 19.

**Quadro (frame).** (1) Um passo de tempo da timeline, `0..frames-1`. (2) A struct `Frame`: o resultado da
avaliação de um passo (ossos, geometria, câmera, flags). Veja `engine`.

**Glue.** Um vínculo que faz vértices de uma malha seguirem vértices de outra (snap, rigid ou offset). Veja
`links`.

**Alça de gradiente.** Um ponto de controle de um gradiente linear ou radial que pode ser animado com
`grad_dx` e `grad_dy`. Veja PROJECT_FORMAT seção 8.2.

**Camada de grupo.** Uma camada que contém outras camadas e lhes repassa sua transformação. Veja `project`.

**Alça (handle).** Um ponto de controle de Bézier associado a uma âncora. Coordenadas absolutas no arquivo. Veja `path`.

**HOG.** Histograma de gradientes orientados, uma característica de forma local (Dalal e Triggs 2005) usada pelo detector
de rostos. Veja `facedet`.

**IK.** Cinemática inversa (inverse kinematics): calcular os ângulos dos ossos de modo que a ponta de uma cadeia alcance um alvo. Veja `rig`.
Relacionados: FABRIK, IK suave, IK com alongamento, dobra.

**IK mix.** A mistura entre a pose de FK (0) e o resultado do IK (1) para uma cadeia. Animada com `ik_mix`.
Veja PROJECT_FORMAT seção 4.2.

**Malha de imagem.** Uma malha de grade ou de triângulos livres sobre uma camada de imagem, usada para pesos de ossos e deformadores.
Veja PROJECT_FORMAT seção 5.8.

**Keyframe (chave).** Uma tripla (quadro, valor, easing) em uma trilha. O easing molda o segmento que começa
na chave. Veja `anim`.

**L*a*b*.** Espaço de cor CIE 1976 concebido para que as distâncias aproximem as diferenças percebidas. Veja
`percep`.

**Relógio de Lamport.** Um contador lógico que ordena eventos em um sistema distribuído sem um relógio
compartilhado (Lamport 1978). Veja `collab`.

**Lattice.** Um deformador com uma grade de pontos de controle sobre um retângulo, cujos deslocamentos dobram o conteúdo
por meio de mistura de Bézier-Bernstein. Veja PROJECT_FORMAT seção 7.1.

**Camada (layer).** Um nó da árvore da cena: vetorial, de grupo, de troca, líquida, booleana, emissor ou de imagem. Veja
PROJECT_FORMAT seção 5.

**Efeito de camada.** Um filtro por camada (desfoque, sombra projetada, tonalização, preenchimento) renderizado em um canvas fora da tela.
Veja `layerfx`.

**Nível de detalhe (LOD).** Uma versão simplificada de um projeto produzida sob um limite de erro declarado
em pixels, para pré-visualização rápida ou saída pequena. Veja `lod`, `osatura lod`.

**Linter.** O verificador consultivo (`osatura lint`) que relata problemas de qualidade da animação com códigos
como L011 e L012. Separado da validação. Veja `lint`.

**Camada líquida.** Uma camada de metaballs cujo contorno é o isocontorno de um campo de soma de influências. Veja
`fx`.

**Registrador LWW.** Registrador last-writer-wins (a última escrita vence): uma célula de CRDT em que a escrita com o maior carimbo
lógico de tempo sobrevive. Usado para chaves de objetos. Veja `collab`.

**Magnet.** Uma ferramenta de bake que puxa posições em direção a um ponto com atenuação smoothstep. Veja `magnet`.

**Marching squares.** Um algoritmo que extrai linhas de contorno de um campo escalar amostrado em uma grade.
Usado para contornos líquidos e vetorização de bitmaps. Veja `fx`, `trace`, `meshtrace`.

**Marcador.** Um quadro ou uma região de quadros nomeados na timeline. As máquinas de estados podem usar uma região de marcador como
fonte. Veja `keyops`; PROJECT_FORMAT seção 10.2.

**Máscara.** Uma camada cujo alfa limita a visibilidade de outra camada; pode ser invertida. Uma camada oculta ainda
pode ser uma máscara. Veja `raster`.

**Merge (CRDT).** Combinar os estados de duas réplicas de modo que ambas convirjam para o mesmo documento. Veja
`collab`.

**Mesh lock.** Um vínculo que faz uma malha seguidora acompanhar uma malha hospedeira com uma intensidade de 0..1. Veja `links`.

**Metaball.** Um blob cuja influência de campo se mistura com a dos vizinhos, de modo que blobs próximos se fundem. Veja camada
líquida.

**Morph (blend shape).** Um conjunto alternativo de âncoras para um caminho, aplicado de forma aditiva com um peso. Veja
PROJECT_FORMAT seção 6.1.

**MVC.** Coordenadas de valor médio (Floater 2003): coordenadas baricêntricas generalizadas para um ponto dentro de
um polígono, usadas pelo deformador cage. Veja `cage`.

**Não destrutivo.** Que deixa o trabalho existente intacto. Os bakers acrescentam canais em vez de reescrever chaves, e
o editor registra ops inversas. Veja ARCHITECTURE seções 4 e 5.

**Normal map.** Uma imagem que armazena a direção da superfície como RGB, usada pelo efeito `relight` e pela
iluminação por camada. Veja `postfx`; PROJECT_FORMAT seção 7.3.

**Op.** Uma edição expressa como `set`, `insert`, `remove` ou `batch` sobre um ponteiro JSON da RFC 6901; ela tem uma
inversa para desfazer. Veja `edit`.

**OT.** Transformação operacional (operational transformation): uma alternativa aos CRDTs que reescreve operações concorrentes
umas em relação às outras, normalmente com um servidor central. O OSATURA não a usa; ele usa um CRDT com um sequenciador
central. Veja ARCHITECTURE seção 10.

**Parallax.** Um fator por camada que escala o movimento da câmera, de modo que camadas distantes se movem menos. Também é o
valor de profundidade usado para a profundidade de campo. Veja PROJECT_FORMAT seção 10.1.

**Partícula.** Um ponto de vida curta emitido por um emissor; sua posição é uma função em forma fechada do
tempo. Veja `fx`.

**Path (alvo).** O nome separado por barras de uma camada aninhada, como `parent/child`; os subalvos usam
`::`, como em `layer::path::morph`. Veja PROJECT_FORMAT seção 5.

**Path (vetorial).** Uma sequência de âncoras, aberta ou fechada, com preenchimento, traço e morphs opcionais. Veja
`path`.

**Restrição pin.** Uma restrição que desliza a cabeça de um osso até a cabeça de outro osso. Veja `rig`.

**Pin warp.** Um deformador em que pins arrastados movem a arte suavemente dentro de um raio, usando uma
função de base radial de suporte compacto (Wendland 1995). Veja `pinwarp`.

**Player.** O wrapper de runtime na facade que adiciona máquinas de estados e bindings a uma animação.
Veja `api`.

**Pré-ordem (preorder).** A ordem de visita da árvore de camadas: um nó e depois seus filhos. A avaliação a usa para que os
pais sejam calculados primeiro.

**Proxy.** Uma imagem de detalhe reduzido usada durante o scrub, construída a partir de um nível de detalhe. Veja `scrub`, `lod`.

**QEM.** Métrica de erro quádrico (Garland e Heckbert): uma medida do erro geométrico causado por
colapsar um vértice, usada na simplificação de malhas. Veja `lod`.

**Raster.** Uma imagem de pixels, ou o módulo que renderiza um quadro em pixels com tiny-skia. Veja `raster`.

**RDP.** Ramer-Douglas-Peucker: simplificação recursiva de linhas que mantém os pontos mais distantes que uma tolerância
em relação à corda. Veja `trace`, `sizefit`.

**Camada de referência.** Uma camada marcada com `reference`; oculta, a menos que o editor mostre as referências. Veja
PROJECT_FORMAT seção 5.1.

**Remap.** Um mapeamento afim de um intervalo, com uma curva e um clamp opcionais, aplicado a um valor de binding.
Veja `binding`.

**Réplica (collab).** A cópia do documento de um participante no CRDT. Veja `collab`.

**Pose de repouso.** As transformações dos ossos sem nenhuma animação aplicada; as matrizes de skin são `world * inverse(rest
world)`. Veja `rig`.

**RGA.** Array crescente replicado (Replicated growable array, Roh et al. 2011): um CRDT de sequência em que cada elemento tem um id
único e as exclusões deixam tombstones. Usado para arrays JSON. Veja `collab`.

**Rig.** O esqueleto e seus auxiliares: ossos, cadeias de IK, restrições e molas. Veja PROJECT_FORMAT
seção 4.

**Scrub.** Arrastar o cabeçote de reprodução; o estúdio guarda quadros em cache por revisão e usa proxies. Veja `scrub`.

**Skew.** Um ângulo de cisalhamento na transformação de uma camada. Veja PROJECT_FORMAT seção 5.1.

**Skin (animação).** Uma variante nomeada de camadas, ossos, restrições ou IK que pode ser ativada em um
grupo, como figurinos. Veja `skins`.

**Skinning.** Deformar a geometria misturando as transformações dos ossos, ponderadas por âncora ou vértice.
O OSATURA normaliza os pesos e trata pesos vazios como estáticos. Veja `engine`, `weights`.

**IK suave (soft IK).** Uma opção de IK que suaviza a extensão perto do alcance total para evitar um estalo ao esticar. Intervalo de 0 a
0,9. Veja `rig`.

**SPDX.** Software Package Data Exchange: uma forma padronizada de declarar a licença de um arquivo, como em
<!-- REUSE-IgnoreStart -->
`SPDX-License-Identifier: MPL-2.0`. Todo arquivo-fonte traz uma. Veja `LICENSING.md`.
<!-- REUSE-IgnoreEnd -->

**Deformador spine.** Um deformador que dobra a geometria ao longo de uma curva projetando âncoras sobre uma curva
de repouso e reanexando-as à curva deformada. Veja PROJECT_FORMAT seção 7.1.

**Mola (osso).** Uma mola amortecida de segunda ordem que faz a rotação de um osso atrasar e ultrapassar o alvo; passa por bake no
carregamento, de modo que qualquer quadro possa ser avaliado. Veja PROJECT_FORMAT seção 4.4.

**SSIM.** Índice de similaridade estrutural (Wang et al. 2004): uma medida de qualidade de imagem a partir de médias locais,
variâncias e covariância. O OSATURA usa uma versão por blocos. Veja `percep`.

**Palco (stage).** O espaço de coordenadas da imagem de saída: pixels, +x para a direita, +y para baixo.

**Máquina de estados.** Um grafo de estados de animação com transições conduzidas por entradas. Existe uma versão com bake,
baseada em log de eventos, e uma versão de runtime. Veja `statemachine`, `machine`.

**Easing step.** Um easing que mantém o valor da chave até a próxima chave. Usado para camadas de troca. Veja
`anim`.

**IK com alongamento (stretch IK).** Uma opção de IK que escala a cadeia até um limite (1..4) para alcançar um alvo distante. Veja
`rig`.

**Traço (stroke).** O contorno de um caminho: tinta, largura, terminação, junção, tracejado e um perfil de largura opcional. Veja
PROJECT_FORMAT seção 6.3.

**Camada de troca (switch).** Uma camada que mostra exatamente um de seus filhos, escolhido por um canal de índice. Usada
para animação quadro a quadro e formas de boca. Veja PROJECT_FORMAT seção 5.4.

**Take.** Um conjunto nomeado de trilhas que forma o conteúdo de um clipe. Veja `clipstack`.

**Tombstone.** Um marcador deixado em um array de CRDT no lugar de um elemento excluído, para que operações concorrentes
ainda possam ser posicionadas. A compactação remove os tombstones que todas as réplicas já viram. Veja `collab`.

**Trilha (track).** A lista de chaves de um canal, com um modo de extrapolação. Veja `anim`.

**Transformação.** Posição, rotação, escala, skew e âncora de uma camada; a matriz local é
`T(position) R Shear S T(-anchor)`. Veja PROJECT_FORMAT seção 5.1.

**Relógio vetorial.** Um vetor de contadores por réplica que diz se um evento aconteceu antes de outro ou
se são concorrentes (Fidge, Mattern). Veja `collab`.

**Pesos (osso).** A influência de cada osso sobre uma âncora ou vértice. Podem ser pintados ou calculados
por difusão. Veja `weights`.

**Wobble.** Um deslocamento de ruído determinístico somado à geometria de uma camada, com amplitude, escala, taxa
e seed. Veja PROJECT_FORMAT seção 5.1.

**Afinamento de Zhang-Suen.** Um algoritmo paralelo que reduz uma forma binária a um esqueleto de um pixel. Usado
pelo autotrace para a vetorização da linha central. Veja `autotrace`.
