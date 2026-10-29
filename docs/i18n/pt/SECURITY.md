<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Segurança

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/docs/SECURITY.md) · [Română](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/ro/SECURITY.md) · [Français](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/fr/SECURITY.md) · [Italiano](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/it/SECURITY.md) · [Español](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/es/SECURITY.md) · [Português](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/pt/SECURITY.md) · [Deutsch](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/de/SECURITY.md) · [Magyar](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/hu/SECURITY.md)

*Modelo de ameaças, proteções embutidas, lacunas conhecidas, como reportar um problema e como implantar o studio com segurança. Documentação CC-BY-4.0. Derivada do código-fonte (`osatura-core/src/{api,assets,fsutil,expr,script}.rs`, `osatura-cli/src/{http,studio,collab_srv}.rs`), de `ROADMAP.md` seção 9 (ondas de revisão 1 a 4) e dos testes citados abaixo. **O OSATURA não passou por nenhuma auditoria de segurança independente.** As ondas de revisão foram feitas pelo próprio projeto; trate esta página como uma descrição da intenção e do que os testes cobrem, não como uma garantia.*

Conteúdo: [1 Modelo de ameaças](#1-modelo-de-ameaças) - [2 Servidor HTTP do studio](#2-servidor-http-do-studio) - [3 Autenticação e colaboração](#3-autenticação-e-colaboração) - [4 Resolução de assets e limites de entrada](#4-resolução-de-assets-e-limites-de-entrada) - [5 Sandbox de expressões e scripts](#5-sandbox-de-expressões-e-scripts) - [6 Arquivos, links simbólicos e dados temporários](#6-arquivos-links-simbólicos-e-dados-temporários) - [7 Programas externos (ffmpeg)](#7-programas-externos-ffmpeg) - [8 Bibliotecas (ABI C, WASM, fachada)](#8-bibliotecas-abi-c-wasm-fachada) - [9 Lacunas conhecidas](#9-lacunas-conhecidas) - [10 Reportando uma vulnerabilidade](#10-reportando-uma-vulnerabilidade) - [11 Checklist de endurecimento para quem implanta](#11-checklist-de-endurecimento-para-quem-implanta) - [12 Proxy reverso com TLS: nginx e Caddy](#12-proxy-reverso-com-tls-nginx-e-caddy)

## 1. Modelo de ameaças

O que o OSATURA processa: **arquivos de projeto** (JSON), imagens, áudio, arquivos de importação Lottie / OCA / estilo glTF, arquivos de mapeamento e, no studio, requisições HTTP de um navegador. No modelo abaixo, os arquivos de projeto são *dados não confiáveis*: um projeto recebido de outra pessoa pode ser hostil, e abri-lo não deve executar código, ler arquivos fora da sua pasta nem esgotar a máquina.

| Ator | Pode | Objetivo presumido | Principais defesas |
|---|---|---|---|
| Autor de um projeto / arquivo de importação hostil | entregar um arquivo a você | ler seus arquivos, travar ou derrubar a ferramenta, executar código, encher o disco | análise sintática estrita, limites de tamanho e de profundidade, recusa de caminhos de assets, nenhum panic na entrada, sandbox de expressões e scripts, saída atômica |
| Página web que você visita enquanto o studio roda em localhost | fazer seu navegador enviar requisições para `127.0.0.1` (CSRF, DNS rebinding) | editar seu projeto, lê-lo, gravar arquivos | lista de Hosts permitidos, verificações de Origin / `Sec-Fetch-Site`, tipo de conteúdo JSON, token com cookie `SameSite=Strict`, CSP |
| Outro usuário na mesma máquina | conectar-se à porta, ler `/tmp` | ler ou sequestrar uma sessão ou arquivos temporários | bind em `127.0.0.1` por padrão, diretórios temporários privados (0700), arquivos de log 0600, token obrigatório para fazer bind em outro endereço |
| Alguém na sua rede, quando o studio está com bind além do localhost | alcançar a porta | usar ou adivinhar o token, esgotar as conexões | token obrigatório (16+ caracteres), comparação em tempo constante, back-off, limites de conexões e de tamanho; **sem TLS** |
| Um colaborador com o token | editar a sessão compartilhada | danificar o projeto | histórico de desfazer, o log encadeado por hashes; os colaboradores têm a confiança do projeto, não há contas nem papéis |
| Chamador hostil das bibliotecas | passar argumentos inválidos à fachada, à ABI C ou WASM | corrupção de memória, panics através da fronteira | validação de argumentos, verificação de tamanhos antes da alocação, panics capturados (onde o alvo faz unwinding) |

Fora do escopo: proteção contra um administrador local malicioso, uma cadeia de ferramentas de build ou dependência comprometida (veja [THIRD_PARTY.md](THIRD_PARTY.md) para o que é distribuído; `cargo deny` policia licenças, proibições e fontes, não vulnerabilidades), canais laterais e a segurança do próprio ffmpeg.

## 2. Servidor HTTP do studio

`osatura studio` serve o editor a partir de uma pequena implementação própria de HTTP/1.1 (`osatura-cli/src/http.rs`, apenas biblioteca padrão, sem framework web de terceiros). Comportamento, com os testes que o fixam (testes de unidade de `osatura-cli`, `tests/robust.rs`, `tests/wave4.rs`, `tests/source_offer.rs` e os smoke tests em Chromium):

**Limites**

| Limite | Valor | Resultado ao exceder |
|---|---|---|
| Tempo para receber o cabeçalho da requisição | 10 s | 408 |
| Tempo para receber o corpo | 20 s | 408 |
| Tempo para gravar uma resposta | 10 s | conexão encerrada |
| Tamanho do cabeçalho da requisição | 16 KB | 431 |
| Número de cabeçalhos | 100 | 431 |
| Tamanho do corpo | 32 MB | 413 |
| Conexões simultâneas | 128 | 503 |
| Corpos de requisição em chunks | não suportados | 411 |
| Cabeçalho `Host` duplicado | recusado | 400 |
| Caractere de controle (CR, NUL, DEL…) em um valor de cabeçalho | recusado (um tab é permitido) | 400 |

Toda resposta traz `Connection: close` (sem estado de keep-alive a esgotar), `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer` e um `Content-Security-Policy`.

**Política de requisições** (`Policy::check`, aplicada a toda requisição):

1. **Lista de Hosts permitidos.** O cabeçalho `Host` deve ser o endereço de bind (`127.0.0.1`, `localhost`, a porta real) ou um nome fornecido com `--allow-host`. Caso contrário, 421 Misdirected Request. É isso que impede o DNS rebinding: uma página em `evil.example` que resolve para `127.0.0.1` envia `Host: evil.example`.
2. **Origin e `Sec-Fetch-Site`.** Requisições com um `Origin` de outro site, ou com `Sec-Fetch-Site: cross-site` / `same-site`, são recusadas com 403. `Origin: https://<allowed host>` é aceito para os hosts em `--allow-host`; atrás de um proxy reverso que serve a página em uma URL pública, passe `--public-origin URL` para que a origem do proxy seja aceita. A origem cruzada continua recusada.
3. **Tipo de conteúdo.** `POST` exige `Content-Type: application/json` (caso contrário, 415), de modo que um formulário HTML simples não consegue enviar dados à API.
4. **Token** (veja a seção 3), quando configurado; obrigatório quando `--bind` não é um endereço de loopback.

**Oferta de código-fonte.** O studio serve `/source` (também quando um token é exigido) e `--source-url` define o endereço (os binários de release oficiais usam por padrão o commit a partir do qual foram compilados), conforme exigido pela seção 13 da AGPL-3.0 (veja [LICENSING.md](LICENSING.md)).

**Caminhos e métodos desconhecidos ou malformados** recebem uma resposta de erro simples; a página do editor é compilada dentro do binário (`studio.html`) e não é lida do sistema de arquivos, portanto não há superfície de path traversal para conteúdo estático.

## 3. Autenticação e colaboração

A opção `--collab-token T` (ou o token exigido por um `--bind` não local) habilita o controle de acesso; `--collab-log LOG` persiste uma sessão compartilhada.

* **Formato do token.** Pelo menos 16 caracteres, de `A-Za-z0-9._~+/=-`. Qualquer coisa com CR, LF, `;` ou outros caracteres é recusada na inicialização (injeção em cabeçalho e cookie).
* **Apresentação.** Cabeçalho `X-Collab-Token: T`, ou o cookie `osatura_token` (`HttpOnly`, `SameSite=Strict`, mais `Secure` quando a requisição chegou sob um nome `--public-origin` `https://`, um `Origin` https ou `X-Forwarded-Proto: https`) definido por `POST /api/login`. Um cookie errado conta como tentativa falha, como um cabeçalho errado.
* **Comparação.** Em tempo constante (ambos os lados passam por hash SHA-256 e os digests são comparados), de modo que o tempo de resposta não revela coincidências de prefixo.
* **Back-off.** Por endereço IP de cliente, a partir da **5ª** falha consecutiva: 2, 4, 8, 16 e depois 30 segundos de respostas 429; até 512 endereços de cliente são rastreados. O token correto é sempre aceito, mesmo durante um back-off (de modo que um atacante não consegue bloquear o dono). O back-off retarda as tentativas de adivinhação; não impede um ataque distribuído. **Atrás de um proxy reverso, todos os clientes aparecem com o endereço do proxy**, portanto compartilham um único contador de back-off: uma enxurrada de requisições inválidas de um visitante atrasa o contador de falhas de todos, mas o token correto continua funcionando.
* **Segredos de participante.** Cada colaborador recebe um segredo aleatório de 128 bits (`X-Collab-Secret`) que identifica seu fluxo; no máximo 32 participantes e 64 fluxos de eventos.
* **Integridade do log.** O log da sessão é um arquivo de texto, uma linha por operação: `<hash> <json>`, encadeado com SHA-256 (formato versão 2). Editar, excluir ou reordenar uma linha quebra a cadeia, o que é informado quando o log é carregado. Cada requisição passa por `fdatasync` antes da resposta, o arquivo é criado com modo 0600 e reescrito atomicamente. A cadeia detecta danos acidentais e adulterações casuais; **não é assinada**, de modo que quem consegue gravar o arquivo pode reconstruir uma cadeia válida.
* **Tokens por participante (onda 7, opcional).** `--collab-secret-file F` substitui o bearer compartilhado único por tokens derivados de um segredo: `ap1.<name>.<expires>.<mac>` com `mac` = HMAC-SHA-256 (RFC 2104, SHA-256 próprio, vetores de teste da RFC 4231 nos testes) sobre o nome e o prazo. O host os emite com `osatura collab-token NAME --secret-file F --ttl SECONDS` (padrão 8 horas, no máximo 30 dias) e entrega a cada pessoa o seu. Um token está vinculado a um nome (um participante que entra recebe esse nome, independentemente do que a página enviar, de modo que ninguém consegue entrar como outra pessoa), expira por conta própria, é verificado por recálculo (sem tabela no servidor, nada a roubar dele além do próprio segredo) e seu cookie recebe um `Max-Age` igual à vida restante. `--collab-revoke NAMES` recusa os tokens desses nomes antes de expirarem; trocar o segredo revoga tudo. O arquivo de segredo deve ser privado (modo 600 em Unix; caso contrário, o studio se recusa a iniciar), ter pelo menos 16 caracteres e nunca é aceito como argumento de linha de comando. `--collab-token` e `--collab-secret-file` podem ser combinados; qualquer um satisfaz a regra de que um bind fora do loopback precisa de uma credencial. A expiração é verificada **por requisição**: um fluxo de eventos já aberto permanece aberto até terminar, e a conexão seguinte é recusada. Um token expirado ou revogado recebe `401` com o motivo e não alimenta o back-off de tentativas de adivinhação.
* **Limite de taxa (onda 7).** `--collab-rate N` permite N requisições por segundo por endereço de cliente, com rajadas de 2N (token bucket); acima disso a resposta é `429` com `Retry-After`. O padrão é 100 por segundo quando uma credencial é exigida e nenhum limite caso contrário; 0 o desativa. É aplicado após a autenticação e por **endereço** (a origem de uma conexão), não por conexão TCP, porque um limite por conexão é contornado abrindo mais conexões; atrás de um proxy reverso todos os clientes compartilham o endereço do proxy, portanto limite também no proxy (seção 12).
* **Avisos sobre o endereço de bind (onda 7).** Um `--bind` fora do loopback imprime um aviso ao iniciar (HTTP puro; para `0.0.0.0` / `::`, que o studio escuta em todas as interfaces; para um token compartilhado, que ele não tem nomes e nunca expira).
* **Grafia do token.** O prazo de validade em um token por participante tem exatamente uma forma escrita (dígitos decimais, sem sinal, sem zeros à esquerda, sem `+`): qualquer outra grafia do mesmo número é recusada como malformada, e a assinatura é avaliada antes do prazo, de modo que um token forjado não aprende nada com a resposta.
* **Hosts (opcional).** `--collab-host NAMES` (requer `--collab-secret-file`) indica os participantes que podem **salvar** (`/api/save`), **compactar** o log da sessão (`/api/collab/compact`) e **revogar** (`/api/collab/revoke`, um `POST` que acrescenta nomes à lista de revogados, no máximo 1000). Todos os demais com um token válido ainda podem editar e acompanhar, e recebem `403` nessas três operações. Sem `--collab-host` nada muda: todo portador de uma credencial válida pode fazer tudo isso, como antes. O `--collab-token` compartilhado não está ligado a um nome e conta como host.
* **Fluxo de eventos.** O fluxo de eventos exige uma credencial (um token), mas não um segredo de participante; o segredo identifica o fluxo do próprio participante, não é uma segunda senha.
* **Conexões por endereço.** Um endereço de cliente pode manter no máximo 16 conexões ao mesmo tempo (além do teto global), de modo que uma única máquina não consiga ocupar todas as vagas. Os endereços de loopback não são contados por padrão, porque um proxy reverso no mesmo host chega ao studio a partir de `127.0.0.1` para cada visitante; limite por visitante no proxy (seção 12).
* **Sem TLS, sem contas.** O servidor fala HTTP puro. Um token que trafega por uma rede pode ser lido por qualquer pessoa no caminho. Coloque o studio atrás de um proxy reverso que termine o TLS ou de um túnel SSH / VPN (seção 11). Não há contas de usuário nem permissões por projeto; o único papel é a lista opcional de hosts acima, e qualquer outro token válido concede acesso de edição.

## 4. Resolução de assets e limites de entrada

* **Assets** (arquivos de imagem, normal maps, sons nomeados por um projeto) são resolvidos por `assets::resolve`: em relação ao diretório de assets do processo (`assets::default_dir`, normalmente a pasta do projeto de entrada). **Caminhos absolutos e qualquer componente `..` são recusados**, a menos que a flag global `--allow-external-assets` seja fornecida. Os arquivos de imagem são limitados a 64 MB, os de áudio a 256 MB.
* **Arquivos de projeto.** `load_project` lê apenas arquivos regulares (sem dispositivos, sem pipes), até 256 MB (`--max-input-bytes N` para alterar). O `MAX_JSON_BYTES` da fachada é 256 MiB.
* **Projetos derivados.** Os comandos que gravam um projeto derivado validam em relação ao diretório do projeto de *entrada*; quando a saída está em outro lugar, os caminhos relativos de assets de arquivos existentes são reescritos ou um aviso informa que a saída não os encontrará (`--allow-external-assets` os torna absolutos).
* **Renderização.** A fachada recusa imagens maiores que 16384 px de lado (`MAX_RENDER_SIDE`) e escalas fora de 0.05 a 4; as camadas C e wasm recusam saídas acima de 256 MiB antes de alocar. Outros limites da fachada: fps em (0, 1000], quadros de 1 a 200000, stage até 32768, 4096 ossos, magnitudes dentro de 1e7, `lod_levels` de 1 a 12, amostras de `fuzz` até 100000, cores de `trace` de 2 a 64 ([API.md](API.md)).
* **Nenhum panic na entrada.** Os testes de entrada hostil estão em `osatura-cli/tests/robust.rs` e nos testes das ondas; testes no estilo fuzz cobrem os parsers. Um panic alcançável a partir da entrada é tratado como vulnerabilidade.
* **Os corpos do studio** são limitados a 32 MB; as edições passam pela mesma validação dos arquivos.

## 5. Sandbox de expressões e scripts

* **As expressões** (vínculos de valores, constraints, drivers) são analisadas por um parser próprio: **código-fonte de até 4096 bytes, profundidade de aninhamento 64**, aritmética, comparação, as variáveis fixas `frame`, `time`, `fps`, `frames`, `pi`, `t`, `get("layer","property")` para drivers e uma tabela fixa de funções embutidas (`sin`, `abs`, `min`, ...); sem laços, sem E/S e sem funções definidas pelo usuário, de modo que a avaliação sempre termina. Resultados não finitos são rejeitados pela política de valores finitos.
* **Os scripts** (comando `script`, feature `scripting`, ativada por padrão) são executados em Rhai com estes limites: **500,000 operações, 32 níveis de chamada, profundidade de expressão (64 no nível superior, 32 em funções), strings de 10,000 caracteres, arrays de 10,000 itens, maps de 1,000 entradas, 5,000 requisições à API, código-fonte do script de 64 KB** e `no_module` (sem `import`, portanto sem acesso a arquivos por módulos). A API do host exposta aos scripts é um conjunto fixo de funções de edição de projeto. Compile com `--no-default-features` para remover por completo o motor de scripting (remove também a dependência `smartstring`).
* Um script continua sendo código que você escolheu executar sobre um projeto: ele não pode ler nem gravar arquivos por conta própria, mas pode tornar o projeto que edita muito grande dentro dos limites acima, e um laço infinito termina no limite de operações.

## 6. Arquivos, links simbólicos e dados temporários

* **Gravações atômicas.** As saídas são gravadas com `fsutil::write_atomic`: um arquivo temporário no mesmo diretório (nome `.<name>.tmp-<pid>-<random>`), `fsync`, `rename`. Uma queda deixa o arquivo antigo ou o novo. **Links simbólicos fornecidos como saída são recusados**, indicando `--follow-symlinks` (isso agora cobre toda saída endereçada pelo usuário: relatórios e goldens de `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, `face-*`, arquivos de pose, snapshots, `vcs`, o log de colaboração e os arquivos de vídeo gravados pelo ffmpeg, que são pré-verificados), de modo que um link plantado não consegue redirecionar uma gravação. Se o diretório não é gravável mas o arquivo é, o recurso alternativo é uma sobrescrita no lugar com `fsync`, que **não é atômica** (documentado).
* **Dados privados.** Os logs de colaboração e outros arquivos que apenas o dono deve ler são criados com modo 0600 (`write_atomic_private`). Os diretórios temporários são criados de forma exclusiva com modo 0700 (`private_temp_dir`), de modo que um nome previsível em um `/tmp` compartilhado não possa ser criado antecipadamente por outro usuário. Os valores aleatórios (tokens para novos participantes, nomes temporários) vêm de `/dev/urandom` (`fsutil::random_bytes`); o gerador determinístico com seed do motor nunca é usado para segredos.
* **Configurações da interface.** O `ui.json` do studio ignora chaves desconhecidas com um aviso e é copiado para `ui.json.bak` (depois `.bak.1`, ...) antes do primeiro salvamento.
* **Windows / não Unix.** Os modos de permissão acima valem em Unix. No Windows e no macOS a suíte de testes passa no CI (as verificações exclusivas de POSIX são ignoradas no Windows), mas as verificações de links simbólicos, permissões e `/dev/urandom` rodam apenas no Linux e no macOS, e ninguém revisou à mão o comportamento no Windows.
* **Binários não assinados e o aplicativo de janela.** Os executáveis de release não são assinados digitalmente; confira os downloads com `SHA256SUMS.txt`. O `osatura-studio` inicia o mesmo servidor local (somente `127.0.0.1`, porta livre aleatória) e o mostra na web view do sistema (WebView2 / WKWebView); sua web view é tão exposta quanto uma aba de navegador nesse endereço e não teve uma revisão de segurança separada.

## 7. Programas externos (ffmpeg)

O ffmpeg e o ffprobe são executados como processos separados **apenas** para exportação de vídeo, decodificação de mp3 / ogg, mixagem de trilha sonora e algumas ferramentas; nunca são incluídos ([LICENSING.md](LICENSING.md)). Cada invocação:

* passa os argumentos como um array (sem shell, sem concatenação de strings em uma linha de comando);
* usa `-protocol_whitelist file` (um uso interno de pipe acrescenta `pipe,fd`), de modo que um arquivo de mídia não consegue fazer o ffmpeg abrir `http://`, `concat:` ou outros protocolos de rede / especiais;
* passa os nomes de arquivo por `fsutil::file_arg`, que antepõe `./` aos nomes que começam com `-` ou contêm `:` ou `|` no primeiro componente do caminho, de modo que um nome de arquivo não possa ser lido como uma opção ou URL;
* para arquivos de som `.wav`, `.mp3` e `.ogg`, força o demuxer com `-f` (`soundtrack::forced_demuxer`), de modo que um arquivo que não é o que o nome diz seja recusado em vez de aberto como algum outro contêiner;
* usa `-loglevel error`.

O próprio ffmpeg analisa formatos de mídia complexos e já teve vulnerabilidades próprias: mantenha-o atualizado e não o alimente com mídia de fontes não confiáveis em uma máquina que guarde segredos. Os comandos com orçamento de tempo (`export-fit --max-seconds`) não conseguem interromper uma única chamada do ffmpeg em execução.

## 8. Bibliotecas (ABI C, WASM, fachada)

* A **fachada** valida todos os argumentos, retorna `ApiError` e converte panics em `ApiError::Internal` onde o alvo faz unwinding. `Animation` e `Player` são `Send + Sync` (verificado em tempo de compilação).
* A **ABI C** ([C_ABI.md](C_ABI.md)) verifica ponteiros nulos, comprimentos e UTF-8 e retorna códigos; o chamador ainda precisa passar ponteiros válidos e respeitar as regras de buffer. A string do último erro é por thread. A ABI foi exercitada sob AddressSanitizer e UndefinedBehaviorSanitizer com os exemplos documentados (sem problemas). Ela não está endurecida contra um chamador que viole os contratos documentados (double close, uso após close, comprimentos errados).
* O **módulo WASM** ([WASM.md](WASM.md)) compila no CI e passa no teste do wrapper sob o Node, e foi executado uma vez em Chromium headless (entrada inválida gera erros, nenhum vazamento em 60 ciclos de abrir/fechar, palcos grandes demais recusados), mas **não foi revisado como componente de navegador e não foi executado no Firefox nem no Safari**, portanto suas propriedades de segurança são conhecidas apenas pela leitura do código e por esse teste. Por construção, não tem acesso a sistema de arquivos nem à rede; um panic ali aborta a instância.
* Nenhuma das duas camadas isola o *projeto* em sandbox: a renderização de um projeto hostil é limitada pelos limites acima, mas um projeto ainda pode pedir muito trabalho dentro deles (muitas camadas, imagens grandes, timelines longas).

## 9. Lacunas conhecidas

Lista honesta; veja [LIMITATIONS.md](LIMITATIONS.md) para a lista mais ampla de limites funcionais.

1. **Sem TLS e sem contas** no studio. Desde a onda 7 o token pode ser por participante e com expiração (seção 3), o que ainda não é um sistema de contas: não há papéis, nem redefinição de senha, nem TLS próprio. Não se adicionou TLS nativo porque nenhum crate TLS de licença compatível consta em `Cargo.lock` (adicionar um traria uma grande dependência nova, e código criptográfico, a um projeto AGPL/MPL sem auditoria); a seção 12 mostra o caminho suportado, um proxy reverso que termine o TLS.
2. **O back-off é por endereço** e compartilhado atrás de um proxy; ele retarda, mas não impede, a adivinhação online. Use um token aleatório longo (o mínimo de 16 caracteres é um piso, não uma recomendação).
3. **O log de colaboração é encadeado, não assinado.**
4. **Não é uma base de código auditada.** Sem teste de penetração de terceiros nem campanha formal de fuzzing; as verificações são os testes e as ondas de revisão do próprio projeto.
5. **Dependências.** A CI executa `cargo deny` para advisories (vulnerabilidades, crates yanked, dependências diretas sem manutenção), licenças, crates banidos (nenhuma pilha de TLS ou de cliente HTTP) e origens. Os avisos de manutenção de dependências transitivas são revisados manualmente (veja `deny.toml`). A verificação só conhece os advisories publicados até agora; execute-a de novo antes de implantar.
6. **O diretório de assets é do processo inteiro.** O `merge` sobre vários diretórios de entrada usa o último diretório para os assets; apenas `write_json` reescreve caminhos de assets (os outros escritores não).
7. **Alternativa não atômica** quando o diretório de saída não é gravável.
8. **As verificações de caminho usam caminhos canônicos.** `assets::resolve` canoniza os arquivos existentes e recusa um arquivo cuja localização real esteja fora do diretório do projeto, de modo que um link simbólico dentro da pasta do projeto que aponte para fora dela é recusado (testado em `assets.rs`); links que permanecem dentro da pasta funcionam. Não se exclui uma condição de corrida entre a verificação e a leitura posterior (time-of-check to time-of-use) por alguém que possa modificar a pasta do projeto simultaneamente.
9. **O ffmpeg** está fora do nosso controle (seção 7).
10. **O WASM** não foi testado em um motor de navegador (ele só roda sob o Node no CI); o modelo de ameaças de navegador de uma página que o incorpore (CSP, limites de memória) não foi examinado.
11. **Negação de serviço por tamanho.** Os limites restringem requisições e arquivos individuais, não o trabalho total: um projeto válido, mas muito pesado, pode ocupar uma CPU por muito tempo (o limite de conexões acima restringe a concorrência, não o custo de uma requisição).

## 10. Reportando uma vulnerabilidade

* **Não abra uma issue pública** para uma vulnerabilidade. Envie um relatório privado ao mantenedor: **`alexio@trom.tf`** (o endereço do aviso de `--version` do programa e dos documentos de licença), ou use o relato privado de vulnerabilidades do GitHub, na aba *Security* do repositório, quando estiver ativado.
* Inclua: a versão (`osatura --version`) e a plataforma, o projeto ou a requisição que provoca o problema (um arquivo mínimo é o melhor), o que você esperava e o que aconteceu, e se deseja receber crédito.
* O que esperar *(intenção, não um contrato: este é um projeto de uma pessoa só)*: uma confirmação de recebimento em prazo razoável, uma correção desenvolvida em privado, um teste de regressão adicionado à suíte, uma nota em [CHANGELOG.md](CHANGELOG.md) e crédito, se você desejar. Por favor, dê tempo para uma correção antes de publicar detalhes.
* Versões suportadas: apenas o pré-lançamento mais recente (atualmente `v0.1.0-alpha.4`) e a árvore atual em `main` são suportados.
* Relatos sobre a exposição de rede do studio quando ele é deliberadamente colocado em um endereço público sem proxy reverso são, antes de tudo, problemas de documentação: veja a seção 11.

## 11. Checklist de endurecimento para quem implanta

Para quem executa `osatura studio` para mais de uma pessoa ou fora do localhost.

1. [ ] Prefira **não** expô-lo. Usuário único: mantenha o bind padrão (`127.0.0.1`) e use um túnel SSH (`ssh -L 8080:127.0.0.1:8080 host`).
2. [ ] Se exposto, **termine o TLS** em um proxy reverso (nginx, Caddy, ...) na frente dele; nunca exponha a porta HTTP pura à Internet.
3. [ ] Use um **token aleatório longo** (por exemplo, 32+ caracteres de `openssl rand -base64 24`), passe-o por um arquivo protegido ou pelo ambiente do serviço em vez de pelo histórico do shell, e troque-o quando pessoas saírem. Lembre-se de que ele dá acesso total de edição.
4. [ ] Inicie com `--bind 127.0.0.1` (ou a interface privada) mais o proxy; defina `--allow-host your.domain` e `--public-origin https://your.domain` para que as verificações de Host e Origin sejam satisfeitas para o proxy e para mais nada.
5. [ ] Prefira **tokens por participante** (`--collab-secret-file`, seção 3) com um `--ttl` curto a um único token compartilhado; mantenha o arquivo de segredo com modo 600 fora da pasta servida, e indique com `--collab-host` as pessoas que podem salvar, compactar e revogar.
6. [ ] Faça o proxy aplicar **limite de taxa** e, se possível, restringir por IP ou acrescentar uma segunda camada de autenticação (HTTP basic sobre TLS, VPN, proxy SSO), porque o back-off embutido enxerga apenas o endereço do proxy.
7. [ ] Execute como um **usuário sem privilégios** em um diretório dedicado que contenha apenas os projetos a servir; sem segredos, sem links simbólicos sensíveis. Use um sistema de arquivos somente leitura no restante, um contêiner ou o sandboxing do systemd (`ProtectSystem`, `PrivateTmp`, `NoNewPrivileges`).
8. [ ] Mantenha `--allow-external-assets` e `--follow-symlinks` **desativados**.
9. [ ] Defina `--source-url` para o local onde o código-fonte (possivelmente modificado) pode ser obtido (seção 13 da AGPL).
10. [ ] Persista com `--collab-log` em um diretório privado, faça backup e mantenha o modo 0600. Trate o log como sensível: ele contém todo o histórico de edição.
11. [ ] Limite os recursos no nível do SO (memória, tempo de CPU, arquivos abertos, cota de disco); os tetos embutidos limitam requisições individuais, não a carga total.
12. [ ] Mantenha atualizados o ffmpeg e a cadeia de ferramentas Rust usada no build; execute `cargo deny check` (ou `cargo audit`) sobre `Cargo.lock` do seu lançamento.
13. [ ] Não alimente o servidor com projetos de fontes não confiáveis a menos que ele rode no sandbox acima. Para implantações sem scripting, compile com `--no-default-features`.
14. [ ] Observe nos logs as respostas 421, 403, 429 e 503; picos indicam sondagem ou configuração incorreta.

## 12. Proxy reverso com TLS: nginx e Caddy

O studio fala HTTP puro. Para qualquer uso além da sua própria máquina, execute-o no loopback e deixe um proxy terminar o TLS. Estas receitas foram escritas a partir da documentação pública dos proxies e do comportamento do studio descrito acima; **não foram executadas no ambiente de autoria** (sem nginx nem Caddy, sem certificados, sem rede): teste-as antes de depender delas.

Inicie o studio no loopback e informe seu host público, para que as verificações de Host e Origin aceitem o proxy e mais nada:

```
osatura studio demo.json --port 8080 --bind 127.0.0.1 \
  --allow-host studio.example.org --public-origin https://studio.example.org \
  --collab-secret-file /etc/osatura/collab.secret --collab-log /var/lib/osatura/session.log
```

Com um `--public-origin` `https://`, o cookie de login recebe o atributo `Secure`. O fluxo de eventos (`/api/collab/events`) é Server-Sent Events: o proxy não deve fazer buffer dele e deve permitir leituras longas.

**nginx** (certificados da sua CA ou do `certbot`):

```
limit_req_zone $binary_remote_addr zone=osatura:10m rate=20r/s;

server {
    listen 80;
    server_name studio.example.org;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    server_name studio.example.org;
    ssl_certificate     /etc/letsencrypt/live/studio.example.org/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/studio.example.org/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    add_header Strict-Transport-Security "max-age=31536000" always;
    client_max_body_size 64m;

    location / {
        limit_req zone=osatura burst=60 nodelay;
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Proto https;
        proxy_set_header Connection "";
    }
    location /api/collab/events {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Proto https;
        proxy_set_header Connection "";
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 1h;
    }
}
```

**Caddy** (obtém e renova o certificado por conta própria; `Caddyfile`):

```
studio.example.org {
    encode zstd gzip
    header Strict-Transport-Security "max-age=31536000"
    request_body {
        max_size 64MB
    }
    reverse_proxy 127.0.0.1:8080 {
        flush_interval -1
        transport http {
            read_timeout 1h
        }
    }
}
```

`flush_interval -1` envia o fluxo de eventos sem buffer. O Caddy define `X-Forwarded-Proto` e repassa o `Host` original. Para um limite de taxa de requisições, use o plugin `caddy-ratelimit` ou coloque o `limit_req` do nginx na frente.

Verificações após a inicialização: `curl -I https://studio.example.org/` responde `401` (página de login) e nunca serve o studio sem um token; `curl -I http://127.0.0.1:8080/ -H 'Host: evil.example'` responde `421`; o cookie definido por `/api/login` traz `Secure; HttpOnly; SameSite=Strict`; o log da própria sessão do studio permanece privado (seção 6). Se o proxy rodar em outra máquina, faça o bind do studio na interface privada que o proxy alcança, proteja a porta com firewall e lembre-se de que o próprio studio ainda fala HTTP puro nesse enlace.
