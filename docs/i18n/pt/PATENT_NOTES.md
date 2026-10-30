# Notas sobre patentes

**Isto não é aconselhamento jurídico nem um parecer de liberdade de operação (freedom-to-operate).** Registra o que NÃO foi verificado, para que ninguém confunda a ausência de uma triagem com uma liberação. As licenças do OSATURA (MPL-2.0 para as bibliotecas, AGPL-3.0-or-later para a CLI) contêm concessões de patente dos contribuidores para o código do próprio autor, mas nada dizem sobre patentes de terceiros. Nenhuma busca de patentes por um profissional foi realizada.

## Triagem ainda não realizada

| Área | Por que pode importar | Situação |
|---|---|---|
| Puppeteering guiado pelo rosto (`faceparam`, `facedet`, `liveinput`: pontos de referência ou proporções do rosto controlando parâmetros do rig) | A animação de personagens 2D guiada por performance é um campo ativo de pedidos comerciais. | Não triada. O código usa medidas publicadas em livros-texto (eye aspect ratio, filtro One-Euro, HOG + regressão logística); isso não exclui reivindicações sobre a combinação. |
| Máquinas de estados e data binding (`statemachine`, `runtime_machine`, `bind`) | Runtimes de animação interativa com máquinas de estados, blend trees, entradas e vinculação de view-model são uma área de recursos comerciais. | Não triada. Design próprio do modelo de dados, não uma liberação de patente. |
| FABRIK, coordenadas de valor médio, pesos biharmônicos limitados (`rig`, `cage`, `weights`) | Algoritmos publicados; alguns autores e empregadores registraram patentes em torno de técnicas de deformação e skinning. | Não triada. FABRIK (Aristidou e Lasenby, 2011) e MVC (Floater, 2003) são usados conforme os artigos; o módulo de pesos usa difusão harmônica simples, não o solucionador biharmônico limitado publicado. |
| Preenchimento de caminhos por stencil-then-cover (`osatura-gpu`) | A técnica tem origem em pesquisas de renderização de caminhos em GPU e em extensões de fabricantes. | Não triada. Usada aqui apenas por meio da API padrão do wgpu com um stencil buffer simples. |
| Codecs de vídeo e áudio via ffmpeg (`export` .mp4 / .webm, `soundtrack`) | H.264 e AAC são cobertos por pools de licenciamento (MPEG LA / Via LA e outros). VP9 e AV1 são divulgados como livres de royalties, mas foram anunciadas reivindicações de pools de patentes e de patentes individuais para ambos. | Não avaliada. Não trate VP9/AV1 como liberados. |

## ffmpeg
O ffmpeg é um **subprocesso externo** fornecido pelo usuário. O OSATURA não o vincula, não o distribui e não deve incluí-lo em nenhuma versão: incluí-lo tornaria o distribuidor responsável pela licença do próprio ffmpeg (LGPL/GPL, conforme a compilação) e pelas licenças dos codecs acima. Cada opção de codec é uma escolha do próprio usuário, em sua própria instalação.

## Antes do lançamento comercial
Peça a um advogado de patentes que revise as áreas acima, nas jurisdições em que o software será distribuído, e registre o resultado aqui e em CLEANROOM.md (regra 5).
