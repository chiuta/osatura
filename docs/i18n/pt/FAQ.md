<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Perguntas frequentes

*Respostas curtas, cada uma tirada de algo que foi executado ou lido na árvore de código-fonte e nos outros documentos. Documentação CC-BY-4.0. Quando uma resposta depende de uma limitação, ela diz isso e aponta para [LIMITATIONS.md](LIMITATIONS.md). As mensagens de erro da última seção foram capturadas de um build real; elas são citadas exatamente, para que você possa buscar nesta página o texto que está vendo.*

Conteúdo: [Instalação e configuração](#instalação-e-configuração) - [ffmpeg](#ffmpeg) - [Desempenho](#desempenho) - [Determinismo](#determinismo) - [Licenciamento e a sua saída](#licenciamento-e-a-sua-saída) - [Segurança dos arquivos e caminhos de recursos](#segurança-dos-arquivos-e-caminhos-de-recursos) - [Por que a saída deve ser diferente da entrada](#por-que-a-saída-deve-ser-diferente-da-entrada) - [Rigging e animação](#rigging-e-animação) - [Estúdio](#estúdio) - [Rosto, fala e captura de movimento](#rosto-fala-e-captura-de-movimento) - [Bugs, segurança e contribuição](#bugs-segurança-e-contribuição) - [Limitações](#limitações) - [Comparações com outras ferramentas](#comparações-com-outras-ferramentas) - [Tabela de solução de problemas](#tabela-de-solução-de-problemas)

---

## Instalação e configuração

**1. Como instalo o OSATURA?**
Arquivos pré-compilados (Windows x86-64, macOS arm64, Linux x86-64 e o módulo WebAssembly) estão preparados como o pré-lançamento em rascunho `v0.1.0-alpha.3` em <https://github.com/chiuta/osatura/releases>; eles ficam disponíveis para download quando o dono publicar o rascunho, e não são assinados digitalmente (o SmartScreen mostra "Editor desconhecido"; o macOS pede aprovação). Não há instalador: descompacte o arquivo e execute `osatura`. Ou compile a partir do código-fonte com uma toolchain Rust stable recente: `cargo build -p osatura-cli` gera `target/debug/osatura`; acrescente `--release` para `target/release/osatura`. Veja [DEVELOPMENT.md](DEVELOPMENT.md).

**2. Quais plataformas funcionam?**
O Linux x86-64 é onde o projeto foi desenvolvido e exercitado. No GitHub Actions, a suíte de testes e o clippy também passam em macOS (arm64) e Windows (x86-64), e os binários de release para esses três são compilados lá; mas nenhuma pessoa usou ainda os executáveis de Windows ou macOS em um desktop real, então eles são "testados pelo CI", e não "suportados" ([LIMITATIONS.md](LIMITATIONS.md), seção 1).

**3. Existe um instalador gráfico ou um aplicativo?**
Não há instalador. A linha de comando é o produto. O estúdio é uma página web local servida por `osatura studio`; no Windows, um duplo clique em `osatura.exe` o inicia e abre o seu navegador. Os arquivos para Windows e macOS também contêm `osatura-studio`, uma pequena janela nativa em torno do mesmo estúdio (ela só foi compilada e autotestada no CI); veja [STUDIO.md](STUDIO.md) e [GETTING_STARTED.md](GETTING_STARTED.md).

**4. Como começo rapidamente?**
Siga [GETTING_STARTED.md](GETTING_STARTED.md): ele constrói um personagem animado a partir de um projeto vazio, com todos os passos executados, e termina com um GIF.

**5. Posso compilar sem o motor de scripts?**
Sim: `cargo build -p osatura-cli --no-default-features` deixa o Rhai de fora, de modo que o comando `script` fica ausente (veja [DEVELOPMENT.md](DEVELOPMENT.md)). O recurso `scripting` é o único lugar onde entra um crate com licença de copyleft fraco (smartstring); veja [THIRD_PARTY.md](THIRD_PARTY.md).

**6. Ele precisa de conexão de rede ou de uma conta?**
Não. O programa não precisa de conta nem de rede para funcionar (o estúdio não tem contas). Por padrão, o estúdio escuta apenas na sua própria máquina (veja a pergunta 51).

**7. Onde o estúdio guarda suas configurações?**
Em `$OSATURA_UI_CONFIG`, se definida; caso contrário em `$XDG_CONFIG_HOME/osatura/ui.json`; caso contrário, no Windows, em `%APPDATA%\OSATURA\ui.json`; caso contrário em `~/.config/osatura/ui.json`. Seus projetos nunca são tocados por ele.

---

## ffmpeg

**8. Preciso do ffmpeg?**
Apenas para a exportação em MP4 e WebM, para `sound mix`, para áudio mp3/ogg e para `face-track` em um arquivo de vídeo. A exportação em GIF, APNG, HTML, sequências de PNG, SVG, glTF e JSON não precisa de mais nada. A lista completa está em [CLI_REFERENCE.md](CLI_REFERENCE.md), seção 1.6.

**9. O ffmpeg é incluído?**
Nunca. O OSATURA inicia o `ffmpeg` do seu `PATH` como um programa separado e nada mais é executado. Qualquer que seja a situação de licença e de patentes do seu build do ffmpeg, cabe a você verificá-la; veja [PATENT_NOTES.md](PATENT_NOTES.md).

**10. O que ele diz quando o ffmpeg está ausente?**
`cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead` (para WebM, a dica termina com `or export an APNG instead`). Nada é gravado.

**11. Quais codificadores meu ffmpeg precisa ter?**
`libx264` para mp4, `libvpx-vp9` ou `libaom-av1` para webm, `aac` e `libopus` para o som nesses formatos, `libvorbis` e `libmp3lame` para `sound mix`.

---

## Desempenho

**12. Por que ele é tão lento?**
Provavelmente você está executando um build de depuração. Os números dos documentos vêm de um build de depuração e são de 10 a 30 vezes mais lentos que um build de release ([CLI_REFERENCE.md](CLI_REFERENCE.md), seção 1.7). Na máquina de teste de 2 núcleos, um build de depuração levou 46 s para gravar os 48 quadros do projeto de demonstração como PNG e 57 s para exportá-lo como GIF. Um build de release recém-feito executou as mesmas duas tarefas em 1,5 s e 2,2 s. Compile com `cargo build --release` e use `target/release/osatura`.

**13. Ele usa a GPU?**
O renderizador padrão é a CPU. Um crate `gpu` e um comando `render-gpu` existem por trás de uma feature flag, mas nenhuma GPU real estava disponível durante o desenvolvimento: em uma GPU por software (llvmpipe), um quadro de 640x480 levou de 253 a 295 ms contra 13,8 ms no caminho da CPU. Trate a renderização em GPU como não testada em hardware ([LIMITATIONS.md](LIMITATIONS.md)).

**14. Quanto custa um rig por quadro?**
Em um build de depuração, o exemplo `biped` teve uma mediana de 64,5 ms por quadro na verificação de desempenho do `ci`. O bake de tecido de uma bandeira pequena levou 0,43 s. Meça o seu próprio projeto com `osatura stats` e `osatura ci`; os rigs variam muito.

**15. Como posso deixar um projeto pesado mais leve?**
Use `lod` para criar variantes mais baratas, `export-fit` para encontrar as configurações de exportação que atendem a um orçamento de bytes e `proxy-bench` para comparar proxies. Veja [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) e [EXPORT_FORMATS.md](EXPORT_FORMATS.md). O `export-fit` pode parar mais cedo com `--max-seconds` e então informa que ficou sem tempo.

**16. Um script com milhares de chamadas falha. Por quê?**
`osatura script` aceita no máximo 5000 requisições por execução e é tudo ou nada: se qualquer requisição for rejeitada, nada é gravado. Divida um trabalho grande em vários scripts que leem a saída anterior.

---

## Determinismo

**17. A saída é reproduzível?**
A renderização foi construída para ser: o mesmo projeto e o mesmo seed produzem os mesmos pixels, e a física, as partículas e o noise usam seed. É isso que permite a `golden` e `ci` comparar renderizações. Isso foi verificado em um único tipo de máquina (Linux x86-64); saída idêntica bit a bit em outras CPUs ou compiladores não é prometida.

**18. Por que um bake de física dá um resultado diferente depois que mudo um número?**
O bake é determinístico para uma dada entrada, mas é uma simulação: qualquer mudança em massa, taxa de quadros, collider ou seed muda a execução inteira. Faça o bake com as mesmas configurações e o mesmo seed para comparar. O bake, por padrão, é mesclado com a animação existente (`merge=keep`: os canais existentes são mantidos e o comando diz `kept existing animation on 'b2' (use merge=replace to overwrite)`); use `merge=replace` para sobrescrever.

**19. O noise entra em loop?**
Não. Um canal `noise` não se repete por conta própria, portanto um loop mostrará um salto na emenda. Use chaves ou uma expressão como `8 * sin(t * 2 * pi)`; nas expressões, `t` vai de 0 a 1 ao longo do clipe, não em quadros ([ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)).

**20. Posso verificar se uma animação faz loop de forma limpa?**
Execute `lint`. `make-loop` remove o deslocamento por completo: em um teste, uma bola rolando ficou estática e o `lint` emitiu um aviso (L005). `loop-solve` fecha a emenda para movimento cíclico, mas deixou um objeto em deslocamento com um salto visível (280 para 280 no seu canal de deslocamento). Para deslocamento contínuo, use o modo de extrapolação `loop_offset`.

---

## Licenciamento e a sua saída

*Este é um resumo em linguagem simples de [LICENSING.md](LICENSING.md), não é aconselhamento jurídico. A revisão da estrutura de licenciamento por um advogado ainda está pendente.*

**21. Quais licenças se aplicam?**
MPL-2.0 para a biblioteca core, a C ABI e os crates WASM e GPU; AGPL-3.0-or-later para o programa de linha de comando e o estúdio (`osatura-cli`); CC-BY-4.0 para a documentação e para os recursos e modelos de exemplo sintéticos.

**22. Quem é o dono das animações e renderizações que eu faço?**
Você. A declaração do projeto é que os projetos, as animações e as renderizações que você cria com o OSATURA pertencem inteiramente a você; as licenças se aplicam ao software, não à sua saída. Exportar um GIF ou executar um comando não gera nenhuma obrigação.

**23. Posso usar a saída comercialmente?**
Sim, de acordo com a declaração acima. A exceção é tudo o que você copiar dos recursos de exemplo: esses são CC-BY-4.0 e exigem atribuição.

**24. Preciso publicar meus projetos se usar as partes AGPL?**
Não. A AGPL cobre o programa. Se você executar um `osatura-cli` ou um estúdio modificado de modo que outras pessoas o usem por uma rede, você deve oferecer a elas o seu código-fonte. O estúdio faz isso com uma página `/source` e uma flag `--source-url` que aponta para a sua cópia.

**25. Posso vincular a biblioteca core a um aplicativo de código fechado?**
O core é MPL-2.0, que é um copyleft em nível de arquivo: você pode usá-lo a partir de código fechado e deve compartilhar as alterações nos próprios arquivos cobertos pela MPL. A CLI e o estúdio são AGPL e são outro assunto.

**26. Posso dizer que minha ferramenta "funciona com o OSATURA"?**
Sim: "compatível com o OSATURA" e "feito com o OSATURA" são permitidos. Um build modificado não deve ser distribuído com o mesmo nome ([TRADEMARK.md](TRADEMARK.md)).

**27. Posso licenciar minha contribuição sob licença dupla?**
Não existe contrato de licença de contribuidor. As contribuições são aceitas sob a licença do projeto com uma assinatura DCO (`git commit -s`), portanto não há um dono que possa relicenciá-las. Detalhes em [CONTRIBUTING.md](CONTRIBUTING.md).

**28. E o código de terceiros?**
Cerca de 360 crates de terceiros (contados em todas as plataformas; o aplicativo de janela acrescenta o toolkit de Windows / macOS), todos com licenças permissivas, exceto seis MPL-2.0: o smartstring (MPL-2.0 ou posterior, apenas com o recurso `scripting`) e cinco que somente o aplicativo de janela pode trazer. As listas estão em [THIRD_PARTY.md](THIRD_PARTY.md) e [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md).

**29. Há preocupações com patentes?**
Veja [PATENT_NOTES.md](PATENT_NOTES.md). São notas de engenharia, não um parecer jurídico.

---

## Segurança dos arquivos e caminhos de recursos

**30. Um comando pode danificar meu projeto?**
As gravações são atômicas: o resultado vai para um arquivo temporário no mesmo diretório, é sincronizado e é renomeado sobre o destino, de modo que uma falha deixa o arquivo antigo. Os comandos de edição revalidam o projeto que estão prestes a gravar e se recusam a gravar um inválido (`output would be invalid: ...`). O comando `script` não aplica nada se qualquer requisição falhar.

**31. É seguro abrir um projeto de outra pessoa?**
Razoavelmente, mas não incondicionalmente. Um projeto é dado, não código; o único programa externo que o OSATURA inicia é o ffmpeg, para os formatos listados na pergunta 8. O carregador recusa dispositivos, pipes e diretórios, limita o tamanho da entrada e recusa caminhos de recursos absolutos ou com `..`. Os scripts Rhai são executados em um sandbox. Ainda assim, leia [SECURITY.md](SECURITY.md) antes de processar arquivos não confiáveis em um serviço.

**32. Onde um projeto procura sons e imagens?**
Relativamente à pasta do arquivo do projeto. Caminhos absolutos e qualquer caminho com `..` são recusados, a menos que você passe `--allow-external-assets`, e você precisa passá-lo novamente em todo comando posterior que carregar esse projeto. Exemplo da referência:
`sound add: asset '/etc/passwd': absolute paths are not allowed (a project may only reference files inside its own directory; use --allow-external-assets to override) (nothing was added)`.

**33. Que tamanho de projeto ele lê?**
Até 256 MiB por padrão. Mude com `--max-input-bytes N`. Arquivos maiores falham com `cannot read demo.json: 51450 bytes exceed the input limit of 100 bytes (raise it with --max-input-bytes N)` (os números são os do exemplo).

**34. Por que ele se recusa a gravar em um link simbólico?**
Para que um link plantado ao lado da sua saída não possa redirecionar uma gravação para outro lugar. A mensagem é `refusing to write through a symbolic link; pass --follow-symlinks to write to the link's target, or name the target file yourself`.

**35. Por que não consigo ler de `<(...)` ou de um pipe?**
As entradas precisam ser arquivos comuns: `cannot read /dev/fd/63: not a regular file (devices, pipes and directories are refused)`. Grave os dados em um arquivo primeiro.

**36. Um campo desconhecido em um projeto o quebra?**
Não: `warning: unknown field bogus (ignored)` é impresso e o status de saída é 0. Os arquivos auxiliares que você grava para um comando (contratos, mapeamentos, máquinas) são mais rígidos: uma chave desconhecida neles é um erro e nada é aplicado.

---

## Por que a saída deve ser diferente da entrada

**37. Por que `render in.png in.png` diz "refusing to overwrite"?**
Para os comandos que leem um projeto e gravam uma imagem, um vídeo ou outro arquivo que não seja de projeto, o nome da saída não pode ser o nome da entrada. A mensagem é `refusing to overwrite the input project X with output of the same name`. Isso evita o deslize clássico de exportar sobre a sua única cópia do projeto.

**38. Então por que `script` e `autorig` podem gravar sobre a própria entrada?**
A proteção cobre as saídas que não são de projeto. Os comandos que leem um projeto e gravam um projeto (`script`, `autorig`, `loop-solve` e outros) podem indicar o mesmo arquivo para a entrada e a saída; a gravação é atômica e, se o comando falhar, nada muda. Se você quiser manter o original, dê um novo nome ou use `osatura vcs` para manter o histórico ([CI.md](CI.md) e [CLI_REFERENCE.md](CLI_REFERENCE.md)).

**39. Por que `render` recusa `out.jpg`?**
`render` grava apenas `.png` ou `.svg`: `render writes .png or .svg; 'out.jpg' has another extension (nothing was written)`. Para outros formatos, use `export`; a lista está em [EXPORT_FORMATS.md](EXPORT_FORMATS.md).

---

## Rigging e animação

**40. Para que lado fica o zero grau de um osso?**
O ângulo 90 do osso aponta para baixo; os ângulos não são medidos a partir do eixo x da forma como um livro de matemática faz. Os canais são deltas em relação à pose de repouso, e `rest_offset` está no referencial do pai ([RIGGING_GUIDE.md](RIGGING_GUIDE.md)).

**41. Minha cadeia de IK não estica ou dá um solavanco. O que são `soft` e `stretch`?**
`soft` (0 a 0,9) suaviza a cadeia até a extensão total e `stretch` (1 a 4) permite que ela se alongue. Ambos existem apenas em JSON, não como opções do Rhai nem da CLI. Com `stretch` acima de 1, a zona suave é o fim do alcance esticado. As cadeias de dois ossos são resolvidas analiticamente; as de três ou mais usam FABRIK.

**42. Minhas restrições dão resultados estranhos.**
As restrições são executadas na ordem da lista, e a ordem importa. Um aim com peso 0,5 dá metade do ângulo (um ângulo de -74,7 vira -37,4). Reordene no JSON se uma restrição posterior deve ver o resultado de uma anterior.

**43. Existe um skinner automático para arte vetorial?**
Não na linha de comando. `Project::auto_skin` existe apenas na biblioteca. Para arte vetorial, você faz o rig com ossos associados a camadas, com lattices, splines ou pins (somente em JSON), ou com malhas traçadas via `mesh-trace`.

**44. `skin-diffuse` e `cloth` reclamam da malha.**
Ambos precisam de uma malha, e o cloth precisa de uma malha em **grade**: `cloth needs a grid mesh; the layer has a free-form (traced) mesh`. Uma camada sem nenhuma malha dá `layer 'flag' has no mesh`.

**45. `foot-lock` diz que precisa de uma raiz.**
`foot lock needs a root controller bone (no parent)`. O osso controlador raiz não pode ter osso pai.

**46. `scale` recusa meus keyframes.**
`scale: frame 27 is outside the project range 0..=24` significa que uma chave resultante cairia fora do projeto; `keyedit scale` é recusado quando fora do intervalo, a menos que você acrescente `--free`. Uma execução de teste usa `-` como nome de saída.

**47. Por que meu projeto ficou mais longo depois de `mocap-retarget`?**
`mocap-retarget` estende a duração do projeto para acomodar o movimento capturado. `follow --chain` pode deslocar chaves além do último quadro, o que o `lint` relata (L007).

**48. Como animo uma camada sem um osso?**
Edite os canais `layer_x` e `layer_y` da camada. No estúdio, ative a chave automática; com ela desativada, uma edição de camada permanece estática, a menos que aquela propriedade já tenha um canal com chaves. As edições de osso sempre criam chaves. Veja [STUDIO.md](STUDIO.md).

---

## Estúdio

**49. Como abro o estúdio?**
`osatura studio project.json` e abra o endereço que ele imprime, normalmente `http://127.0.0.1:<port>`. É um aplicativo de página única servido pelo pequeno servidor HTTP próprio do OSATURA (apenas biblioteca padrão); não precisa de internet.

**50. O estúdio altera o meu arquivo enquanto trabalho?**
Não. O arquivo em disco só muda quando você pressiona Save. Fechar a página sem salvar perde as edições (o log de colaboração, se ativado, guarda o projeto inteiro, mas não o histórico de desfazer).

**51. Outras pessoas podem se conectar?**
Somente se você associar um endereço que não seja de loopback, e então um token é exigido: `refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (and still sends everything in clear text)`. O token deve ter pelo menos 16 caracteres, por exemplo gerado com `openssl rand -hex 16`. Não há criptografia: use um túnel SSH ou um proxy reverso com TLS.

**52. Onde coloco o token?**
No cabeçalho `X-Collab-Token` ou no cookie `osatura_token` (HttpOnly, SameSite=Strict e Secure quando a requisição chega sob uma `--public-origin` https). Ele nunca é aceito na URL. A partir da quinta tentativa malsucedida, esse endereço de cliente recebe HTTP 429 com um atraso crescente (2, 4 ... até 30 s).

**53. O que significam os erros HTTP 403, 415 e 421?**
421: o cabeçalho Host não é um dos que o estúdio espera. 403: um `Origin` ou `Sec-Fetch-Site` externo. 415: um POST que não é JSON. São proteções contra páginas no seu navegador chamando o servidor local; veja [STUDIO.md](STUDIO.md) e [SECURITY.md](SECURITY.md).

**54. Funciona em qualquer navegador?**
Foi testado apenas no Chromium headless. Os outros não foram testados. Não há comando de zoom do canvas, e existe um painel dentro do estúdio para rastreamento facial por webcam, mas uma câmera real nunca esteve disponível para teste ([LIMITATIONS.md](LIMITATIONS.md)).

---

## Rosto, fala e captura de movimento

**55. O OSATURA rastreia rostos?**
Há um pequeno detector integrado, treinado apenas com rostos sintéticos. Em 50 cenas de teste sintéticas (seed 9), ele encontrou todos os rostos, com um erro médio de pontos de referência de 0,028 do tamanho do rosto, e gerou 2 falsos alarmes em 50 cenas sem rosto. Em desenhos de estilo cartoon, ele se saiu mal: 0 de 6 rostos de robô quadrado foram encontrados, e os casos do emoji redondo e do oval de mangá tiveram erros de 0,164 e 0,166. Ele nunca foi executado em fotografias reais nem em uma webcam real. Observe que [LIMITATIONS.md](LIMITATIONS.md) L-13 e o texto do painel de webcam do estúdio dizem que nenhum detector de rostos é incluído; isso está desatualizado em relação a este detector sintético.

**56. Como funciona o lip-sync?**
As indicações de boca escolhem entre cinco formas (repouso, pequena, aberta, redonda, larga). Um tempo de indicação com ponto decimal são segundos; um inteiro sem ponto é um quadro. `--rhubarb` só muda como a letra E é classificada (redonda em vez de larga). Áudio mais longo que o projeto é truncado.

**57. Posso animar a partir de vídeo?**
`face-track` lê vídeo por meio do ffmpeg. `face-bake` grava os sinais como canais com chaves (o resumo diz "+N channels (K keys in them), M trigger keys set") e trunca no comprimento do projeto, com um aviso no stderr contando os quadros que não passaram pelo bake. Um sinal desconhecido dá `unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)`.

---

## Bugs, segurança e contribuição

**58. Como relato um bug?**
Abra uma issue com a linha de comando, a saída exata (inclusive o stderr), o status de saída, o build (`osatura --version`) e, se puder, um projeto mínimo. Execute antes `osatura check project.json` e `osatura lint project.json`: muitos problemas são relatados ali. Veja [CONTRIBUTING.md](CONTRIBUTING.md).

**59. Como relato um problema de segurança?**
De forma privada, não em uma issue pública; veja [SECURITY.md](SECURITY.md). O endereço no repositório é `alexio@trom.tf`; é o mesmo endereço do contato do mantenedor e não foi substituído por uma caixa de correio de equipe de segurança.

**60. Existe lista de e-mail, chat ou contrato de suporte?**
Não. Os documentos indicam um contato do mantenedor; o primeiro lançamento ainda é um pré-lançamento em rascunho.

**61. Como contribuo?**
Assine cada commit (`git commit -s`), mantenha os testes passando (`cargo test` e, para trabalho de animação, `osatura ci`) e leia [DEVELOPMENT.md](DEVELOPMENT.md). A documentação é CC-BY-4.0 e aceita correções com prazer, especialmente onde uma página e o programa discordam.

---

## Limitações

**62. Qual é a limitação mais importante?**
Ele foi construído e testado por um único projeto em um único ambiente. Tudo o que não está listado como executado é "não testado". Leia [LIMITATIONS.md](LIMITATIONS.md); ele tem ids que você pode citar em issues.

**63. Quais formatos de exportação exigem cuidado?**
O vídeo precisa do ffmpeg. O Lottie e o glTF cobrem um subconjunto do que o OSATURA consegue desenhar e animar, portanto verifique o que se perde em [EXPORT_FORMATS.md](EXPORT_FORMATS.md). O erro de `export` para uma extensão desconhecida lista `.gif .html .json .mp4 .apng .webm .png .glb`, o que omite `.lottie`; `.lottie` (dotLottie) é um destino válido, veja [EXPORT_FORMATS.md](EXPORT_FORMATS.md), seção 6.

**64. O build WASM é utilizável?**
Ele compila para `wasm32-unknown-unknown` no GitHub Actions e o teste do wrapper passa contra o módulo real sob o Node 20, e está anexado ao lançamento em rascunho como `osatura-web.tar.gz`. Ele não foi executado em um navegador, e sua velocidade e memória não foram medidas ([WASM.md](WASM.md), [LIMITATIONS.md](LIMITATIONS.md)). Trate-o como parcialmente verificado.

**65. Ele importa arquivos de projeto de outras ferramentas?**
Ele importa SVG, OpenRaster, JSON Lottie, imagens, GIF/APNG, captura de movimento BVH e MIDI ([EXPORT_FORMATS.md](EXPORT_FORMATS.md), seções 9 e 10). Não importa PSD, arquivos zip `.lottie` nem pastas OCA. As importações criam um novo projeto. Observe que `import-lottie in.json in.json` não tem proteção contra sobrescrita e substitui a sua entrada. Não espere que um arquivo importado corresponda ao original pixel a pixel.

**66. O JSON do projeto é estável?**
É o "format 1". Um arquivo mais novo é recusado: `project format 2 is newer than this build supports (1); update OSATURA`. Como existe apenas um pré-lançamento alpha, o próprio format 1 ainda pode mudar antes de uma versão estável; veja [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

---

## Comparações com outras ferramentas

**67. Em que ele difere de um editor de animação de desktop baseado em timeline?**
Esses são programas interativos, centrados no desenho. O OSATURA é centrado em comandos: toda operação é um comando ou um script, o projeto é um JSON que pode ser comparado com diff, e as renderizações são determinísticas, de modo que podem ser testadas em CI. O estúdio é uma visão web fina sobre o mesmo motor e é muito menos refinado que um editor maduro.

**68. Em que ele difere de uma ferramenta de animação esquelética 2D para jogos?**
Essas ferramentas se concentram em malhas, skins e bibliotecas de runtime para motores de jogo específicos. O OSATURA também tem ossos, IK, restrições e malhas, mas uma história de runtime menor: seu próprio player em HTML, uma C ABI e um crate WASM que não foi compilado. Escolha de acordo com o runtime que você precisa atingir.

**69. Em que ele difere de um compositor de motion graphics?**
Um compositor é construído em torno de camadas com efeitos e expressões sobre filmagens de vídeo. O OSATURA tem efeitos de camada e expressões, mas seu centro é o rigging de personagens e a arte vetorial, e ele não faz composição de vídeo, 3D nem correção de cor de filmagens.

**70. Quando o OSATURA é a escolha errada?**
Quando você precisa hoje de desempenho em tempo real em uma GPU, de um fluxo de trabalho polido com mesa digitalizadora, de animação desenhada à mão quadro a quadro, de um produto com suporte de fornecedor, com versão estável lançada e suporte, ou de comportamento confirmado fora do Linux x86-64.

---

## Tabela de solução de problemas

Cada mensagem abaixo foi produzida por um build real e termina com status de saída 1, salvo indicação em contrário. A redação de uma mensagem pode mudar antes de uma versão lançada.

| Mensagem | Causa | Correção |
|---|---|---|
| `cannot read nope.json: No such file or directory (os error 2)` | Caminho ou nome errado | Verifique o caminho e o diretório atual |
| `invalid JSON in broken.json: EOF while parsing a value at line 2 column 0` | O arquivo não é um JSON completo | Corrija a sintaxe na linha indicada; restaure de um backup ou com `osatura vcs` |
| `badparent.json is not a valid project:` then `bone 'a' has parent 'ghost' which is missing or listed after it` | Um osso pai está ausente ou listado depois do seu filho | Defina os pais antes dos filhos; corrija o nome |
| `invalid JSON in fmt2.json: project format 2 is newer than this build supports (1); update OSATURA` | Arquivo de um build mais novo | Use um build mais novo |
| `warning: unknown field bogus (ignored)` (saída 0) | Uma chave que este build não conhece | Remova ou corrija a chave; verifique se há erro de digitação |
| `refusing to overwrite the input project X with output of the same name` | O nome do arquivo de saída é igual ao da entrada | Escolha outro nome de saída |
| `render writes .png or .svg; 'out.jpg' has another extension (nothing was written)` | Extensão errada para `render` | Use `.png`/`.svg`, ou `export` para outros formatos |
| `unknown option --bone for 'render' (did you mean --bones?); see: osatura (no arguments) for the usage` | Opção digitada errado | Use a opção sugerida |
| `unknown export format '.bmp' (use .gif .html .json .mp4 .apng .webm .png .glb)` | Extensão não exportável | Escolha um formato listado (`.lottie` também é válido) |
| `--scale needs a number in (0, 16]` | Escala fora do intervalo | Use um valor acima de 0 e até 16 |
| `bad frame number 'x'` | Um argumento de quadro não é um número inteiro | Passe um inteiro |
| `cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead` | ffmpeg fora do `PATH` | Instale o ffmpeg ou exporte GIF, HTML ou APNG |
| `the project has no sound tracks (see: sound add)` | Comando de som em um projeto sem som | Adicione uma faixa com `sound add` |
| `channel hand_ctrl.BoneOffsetY is locked` | O canal está bloqueado | Desbloqueie-o ou edite outro canal |
| `request 1 (hand_ctrl.BoneOffsetY at 0) was rejected, nothing changed: channel hand_ctrl.BoneOffsetY is locked` | O mesmo, vindo de `script` | Desbloqueie primeiro; nada foi aplicado |
| `request 1 (channel head.BoneAngle) was rejected, nothing changed: ... expression error: unknown variable 'tt'` | Erro de digitação em uma expressão | As variáveis são frame, time, fps, frames, pi, t |
| `... unexpected end of expression` | Expressão incompleta | Feche os parênteses, complete a fórmula |
| `points need an even number of coordinates: 3..1000 points` | Número ímpar de coordenadas ou pontos de menos | Forneça pares x,y: de 3 a 1000 pontos |
| `cannot read /dev/fd/63: not a regular file (devices, pipes and directories are refused)` | Substituição de processo ou pipe como entrada | Salve primeiro em um arquivo comum |
| `sound add: asset '/etc/passwd': absolute paths are not allowed (...)` | Recurso fora da pasta do projeto | Copie-o para ao lado do projeto ou passe `--allow-external-assets` |
| `cannot read demo.json: N bytes exceed the input limit of M bytes (raise it with --max-input-bytes N)` | Arquivo acima do limite de entrada | Aumente `--max-input-bytes` se você confia no arquivo |
| `refusing to write through a symbolic link; pass --follow-symlinks ...` | A saída é um link simbólico | Indique o destino ou passe `--follow-symlinks` |
| `cloth needs a grid mesh; the layer has a free-form (traced) mesh` | Cloth em uma malha traçada | Construa uma malha em grade para a camada |
| `layer 'flag' has no mesh` | Um comando de malha em uma camada simples | Crie uma malha primeiro |
| `no bone 'b9'` | O nome do osso não existe | Verifique os nomes com `stats` ou `explain` |
| `unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)` | Nome de sinal errado | Use um nome da lista |
| `foot lock needs a root controller bone (no parent)` | A raiz escolhida tem um pai | Use um controlador sem pai |
| `scale: frame 27 is outside the project range 0..=24` | As chaves escaladas caem fora do projeto | Alongue o projeto, escale menos ou use `--free` |
| `--extrapolate must be hold, linear, loop, ping_pong or loop_offset` | Modo inválido | Use um dos cinco |
| `cannot listen on 127.0.0.1:8861: Address already in use (os error 98)` | Porta ocupada | Escolha outra `--port` ou encerre o outro processo |
| `cannot listen on 127.0.0.1:99999: invalid port value` | Porta acima de 65535 | Use de 1 a 65535 |
| `refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (...)` | Associação pública sem token | Acrescente `--collab-token` ou permaneça no loopback |
| `--collab-token must be at least 16 characters (e.g. \`openssl rand -hex 16\`)` | Token fraco | Gere um mais longo |
| `--source-url must be an http:// or https:// URL without spaces or quotes` | URL malformada | Use uma URL simples |
| `--public-origin must be a bare origin (scheme, host and optional port, no path), e.g. https://studio.example.org` | A origem tem um caminho | Informe apenas esquema, host e porta |
| `output would be invalid: ...` | A edição deixaria um projeto inválido | Leia os motivos após os dois-pontos; nada foi gravado |
| `budget not met: the smallest export found is 14952 bytes (raw), above the budget of 8000 bytes ...` | `export-fit` não consegue atingir o orçamento | Aumente o orçamento ou simplifique o projeto (o menor arquivo ainda é gravado) |
| `time budget exhausted (60 s ...)` | `export-fit` ficou sem tempo | Aumente `--max-seconds` |
| `fuzz: failure rate 98.00% exceeds --max-fail-rate 50.00%` | Poses quebraram o rig e você pediu um gate (`--gate` ou `--max-fail-rate`) | Veja os casos que falham; aumente a taxa somente se for intencional; `fuzz` simples apenas relata (saída 0) |
| `no .osatura-vcs repository found from ... (run: osatura vcs init <project.json>)` | Ainda não há histórico | Execute `vcs init` |
| `frame 999 out of range` (API do estúdio) | Quadro além do projeto | Use um quadro de 0 até o último quadro |
| `bad kind 'x' (vector, raster, png)` (API do estúdio) | Tipo errado | Use um tipo listado |
| `nothing selected` (API do estúdio) | Um comando de seleção sem seleção | Selecione camadas ou ossos primeiro |

Os códigos de saída de `osatura ci` são 0 aprovado, 1 reprovado, 2 erro.

---

*Documentação CC-BY-4.0. Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio). Você pode compartilhá-la e adaptá-la com atribuição. Veja também: [INDEX.md](INDEX.md), [GLOSSARY.md](GLOSSARY.md).*
