# Estúdio do OSATURA

O estúdio é um editor no navegador servido pelo próprio binário `osatura`: `osatura studio project.json`.
Ele tem um canvas com gizmos de seleção e de transformação, uma lista de camadas, uma lista de ossos, um dope sheet e um gráfico de curvas,
onion skin e trilhas de movimento, uma paleta de comandos com atalhos reconfiguráveis, sugestões de pose, painéis de entrada ao vivo e
de rosto, botões de exportação para GIF/HTML/Lottie e uma sessão de edição compartilhada multiusuário. Ele não tem contas, nem TLS, nem
banco de dados: é um pequeno servidor HTTP de biblioteca padrão na frente do mesmo editor que a linha de comando usa.

Tudo aqui foi conferido com o binário real (`osatura 0.1.0`, build de depuração, Linux) e, quanto à interface do
usuário, no Chromium headless controlado pelo Playwright. As capturas de tela foram feitas dessa forma (1400x900, o projeto de demonstração
integrado). Apenas o **Chromium** foi testado; veja a seção 11. No Windows e no macOS, os arquivos de release também contêm `osatura-studio`, o mesmo estúdio em uma janela nativa (compilado e autotestado no CI, ainda não usado por uma pessoa; veja [GETTING_STARTED.md](GETTING_STARTED.md) e [LIMITATIONS.md](LIMITATIONS.md) L-75), e no Windows um duplo clique em `osatura.exe` inicia o estúdio e abre o navegador (L-76).

Licença deste documento: CC-BY-4.0. Para a linha de comando, veja [GETTING_STARTED.md](GETTING_STARTED.md) e
[CLI_REFERENCE.md](CLI_REFERENCE.md); para a segurança do servidor, veja também [SECURITY.md](SECURITY.md).

![O estúdio com o projeto de demonstração: barra de ferramentas no alto, o canvas com uma camada de braço selecionada e seus ossos desenhados, o painel Layers à direita e a timeline abaixo com losangos de chaves.](../../img/studio-overview.png)

## 1. Iniciando o estúdio

```
osatura studio <project.json> [--port N] [--ui-config F] [--bind ADDR] [--allow-host NAMES]
                [--public-origin URL] [--source-url URL] [--collab-token T] [--collab-log LOG [--collab-nosync]]
```

```
$ osatura demo demo.json
$ osatura studio demo.json --port 8080
AGPL-3.0 source offer: http://127.0.0.1:8080/source
OSATURA studio on http://127.0.0.1:8080  (Ctrl-C to stop; saves go to demo.json)
UI settings (layout, shortcuts) in /home/you/.config/osatura/ui.json      (your own home directory)
```

Abra o endereço em um navegador. O projeto é validado na inicialização; um arquivo que não passa em `check` é recusado com a
mesma mensagem. Campos desconhecidos geram um aviso (`warning: unknown field bogus (ignored)`).

| Flag | Padrão | Significado |
|---|---|---|
| `--port N` | 8080 | Porta TCP. Uma porta ocupada dá `cannot listen on 127.0.0.1:8861: Address already in use (os error 98)`; uma porta inválida dá `cannot listen on 127.0.0.1:99999: invalid port value`. |
| `--bind ADDR` | `127.0.0.1` | Interface em que escutar. Qualquer endereço que não seja de loopback **exige `--collab-token`**. |
| `--ui-config F` | `$OSATURA_UI_CONFIG`, senão `$XDG_CONFIG_HOME/osatura/ui.json`, senão no Windows `%APPDATA%\OSATURA\ui.json` (antes do alpha.3 não havia nenhum: as configurações eram mantidas só durante a sessão), senão `~/.config/osatura/ui.json` | Onde ficam guardados o layout, os atalhos, os comandos recentes e as configurações de overlays. É um arquivo por usuário, nunca parte do projeto. |
| `--allow-host NAMES` | nenhum | Nomes de `Host` adicionais a aceitar (separados por vírgula), para acesso sob um nome DNS. |
| `--public-origin URL` | nenhum | A origem simples (`https://studio.example.org`, sem caminho) que um proxy reverso TLS apresenta; seu `Host` e seu `Origin` passam então a ser aceitos. |
| `--source-url URL` | nenhum; nos binários de release oficiais, o commit a partir do qual foram compilados (`https://github.com/chiuta/osatura/tree/<commit>`) | Onde o código-fonte correspondente ao *seu* build pode ser baixado (AGPL seção 13); exibido em `/source` e por `--version`. Um valor explícito sempre prevalece. |
| `--collab-token T` | nenhum | Segredo compartilhado, **de pelo menos 16 caracteres** (`openssl rand -hex 16`). Obrigatório para associações que não sejam de loopback. |
| `--collab-log LOG` | nenhum | Persiste a sessão de edição compartilhada em um arquivo e a retoma após uma reinicialização. |
| `--collab-nosync` | desativado | Com `--collab-log`: não executa `fsync` após cada edição (mais rápido; uma queda de energia pode perder as últimas edições). Sem `--collab-log`, é recusado na inicialização (`--collab-nosync needs --collab-log`). |

O aplicativo de janela e o início por duplo clique criam o primeiro projeto em `OSATURA_DOCUMENTS` quando essa variável está definida; caso contrário, em `Documents\OSATURA` (usado pelo pacote portátil).

**Salvando.** As edições ficam na memória. O arquivo de projeto em disco só muda quando você pressiona **Save** (`Ctrl+S`), que grava de forma atômica
(arquivo temporário e depois renomeação, nunca através de um link simbólico). Isso vale também durante uma sessão compartilhada (seção 8).

## 2. Modelo de segurança (leia antes de associar a uma rede)

O estúdio foi feito para uma pessoa em sua própria máquina. O que o protege:

* **Lista de hosts permitidos.** Somente as requisições cujo cabeçalho `Host` seja um dos nomes do próprio servidor são respondidas; qualquer outra recebe **421**.
  Com `--allow-host` e `--public-origin`, mais nomes são aceitos.
* **Proteção entre sites.** Uma requisição com um `Origin` externo, ou `Sec-Fetch-Site: cross-site`, recebe **403**. Um POST cujo corpo não seja
  `application/json` recebe **415** (POSTs sem corpo, como `/api/undo`, não precisam de tipo).
* **Cabeçalhos de segurança** nas respostas: uma `Content-Security-Policy` restritiva (`default-src 'self'` e sem framing), `X-Frame-Options: DENY`,
  `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `Cache-Control: no-store`.
* **Token para qualquer acesso fora da máquina.** Com `--collab-token`, toda requisição precisa do token no cabeçalho `X-Collab-Token` ou no
  cookie `osatura_token` definido por `POST /api/login` (HttpOnly, SameSite=Strict). O token **nunca é aceito na URL**. Tokens errados são retardados
  por endereço de cliente (a partir da quinta falha seguida, esse endereço recebe **429** por 2, 4, ... até 30 segundos), mas o recuo não consegue deter um
  atacante que continue tentando: use um token aleatório longo.
* **Segredos de participante.** Entrar em uma sessão compartilhada devolve um segredo aleatório de 128 bits; toda chamada feita como esse participante deve enviá-lo como
  `X-Collab-Secret`. Saber o número de um participante não basta.

Verifiquei cada um desses pontos com `curl` contra um servidor em execução:

```
bad Host (evil.example):               421
allowed host (--allow-host):           200
cross-origin POST (Origin: evil.example): 403
POST from the --public-origin:         200
Sec-Fetch-Site: cross-site:            403
POST with Content-Type: text/plain:    415
no token, GET / or /api/state:         401
wrong X-Collab-Token:                  401
right X-Collab-Token:                  200
token in the URL (?token=...):         401
/source without a token:               200
unknown route:                         404   (GET /favicon.ico answers with a small SVG icon, without a token)
```

Recusas na inicialização (código de saída 1, nada é iniciado):

```
refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (and still sends everything in clear text)
--collab-token must be at least 16 characters (e.g. `openssl rand -hex 16`)
--source-url must be an http:// or https:// URL without spaces or quotes
--public-origin must be a bare origin (scheme, host and optional port, no path), e.g. https://studio.example.org
```

**Limites honestos.** O tráfego é HTTP simples: o token e todo o resto trafegam em texto claro, portanto coloque o estúdio atrás de um proxy reverso TLS
(`--public-origin`) se ele for acessível por qualquer outra pessoa. O `Set-Cookie` do token recebe a **flag `Secure`** quando a requisição chega sob um nome `--public-origin` `https://` (ou com um `Origin` https / `X-Forwarded-Proto: https`); o HTTP simples no loopback nunca a recebe,
e atrás de um proxy todo cliente aparece com o endereço do proxy, de modo que o recuo limita todos ao mesmo tempo. `/source` é aberto sem token
(a AGPL pede isso). Não há contas de usuário: quem tem o token pode alterar o projeto e gravar o caminho de salvamento.

### A oferta de código-fonte da AGPL

O estúdio é um software AGPL-3.0-or-later: se você o modificar e deixar outras pessoas usá-lo por uma rede, você deve oferecer a elas o código-fonte
da sua versão. `/source` é essa oferta; defina `--source-url` com o local onde o código-fonte do seu build está. Sem isso (um build feito por você mesmo com um `cargo build` simples não tem nenhum), a página informa esse fato e cita o mantenedor. Os binários de release compilados por `.github/workflows/release.yml` trazem o endereço do próprio commit (`OSATURA_SOURCE_URL` em tempo de compilação; um fork que compila com o próprio workflow aponta para si mesmo).

## 3. Um tour pela interface

### Barra de ferramentas

Undo, Redo, **Save**; caixas de seleção `bones` (desenhar o esqueleto), `auto-key`, `grid` (espaçamento em unidades do canvas, 0 = desligado), `snap` (às outras camadas e ao canvas),
`whole groups` (um clique em parte de um grupo seleciona o grupo de nível superior), `points` (mostra âncoras, alças, pontos de cage/lattice/pin e vértices de malha das
camadas selecionadas); um seletor `box` para o que uma caixa arrastada seleciona (`touches`, `contains`, `bounds touch`; começa em `touches`); **Lock**; e os botões **Overlays**, **Suggest**, **Layout**,
**Commands** (paleta) e **Keys** (editor de atalhos). Uma segunda linha tem o seu nome e **Collab**, e os botões de exportação **GIF**, **HTML player** e **Lottie**,
que baixam o que `osatura export` gravaria (o player HTML inclui o som mixado quando há som e o `ffmpeg` consegue gerar o mp3). O botão **Simple view** oculta as caixas de seleção de edição, Overlays, Suggest e o botão da paleta de comandos, de modo que restam apenas desfazer, salvar, painéis e exportações; a escolha fica lembrada neste navegador (ou janela) e a barra completa volta com **Full view**.

### Painéis e layouts

A janela é um dock de painéis, cada um com um id estável: `stage` (o canvas), `layers`, `selection`, `bones`, `live` (Live input), `face` (Webcam face), `json`
(o projeto inteiro como JSON editável, com um botão *Apply JSON*) e `timeline`. Existem quatro presets de layout:

```
animate  stage over timeline on the left, all panels as tabs on the right
rig      stage with a folded timeline; layers+bones and selection+json on the right (live and face hidden)
compact  stage over a tabbed timeline and panels
live     stage and timeline on the left; live, face, bones and selection on the right (layers and json hidden)
```

Troque de preset pelo menu **Layout** ou pela paleta; arraste abas para reacoplar ou destacar um painel; "reset layout" volta ao padrão.
O layout é salvo na configuração da interface, por usuário. `osatura ui layout presets|show NAME|validate FILE.json` imprime e valida os mesmos dados
(um layout deve manter todos os painéis acessíveis; um incompleto é rejeitado pelo servidor com 400).

### Painel Layers

![O painel Layers com uma camada bloqueada (sun), uma camada oculta (hills) e braço e perna selecionados juntos.](../../img/studio-layers.png)

Cada linha tem duas caixas de seleção: **visible** e **lock** (salvo *com o projeto*; as camadas bloqueadas são ignoradas por cliques e caixas, e recusam edições).
A captura de tela mostra `sun` bloqueada, `hills` oculta e duas camadas selecionadas com Shift-clique.

### Painel Selection

Enuncia as regras na própria página: *O clique seleciona o item mais acima (os bloqueados e ocultos são ignorados); Shift-clique alterna; arrastar em espaço vazio
desenha uma caixa (mantenha Alt para desenhar um laço à mão livre, Shift acrescenta). Arraste a caixa para mover, um quadrado para escalar (Shift mantém as proporções, Alt escala a partir do
centro), o botão redondo para girar (Shift prende em passos de 15 graus). Mantenha Ctrl ao arrastar para desligar o snapping. As teclas de seta deslocam 1 unidade (Shift: 10),
Delete remove camadas, Esc cancela um arrasto ou limpa a seleção.*

### Timeline: dope sheet e gráfico

![O dope sheet com uma seleção em caixa de três chaves no quadro 12 (as chaves são desenhadas sólidas, as outras vazadas).](../../img/studio-dopesheet.png)

![A visão do gráfico de curvas: três curvas bone_angle (arm_up, arm_low, hand) em cores diferentes, com três chaves selecionadas no quadro 12 mostrando suas alças de tangente; a linha de status diz "3 keys selected | frames 12".](../../img/studio-graph.png)

`Graph` (ou `Alt+V`) alterna entre o dope sheet e o gráfico de curvas. Os controles da timeline: `Play`, `|<` (primeiro quadro do intervalo), `loop`, `range` com
`Set in` / `Set out` (reproduz apenas entre a entrada e a saída), `snap` (arredonda as chaves movidas para quadros inteiros), `on collision` (`replace`, `merge`, `refuse` quando chaves movidas caem sobre
chaves existentes), `Copy` / `Paste` / `Duplicate` / `Delete`, `interp` + `Apply` (define o ease das chaves selecionadas), `Auto-smooth` (faz o bake de tangentes sem overshoot nas
chaves selecionadas), `ripple` N com `Insert` / `Remove` (e `grow project` para alterar a duração do projeto junto com eles), zoom `-` `+` `Fit` e `shy` (mostra canais marcados como shy).

Mouse: clique em uma chave para selecionar, Shift-clique para adicionar, arraste uma caixa para selecionar, arraste chaves para movê-las; **Alt-arrastar ou o botão do meio faz panorâmica**, **Ctrl+roda dá zoom**,
**Shift+roda rola** na vertical; no gráfico, **Shift restringe o arrasto da chave a um eixo**. A tolerância de snap para o snapping do canvas é de 6 unidades do canvas por padrão.
Toda edição de chave passa pelas mesmas operações validadas que `osatura keyedit` (veja [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)), portanto é um único passo de desfazer, e canais bloqueados a recusam.

### Overlays: onion skin, trilhas, proxies

![Onion skin e trilhas de movimento ativados para o braço no quadro 14, com o painel Overlays aberto.](../../img/studio-overlays.png)

`Overlays` (`Alt+7`) reúne o onion skin (`Alt+N`: quadros antes/depois em tons de vermelho/verde, passo fixo ou quadros-chave, modo contorno, atenuação, somente camadas selecionadas),
as trilhas de movimento (`Alt+M`: caminhos de osso, de pivô de camada ou de ponto de controle, com marcas de tique por quadro) e os **proxies de scrub** (`Alt+Q`: imagens de qualidade reduzida enquanto você arrasta o
cabeçote de reprodução, com um cache de imagens e uma cópia opcional de nível de detalhe). O painel informa, por exemplo, `picture cache: 2 frames, 17 KiB, 0 hits / 7 misses, emptied 4x by edits`.
As configurações são guardadas na configuração da interface e sobrevivem a uma reinicialização. Os overlays nunca fazem parte das exportações.

### Paleta de comandos e atalhos

![A paleta de comandos aberta com Ctrl+K e filtrada pela palavra "toggle", listando os comandos correspondentes com as dicas de atalho.](../../img/studio-palette.png)

`Ctrl+K` (ou `Ctrl+Shift+P`) abre a paleta: digite para filtrar (aproximado, com as letras correspondentes realçadas), as setas movem, Enter executa, Esc fecha. Sem nada digitado, ela
lista primeiro os comandos recentes. Há **84 comandos**; `osatura ui commands` os lista e `osatura ui search onion` os classifica exatamente como a paleta faz.

![O editor de atalhos de teclado aberto com Ctrl+vírgula, listando os comandos por categoria com seus acordes de teclas.](../../img/studio-shortcuts.png)

`Ctrl+,` (ou **Keys**) abre o editor de atalhos: clique em um acorde, pressione a nova combinação de teclas. Um conflito cita o outro comando e nada muda até que você
confirme "Take it over". Os acordes reservados do navegador são recusados: `Ctrl+W`, `Ctrl+T`, `Ctrl+N`, `Ctrl+Q`, `Ctrl+R`, `Ctrl+Tab`, `Ctrl+Shift+Tab`, `Ctrl+Shift+W`, `Ctrl+Shift+N`, `F5`, `F11`, `F12`,
`Alt+F4`. A reconfiguração é guardada na configuração da interface e sobrevive a um recarregamento. O servidor rejeita com 400 um mapa com conflitos, um comando desconhecido, um layout incompleto ou um corpo que não seja um objeto.

### Suggest

![O painel Suggest mostrando três candidatas de pose intermediária classificadas (linear, pca, manifold) entre os quadros 12 e 24, cada uma com uma pontuação e um botão Apply.](../../img/studio-suggest.png)

`Suggest` (`Alt+8`) propõe poses aprendidas a partir das próprias poses-chave do projeto: uma intermediária entre dois quadros, a conclusão de uma pose parcialmente fixada ou "o que costuma
vir a seguir". No projeto de demonstração, entre os quadros 12 e 24, ele classificou `linear` 1,00, `pca` 1,00 e `manifold` 0,99, construídas a partir de 5 poses em 2 eixos (99 % da variância).
`Apply` grava a pose como chaves, em um único passo de desfazer. O mesmo motor é `osatura pose-suggest` e `/api/suggest`. São métodos clássicos (mistura linear, PCA, uma variedade
de kernel, uma pequena rede opcional que você mesmo treina); com um punhado de poses, os três concordam.

### Painéis Live input e Webcam face

* **Live input** (`Alt+4`): o mouse sobre a imagem, teclas, gamepad e Web MIDI são mapeados para propriedades por uma tabela; `Record` faz o bake de uma performance em chaves, em um único passo de
  desfazer. O Web MIDI precisa de um navegador que o suporte (o Chromium suporta).
* **Webcam face** (`Alt+5`): pontos de referência alimentam a mesma tabela de mapeamento (fonte `face`, ids como `mouth_open`, `eye_r_open`, `head_yaw`). Os botões são
  *Start camera*, *Synthetic face*, *Synthetic face (detector)*, *Calibrate (hold neutral)* e *Face example mapping*. O caminho da câmera precisa de uma função provedora de pontos de referência
  que você fornece (`window.osaturaLandmarkProvider`, veja o comentário no código-fonte da página); o vídeo nunca sai da página e apenas os pontos de referência vão para o servidor local.
  Os dois modos sintéticos funcionam de imediato. *Synthetic face (detector)* renderiza a cabeça sintética no servidor e a encontra de novo com o detector integrado,
  que foi **treinado apenas com rostos sintéticos** (cabeças estilizadas, não fotografias); veja a seção de rosto em [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).

### Demonstração guiada

O botão **Demo** (comando `demo.panel`) abre uma demonstração guiada de 13 passos que constrói a pessoa caminhando de `osatura demo-person`: cenário, corpo, cabeça e rosto, braços com cinco dedos por mão, o esqueleto, as pernas caminhando, os braços balançando, tronco e cabeça, uma piscada e um sorriso, o chão em movimento, uma verificação e a exportação. Cada passo carrega essa etapa do projeto no editor com uma única operação de substituição total, de modo que você pode reproduzi-la, percorrê-la, selecionar ossos e camadas e ler a timeline. As etapas são recortes do projeto concluído; os comandos e a saída mostrados no painel foram executados com o binário real.

* **Nada é gravado em disco.** Salvar (e Ctrl+S) fica desativado enquanto a demonstração está carregada. O único caminho de escrita do studio, `POST /api/save`, nunca é chamado pela demonstração.
* **Seu projeto volta.** *Restore my project* recoloca o projeto que estava aberto antes (uma única operação de substituição total, um passo de Desfazer). Se você fechar a aba ou recarregar enquanto a demonstração está carregada, o próximo carregamento da página percebe que o servidor ainda guarda a demonstração, mantém Salvar desligado e oferece *Restore my project* de novo; o original fica guardado no armazenamento local do navegador para isso.
* A demonstração se recusa a iniciar dentro de uma sessão de colaboração, porque substitui o projeto inteiro.
* Regenere as etapas com `node scripts/build-studio-tour.mjs --bin target/release/osatura` e depois recompile a CLI (a página é incorporada no binário).

## 4. Atalhos de teclado (padrões)

Esta é a saída de `osatura ui shortcuts` para uma configuração nova: **72 associações, 84 comandos, 0 conflitos**. Os atalhos pertencem a um **contexto**:
`global` (em todo lugar), `canvas` (foco na imagem) e `timeline` (foco na timeline). Os campos de texto mantêm suas próprias teclas.

### Global

| Atalho | Comando | Atalho | Comando |
|---|---|---|---|
| `Space` | reproduzir / pausar (`playback.toggle`) | `Alt+B` | alternar ossos |
| `Home` / `End` | primeiro / último quadro | `Alt+G` | alternar grupos inteiros |
| `,` / `.` | quadro anterior / seguinte | `Alt+K` | alternar auto-key |
| `Shift+,` / `Shift+.` | voltar / avançar 10 quadros | `Alt+M` | alternar trilhas de movimento |
| `[` / `]` | definir entrada / saída do intervalo | `Alt+N` | alternar onion skin |
| `Alt+R` | alternar intervalo de reprodução | `Alt+O` | alternar reprodução em loop |
| `Alt+1` | alternar painel Layers | `Alt+P` | alternar pontos de controle |
| `Alt+2` | alternar painel Selection | `Alt+Q` | alternar proxies de scrub |
| `Alt+3` | alternar painel Bones | `Alt+S` | alternar snap do canvas |
| `Alt+4` | alternar painel Live input | `Alt+T` | alternar painel Timeline |
| `Alt+5` | alternar painel Webcam face | `Alt+V` | alternar dope sheet / gráfico |
| `Alt+6` | alternar painel JSON | `Alt+Shift+S` | alternar snap da timeline |
| `Alt+7` | painel Overlays | `Alt+=` / `Alt+-` | ampliar / reduzir a timeline |
| `Alt+8` | painel Suggest | `Ctrl+S` | salvar |
| `Ctrl+K` / `Ctrl+Shift+P` | paleta de comandos | `Ctrl+,` | atalhos de teclado |
| `Ctrl+Z` | desfazer | `Ctrl+Y` / `Ctrl+Shift+Z` | refazer |

### Canvas

| Atalho | Comando |
|---|---|
| `Arrow keys` | desloca a seleção em 1 unidade (`Shift`: 10 unidades) |
| `Delete` / `Backspace` | exclui a seleção |
| `Escape` | limpa a seleção |
| `Ctrl+A` | seleciona tudo |
| `Alt+L` | bloqueia / desbloqueia as camadas selecionadas |
| `I` | adiciona uma chave no cabeçote de reprodução (`anim.add_key`) |

### Timeline

| Atalho | Comando |
|---|---|
| `ArrowLeft` / `ArrowRight` | quadro anterior / seguinte (`Shift`: 10) |
| `-` / `=` | reduzir / ampliar |
| `F` | ajustar o zoom para caber |
| `Delete` / `Backspace` | exclui as chaves selecionadas |
| `Escape` | desmarca as chaves |
| `Ctrl+A` | seleciona todas as chaves |
| `Ctrl+C` / `Ctrl+V` / `Ctrl+D` | copiar / colar / duplicar chaves |
| `Alt+ArrowLeft` / `Alt+ArrowRight` | move as chaves selecionadas um quadro antes / depois |
| `Alt+ArrowUp` / `Alt+ArrowDown` | eleva / abaixa as chaves selecionadas (seu valor) |
| `I` | adiciona uma chave no cabeçote de reprodução |

Modificadores no canvas: **Shift** restringe (trava de eixo ao mover, passos de 15 graus ao girar, proporcional ao escalar, a ponta de um osso então apenas gira,
e Shift-clique alterna a seleção enquanto uma caixa acrescenta a ela); **Alt** escala a partir do centro (e Alt-arrastar em espaço vazio desenha um laço); **Ctrl** (Cmd no macOS)
desliga o snapping.

## 5. Seleção e gizmos

Um clique seleciona o item mais acima sob o ponteiro: uma camada, um osso (sua cápsula, um pouco mais larga que a linha desenhada) ou, com `points` ativado, um ponto de controle. As camadas bloqueadas e ocultas
são ignoradas, de modo que um clique atravessa para o que está abaixo. Uma caixa seleciona conforme o modo do seletor `box`: `touches` (qualquer coisa que a caixa sobreponha), `contains` (apenas o que está completamente dentro) ou `bounds touch` (as caixas delimitadoras se sobrepõem); Alt-arrastar desenha um laço no lugar.

Para uma camada, a caixa de transformação segue os eixos da própria camada: 8 alças escalam (as bordas, um eixo; os cantos, ambos; arrastar além da borda oposta inverte com uma escala negativa),
o botão redondo gira em torno do centro, e arrastar por dentro move. Para um osso, o círculo da cabeça move o osso e a ponta o gira (e o estica, a menos que Shift seja mantido).
Para pontos de controle (âncoras, alças de Bézier, pontos de deformador, vértices de malha), o mesmo gizmo move os pontos selecionados. O snapping cobre uma grade (desligada por padrão), as bordas e os centros
de outras camadas e as bordas do canvas, dentro de 6 unidades do canvas. **Um arrasto é um passo de desfazer** (uma operação em lote).

## 6. Auto-key: exatamente o que recebe chaves

Se uma edição no canvas altera um valor *estático* ou cria *chaves* é decidido assim. Conferi isso com a API HTTP no projeto de demonstração (o `mode` de cada edição é informado):

| Situação | Resultado |
|---|---|
| Edição de camada, `auto-key` desligado, a propriedade não tem canal com chaves | alteração **estática** da transformação da camada (nenhum canal é criado) |
| Edição de camada, `auto-key` ligado | as propriedades editadas recebem **chaves** no quadro atual (novos canais `layer_x` e `layer_y` foram criados: de 14 para 16 canais) |
| Propriedade de camada ou de ponto que **já tem um canal com chaves**, `auto-key` desligado | ainda recebe **chaves** no quadro atual (uma edição no quadro 9 acrescentou uma segunda chave aos canais do quadro 5) |
| Edição de **osso** | sempre recebe **chaves**, qualquer que seja a configuração de `auto-key` (canais de deslocamento de osso foram criados: de 16 para 18) |

Assim, uma propriedade animada nunca pode ser sobrescrita silenciosamente por um valor estático, e uma pose de osso nunca é "perdida" por esquecer o interruptor. Ligue `auto-key` (`Alt+K`)
quando quiser animar uma propriedade que ainda é estática. A tecla `I` (`anim.add_key`, "Add key at playhead") adiciona uma chave no quadro atual.

## 7. Desfazer e refazer

Desfazer e refazer (`Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`) atuam sobre operações inteiras: um arrasto de gizmo, um deslocamento, uma edição de chave e um Apply são, cada um, um passo. Sem uma sessão compartilhada, eles
são lineares. **Em uma sessão compartilhada, o desfazer é por usuário**: ele reverte apenas a sua última edição e deixa no lugar as edições das outras pessoas (seção 8). O histórico de desfazer não é salvo com o
projeto e se perde quando o servidor para.

## 8. Edição compartilhada (colaboração)

### Passo a passo: duas pessoas, um projeto

![Dois usuários de navegador em uma sessão: a barra de ferramentas mostra dois chips de participantes, Ana (você) no quadro 14 e Bo no quadro 0, e a caixa de seleção e o cursor de Bo são desenhados no canvas em torno da camada da perna.](../../img/studio-collab.png)

1. Inicie o estúdio. Para pessoas em outras máquinas, você precisa adicionar um token e associar a um endereço de rede:
   `osatura studio demo.json --bind 0.0.0.0 --port 8080 --collab-token $(openssl rand -hex 16) --collab-log session.log`,
   de preferência atrás de um proxy TLS (seção 2). Em uma única máquina, nenhum token é necessário.
2. No primeiro navegador, digite seu nome ao lado de **Collab** ("Ana") e pressione **Collab**. O botão vira **Leave** e um chip aparece para você. No segundo navegador,
   digite "Bo" e entre. Cada chip mostra o quadro atual do participante (`Ana (you) @14`, `Bo @0`).
3. Selecione coisas. A outra pessoa vê a sua caixa de seleção e o seu cursor na sua cor, com o seu nome (Bo selecionou `leg` e moveu o ponteiro; Ana vê uma caixa roxa
   com o nome "Bo" e um ponto ao lado de `leg` na lista de camadas).
4. Edite ao mesmo tempo. No teste: Ana deslocou a camada do braço em 10 unidades (`Shift+ArrowRight`); Bo viu `[10,0]` aparecer em `/api/project`. Bo então ocultou a camada `sun`, o que Ana viu
   imediatamente.
5. Ana pressionou `Ctrl+Z`. **Apenas o deslocamento de Ana foi desfeito** (o braço voltou a `[0,0]`); o sol oculto por Bo permaneceu oculto.
6. As edições feitas enquanto alguém trabalhava a partir de uma visão mais antiga são mescladas, não rejeitadas: executei a simulação do próprio projeto (`osatura collab-sim`, 6 réplicas, 600 edições
   concorrentes aleatórias, 12 outras ordens de entrega) e todas as réplicas terminaram idênticas e o projeto mesclado compilou.
7. Pressione **Save** (ou `Ctrl+S`) em qualquer um dos navegadores para gravar o estado compartilhado no arquivo de projeto; nada é gravado no arquivo antes disso.

Apenas o Chromium foi testado, com duas páginas em uma máquina. Não testei máquinas diferentes, conexões lentas nem mais de dois navegadores reais.

### Persistência: `--collab-log`

```
$ osatura studio demo.json --collab-log session.log
...
$ # (an edit is made in a session, the server is stopped and started again)
collaboration log session.log: resumed at index 1 (1 envelopes after the checkpoint)
```

O log é um arquivo de linhas (modo `0600`, somente o dono) que guarda um checkpoint encadeado por hash do **projeto inteiro** (cerca de 178 KB para a demonstração) mais uma linha por edição aceita. As anexações são
`fsync`adas por requisição, a menos que `--collab-nosync`. Após uma reinicialização, o estado da sessão é retomado (mudei o nome do projeto em uma sessão; após a reinicialização, `/api/project` ainda tinha o
novo nome) enquanto o arquivo em disco permaneceu inalterado (`OSATURA demo`) até o Save. **Os participantes e o histórico de desfazer não são persistidos**: as páginas entram novamente com novos ids. Um log danificado ou um log
de outro projeto interrompe o servidor com um erro em vez de ser sobrescrito. Uma falha durante uma anexação perde no máximo uma última linha cortada.

**O log contém o seu projeto inteiro em texto claro; proteja-o como o próprio projeto.**

## 9. Referência da API HTTP

A página do estúdio usa estas rotas e você também pode usá-las (scripts, testes). Todos os corpos e respostas são JSON, salvo indicação em contrário; os erros têm a forma `{"ok": false, "errors": ["..."]}` com status 400 (ou 401, 403,
404, 415, 421, 429, 503, conforme descrito na seção 2). As respostas de sucesso trazem `"ok": true` onde indicado. `state`, abaixo, significa o documento devolvido por `GET /api/state`:
`{bones:[{name,rest_angle}], channels:[...], fps, frames, width, height, can_undo, can_redo, rev, ok}`; `rev` aumenta a cada alteração. Todas as rotas estão sujeitas às verificações da seção 2.

### Leitura

| Rota | Parâmetros | Resposta |
|---|---|---|
| `GET /` | | a página do estúdio (HTML) |
| `GET /source` | | a página da oferta de código-fonte AGPL; aberta, sem token |
| `GET /api/state` | | `state` |
| `GET /api/project` | | o JSON completo do projeto |
| `GET /api/layers` | | `{ok, layers:[{path, depth, kind, visible, locked, locked_eff, reference, ptr}]}` (`locked_eff` conta também os grupos bloqueados acima) |
| `GET /frame/N.svg` | `bones=1` desenha o esqueleto | um quadro como SVG (N pode ser fracionário); cabeçalhos `X-Cache`, `X-Render-Ms` |
| `GET /api/frame` | `frame`, `proxy` 0-4 (0 = qualidade total, 1-4 proxies de scrub), `kind` = `vector` (padrão) \| `raster` \| `png`, `bones=1`, `lod=1&screen=WxH&lodpx=P` (desenha a partir de uma cópia de detalhe reduzido dentro de P pixels, 0-16) | a imagem; cabeçalhos `X-Cache` (hit/miss), `X-Render-Ms`, `X-Lod-Level`, `X-Lod-Bound-Px`, `X-Proxy-Level`. Um quadro fora do intervalo: `{"errors":["frame 999 out of range"]}`; um kind inválido: `bad kind 'x' (vector, raster, png)` |
| `GET /api/cache` | | `{ok, revision, entries, bytes, hits, misses, invalidations, lod:{builds, engines, invalidations, last_build_ms, levels, unavailable}}` |
| `GET /api/overlays` | `frame`, `sel` (nomes de camadas separados por vírgula) | `{ok, frames, settings, ...}` mais fantasmas e SVG do quadro quando um é indicado |
| `GET /api/sounds` | | `{ok, total, sounds:[{name, start, gain, duration, peaks, error?}]}` picos da forma de onda após corte, fades e ganho |
| `GET /api/vcs/log` | `limit` (máx. 1000), `rev` | `{ok, branch, commits:[{hash, author, time, message, parents}]}`; `{"ok":false,"errors":["no .osatura-vcs repository found ..."]}` sem repositório (veja `osatura vcs`) |
| `GET /api/vcs/diff` | `a` (padrão `HEAD`), `b` | `{ok, changes}` |
| `GET /api/timeline` | | `{ok, fps, frames, groups, rows, tracks}` o modelo da dope sheet |
| `GET /api/graph` | `ch` (índices de canais separados por vírgula), `f0`, `f1`, `n` (amostras, 2-2000, padrão 200) | `{ok, channels, f0, f1, range, skipped}` amostras de curvas e alças de tangente |
| `GET /api/live` | | `{ok, mapping:{params,triggers}, recorded, recording}` |
| `GET /api/ui` | | `{ok, layout, presets, preset_names, default_layout, shortcuts, commands, conflicts, reserved, overlays, recents, panels, persisted, path, warning, ...}` |
| `GET /api/ui/search` | `q`, `limit` (1-200, padrão 50) | `{hits:[{id, title, category, description, bindings, score, positions}]}` a classificação da paleta |
| `GET /export.gif`, `/export.html`, `/export.html.nosound`, `/export.lottie.json` | | o arquivo, como download (`animation.gif`, `player.html`, `animation.json`) |

### Edição

| Rota | Corpo | Resposta |
|---|---|---|
| `POST /api/op` | `{"op":"set","path":"/fps","value":30}` — `op` é `set`, `insert` (caminho de array, `.../-` acrescenta no fim), `remove` ou `batch` (`{"op":"batch","ops":[...]}`, um único passo de desfazer). Os caminhos são JSON Pointers no projeto. | `state`. Um resultado inválido é recusado com `{"errors":["invalid project: invalid value: integer `-3`, expected u32"]}` e nada muda |
| `POST /api/undo`, `POST /api/redo` | nenhum | `state` (por usuário em uma sessão compartilhada) |
| `POST /api/key` | `{"target","prop","frame","value","ease"?}` (`ease` como `{"type":"smooth"}`; padrão linear) | `state` |
| `POST /api/delete_key` | `{"target","prop","frame"}` | `state` |
| `POST /api/save` | nenhum | `{"ok":true,"saved":"<path>"}`; grava de forma atômica. Somente o host, quando o estúdio roda com `--collab-host` (caso contrário, **403**) |
| `POST /api/suggest` | `{"kind":"between"\|"complete"\|"next","a","b","t","frame","constraints","free","top","snap","prior","apply":RANK, "library"?, "model"?}` | `{ok, suggestions}`; com `apply`, também o `state` da edição. Uma requisição sem `kind` gera `bad request: missing field `kind`` |

### Seleção e gizmos (sem estado: a página guarda a seleção)

Um **alvo** (target) é `{"kind":"layer","path":"arm"}`, `{"kind":"bone","name":"hand"}` ou `{"kind":"point","layer":"...","point":...}`.

| Rota | Corpo | Resposta |
|---|---|---|
| `POST /api/pick` | `{"frame","x","y","options"?}` (`options`: `tolerance`, `bone_radius`, `point_radius`, `layers`, `bones`, `point_layers`, `locked`, `locked_bones`) | `{ok, hit, hits:[{distance, target}]}` o mais acima primeiro |
| `POST /api/marquee` | `{"frame","rect":[x0,y0,x1,y1]` ou `"lasso":[[x,y],...]` (3 a 5000 pontos)`,"mode":"bbox"\|"contain"\|"intersect","options"?}` | `{ok, targets}` |
| `POST /api/select` | `{"frame","selection":[...],"targets":[...],"mode":"replace"\|"add"\|"toggle"\|"subtract"}` | `{ok, selection, bounds, gizmo:{corners, handles}, points}` |
| `POST /api/gizmo` | `{"frame","selection","handle","start":[x,y],"delta":[dx,dy],"modifiers":{"constrain","center","no_snap"},"snap":{"grid","objects","canvas","tolerance"},"auto_key","locked","pivot","commit":bool}` — `handle` é `{"type":"move"}`, `rotate`, `{"type":"scale","at":"nw"}`, `bone_head` ou `bone_tip` | sem `commit`: `{ok, drag:{changed, edits, guides, matrix, summary, box_after, outline}}` (uma prévia); com `commit:true`: `state` mais `drag` e `selection`, um único passo de desfazer |
| `POST /api/nudge` | `{"frame","selection","delta":[dx,dy],"auto_key","locked"}` | `state` + `drag` (edições com `mode` igual a `static` ou `key`) + `selection` |
| `POST /api/delete` | `{"frame","selection"}` | `state` + `selection`; `{"errors":["nothing selected"]}` quando vazia |

### Timeline

| Rota | Corpo | Resposta |
|---|---|---|
| `POST /api/timeline/select` | `{"selection":[{ch,frame}...],"mode":"replace"\|"add"\|"toggle"\|"subtract","all":true` ou `"channel":N` ou `"group":N` ou `"box":{f0,f1,row0,row1}`, `"include_locked"}` | `{ok, selection:[{ch,frame}...]}` |
| `POST /api/timeline/op` | `{"op":{...},"preview":bool}` — `op` é um de `move_keys`, `scale_keys`, `copy`, `paste`, `duplicate`, `delete`, `set_ease`, `set_value`, `add_key`, `ripple`, `offset`, `extrapolate`, `make_loop`, `handle`, `smooth`, `flags` (um nome desconhecido lista estes) | com `preview:true`: `{ok, preview:true, plan}` sem alterar nada; caso contrário, `state` + `plan:{changed, notes, op, clip}` |

Por exemplo, `{"preview":true,"op":{"op":"delete","keys":[{"ch":0,"frame":12}]}}` devolve o plano (`changed: 1`) e deixa o projeto intacto.

### Overlays, live, rosto, UI

| Rota | Corpo | Resposta |
|---|---|---|
| `POST /api/overlays` | `{"settings":{"onion":{"enabled":true,"before":3}},"frame","selection":[names],"reset":bool}` | `{ok, frames, ghosts, settings, svg}`; as configurações são salvas na configuração da UI |
| `POST /api/trails` | `{"targets":[{"kind":"bone","name":"hand"}],"from","to"` ou `"frame","before","after"`, `"smooth","svg":bool,"hit":{x,y,radius}}` | `{ok, from, to, trails:[{curve,frames,keys,...}], svg?, hit?, path_hit?}` |
| `POST /api/live/mapping` | `{"mapping":{params,triggers},"library"?}` (mesmo formato de `live-map`) | status live |
| `POST /api/live/push` | `{"events":[{source,id,value,time}],"midi":[raw...]}` | status live com o efeito dos eventos |
| `POST /api/live/record` | `{"on":true}` inicia; `{"on":false,"tol":0.01}` para e **grava a performance no projeto** (bake) como um único passo de desfazer | status / `state` + resumo `live` |
| `POST /api/face/config` | `{"config"?,"calibration"?}` | `{ok}` |
| `POST /api/face/calibrate` | `{"on":true}` / `{"on":false}` | `{ok, frames, calibration}` |
| `POST /api/face/push` | `{"frames":[...landmarks...],"layout"?,"size"?}` (no máximo 10000 quadros por envio) | status live |
| `POST /api/face/detect` | `{"image":"<base64 PNG/JPEG or data: URI>","track":bool,"push":bool,"t"}` | `{ok, found, layout:"osatura23", size, points, box, note:"detector trained on synthetic faces only"}` |
| `POST /api/face/synth` | `{"t","size":32-512,"seed"}` | `{ok, image:"data:image/png;base64,...", layout, size, points}` um quadro da cabeça sintética com seus landmarks exatos |
| `POST /api/ui` | qualquer um de `{"layout":{...}\|null,"shortcuts":{...}\|null,"recents":[...],"overlays":{...},"reset":"layout"\|"shortcuts"\|"overlays"\|"all"}` | documento de `GET /api/ui`; campos desconhecidos (`unknown field 'bogus'`), conflitos e layouts incompletos são recusados e nada muda |
| `POST /api/login` | `{"token":"..."}` | `{ok:true}` e `Set-Cookie: osatura_token=...; Path=/; HttpOnly; SameSite=Strict`; token errado: 401 (429 a partir da 5ª falha); sem token configurado, `{"ok":true,"token_required":false}` |

### Colaboração

As chamadas feitas como participante enviam `X-Collab-Replica: <id>` e `X-Collab-Secret: <secret>` (ambos devolvidos por join). Um id de réplica com segredo incorreto ou ausente recebe **403**.

| Rota | Corpo / consulta | Resposta |
|---|---|---|
| `POST /api/collab/join` | `{"name":"Ana","mode":"thin"\|"replica"}` | `{ok, replica, secret, colour, name, index, mode, clock, users, presence, doc}` (`snapshot` no modo replica). A primeira entrada inicia a sessão a partir do documento do editor |
| `POST /api/collab/leave` | `{"replica"}` | `{ok, left}` |
| `GET /api/collab/state` | `replica`, `doc=1`, `snapshot=1` | `state` + `collab_active` (+ users, presence) |
| `GET /api/collab/log` | `since=N` | `{ok, index, entries:[{index, by, envelope}]}` |
| `POST /api/collab/ops` | `{"replica","ops":[...]` ou `"op":{...}`, `"base"?}` ou `{"replica","envelopes":[...]}` | `state` + `applied`, `index`. Participante desconhecido: `unknown participant N: join first`; lista vazia: `no ops`; edições inválidas do projeto são recusadas |
| `POST /api/collab/undo`, `.../redo` | `{"replica"}` | `state` + `collab_can_undo`, `collab_can_redo` (somente as suas próprias edições) |
| `POST /api/collab/presence` | `{"replica","n","frame","cursor":[x,y],"selection":[names up to 64]}` | `{ok}` |
| `POST /api/collab/compact` | `{}` | resumo da compactação (`ok`, `logged`). Somente o host, quando o estúdio roda com `--collab-host` (caso contrário, **403**) |
| `POST /api/collab/revoke` | `{"names":["ana","bob"]}` (1 a 64 nomes) | `{ok, revoked, revoked_total}`: os tokens desses nomes passam a ser recusados e os participantes que entraram com eles são removidos. Exige tokens por participante (`--collab-secret-file`) e um token de host (`--collab-host`), ou o `--collab-token` compartilhado; caso contrário, **403**. Um fluxo de eventos já aberto termina quando o cliente se desconecta |
| `GET /api/collab/events` | `since=N` ou `Last-Event-ID` | um fluxo de **server-sent events**: reprodução a partir de `since`, depois usuários, presença e todos os eventos seguintes; `resync` se o cliente estiver muito atrasado; no máximo 64 fluxos (`503 too many event streams`) |

Em uma sessão ativa, as rotas de edição comuns (`/api/op`, `/api/undo`, `/api/redo` e as próprias edições do estúdio por nudge, key ou gizmo) também são registradas para os outros participantes.

## 10. Solução de problemas

| Sintoma | Causa e correção |
|---|---|
| O navegador mostra 421 ou "misdirected request" | Você abriu o estúdio com um nome que ele não conhece (um nome DNS, um endereço da LAN). Adicione-o com `--allow-host name` ou abra `http://127.0.0.1:PORT/`. |
| 403 em todas as requisições atrás de um proxy | O proxy altera `Origin`/`Host`. Informe a origem pública: `--public-origin https://studio.example.org`. |
| 401 | `--collab-token` está definido: abra a página e informe o token uma vez, ou envie `X-Collab-Token`. Um token na URL não é aceito. |
| 429 | Cinco tokens errados seguidos vindos do seu endereço; aguarde até 30 s. Atrás de um proxy, todos os usuários compartilham um único endereço. |
| `refusing to listen on 0.0.0.0:...` | Bind fora do loopback sem token. Adicione `--collab-token` (16 ou mais caracteres). |
| `cannot listen on 127.0.0.1:8080: Address already in use` | Outro processo está usando a porta; escolha outra com `--port`. Um estúdio antigo de uma execução anterior é a causa habitual. |
| A página carrega, mas a imagem está em branco | Veja o terminal; um projeto que falha em `check` é recusado na inicialização. Recarregue depois de corrigir. |
| Uma mensagem do console do navegador `Failed to load resource ... 404` | Versões antigas não tinham `/favicon.ico`; agora ele é servido. Um 404 que persista vem de outra coisa. |
| O texto do painel Face | O painel agora informa que o detector treinado com dados sintéticos está incluído, que não foi validado com fotos nem webcams e que uma webcam real exige seu próprio provedor de landmarks. |
| O layout está quebrado ou faltam painéis | Menu `Layout`, redefinir layout, ou apague o `ui.json` mostrado na inicialização. Um arquivo de configuração que não pode ser lido por completo é copiado para `ui.json.bak` (depois `.bak.1`, ...) antes de usar os padrões, e `GET /api/ui` tem um campo `warning` sobre isso. |
| Minhas edições sumiram depois de reiniciar | Nada é gravado até **Salvar**. Com `--collab-log`, a *sessão* sobrevive a uma reinicialização, mas o arquivo em disco continua sendo gravado somente por Salvar. |
| `Ctrl+W`, `Ctrl+T` e semelhantes não podem ser atribuídos | Reservados pelos navegadores (lista na seção 3). |
| Ctrl+Z não desfaz a alteração de outra pessoa em uma sessão | É intencional: o desfazer é por usuário. |
| As exportações pedem `ffmpeg` | Somente para som em HTML (mp3) e para mp4/webm na linha de comando; GIF, Lottie e HTML sem som não precisam de nada. |

## 11. Acessibilidade e suporte a navegadores

**Navegadores.** A interface foi testada somente no **Chromium** (a build incluída no Playwright, executada em modo headless, no Linux). Firefox e Safari não foram experimentados e podem funcionar ou não.
O Web MIDI exige um navegador que o implemente (o Chromium implementa); a câmera exige `getUserMedia` e um contexto seguro (HTTPS ou `localhost`). A página usa server-sent events para a
sessão compartilhada. Eventos de ponteiro são usados para arrastar; telas sensíveis ao toque não foram testadas.

**Acessibilidade, com honestidade.** Não auditei o estúdio com um leitor de tela nem em uma sessão apenas com teclado. O que existe: um atributo `lang` e a meta tag viewport, caixas de seleção com rótulo, um
toast com `role="status"`, alguns atributos ARIA, **um conjunto completo de atalhos de teclado e uma paleta de comandos** que alcançam todos os comandos, atalhos reconfiguráveis, e a linha de comando `osatura`, que faz
tudo o que o editor faz sem um ponteiro. O que falta: o canvas é uma imagem e a maior parte da edição no canvas é feita com o ponteiro (existe deslocamento com as teclas de seta), não há tema claro e nenhum
tratamento de `prefers-reduced-motion` ou `prefers-color-scheme`, os contornos de foco são os padrões do navegador e não os verifiquei, e não medi o contraste de cores. Se você depende de tecnologia assistiva,
prefira a CLI e o painel JSON, e informe ao projeto o que não funciona.

## 12. Arestas conhecidas

* Corrigido desde a primeira versão: o texto do painel da webcam (Face), o 404 de `/favicon.ico`, `--collab-nosync` sem `--collab-log` e a ausência da flag de cookie `Secure` atrás de um `--public-origin` https.
* `osatura render p.json 99 o.png` em um projeto de 24 quadros agora também é um erro (adicione `--clamp` para renderizar o último quadro); o `/api/frame` do estúdio recusa um quadro fora do intervalo (`frame 999 out of range`) e agora aceita somente os quadros `0..frames-1`.
* As alterações de layout e de atalhos são armazenadas por usuário na configuração da UI, não no projeto, portanto não viajam com o arquivo.
