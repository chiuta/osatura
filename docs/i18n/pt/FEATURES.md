# OSATURA — matriz de recursos + listas de verificação de referência e da concorrência

> Redação das versões: todos os crates estão em `0.1.0` (os primeiros builds são o pré-lançamento em rascunho `v0.1.0-alpha.3`); "v0.5 / v0.6 / v0.7" neste repositório são *níveis de recursos* (veja [CHANGELOG.md](CHANGELOG.md)), e a seção 9 de `ROADMAP.md` é a autoridade sobre o que está construído.

Estado honesto do que existe. **✅ pronto e testado · 🟡 parcial · ❌ não implementado.**

> Nota de escopo: o OSATURA é um motor implementado de forma independente (a partir de especificações e artigos públicos; as listas de recursos da concorrência foram consultadas para definir o escopo; nenhum código-fonte de terceiros foi copiado; veja CLEANROOM.md para a divulgação da pesquisa), não uma reimplementação de nenhum
> produto comercial, e **não** atinge a paridade com uma suíte comercial de 25 anos
> (GUI madura, ferramentas de desenho, entrada ao vivo, 3D, ecossistema). As primeiras tabelas são organizadas por
> capacidade; a **segunda metade deste arquivo** é uma lista de verificação, versão por versão, dos recursos
> publicamente documentados do produto de referência (fontes listadas no final), com
> o estado do OSATURA para cada um. As "inovações" são afirmadas apenas em relação à técnica genérica,
> e não como comprovadamente ausentes de qualquer outra ferramenta.

---

# ★ ESCADA DE CONSTRUÇÃO — do simples ao complexo, com o estado de direitos autorais / PI (nível de recursos v0.7, 20-22 out 2026)

Esta seção é o **único lugar para ver o que pode ser construído, em que ordem e se é seguro**.
Ela substitui as listas dispersas de "candidatos n.º" mais adiante; os detalhes e os IDs ficam em `ROADMAP.md`
(R-xxx / I-xx), o método clean-room em `CLEANROOM.md`, as licenças em `LICENSING.md`.

**Estado:** ✅ construído e testado · 🟡 parcialmente construído · ⬜ planejado (não iniciado)
**Estado de PI (após as alternativas da ROADMAP §8):**
🟢 seguro — padrão aberto, matemática de livro didático ou design próprio ·
🟠 seguro *se* você mantiver uma condição (indicada na última coluna) ·
🔴 não construir.
*Triagem de engenharia, não aconselhamento jurídico; um advogado deve confirmar cada 🟠 antes de um lançamento comercial.*

## Nível 1 — Fundamentos (de horas a dias cada; nenhuma exposição de PI)
| Recurso | Estado | PI | Base segura / condição |
|---|---|---|---|
| JSON de projeto, validação, `evaluate(frame)` pura | ✅ | 🟢 | esquema próprio |
| Caminhos Bézier, preenchimentos, traços, gradientes, tracejados, traços de largura variável | ✅ | 🟢 | geometria de livro didático |
| Ossos FK, limites, IK de 2 ossos/FABRIK, restrições | ✅ | 🟢 | cinemática de livro didático |
| Keyframes, easing, expressões, ruído | ✅ | 🟢 | design próprio |
| Editor de patch-ops com desfazer/refazer exato | ✅ | 🟢 | ponteiros JSON no estilo RFC 6902 |
| Exportação SVG, PNG, GIF, player HTML, Lottie | ✅ | 🟢 | especificações públicas |
| Log de proveniência, `stats` / `diff` semântico, regressão por quadros de referência (hash exato + tolerância), limites de recursos, teste de mutation-fuzz, controle de versão do formato de arquivo (R-001…R-005) | ✅ | 🟢 | design próprio |
| Vocabulário próprio padrão de papéis + etiquetador baseado em nomes (`tag`) (R-006) | ✅ | 🟢 | nomes próprios, palavras simples de anatomia |
| Exportação de JSON Schema (esquema R-003) | ⬜ | 🟢 | esquema próprio |

## Nível 2 — Noções básicas de autoria e saídas abertas (dias)
| Recurso | Estado | PI | Base segura / condição |
|---|---|---|---|
| APNG, sprite sheet PNG+JSON | ✅ | 🟢 | especificações PNG/APNG |
| WebM VP9 / AV1 com alfa (via o seu ffmpeg) | ✅ | 🟢 | codecs com especificação aberta; situação de patentes não avaliada |
| Exportação OCA (Open Cel Animation) | ✅ | 🟢 | especificação pública; importação ⬜ |
| Importação OpenRaster (.ora) | ✅ | 🟢 | especificação aberta, limites contra zip-bomb |
| MP4 / H.264 | ✅ | 🟠 | somente via o ffmpeg *do próprio usuário*; nunca incluir um encoder |
| Deslocar/escalar/inverter/copiar/colar keys, easing em lote, deslocamento de tempo por canal, marcadores e regiões, timecode SMPTE (R-010, R-011, R-044) | ✅ | 🟢 | edição genérica |
| Snapshots de revisão com poda e recuperação após falha (R-014); layer comps (R-015); eventos de timeline com payloads (R-045); reparentar preservando a pose de repouso (parte de R-013) | ✅ | 🟢 | edição genérica |
| Alinhar/distribuir, camada de referência, tags shy/solo/lock (R-012, R-013, R-016) | ✅ | 🟢 | edição genérica |
| Onion skin, trilhas de movimento (sobreposições nunca exportadas) (R-020) | ✅ | 🟢 | design próprio |
| Cadeia de pós-efeitos: blur, sombra projetada, brilho, contorno, posterização, pixelização, níveis/gama, fade (parte de R-022) | ✅ | 🟢 | processamento de imagem de livro didático |
| Trim paths, filtros por camada, máscaras, profundidade de campo (R-021, R-024, R-025, R-034) | ✅ (trim paths pré-calculados; limites dos efeitos de camada: veja ROADMAP §9) | 🟢 | matemática de imagem padrão |
| Importação SVG (caminhos, formas, grupos, transformações, preenchimentos sólidos; gradientes/texto/clip reportados como avisos) (R-032) | ✅ | 🟢 | gramática W3C SVG 1.1 |
| Importação BVH (um eixo de rotação → ângulos de osso, redução limitada de keys) (R-036) | ✅ | 🟢 | formato de texto aberto |
| Exportação do contêiner dotLottie (R-03A) | ✅ | 🟢 | especificação pública do contêiner (zip + manifesto) |
| Exportação de SVG animado, importação de sequência de imagens, saída OpenEXR/16 bits, exportação glTF (R-032, R-037…R-039) | 🟡 (glTF: formas do quadro 0 mais ossos e pesos de morph, veja ROADMAP R-039) | 🟢 | formatos públicos |
| Importação de pistas de boca do Papagayo (Moho-switch) e do Rhubarb (R-03B) | ✅ | 🟢 | lê apenas a saída em texto simples deles |

## Nível 3 — Profundidade de rig e animação (semanas)
| Recurso | Estado | PI | Base segura / condição |
|---|---|---|---|
| Molas (pré-calculadas), lattices (FFD), deformador de spine, camadas switch, partículas, líquido, booleanas | ✅ | 🟢 | livro didático |
| Lip-sync de áudio → formas da boca, gerador de ciclo de caminhada, espelhar, retime, clipe, crossfade | ✅ | 🟢 | design próprio |
| **Pin warp** (substitui o warp estilo cage / ARAP) | ✅ | 🟢 | RBF de Wendland 1995 |
| **Pesos de skin automáticos por difusão de calor** | ✅ | 🟢 | equação de Laplace |
| **Gravação de performance** com redução limitada de keys | ✅ | 🟢 | Douglas–Peucker |
| **Mapeamento de entrada** (sinais CSV/JSON → parâmetros) | ✅ | 🟢 | nenhum rastreador incluído |
| IK suave/com alongamento (R-051), drivers de pose: slider + joystick 2D (R-050), troca de vista guiada por ângulo (R-056), auto-rig por tags (R-058) | ✅ | 🟢 | rigging genérico; matemática de livro didático, design próprio |
| Restrições de caminho, IK com travamento de pé, keys de troca de pai, skins, ímãs (R-040…R-049, R-052…R-055, R-057) | 🟡 (construído: R-040…R-043, R-044, R-047, R-048, R-052…R-055, R-057; veja a seção 9 do ROADMAP para as partes marcadas como NÃO concluídas) | 🟢 | rigging genérico |
| Geradores de idle (piscar, respirar, balançar) e cadeias de follow-through (R-047, R-048) | ✅ | 🟢 | com seed, determinístico |
| Vento, gravidade, colisões, tiras de cabelo/tecido (R-070…R-072) | 🟡 (R-070, R-071 concluídos; as tiras de R-072 são apenas ossos, o tecido é uma folha plana) | 🟢 | livros didáticos de massa-mola / PBD |

## Nível 4 — Ferramentas inteligentes (semanas a meses)
| Recurso | Estado | PI | Base segura / condição |
|---|---|---|---|
| **Vetorização de imagem → vetor** (substitui o auto-trace por ML) | ✅ | 🟢 | marching squares + RDP |
| **Máquina de estados por log de eventos** + bake para canais (substitui clones de máquinas de estados de runtime) | ✅ | 🟢 | semântica própria — manter os nossos nomes |
| **Retargeting por alcance do efetuador** | ✅ | 🟢 | método próprio |
| **Solver de correspondência de pose** (estratégia evolutiva, ciente dos limites das juntas) | ✅ | 🟢 | Beyer & Schwefel |
| **Verificação de flash / flash vermelho WCAG 2.3.1 + relatório de movimento de câmera + gêmeo de movimento reduzido** | ✅ | 🟠 | somente definições públicas; diga "auxílio de engenharia", nunca "certificado" |
| **Linter de rig**, **contratos de movimento**, **solver de loop contínuo**, **coach de princípios** (I-02, I-04, I-05, I-14) | ✅ | 🟢 | ideias próprias |
| **Ajuste de mola por desenho do overshoot** (I-08) — ajusta frequência/amortecimento com o integrador do próprio motor | ✅ | 🟢 | ideia própria |
| **Explique este pixel** para ossos e camadas (I-07) | ✅ | 🟢 | ideia própria |
| Antes | depois | imagem de revisão de diferenças (primeiro passo de I-06; a métrica perceptual é `ci diff`, SSIM e CIEDE2000) | ✅ | 🟢 | ideia própria |
| Iluminação com normal map, parallax, profundidade de campo (R-064, R-065, R-024) | ✅ | 🟢 | livro didático |
| Lip-sync alinhado à transcrição (R-081) | ✅ (alinhador próprio, `speech.rs`) | 🟠 | use um alinhador com licença permissiva; verifique a licença do modelo |
| Rastreamento facial/corporal por webcam alimentando o mapeador de entrada (R-082) | 🟡 (detector próprio treinado apenas com rostos sintéticos; webcams e fotografias reais não testadas) | 🟠 | o usuário traz um rastreador permissivo; não incluir pesos de modelo, a menos que a licença permita |
| Retargeting completo de mocap BVH com correspondência automática por hierarquia (R-083) | ✅ | 🟢 | baseia-se na importação BVH |
| Importação PSD (R-035) | ⬜ | 🟠 | somente leitura, especificação publicada, sem marcas da Adobe; ou simplesmente use ORA |

## Nível 5 — Plataforma (meses)
| Recurso | Estado | PI | Base segura / condição |
|---|---|---|---|
| Studio no navegador com desfazer/refazer | 🟡 | 🟢 | UI própria; sem copiar layouts ou nomes de fornecedores |
| Runtime WASM, API Rust estável + ABI C, scripting em sandbox (R-090…R-093) | 🟡 (R-091, R-092, R-093 concluídos; R-090: o módulo wasm compila e passa no teste do wrapper sob o Node no CI, e foi executado apenas em Chromium headless) | 🟢 | API própria |
| **Servidor de agente / MCP** sobre patch ops: inspecionar, editar (atômico, desfazível), lint, segurança, renderizar, salvar somente no próprio arquivo (R-094) | ✅ | 🟢 | protocolo JSON-RPC aberto |
| GUI completa: gizmos, timeline, editor de curvas (R-100…R-103) | 🟡 (R-101, R-102 concluídos; as páginas de R-100 e R-103 existem, mas o autor nunca as executou em um navegador real) | 🟢 | widgets genéricos; visual próprio |
| Controle de versão de animação com merge semântico, diff perceptual de CI, explique-este-pixel (I-01, I-06, I-07) | ✅ (I-01 `osatura vcs`, I-06 `osatura ci`, I-07 `explain`) | 🟢 | ideias próprias |
| Fuzzing de poses com mapas de calor de estresse, exportação com tamanho-alvo, nível de detalhe determinístico com limite de erro declarado (I-03, I-10, I-11) | 🟡 (os três construídos; o limite de I-11 não cobre efeitos, caminhos com skin, valores guiados por IK nem poses de máquina de estados) | 🟢 | ideias próprias, matemática de livro didático |

## Nível 6 — Fronteira (nível de pesquisa)
| Recurso | Estado | PI | Base segura / condição |
|---|---|---|---|
| Renderizador GPU com testes de paridade com a CPU (R-110) | 🟡 (crate `osatura-gpu` construído e testado; execução em hardware limitada, veja ROADMAP) | 🟢 | wgpu (permissiva) |
| Edição multiusuário em tempo real sobre o op-log (R-111) | 🟡 (concluída, com log de sessão persistido opcional `--collab-log`; participantes e histórico de desfazer não são persistidos; HTTP simples, token compartilhado) | 🟢 | literatura de CRDT |
| Runtime de data-binding / layout (R-091, R-095) | ✅ (design próprio; triagem de patentes não realizada, docs/PATENT_NOTES.md) | 🟠 | fazer a triagem de patentes antes de distribuir |
| Ferramentas assistidas por ML (R-112) | 🟡 | 🟠 | somente métodos clássicos, sem pesos de modelo: `autotrace` (k-means, ajuste de Bézier de Schneider, modo de linha Zhang-Suen) e sugestões aprendidas das suas poses (PCA, kernels, pequeno MLP opcional em formato de pesos próprio); sem deep learning |

## Nunca construir (🔴)
- Ler ou gravar arquivos de projeto nativos do **Live2D, Spine, Moho, Harmony, Rive, Cartoon Animator** (proprietários; exposição a EULA/engenharia reversa) — use Lottie, OCA, ORA, APNG, SVG no lugar.
- **Solvers rápidos de deformação de arte no estilo ARAP**, como descritos nas patentes da Adobe — pin warp, FFD e skinning cobrem o caso de uso.
- Copiar de qualquer fornecedor o **layout de UI, ícones, nomes, conteúdo de exemplo ou números mágicos/terminologia de formatos de arquivo**.
- Imitar a ferramenta de fotossensibilidade **Harding** ou alegar conformidade com normas de transmissão.
- Incluir **encoders H.264/H.265/AAC**.

## Condições que se aplicam a toda linha 🟢
Log de proveniência mantido (`CLEANROOM.md`; implementação independente, não um clean-room estrito); somente dependências permissivas (regenere com `scripts/licence-audit.sh`; atualmente 363 crates de terceiros em todas as plataformas, todos da classe MIT / Apache-2.0 / BSD / Zlib / Unlicense, exceto seis crates MPL-2.0 (smartstring, via o recurso `scripting`, ativado por padrão, e cinco via o aplicativo de janela `osatura-gui`); veja docs/THIRD_PARTY.md); nenhuma arte ou áudio de terceiros distribuído sem o respectivo arquivo de licença; o nome **OSATURA** liberado em uma busca de marca registrada antes do lançamento.


# Tabelas detalhadas de capacidades
## Desenho e formas
| Recurso | Estado | Observações |
|---|---|---|
| Caminhos Bézier cúbicos, abertos/fechados, alças | ✅ | |
| Preenchimento: sólido, gradiente linear e radial | ✅ | os gradientes seguem a transformação da camada |
| Traço: largura, terminações, junções, tracejados | ✅ | |
| Traço de largura variável (perfil de largura) | ✅ | contorno poligonal; caminhos fechados → dois anéis |
| Suavização Catmull-Rom, auxiliares de retângulo/círculo/elipse/polígono | ✅ | |
| Booleanas de polígono (união/interseção/diferença/xor) | ✅ | camada `boolean`; buracos conectados em um único polígono; o resultado é poligonal (achatado) |
| Camadas metaball / líquido | ✅ | marching squares, os ossos podem guiar as bolhas |
| Camadas de imagem (PNG), embutidas ou em arquivo | ✅ | |
| Pincéis bitmap/vetoriais, preenchimentos de textura, texto/fontes | ❌ | |
| Desenho à mão livre, ferramentas de edição de pontos, vetorização de imagem | ❌ | sem GUI de desenho |

## Rigging
| Recurso | Estado | Observações |
|---|---|---|
| Hierarquia de ossos, FK, pose de repouso | ✅ | |
| Limites de ângulo / escala | ✅ | |
| IK: analítico de 2 ossos (dobra ±) | ✅ | |
| IK: cadeias de N ossos (FABRIK) | ✅ | |
| Mistura FK↔IK (animável) | ✅ | |
| Restrições: aim, copy-rotation, pin (peso animável) | ✅ | |
| Skinning de mistura linear de âncoras + alças, auto-skin | ✅ | |
| Camadas presas a ossos, hierarquia de pais / grupos de camadas | ✅ | |
| Movimento secundário: molas amortecidas pré-calculadas | ✅ | determinístico, acesso aleatório (pré-calculado ao carregar) |
| Canais guiados por pose ("driven keys") | 🟡 | uma curva mapeia qualquer propriedade para outra; sem UI/ações dedicadas de smart bone |
| Física de corpo rígido, tecido, corpo mole | ❌ | |

## Deformação
| Recurso | Estado | Observações |
|---|---|---|
| Deslocamentos por ponto (animáveis) | ✅ | endereçados como `layer::path::index` |
| Morph targets / blend shapes (pesos aditivos) | ✅ | |
| Lattice de forma livre (FFD de Bernstein) | ✅ | pontos de controle animáveis |
| Deformador de dobra de spine / curva | ✅ | raio suave opcional |
| Oscilação "boil" (tremor desenhado à mão) | ✅ | ruído coerente, determinístico |
| Warp de malha de imagem com pesos de osso | ✅ | malha em grade; sem editor de malha |

## Animação
| Recurso | Estado | Observações |
|---|---|---|
| Keyframes + easing (linear/step/ease/cubic-bezier) | ✅ | |
| Extrapolação: hold/linear/loop/ping-pong/loop-offset | ✅ | |
| Canais de expressão | ✅ | pequena linguagem de expressões segura; pode ler outros canais |
| Canais de ruído, mistura add/multiply/replace, pesos | ✅ | |
| Camadas switch + exposição (quadro a quadro, trocas de boca) | ✅ | |
| Retime/deslocamento de clipe, retarget, espelhar E↔D, bake de crossfade | ✅ | funções + CLI `mirror`; sem UI de NLA |
| Ciclo de caminhada procedural | ✅ | gerador senoidal simples, não é uma ferramenta profissional de caminhada |
| Lip-sync de áudio a partir de WAV | 🟡 | heurística de volume/brilho → 5 formas de boca, **não** reconhecimento de fonemas |
| Emissor de partículas determinístico | ✅ | forma fechada: qualquer quadro é avaliado diretamente |
| Onion skin, trilhas de movimento | ✅ | CLI `onion` e `trail`, sobreposições do studio; nunca exportadas |
| Reprodução de áudio / scrubbing de forma de onda | 🟡 | comando `waveform` e reprodução de áudio no studio; nenhuma alegação de UI de scrubbing de forma de onda |

## Câmera e composição
| Recurso | Estado | Observações |
|---|---|---|
| Câmera 2D com pan/zoom/rotação (animável) | ✅ | |
| Parallax multiplano por camada de nível superior | ✅ | |
| Opacidade, modos de mesclagem, máscaras alfa (+inverter) | ✅ | |
| Camadas / câmera 3D reais, estéreo | ❌ | |

## Saída e interoperabilidade
| Recurso | Estado | Observações |
|---|---|---|
| Quadros SVG | ✅ | |
| Quadros PNG (raster tiny-skia) | ✅ | |
| GIF animado | ✅ | |
| MP4 | ✅ | requer `ffmpeg` no PATH |
| Player HTML autônomo | ✅ | todos os quadros embutidos, scrubber |
| Lottie JSON | 🟡 | geometria pré-calculada por quadro; gradientes → primeira cor; sem máscaras/modos de mesclagem/imagens; **não validado em um player Lottie real** |
| Importação de arquivos de outras ferramentas, PSD, FBX etc. | ❌ | (deliberadamente: sem formatos proprietários) |
| API de scripting / plugins | 🟡 | `script` Rhai em sandbox por meio de requisições validadas, mais expressões; sem API de plugins |

## Edição
| Recurso | Estado | Observações |
|---|---|---|
| Modelo de edição por JSON-patch com desfazer/refazer | ✅ | toda edição é validada; edições inválidas são revertidas |
| Auxiliares para definir/excluir keyframes | ✅ | |
| Studio web (navegador) | 🟡 | pré-visualização, scrubber, keying do ângulo por osso, editor JSON, desfazer/refazer, salvar, exportações. **Sem desenho, sem edição de pontos/ossos no canvas, sem editor de curvas.** |

## Coisas que são "melhorias" modestas sobre a linha de base de livro didático
(sem alegar que estejam ausentes em outros lugares)
- Toda edição de projeto é um patch JSON validado e invertível → desfazer nunca pode corromper um projeto.
- Toda simulação (molas, partículas) é determinística com acesso aleatório: faça scrub ou renderize qualquer quadro em qualquer ordem e obtenha pixels idênticos.
- Qualquer propriedade — inclusive pontos de controle de deformadores e pesos de morph — é endereçável por um único caminho `layer::sub::id`, de modo que expressões e curvas guiadas podem ligar qualquer coisa a qualquer coisa.
- A validação informa *todos* os problemas de uma só vez, com os nomes dos itens envolvidos.
- Um motor → SVG, raster, GIF, MP4, HTML, Lottie; o mesmo `evaluate(frame)` alimenta todos.

## Testes
`cargo test` — 142 testes unitários/de integração (matemática, tracks, expressões, solvers de rig, deformadores,
pixels de renderização, exportadores, desfazer/refazer do editor, áudio, partículas, booleanas, metaballs).

---

# Lista de verificação em relação ao produto de referência, versão por versão

Referência: Moho (antes Anime Studio), 1999 → 14.5 "SHOWTIME" (12 out 2026). As listas de recursos vêm
de páginas de lançamento do fornecedor e da imprensa especializada (veja Fontes). Lacunas no registro: Anime Studio 6–8,
12.5 e 14.1 não tinham uma lista pública detalhada que eu pudesse encontrar, por isso são cobertos apenas pelo
histórico geral. O estado é o do OSATURA: ✅ existe equivalente · 🟡 parcial / abordagem diferente · ❌ ausente.

## Primeiras versões (1999 – Anime Studio 8, apenas histórico geral)
| Recurso | OSATURA |
|---|---|
| Partículas | ✅ camada de emissor determinística |
| Máscaras de camada | ✅ máscaras alfa + inverter |
| Onion skins | ✅ (CLI `onion`, sobreposição no studio) |
| Restrições de osso | ✅ aim / copy-rotation / pin, limites de ângulo e escala |
| Scripting em Lua | 🟡 (Rhai, não Lua; canais de expressão também) |
| Lip-sync | 🟡 heurística de amplitude a partir de WAV |
| Física | 🟡 molas + gravidade de partículas; sem corpos rígidos |
| Criação / importação 3D | ❌ |
| Importação do Photoshop | ❌ |
| Suporte a Wacom | ❌ (sem GUI de desenho) |
| Smart bones, IK | 🟡 canais guiados / ✅ 2 ossos + FABRIK |

## Anime Studio 9 (2013)
| Recurso | OSATURA |
|---|---|
| 64 bits, aceleração por GPU | 🟡 Rust nativo de 64 bits; apenas rasterizador por CPU (sem GPU) |
| Smart Bones (dials de controle de dobra de junta) | 🟡 canais guiados mapeiam qualquer valor para qualquer outro; sem UI/ações de dial |
| Gráficos de movimento editáveis | ❌ (as curvas são dados com easing cubic-bezier; sem editor de curvas) |
| Timeline reformulada | ❌ (o studio tem apenas um scrubber + marcas de keys) |
| Multitoque Wacom | ❌ |

## Anime Studio 10 (2014)
| Recurso | OSATURA |
|---|---|
| Restrições de ângulo independentes (globais) | 🟡 copy-rotation para um osso de controle aproxima isso |
| Squash & stretch | 🟡 a escala do osso é animável; sem squash/stretch automático |
| Dials / configuração de Smart Bone | 🟡 veja acima |
| Interpolação bounce / elastic / stagger | 🟡 bounce, spring (tipo elastic), back, cubic-bezier ✅; stagger ❌ |
| Balde de tinta, pincel de mancha, borracha, redução de pontos | ❌ |
| Personagens pré-carregados / conteúdo de biblioteca | ❌ |
| Importação de objetos 3D | ❌ |

## Anime Studio 11 (2015)
| Recurso | OSATURA |
|---|---|
| Animação quadro a quadro | ✅ camadas switch + canais de exposição |
| Referenciamento de camadas (duplicatas vinculadas) | ❌ |
| Ordenação animada de formas | ❌ (a ordem de desenho é fixa por camada; camadas switch podem trocar) |
| Curvas de movimento editáveis | 🟡 dados de curva sim, sem UI de curvas |
| Importação PSD (incl. atualização ao vivo) | ❌ |
| Exportação FBX / Unity | ❌ |
| Importação/gravação de áudio + lip-sync automático | 🟡 importação de WAV + lip-sync heurístico; sem gravação |
| Mesclar traços, ferramenta Color Points | ❌ |

## Moho 12 (2016) e 12.x
| Recurso | OSATURA |
|---|---|
| Alças Bézier | ✅ |
| Ferramentas de mão livre aprimoradas | ❌ |
| Smart Warp (malhas personalizadas) | 🟡 malha de grade de imagem com pesos de osso; deformadores lattice / spine para vetores; sem editor de malha |
| Motion blur | ❌ |
| Ossos pin | 🟡 restrição pin, não ossos pin de ponto único |
| Importar/exportar ações entre rigs semelhantes | 🟡 funções `clip`, `retarget`, `mirror` em conjuntos de canais; sem UI de biblioteca de ações |
| Animar várias camadas ao mesmo tempo | ✅ qualquer número de canais por timeline |
| UI redesenhada, ancoragem de ossos, congelamento automático de posição | ❌ |

## Moho 13 (2019)
| Recurso | OSATURA |
|---|---|
| Quadro a quadro bitmap + onion skin | 🟡 camadas de imagem PNG dentro de camadas switch; sem onion skin, sem desenho bitmap |
| Ferramentas de desenho bitmap / pincéis personalizados | ❌ |
| Ferramentas de storyboard e animatic | ❌ |
| Janela de ações | ❌ (apenas clipes no nível dos dados) |
| Luzes 3D, materiais, toon shading | ❌ |
| Mais predefinições de saída | 🟡 formatos de exportação da CLI, sem gerenciador de predefinições |
| Personagens incluídos | ❌ |

## Moho 13.5 (2021) e 13.5.x
| Recurso | OSATURA |
|---|---|
| Vitruvian Bones (troca desenhos *e* conjuntos de ossos) | 🟡 camadas switch trocam desenhos; sem troca de conjuntos de ossos |
| Malhas quad (perspectiva) | 🟡 malha de grade + lattice dão warps de forma livre; sem quad projetivo |
| Dinâmica de vento (intensidade, direção, turbulência) | ❌ (molas não têm vento; canais de ruído podem simulá-lo) |
| Keyframing relativo | ❌ |
| Variantes de visemas de lip-sync escolhidas aleatoriamente | ❌ (o mapeamento pode mesclar formas, sem variantes aleatórias) |
| Exportar áudio como WAV | ❌ |
| Copiar/colar de deep-keyframes, ajuste de pivô, marcadores de timeline | ❌ (sem GUI) |
| Licenças flutuantes | n/a |

## Moho 14 (2023)
| Recurso | OSATURA |
|---|---|
| Novo motor gráfico, efeitos em tempo real (blur, sombra, ruído) | ❌ sem filtros de blur/sombra/ruído; modos de mesclagem ✅ |
| Liquid Shapes (mesclar/subtrair/interseccionar/misturar) | 🟡 líquido metaball ✅ + booleanas de polígono ✅; não é o mesmo modelo de formas |
| Curvers / Compressible Curvers | 🟡 o deformador de dobra de spine é o análogo mais próximo |
| Importação SVG aprimorada | ❌ (apenas **exportação** SVG) |
| Melhorias de mão livre, novos estilos, mesclagem de formas | ❌ / ❌ / 🟡 (booleanas) |
| Multi-Stroke Fill, Connect & Create | ❌ |
| Smart Line Boil, opções de pincel | 🟡 oscilação "boil" com taxa ajustável ✅; opções de pincel ❌ |
| Cores de pontos, Add Point aprimorado | ❌ |
| Dynamic Bones (+ gravidade) | 🟡 ossos de mola amortecida ✅; sem termo de gravidade |
| Restrições com atraso | 🟡 as molas dão atraso em relação a um alvo; sem restrição de atraso explícita |
| Novas opções de timeline | ❌ |
| Melhorias no quadro a quadro | 🟡 |
| Atualização da interface de scripting (Lua 5.4) | ❌ |

## Moho 14.1 – 14.2 (2023-11, 2024-06)
| Recurso | OSATURA |
|---|---|
| (14.1: nenhum detalhe público encontrado) | — |
| Deslocamento animado do perfil de curva | ❌ (os perfis de largura são estáticos) |
| Smart boil para ruído vetorial | 🟡 a oscilação avança a uma taxa definida; sem modo "somente durante o movimento" |
| Ferramenta Transform Bone reformulada | ❌ (sem ferramentas de canvas) |
| Exibição de pixelização | ❌ |
| Modo de mesclagem Colour Burn | ✅ |
| Menor espaçamento mínimo de pincel, camadas recortadas de PSD | ❌ |
| Numeração da timeline baseada na taxa de quadros | ❌ |
| Opção de exibição "Desativar camadas mascaradas" | ❌ |
| Melhor lip-sync em baixas taxas de quadros | 🟡 |
| Nomes multilíngues para ossos/camadas/marcadores | ✅ (nomes em UTF-8) |
| Antialiasing da máscara Exclude-Strokes, onion skins coloridos em imagens | ❌ |
| Smart Bones controlando a dinâmica de ossos | ❌ (as configurações de mola não são animáveis) |
| Scripts Lua na inicialização; flag de console no Windows | ❌ / n/a |
| Exportação ProRes com alfa | ❌ (somente GIF/PNG/MP4) |

## Moho 14.3 (2024-12)
| Recurso | OSATURA |
|---|---|
| Opacidade de pincel por ponto (guiada por pressão) | ❌ |
| Variação de cor pela pressão da caneta | ❌ |
| Controles de ferramenta de mão livre/largura de linha | ❌ |

## Moho 14.4 "SIDE QUEST" (2025-11)
| Recurso | OSATURA |
|---|---|
| Exportação para Unreal / Unity / Godot | ❌ |
| Exportação glTF para Blender / ferramentas 3D | 🟡 (`.glb`: formas do quadro 0, ossos, pesos de morph) |
| Smart Bones de 360° | ❌ |
| Create Style from Shape | ❌ |
| Opções de partículas (replicar ao longo da curva, atraso, emissão ordenada, distribuição) | 🟡 atraso inicial ✅, emissão ordenada determinística ✅; replicar ao longo da curva ❌ |
| Layer Selector, queda do Magnet, importação PSD, suavização de mão livre, adições ao Lua | ❌ |
| Criação de ciclos mais fácil | ✅ extrapolação em loop + gerador de ciclo de caminhada |

## Moho 14.5 "SHOWTIME" (2026-10-12)
| Recurso | OSATURA |
|---|---|
| Live Playback (webcam / gamepad / microfone, webcam virtual, OBS) | ❌ |
| Rastreamento facial → ossos / Smart Bones / malha | 🟡 (detector treinado com dados sintéticos, `face-bake`; não validado em rostos reais) |
| Gravação ao vivo na timeline, controle de vários personagens | ❌ |
| Rastreamento de vídeo (sequências de imagens) | ❌ |
| Algoritmos de mão livre, linhas retas, aparar caudas | ❌ |
| Balde de tinta refeito (Vector / Trace / Recolor) | ❌ |
| Movimento reverso do Magnet, seleção de pontos por forma | ❌ |
| Categorias de pincel personalizadas | ❌ |
| Cor/opacidade do onion skin, Shift & Trace, onion skins magnéticos | ❌ |
| Filtros / seleção múltipla na janela de camadas | ❌ |
| Desempenho: desfazer/refazer mais rápidos, arquivos mais leves | 🟡 desfazer é um inverso barato de JSON-patch; os arquivos são JSON simples |
| Exportação MP4/H.264 | ✅ via ffmpeg |
| Perfis de cor PNG, importação/exportação EXR | ❌ |
| Exportador em lote, flags de CLI `-exr` / `-pngprofile` | 🟡 CLI `frames` / `export`, sem fila de lote |
| Taxa de quadros com 3 casas decimais (p. ex. 23,976) | ✅ |
| A exportação SVG trata traços de largura variável e formas líquidas | ✅ (os traços viram contornos, os líquidos viram caminhos) |
| Liquid hide-edge, formas líquidas somente de traço, exposição de traço | ❌ |
| Linux via Wine/Proton | ✅ Rust nativo (Linux/macOS/Windows); arquivos pré-compilados para Windows x86-64, macOS arm64 e Linux x86-64 no pré-lançamento em rascunho; testado pelo CI nos três, por uma pessoa apenas no Linux |

## Resumo (somente o produto de referência; a concorrência vem a seguir)
Dos recursos documentados do produto de referência, o OSATURA cobre o núcleo de **rigging, deformação,
animação procedural, matemática de composição e exportação** (aproximadamente: esqueleto/IK/restrições,
deformação por malha e lattice, keyframes e expressões, máscaras/mesclagens/gradientes, partículas,
metaballs/booleanas, GIF/MP4/HTML/Lottie), mas **não tem ferramentas de desenho, entrada ao vivo, 3D
nem E/S de PSD/FBX** (na época da primeira pesquisa; desde então foram adicionados exportação glTF, saída EXR, scripting Rhai, um studio com gráfico/timeline, onion skin e efeitos de blur, veja `ROADMAP.md` seção 9).
Esses são os blocos grandes, em sua maioria ligados à GUI, do produto e constituem a lacuna honesta.

## Fontes
- Anúncio do Moho 14.5 SHOWTIME, moho.lostmarble.com (blog, 12 out 2026)
- Página de recursos do Moho Pro, mohoanimation.com/pages/features
- Appcast do Moho 14.4 SIDE QUEST, lostmarble.com/appcasts/Moho14.4Update.html
- Moho 14.3: Digital Production, 27 dez 2024
- Post de lançamento do Moho 14.2, moho.lostmarble.com; lista de tutoriais "Moho 14: What is new?"
- Novos recursos do Moho Pro 13.5, moho.lostmarble.com; Lester Banks (abr 2021); notas do 13.5.3
- Moho 13: Lester Banks (jun 2019)
- Moho Pro 12: 80.lv; Digital Production (nov 2016)
- Anime Studio 11: Architosh (mai 2015); 10: Architosh (mar 2014); 9: Renderosity
- Datas das versões: mohoscripting.com/moho_versions; histórico geral: Wikipedia "Moho (software)"

---

# Adendo: histórico do Moho / Anime Studio, versões 1 – 11 (preenchido)

Fonte: o PDF de histórico de versões hospedado pelo fornecedor (lostmarble.com, "Anime Studio history", 2015) mais as
páginas de imprensa do Anime Studio 7 e 8. Ele substitui as linhas de "apenas histórico geral" próximas ao topo da
lista de verificação de referência. Não encontrado nem aqui: detalhes de 8.x, 10.1.x, 11.1, 12.5, 14.1.

| Versão | Recursos documentados | OSATURA |
|---|---|---|
| Moho 1 (1999) | BeOS, port para Windows, saída Flash | ❌ Flash/SWF (obsoleto) |
| 2.x | E/S QuickTime, renderiza até 4096², sombreamento de camada, camadas switch, amostras de cor em estilos, vinculação flexível de ossos, importação aprimorada do Illustrator | ✅ camadas switch, 🟡 vinculação por pesos, ❌ importação do Illustrator, ✅ grandes escalas de raster (≤16×) |
| 3.x | Partículas, image wrapping, máscaras de camada, onion skins, câmera virtual com profundidade, image warping, camadas switch aprimoradas, suporte a mesa digitalizadora, timecode SMPTE, ocultar arte fora da tela, máscaras reversíveis, randomizador de espessura de linha, zonas seguras, cisalhamento de camada | ✅ partículas, máscaras (+inverter), câmera, warp de malha de imagem, skew de camada; ❌ onion skin, mesa digitalizadora, timecode, zonas seguras, randomizador de espessura |
| 4.x | Curvas de animação gráficas, restrições de osso, renderização em lote, áudio, parâmetros animáveis, ciclos de animação, escala de keyframes, projetos redimensionáveis, profundidade de partículas, multidões aleatórias, vinculação flexível automática, versão para Linux | ✅ restrições, ciclos (extrapolação em loop), parâmetros animáveis, projetos redimensionáveis, retime de `clip` ≈ escala de keyframes, Linux; 🟡 multidões (partículas), 🟡 áudio (somente lip-sync); ❌ UI de editor de curvas |
| 5.x | Nova UI, exportação SWF, camadas 3D e de notas, ações, estilos, scripting em Lua | ❌ camadas 3D / de notas / Lua; 🟡 ações (funções de clip); ✅ os estilos são tinta por caminho |
| Anime Studio 6 | Rastreamento de movimento, lip-sync integrado, pincel de dispersão, Lua 5.1, sombreamento automático, **morph targets**, sequenciador, vídeo HD | ✅ morph targets; 🟡 lip-sync; ❌ rastreamento de movimento, pincel de dispersão, sombreamento automático, sequenciador |
| 6.1 / 6.2 | Upload para o Facebook, importação de GIF transparente | ❌ / 🟡 exportação GIF ✅, importação ❌ |
| Anime Studio 7 | Física simulada (gravidade, densidade, atrito, elasticidade), design de formas 3D, gravação de áudio, biblioteca de conteúdo, importação de cenas do Poser, **ferramenta follow path**, gather media | 🟡 molas/gravidade de partículas; ❌ física rígida, 3D, gravação, biblioteca, Poser, follow-path |
| Anime Studio 8 | Vetorização automática de imagem, Character Wizard, modo Iniciante, conexão de mídia em tempo real, estilos de renderização globais, rigging avançado de ossos | ❌ tudo (sem vetorização, wizard, link ao vivo, estilos globais) |
| Anime Studio 9 / 9.5 | Gráficos de movimento editáveis, Smart Bones, **layer comps**, onion skins aprimorados, 64 bits + GPU, keyframing aprimorado, timeline colorida; 9.5: melhorias nos Smart Bones, depth shifting, controles de camadas aninhadas, flexi-binding, melhorias nas camadas switch | veja a tabela do AS 9; ❌ layer comps, depth shifting |
| Anime Studio 10 / 10.1 | Novas restrições, configuração de Smart Bone, bounce/elastic/stagger, processo de renderização separado, ferramentas combinadas de osso+ponto, pré-visualização de animação, multidocumento, editor de atalhos, seleção de várias formas, quadro zero, ocultar pontos/ossos, edição de várias camadas, **curvas de largura variável**, transparência de textura, fonte de partículas ajustável, profundidade de campo, blur fracionário; 10.1: vetorização automática de imagem, estilos globais, patch layers, importação PSD, mídia em tempo real, Character Wizard | ✅ traços de largura variável, ✅ animação de várias camadas; 🟡 profundidade de campo ❌ blur ❌; ❌ patch layers, os demais como acima |
| Anime Studio 11 | Quadro a quadro, referenciamento de camadas, ordenação animada de formas, **alvos de osso animados**, **parentesco de osso animado**, inversão de ossos, agrupamento de camadas de seleção, escala de camada normalizada, exportação em lote, novo formato de arquivo | ✅ FbF, ✅ alvo de IK animado (o alvo é só um osso), ❌ *reparentar* animado, 🟡 inversão de ossos (`mirror`), ✅ lote via CLI, ✅ formato de arquivo em JSON simples |

**Novas lacunas identificadas aqui** (adicionar ao roadmap): layer comps (estados salvos de visibilidade de camadas),
reparentar ossos de forma animada, patch layers, depth shifting, camadas follow-path, sobreposições de zona segura,
timecode SMPTE, sombreamento automático, pincel de dispersão, vetorização automática de imagem.

---

# Recursos da concorrência não cobertos pela lista de verificação acima

Produtos pesquisados: Toon Boom Harmony 24, Spine 4.2, Live2D Cubism 5.x, Cartoon Animator 5.3, Rive,
Synfig Studio, OpenToonz, TVPaint Animation 12, Adobe Animate, Cavalry, Procreate Dreams, Duik (After
Effects), Blender Grease Pencil. Os itens abaixo são os que **ainda não constam** na lista de verificação
de referência. A qualidade das fontes varia: a maioria das linhas vem de páginas de recursos/lançamentos do fornecedor que eu consultei;
as linhas marcadas com † vêm de conhecimento geral ou apenas de um título/sumário e não foram confirmadas em uma página.
O estado é o do OSATURA: ✅ · 🟡 · ❌.

## Rigging, restrições, deformação
| Recurso | Visto em | OSATURA |
|---|---|---|
| Ferramentas de pesos: soldar pesos entre malhas, copiar/colar pesos entre vértices | Spine 4.2 | ❌ (somente auto-skin, sem pintura) |
| Restrições de física de ossos (inércia, vento, gravidade, mix) | Spine 4.2 | 🟡 molas amortecidas (frequência/amortecimento); sem vento/gravidade/inércia |
| Restrição de caminho: ossos seguem uma spline Bézier composta | Spine, Rive (Follow Path) | ❌ |
| Mixes de restrições, duplicação com ossos, multiedição, pastas | Spine 4.2 | 🟡 mix de IK e peso de restrição animáveis; sem pastas/multiedição |
| Skins (trocar conjuntos de attachments em um esqueleto) | Spine, solos do Rive | 🟡 camadas switch |
| "Ferramenta de pose" de IK multiosso (arrastar a ponta de um membro) | Spine | ❌ (requer GUI de canvas) |
| Attachments de bounding box / hit-test | Spine | ❌ (desnecessário para saída offline) |
| Free-Form Deformation, pesos de malha | Spine, Harmony | ✅ lattice + malha de imagem com pesos |
| Cadeia de deformadores: Bone, Game Bone, Curve, Envelope, Free-Form, Mesh Warp | Harmony | 🟡 bone ✅ curve (spine) ✅ free-form ✅ mesh warp ✅; envelope/cage ❌; game-bone ❌ |
| Shape-Aware Deformation (pontos, ossos, cages) | Harmony 24 | ❌ |
| Bake de todos os tipos de deformação | Harmony 24 | 🟡 molas pré-calculadas; geometria pré-calculada por quadro na exportação Lottie |
| Master controller / seletor de transformação para grupos de deformação | Harmony 24 | ❌ |
| Pegs e hierarquia de camadas | Harmony, Animate | ✅ grupos, camadas presas a ossos, parentesco |
| Auto-muscle, auto foot-roll, saída cinemática † | Harmony | ❌ |
| Deformadores de rotação / warp + parâmetros de keyform, glue | Live2D | 🟡 parâmetros ≈ canais guiados, warp ≈ lattice; glue ❌ |
| Blend shapes guiando rotação, opacidade, ordem de desenho, cor multiply/screen | Live2D 5.0 | 🟡 os pesos de morph guiam apenas geometria |
| Auto-mesh com controle de densidade uniforme para formas finas | Live2D 5.0 | ❌ (somente grade regular) |
| O warp deformer se redimensiona automaticamente para conter o keyform filho | Live2D 5.3 | ❌ |
| Pincel de deformação (formas quadrada/linha/expansão) | Live2D 5.3 | ❌ |
| Deformador facial assistido por IA / geração de movimento facial | Live2D 5.0 | ❌ |
| Joysticks (controle 2D mapeado para animações) | Rive | 🟡 o canal guiado mapeia uma entrada; sem joystick de 2 entradas |
| Restrições entre objetos arbitrários (translação/rotação/escala/distância) | Rive | 🟡 somente aim, copy-rotation, pin |
| Rigging de bitmap com intercalação automática | TVPaint 12 | 🟡 malha de imagem ✅; intercalação automática ❌ |
| Ferramenta Plastic (rigging baseado em malha) | OpenToonz | 🟡 malha de imagem + pesos |
| Auto-rigging, controladores, dinâmica de molas | Duik † | 🟡 molas ✅; auto-rig ❌ |

## Animação, timeline, interatividade
| Recurso | Visto em | OSATURA |
|---|---|---|
| Mistura de animações: blend/crossfade de animações simultâneas em runtime | Spine | 🟡 modos Add/Multiply + pesos; `crossfade` pré-calculado; sem transições em runtime |
| Dopesheet + editor de curvas + pré-visualização em runtime | Spine, Rive, OpenToonz | ❌ (os dados suportam; sem UI) |
| Máquina de estados: entradas, listeners, transições com condições, estados de blend, camadas | Rive | ❌ |
| Eventos, gatilhos de áudio, assets dinâmicos | Rive | ❌ |
| Data binding (elementos de design ↔ dados ao vivo) | Rive | ❌ (expressões podem ler canais, não dados externos) |
| Scripting embutido (Luau, JavaScript, ToonzScript) | Rive, Cavalry, OpenToonz | ❌ |
| Layouts responsivos / componentes / bibliotecas | Rive | ❌ |
| Trim Paths, feathering vetorial, N-slicing | Rive | ❌ |
| Temporização independente de resolução, tweening vetorial | Synfig | ✅ |
| Vinculação de parâmetros entre camadas por expressões | Synfig | ✅ |
| Grafo de nós procedural (geradores, transformadores, efetuadores), duplicadores | Cavalry | ❌ (sem grafo de nós; sem instanciação de camadas) |
| Animação guiada por dados a partir de JSON/CSV | Cavalry | ❌ |
| Performing: gravar gestos ao vivo como keyframes | Procreate Dreams | ❌ |
| Atalhos de gatilho no palco de puppets, motion pilot (caminhadas guiadas pelo cursor) | Cartoon Animator 5.3 | ❌ |
| Seletor de câmera: câmeras salvas, cortes por atalho, track de troca de câmera | Cartoon Animator 5.3 | 🟡 uma câmera; cortes podem ser simulados com keys step |
| Movimentos cinematográficos (dolly zoom, rastreamento de um alvo) | Cartoon Animator 5.3 | 🟡 câmera animável; sem rastreamento de alvo |
| Mocap facial (webcam / iPhone), corpo guiado pela cabeça, volume do microfone | Cartoon Animator, Live2D | ❌ (volume do microfone somente via WAV offline) |
| Modo de livestream (OBS/Twitch, mapeamento de Stream Deck) | Cartoon Animator 5.3 | ❌ |
| Lip-sync Motion Sync gerando keys na timeline | Live2D 5.0 | 🟡 WAV heurístico → keys de exposição |
| Integração de lip-sync com Rhubarb | OpenToonz | ❌ (somente heurístico) |
| Extração de root motion (compensação de delta) | Spine 4.2 | 🟡 extrapolação loop-offset |
| Sincronização das alças de curvas em loop | Spine 4.2 | ✅ extrapolação em loop (sem UI de alças) |
| Morphing com morph hints | Harmony | 🟡 morph targets + lerp de caminho; sem hints |
| Câmera 2D baseada em keys com temporização na timeline | TVPaint 12 | ✅ |

## Ferramentas de desenho, pintura e bitmap
| Recurso | Visto em | OSATURA |
|---|---|---|
| Pincel: randomização (quantidade de carimbos, deslocamento), papéis duplos, pincéis multitexturizados | Harmony 24, TVPaint 12 | ❌ |
| Opacidade do lápis por pressão/inclinação/velocidade/afilamento/fade | Harmony 24 | ❌ |
| Guias de simetria (vertical, horizontal, dupla) | Harmony 24 | ❌ |
| Ferramenta Cutter (laço, aparar), editor de contorno, fechar lacunas | Harmony | ❌ |
| Guias de desenho, guias de perspectiva/régua † | Harmony | ❌ |
| Animação sem papel: onion skin, flipping, mesa de luz | Harmony, Procreate Dreams, TVPaint | 🟡 (onion skin sim; sem ferramentas de desenho nem mesa de luz) |
| Limpeza de desenhos escaneados, escaneamento, captura por câmera, stop-motion | OpenToonz | ❌ |
| Pincel de seleção, biblioteca de imagens | TVPaint 12 | ❌ |
| Nitidez na transformação | TVPaint 12 | n/a |
| Sensibilidade à pressão da caneta | Synfig | ❌ |
| Sombreamento de gradiente baseado em caminho | Synfig | ❌ |
| Texto com conteúdo variável/dinâmico em runtime, em qualquer idioma | Rive, Procreate Dreams | ❌ |
| Fontes personalizadas, títulos, créditos | Procreate Dreams | ❌ |
| Gerenciamento de paletas e estilos | OpenToonz | ❌ |
| Mais de 50 tipos de camada (fractais, filtros, distorções) | Synfig | ❌ (≈7 tipos de camada) |

## Efeitos e composição
| Recurso | Visto em | OSATURA |
|---|---|---|
| Rede de composição em visão de nós com portas nomeadas | Harmony 24 | ❌ (árvore de camadas + máscaras/mesclagens) |
| Mattes, grupos sob composite | Harmony | ✅ máscaras, grupos isolados |
| Leitura/gravação de CryptoMatte | Harmony 24 | ❌ |
| Toon shader (contorno, posterização) | Harmony 24 | ❌ |
| Unsharp mask, troca de canais | Harmony 24 | ❌ |
| Família de blur (radial, spin, direcional), bokeh, ruído fractal, tile | OpenToonz | ❌ |
| FX de partículas | OpenToonz | ✅ camada de emissor |
| 15 modos de mesclagem RGB + 5 alfa, desenho offscreen por parte | Live2D 5.3 | 🟡 13 modos de mesclagem; renderização de grupos isolados ✅ |
| Clipping (IDs de clip por parte) | Live2D, Rive | ✅ máscaras alfa |
| Imagens HDR / HDRI | Synfig | ❌ |
| Importação de vídeo, ProRes, rotoscopia sobre filmagem | Procreate Dreams | ❌ |
| Motor de áudio (locução, música, SFX) | Procreate Dreams | ❌ |

## 3D, pipeline, interoperabilidade, runtimes
| Recurso | Visto em | OSATURA |
|---|---|---|
| Cena 3D, pose de ossos FBX, grafo de cena Vulkan, desenho em plano de perspectiva | Harmony 24 | ❌ |
| Renderização externa via Maya / Blender | Harmony 24 | ❌ |
| Importação vetorial: PDF / AI / SVG; EXR multicamada; alfa DPX; OpenColorIO (ACES) | Harmony 24 | ❌ |
| Importação PSD sem Photoshop | Spine 4.2, Rive | ❌ |
| Importação por colagem de Lottie / Figma | Rive | ❌ (apenas **exportação** Lottie) |
| Texture packer / atlas | Spine | ❌ |
| Exportação de esqueleto JSON/binária + 19 runtimes oficiais / mais de 40 da comunidade | Spine | 🟡 JSON próprio; sem runtimes |
| Runtimes de código aberto (Web, Apple, Android, React, Flutter, C++, Unity, Unreal) | Rive | ❌ |
| Modo Pixels (alinhar sprites aos pixels da tela) | Spine 4.2 | ❌ |
| Exportação: WebM com transparência | Live2D 5.3 | ❌ |
| Exportação: SVG/PDF, sequências PNG/EXR, profundidade de 16/32 bits, lote, ProRes | Cavalry | 🟡 SVG, PNG ✅; PDF/EXR/ProRes/alta profundidade de bits ❌ |
| Renderização na nuvem, links de compartilhamento, embeds | Rive | ❌ |
| Renderização distribuída (Toonz Farm), controle de versão | OpenToonz | ❌ |
| Exportação PDF / JSON do TVPaint | OpenToonz | ❌ |
| Colaboração em equipe em tempo real, remix pela comunidade | Rive | ❌ |
| Exportação HTML5 Canvas / web interativa | Animate † | 🟡 player HTML (pré-renderizado, não interativo) |
| Linux/macOS/Windows nativos | Synfig, OpenToonz | ✅ |
| Licença de código aberto | Synfig, OpenToonz | ✅ MIT/Apache-2.0 |
| Visão de parentesco de camadas, símbolos/instâncias de biblioteca † | Animate | 🟡 parentesco ✅; sem biblioteca de instâncias |
| UI modular destacável, workspaces personalizados, tema escuro/claro | TVPaint 12, Live2D 5.0 | ❌ |

## Roadmap candidato (ordenado por valor ÷ esforço, sem exigir GUI primeiro)
1. Trim paths + seguimento de caminho (ossos/camadas ao longo de uma Bézier) — Rive, Spine
2. Onion skin + sobreposições de trilha de movimento — quase toda ferramenta 2D
3. Importação SVG (caminhos, gradientes) — Moho 14, Harmony, Synfig
4. Termos de vento / gravidade para molas, dinâmica de ossos guiada por entradas no estilo Smart Bone
5. Blur, sombra projetada, brilho, contorno, posterização (filtros de camada)
6. Instanciação de camadas / duplicadores (clonar N ao longo de um caminho, com deslocamentos por cópia)
7. Máquina de estados interativa no runtime HTML/JSON (entradas, transições, blend)
8. Deformador envelope (cage), ferramentas de copiar/soldar/suavizar pesos, auto-mesh de densidade uniforme
9. Cortes de câmera / câmeras salvas, câmera que rastreia um alvo
10. Exportação glTF do rig; exportação EXR/PDF/WebM-alfa
11. Camadas de texto (requer carregamento de fontes) com seguimento de caminho
12. GUI do studio: edição de pontos/ossos no canvas, editor de curvas, onion skin

## Fontes adicionais (concorrência)
- Notas de lançamento do Toon Boom Harmony 24 e Premium Getting Started Guide (docs.toonboom.com)
- Spine 4.2 "The physics revolution" e "What is Spine?" (esotericsoftware.com)
- Live2D Cubism 5.0 e 5.3 "New Features" (docs.live2d.com)
- Destaques de recursos do Cartoon Animator 5.3 (discussions.reallusion.com)
- Página de recursos do Rive (rive.app/features)
- Recursos do Synfig Studio (zoftwarehub.com); documentação "What's New" do OpenToonz
- TVPaint Animation 12 (tvpaint.com); análise do Cavalry (superrendersfarm.com)
- Adobe Animate "Layer parenting" (adobe.com/learn); Procreate Dreams (Cartoon Brew)
- Duik: página de tag de Lester Banks (somente título †); Blender Grease Pencil: página do manual não recuperável (não coberto)

---

# Adendo — Históricos de versões da concorrência (rodada 2, 2026-10-19)

Legenda como acima: ✅ pronto, 🟡 parcial, ❌ ainda não. Só são listados recursos confirmados em páginas de fornecedores/projetos ou na imprensa. Sites piratas não foram usados; web.archive.org não é acessível a partir deste ambiente, de modo que algumas versões mais antigas continuam sem documentação (veja "Lacunas conhecidas").

## Toon Boom Harmony 20 (notas de lançamento do fornecedor)
| Recurso | OSATURA |
|---|---|
| Editor Centerline: exclusão de pontos, opção de suavidade | ❌ |
| Linhas de lápis texturizadas → traços de pincel texturizados; conta-gotas de textura de lápis | ❌ |
| Ferramenta Line curva / em S; pincel Auto-Fill; pincel Repaint | ❌ |
| Guias de alinhamento, snapping limitado às camadas selecionadas | ❌ |
| Quick Ease / Ease In / Ease Out em keyframes selecionados | 🟡 easing por key ✅; comandos de easing em lote ❌ |
| Marcadores de cena (sem duração, ripple edit) | ❌ |
| Visão de substituição de desenhos (renomear/filtrar) | 🟡 camadas Switch ✅ |
| Nós de sombreamento de superfície: oclusão ambiente, sombra projetada, mapa de superfície/normal | ❌ |
| **Nó Weighted Deform** | 🟡 malha de imagem com pesos ✅ |
| Nó Point Kinematic Output (amostragem de 1/2/3 pontos) | ❌ |
| Nó de cache de pré-renderização | 🟡 cache de bake de molas ✅ |
| Nós de fade de cor / curvas / níveis / gama | ❌ |
| Forma de partículas em região planar usando um desenho vetorial | 🟡 emissor ✅; forma personalizada ❌ |
| Master controllers: deformador sobre deformador, Function Wizard, Slider Wizard | 🟡 canais ✅; UIs de wizard ❌ |
| Scripting em Python (PythonManager) | 🟡 patch ops JSON + CLI; sem scripting embutido |
| Gerenciamento de espaço de cor, renderização em 16 bits, filme de pós-processamento por script | ❌ (pós-processamento com ffmpeg à mão) |
| Control Center: cópia/importação/execução de scripts de cenas em lote, fila de renderização | ❌ |

## OpenToonz 1.7
| Recurso | OSATURA |
|---|---|
| Modo de linha reta para o pincel | ❌ |
| Suporte a pincéis do GIMP + MyPaint | ❌ |
| Modo de preenchimento Selection+Freehand | ❌ |
| Colar imagem da área de transferência | ❌ |
| Painel de pré-produção / storyboard | ❌ |
| Painéis personalizados, janela principal translúcida, girar/espelhar o visualizador | ❌ |
| Escala do Xsheet + navegação por tags | 🟡 dados de timeline ✅; sem GUI |
| Exportação com informações de movimento de câmera | 🟡 câmera no Frame ✅ |
| **Exportação OCA (Open Cel Animation)** | ❌ |
| Exportação MOV e APNG | 🟡 MP4/GIF ✅; APNG/MOV ❌ |
| Conversão em lote TZP→TLV | ❌ |
| Floor Bump Iwa, Flow Fx; renderização float + cor linear | ❌ |
| Suporte ao Haiku OS | n/a |

## Synfig 1.4.2 (e 1.4.0 da rodada 1)
| Recurso | OSATURA |
|---|---|
| Exportação Lottie incl. camadas avançadas de contorno e blur, gama | 🟡 exportação Lottie ✅ (formas/transformação); blur ❌ |
| Editor de atalhos de teclado, atalhos de ferramenta de tecla única | ❌ |
| TimeLoop editável no workspace | 🟡 clipes em loop ✅ |
| Duplicação de camadas Skeleton / Skeleton Deformation com vínculos de ossos | ✅ (os ossos são dados do projeto) |
| Importador SVG que trata atributos de estilo | ❌ (sem importação SVG) |

## Itens da rodada 1 (páginas de fornecedores, abreviado)
- **Spine 4.2/4.3**: restrições de física, sliders, mistura de restrições, pixel snapping — molas ✅ (pré-calculadas), sliders/mix 🟡 (`mix` de IK ✅), modo pixels ❌.
- **Live2D Cubism 4.0–5.3**: ArtPath, onion skin, exportação WebM-alfa — ArtPath ❌; onion skin e WebM-alfa ✅.
- **Harmony 21/22**: deformadores com pesos, texturas de line-boil — 🟡 / ✅ (oscilação).
- **Cartoon Animator 5** — veja as tabelas da concorrência acima.

## Novos candidatos ao roadmap desta rodada
13. Exportação OCA (Open Cel Animation) e APNG — OpenToonz 1.7
14. Comandos de easing em lote / "quick ease" no editor — Harmony 20
15. Filtros de correção de cor (fade, níveis, curvas, gama) — Harmony 20
16. Sombreamento de superfície (sombra projetada / AO) para camadas vetoriais — Harmony 20
17. Scripting embutido (Python/Lua) sobre a API de patch ops — Harmony 20
18. Nó de cache de pré-renderização / avaliação em cache — Harmony 20
19. Marcadores de cena + ripple editing na timeline — Harmony 20
20. Layer comps, reparentar ossos de forma animada, patch layers, follow-path, timecode SMPTE, zonas seguras, vetorização automática, pincel de dispersão (lacunas encontradas na pesquisa do Moho)

## Lacunas conhecidas (não recuperáveis em fontes acessíveis)
Anime Studio 8.x, 10.1.x, 11.1; detalhes do Moho 12.5 e 14.1; Spine 3.0–4.1 (o blog do 3.7 é só uma página de vídeo); Harmony 17, 23, 25–27 (páginas de documentação com erro); Synfig 1.4.1; TVPaint 11; Cartoon Animator 4/5.1/5.2; changelog do Rive.

## Fontes (rodada 2)
- docs.toonboom.com — notas de lançamento do Harmony 20
- en.ubunlog.com — artigo sobre o OpenToonz 1.7
- synfig.readthedocs.io — notas de lançamento do Synfig 1.4.2
- esotericsoftware.com — resumo de recursos do Spine 3.7 (página de vídeo, sem lista)

---

# Adendo — Fontes do YouTube (rodada 3, 2026-10-19)

O WebSearch restrito a youtube.com retorna títulos de vídeos; o WebFetch retorna a **descrição/lista de capítulos** de cada vídeo (sem transcrições). Só são listados os recursos ali nomeados; "novo na versão X" é como declarado por quem enviou o vídeo (muitas vezes o fornecedor).

## Moho 14.1 (descrição do vídeo)
| Recurso | OSATURA |
|---|---|
| Mostrar/ocultar alças na ferramenta Curvature | ❌ (sem GUI de canvas) |
| Efeitos de forma Shaded e Halo visíveis no modo de edição | 🟡 gradientes ✅; efeitos halo/shade ❌ |
| Efeito de pixelização nas Live Project Settings | ❌ |
| Preview Animation: resolução maior, borda de saída oculta | 🟡 pré-visualização raster ✅ |
| Diálogos não modais para scripts Lua | ❌ (sem scripting) |
| Modo de mesclagem "Linear Dodge (Add)" do PSD; modos de mesclagem atualizados | 🟡 verificar a lista de mesclagens; add/dodge ❌ |
| Indicador de documento não salvo na barra de título | ✅ o studio acompanha o estado de desfazer |
| Correções: avaliação de smart bones, motion blur de subquadro + dinâmica de ossos, alças Bézier de comprimento zero | n/a |

## Moho 12.5
As páginas de título/descrição não trouxeram lista de recursos (vídeo desatualizado; download gratuito do 12.5 retirado). Títulos relacionados mencionam tutoriais de Smart Bones, rigs 2.5D com Smart Warp e instalação das "Mynd Tools" — Smart Bones 🟡 (canais/morphs), Smart Warp 🟡 (lattice FFD ✅).

## Spine
| Versão | Recurso | OSATURA |
|---|---|---|
| 3.7 | Suporte a áudio | ❌ (WAV analisado apenas para lip-sync) |
| 3.7 | Exportações reformuladas | 🟡 |
| 3.7 | Digitar para pesquisar | ❌ |
| 3.7 | IK elástico / comprimido / uniforme | ❌ (o IK preserva o comprimento) |
| 3.7 | Combinar skins, duplicar skins, placeholders | 🟡 camadas Switch |
| 3.7 | Copiar/colar vértices | ❌ |
| 3.7 | Renderização em grade de pixels | ❌ |
| 3.7 | Mistura aditiva de animações | 🟡 somente morphs aditivos |
| 3.7 | Runtime spine-cpp; player web | ❌ / 🟡 player HTML |
| 3.8 | Mesh tracing | 🟡 malha de grade automática; sem vetorização de imagem |
| 3.8 | Empacotamento poligonal de texturas | ❌ |
| 3.8 | **Soft IK** | ❌ |
| 3.8 | Ossos e restrições de skin | ❌ |
| 3.8 | Projeto mix-and-match | ❌ |
| 3.8 | Pastas, réguas, exportação PSD aprimorada, tela de boas-vindas | ❌ |
| 3.8 | .json/.skel menores; carregamento binário no spine-ts | ❌ (somente JSON) |
| 4.0 beta | **Editor de gráfico / curvas** | 🟡 easing por key ✅; sem editor |
| 4.1 beta | Função Sequence (attachment de sequência de imagens) | ❌ |

## Toon Boom Harmony 25 / 25.2
| Recurso | OSATURA |
|---|---|
| Ferramenta Pencil Retouch | ❌ |
| Importar pincéis do Photoshop | ❌ |
| **Breakdown Assistant para animação cut-out** | ❌ (o gerador de ciclo de caminhada é um recurso relacionado) |
| Bloquear a régua da timeline; preferências de modelos de ação | ❌ |
| Melhor suporte a arquivos de imagem, 3D e USDZ | ❌ |
| Art Layer Manager (trocar, limpar, mesclar, duplicar camadas de arte; 25.2+) | ❌ (duplicar camada via patch ops ✅) |
| Master Controller Grid Wizard | ❌ |

## Série Cartoon Animator 5
| Recurso | OSATURA |
|---|---|
| 5.1 Edição de ida e volta de PSD e vetor | ❌ |
| 5.1 Importação de sequência de imagens e GIF | ❌ |
| 5.2 **Motion Pilot** (manipular o movimento arrastando) | ❌ |
| 5.2 Animação por Motion Path / edição de caminho | ❌ (roadmap n.º 1) |
| Dinâmica de molas, FFD, animação vetorial (existentes) | ✅ / ✅ / ✅ |

## Novos candidatos ao roadmap desta rodada
21. Soft IK + modos de IK elástico — Spine 3.8/3.7
22. Camada de sequência de imagens (attachment de lista de quadros) e importação GIF/APNG — Spine 4.1, CA 5.1
23. Filtro de saída de pixelização / pixel-snap — Moho 14.1, Spine 4.2
24. Gravação no estilo Motion Pilot: capturar o arrasto ao vivo de um osso/alça em keys — CA 5.2
25. Breakdown assistant (intercalação automática com arcos para rigs cut-out) — Harmony 25
26. Mesh tracing a partir do contorno alfa — Spine 3.8
27. Conjuntos de skins (trocar grupos de attachments juntos) — Spine 3.7/3.8
28. Faixa de áudio no projeto + sincronização de reprodução (não só análise) — Spine 3.7
29. Hooks de scripting em Lua/Python — Moho, Harmony 20

## Fontes (rodada 3, YouTube)
- Moho 14.1: A Complete Breakdown — youtube.com/watch?v=QT4AiV1wO1k; Moho 14.1 Update — watch?v=FCiBKBjDaNg
- Resumo de recursos do Spine 3.7 — watch?v=Fi0ks2LeYSM; resumo de recursos do Spine 3.8 — watch?v=kU26B5465SQ; editor de curvas do Spine v4-beta — watch?v=37CeFRzrjqc
- Mudanças do Harmony 25 — watch?v=kJ58C2DNIk8; Art Layer Manager 25.2+ — watch?v=AZc-8zHucOo
- Cartoon Animator 5.2 Motion Pilot — watch?v=Eu7vO2uGGN4; importação de sequência/GIF do 5.1 — watch?v=O8EGwVwqgfs

---

# Adendo — Busca exaustiva de lacunas (rodada 4, 2026-10-19)

Fontes: notas de lançamento oficiais do Synfig/Rive, títulos/descrições do YouTube. Só são listados os recursos nomeados.

## Anime Studio 8 (série "Pro Feature" do YouTube, títulos + descrições)
| Recurso | OSATURA |
|---|---|
| Vetorização automática de imagem (bitmap → vetor editável) | ❌ (roadmap n.º 20) |
| Character Wizard (criação guiada de personagens) | 🟡 rigs de demonstração/gerador de caminhada; sem wizard |
| Importar arquivos do Photoshop (PSD) | ❌ |
| Real-Time Media Connection (link ao vivo com mídia externa) | ❌ |
| Ferramenta Scale Points | 🟡 patch ops em pontos |
| Predefinições de projeto HD 1080p | ✅ qualquer tamanho |
Anime Studio 10.1 / 11.1: nenhuma fonte acessível encontrada.

## Synfig 1.4.1 – 1.4.4
| Versão | Recurso | OSATURA |
|---|---|---|
| 1.4.1 | Camadas de som mixadas no vídeo exportado | ❌ (sem faixa de áudio) |
| 1.4.1 | Duplicar camada copia todas as camadas selecionadas | ✅ (ops em lote) |
| 1.4.3 | Aviso de bone-link: vincular a transformação em vez da origem | ✅ |
| 1.4.3 | Importação de Papagayo (.pgo); correções na importação SVG; taxa de quadros de sequência de imagens | ❌ / ❌ / ❌ |
| 1.4.3 | Correções na renderização de GIF, compatibilidade com FFmpeg 3.x | ✅ GIF/MP4 |
| 1.4.3 | Salvamento automático (a partir de 15 s) | ❌ (salvamento explícito) |
| 1.4.4 | Correção na exportação do último quadro | ✅ coberto por testes |

## Rive (notas de lançamento oficiais, set–out 2026) — o concorrente mais ativo
| Recurso | OSATURA |
|---|---|
| Conversor String Template (texto a partir de valores de view-model, formatação de números) | ❌ |
| Traços tracejados (Dash Effect), importados do SVG | ❌ |
| Segurar Q/W/E + arrastar para mover/girar/escalar | ❌ (sem GUI) |
| Formas de pincel com keys no modo Animate | ❌ |
| Copiar/colar keyframes de restrições; conversores mantêm fórmulas | 🟡 restrições ✅, cópia ❌ |
| Data binding relativo; view models; propriedades Blob | ❌ |
| Scripting Luau (fetch, text shaping, canvas GPU, shaders/WGSL, malhas de imagem instanciadas) | ❌ |
| Estados de blend / Mix-by-view-model na timeline | 🟡 bake de crossfade |
| Listeners de teclado, foco, gamepad; Text Input; semântica de acessibilidade | ❌ |
| Layouts Grid e Stack, listas virtualizadas, restrições de rolagem e física de rolagem | ❌ |
| Set Parent (Alt+P) mantendo posição e ordem de desenho | 🟡 reparentar via patch ops; manter o mundo ❌ |
| Align to Key Object; deslocamento de osso raiz; Scroll to selection | ❌ |
| Intensidade aditiva para preenchimentos/traços; canais RGBA separados com keys/bindings | 🟡 morphs aditivos; RGBA com keys ✅ |
| Traços internos/externos (acesso antecipado) | ❌ |
| Auto Trace (melhor para pixel art) | ❌ |
| Conversão de Flash `.fla`/XFL → Rive | ❌ |
| Predefinições de renderização: máquina de estados → vídeo | 🟡 clipe → MP4/GIF |
| Exportação `.riv` no plano gratuito; CLI (`rive push/publish/ls/docs`), extensão do VS Code, simulação de entrada headless | 🟡 CLI ✅ (comandos próprios) |
| Agente de IA no editor | ❌ |
| Pré-visualização no Android; runtimes React Native / Apple / Android / Flutter | ❌ |
| Histórico de revisões e restauração, sincronização offline, desfazer que mantém o zoom | 🟡 desfazer/refazer ✅ |

## TVPaint Animation 11 (descrições do YouTube)
Criação de atalhos, under-drawings removíveis, suavização de linhas, gravação de desenho, clique e arraste do ponto de pivô do KeyFramer (11.7.1), Sprite Generator, Inlay Texture FX, guias, vídeo "end of update releases" — tudo ❌ (ferramentas de pintura raster; o OSATURA é vetor/rig).

## Toon Boom Harmony 17 (YouTube)
Free Form Deformer 🟡 (lattice FFD ✅), Master Controllers turbinados 🟡, RGB Difference Keyer ❌, Motion Blur ❌ (roadmap n.º 5).

## Cartoon Animator 4 / Spine 3.6 / Harmony 22–23
Foram encontrados vídeos (CA 4: plugin de mocap Rokoko, atualização de movimento 3D para 2D, normal map e iluminação no 4.5; caminhos do Spine 3.4; vídeo de recursos do Spine 3.6; ferramentas de desenho do Harmony 22), mas as descrições não listam recursos → nada é alegado. Notáveis apenas pelo título: **retargeting de motion capture** (CA 4) ❌, **normal maps/iluminação** (CA 4.5) ❌, **restrições de caminho** (Spine 3.4) ❌.

## Novos candidatos ao roadmap desta rodada
30. Dash effect + traços internos/externos
31. Runtime de scripting embutido (Lua/Luau) com API sem GPU
32. Data binding / view models + entradas de máquina de estados (teclado/ponteiro/foco) para o player HTML
33. Restrições de rolagem / contêineres de layout (grid, stack) para animação no estilo de UI
34. Importação de lip-sync Papagayo (.pgo); importação no estilo Rhubarb
35. Set Parent mantendo a transformação de mundo
36. Salvamento automático + histórico de revisões no studio
37. Motion Blur e filtros de RGB keyer
38. Restrições de caminho (osso ao longo de uma Bézier) — Spine 3.4, funde-se com o roadmap n.º 1
39. Retargeting de motion capture para o rig 2D (BVH/JSON) — CA 4
40. Iluminação com normal map para camadas de imagem — CA 4.5
41. Faixa de som mixada na exportação MP4 — Synfig 1.4.1
42. Importação de .fla / XFL e PSD

## Ainda não recuperável
Anime Studio 10.1 / 11.1, notas detalhadas do Moho 12.5, páginas de detalhes do Spine 3.0–3.6, Harmony 23, notas detalhadas do Cartoon Animator 4.x, texto do changelog do TVPaint 11.

## Fontes (rodada 4)
- synfig.readthedocs.io — notas de lançamento 1.4.1, 1.4.3, 1.4.4
- releasebot.io/updates/rive — Rive Editor 0.9–0.9.272, CLI 1.2–1.6
- YouTube: série Anime Studio 8 Pro Feature (q_HH2zOE-Yk, CUemDIlPH6Y, 408CLy652mU, l9eXLUO9Dus); demo do TVPaint 11 (MoqNrCAUy5k); Harmony 17 (1CU_3RJBcl4); playlist do Cartoon Animator 4

---

# Adendo — Redes sociais e canais da comunidade (rodada 5, 2026-10-19)

**Realidade do acesso:** as páginas do X/Twitter estão bloqueadas para busca pelo robots.txt (só são visíveis trechos/títulos de resultados de busca); o Facebook idem; as buscas no Reddit com filtro de domínio foram rejeitadas pelo proxy de busca e as buscas sem filtro não retornaram threads do Reddit; o fórum da Lost Marble serve uma casca de redirecionamento em JavaScript sem conteúdo legível. Nada foi contornado. O que foi legível:

## Canais oficiais do Moho (notícias de moho.lostmarble.com, trechos do X)
| Item | Observações | OSATURA |
|---|---|---|
| Moho 13.5.3 (abr 2022) | lip-sync automático aprimorado, interface melhor, ossos menos intrusivos | ✅ lip-sync (WAV heurístico) |
| Moho 13.5.5 (X, mai 2022) | **Relative Paths** para assets vinculados | ✅ o diretório de assets é relativo |
| Moho 14.3 "Mantis Shrimp" (dez 2024) | atualização focada em pincéis | ❌ |
| Moho 14.4 "Side Quest" (nov 2025) | exportação para engines de jogos e software 3D | 🟡 Lottie/JSON próprio; glTF/FBX ❌ |
| Moho 14.5 "Showtime" (out 2026) | onion skins magnéticos, Shift & Trace, opções de Liquid Shapes, EXR/PNG com perfis de cor, modo live-playback para transmissão/controle de personagens em tempo real | veja a matriz principal; modo ao vivo ❌ |
| Programa de Embaixadores (jun 2026); posição oficial: nenhuma ferramenta de IA planejada (mar 2025) | política | n/a — o OSATURA também não tem recursos de IA |
| Webinars (Víctor Paredes etc.) | tutoriais | n/a |

## Comparações da comunidade (fórum do Spine, "Spine vs Moho 12")
- Os usuários elogiam no Moho: física integrada, mascaramento de camadas, smart bones, edição vetorial.
- Os usuários criticam: FFD considerado fraco; **sem runtime real**, exportação para jogos limitada a FBX.
- Creature elogiado pelo motion capture integrado (FFD criticado).
- As restrições de física do Spine 4.2 recebem reações entusiasmadas no X (posts de usuários).
OSATURA: física ✅ (molas pré-calculadas), mascaramento 🟡 (clip via booleanas), smart bones 🟡, edição vetorial ✅, runtime ❌, mocap ❌.

## Títulos de listas de desejos de fóruns vistos (conteúdo ilegível)
"Concentrate on Timeline/Workflow improvements" (2024), "Road map for Moho", "Help Shape the Future of Anime Studio", "13.5.2 performance issue" — sugere que timeline/fluxo de trabalho e desempenho são as áreas mais solicitadas; não é uma lista de recursos verificada.

## Adições ao roadmap
43. Modo de live-playback / streaming em tempo real (entrada de pose por WebSocket → quadros renderizados) — Moho 14.5
44. Biblioteca de runtime em uma linguagem pequena e portável (build WASM do avaliador) — fraqueza mais citada do Moho
45. Mascaramento de camadas como tipo de camada de primeira classe (não apenas operações booleanas)
46. Ergonomia de timeline/fluxo de trabalho no studio (editor de curvas, seleção múltipla de keys) — listas de desejos da comunidade

## Fontes (rodada 5)
- moho.lostmarble.com/blogs/news (17 posts, dez 2021 – out 2026)
- Trechos de busca do X: posts de @MohoAnimation (Relative Paths do 13.5.5; preços do Moho 14), posts de usuários sobre a física do Spine 4.2
- Tópico do fórum do Spine "8033 spine vs moho 12" (de.esotericsoftware.com)
- Títulos de tópicos do fórum da Lost Marble (conteúdo ilegível)

---

# Adendo — Listas de desejos e quadros de pedidos de recursos (rodada 6, 2026-10-19)

Só são listados os pedidos realmente legíveis. A coluna "OSATURA" indica se o pedido é algo que o motor já cobre.

## Moho (fórum da Lost Marble)
Legível: apenas o índice do fórum. Existem as seções "Feature Request Discussions" e "Unofficial Bug Reports and Feature Requests"; título de pedido mais recente visível: **"Dock Windows – Custom Interface"** (18 out 2026). O longo tópico "Help Shape the Future of Anime Studio" (828 respostas, 2011–2021) e outros tópicos de listas de desejos ("Road map for Moho", "My wish for Moho in 2024 – Concentrate on Timeline/Workflow improvements", "Request for top 10 Anime Studio Pro 6 features") são cascas de redirecionamento em JavaScript → conteúdo ilegível. Comparações específicas entre fóruns (fórum do Spine): física, mascaramento, smart bones desejados no Spine; Moho criticado pelo FFD fraco e pela falta de runtime.
| Pedido | OSATURA |
|---|---|
| Janelas acopláveis / interface personalizada | ❌ (layout fixo do studio web) |
| Melhorias de timeline / fluxo de trabalho | 🟡 |
| Runtime real para jogos | ❌ |
| FFD mais forte | ✅ lattice de Bernstein + malha |

## Cartoon Animator (Reallusion Feedback Tracker + tópico de lista de desejos)
| Pedido (votos quando exibidos) | OSATURA |
|---|---|
| Curva de keyframe padrão nas preferências | ✅ easing por key |
| Edição de keyframes de vários canais | 🟡 ops em lote de patch |
| Configurações de painel fixas | ❌ |
| Marcadores e canais codificados por cor (esquerda vermelho / direita verde) | ❌ (espelhar E↔D ✅) |
| "Uber key" para mostrar todos os canais animados | ❌ |
| Suporte a SVG (Affinity, Inkscape); colar caminhos vetoriais | ❌ |
| Ferramenta/painel de alinhamento | ❌ |
| Profundidade de campo / filtro de blur | ❌ |
| Importação de GIF/APNG/vídeo por arrastar e soltar, posicionada na timeline | ❌ |
| Luzes spot/ambiente com sombras | ❌ |
| Rolagem automática da timeline | n/a |
| Texto melhor: kerning/entrelinha animáveis | ❌ |
| Cor SVG animável | ✅ |
| Ocultar globalmente / canais "shy" | ❌ |
| Painéis acoplados | ❌ |
| Várias cenas por projeto | ❌ |
| **Painel Auto Animate (movimento idle procedural em ossos)** | 🟡 oscilação de ruído coerente ✅ |
| Camada de desenho ao vivo / propriedades dinâmicas editáveis (18) | ❌ |
| Hitbox e física (14) | 🟡 molas |
| Importação de MOV transparente (14) | ❌ |
| Panorama de 360° (5); ideias de integração com IA (6) | ❌ |

## Quadro de pedidos de recursos do Live2D Cubism Editor (30 tópicos, 2023–2026)
| Pedido | OSATURA |
|---|---|
| Suporte a Linux | ✅ |
| Atalhos de teclado na edição manual de malha | n/a |
| Limite de blendshape até −100% | ✅ morphs sem limite |
| Redimensionamento proporcional; ferramenta de gravidade; travamento de malha a malha | ❌ / ❌ / ❌ |
| Glue para deformadores + pesos de glue precisos | 🟡 pesos ✅ |
| Importar física sem sobrescrever grupos existentes | ❌ |
| Travar parâmetros no animador; edição de loop vs repetição | ❌ |
| Opções "Expand Warp based on Child"; alterar o centro de rotação do warp | ❌ |
| Ferramenta do tipo "Set Vertex Flow", seleção de arestas em loop, mapeamento de malha por traço de densidade variável | ❌ (auto-mesh uniforme) |
| Suporte a pixel art | ❌ |
| Opacidade de vértice; filtragem de partes e exibição mais limpa | ❌ |
| Mover keyframes com facilidade | ✅ patch ops |
| Descartar da memória o atlas em cache | n/a |
| Janela de física: % máximo de saída, edição em massa de faixas de pêndulo | 🟡 |
| RISC-V / Cubism Core embarcado | ❌ |
| Drawables opcionais na especificação CDI3 | n/a |

## Toon Boom Harmony
O fórum de usuários está fechado e redireciona para o Discord (não pesquisável); os antigos tópicos do "Feature Requests board" ("One can only dream: A Feature List", "Software Upgrade Suggestions Part 2", "Bitmap Layer Overhaul LONG OVERDUE", "Various Quality of Life suggestions") mostram apenas o aviso de encerramento. Nenhum pedido recuperável.

## Synfig (lista de ideias do GSoC 2019–2022, números de issues do GitHub vinculados)
| Pedido | OSATURA |
|---|---|
| Copiar/colar waypoints (#2139) | ✅ keys via ops |
| Troca de abas com Ctrl+Tab, atalhos de janela, seleção da área de reprodução, busca de propriedades, alternar réguas, alteração de parâmetro com o mouse | ❌ (GUI) |
| Desempenho: renderização/exportação mais rápidas; cache de parâmetros de camada; árvore incremental de tarefas de renderização | ✅ evaluate de função pura; sem cache ❌ |
| Port para GTK 4, builds Conan/vcpkg | n/a |

## Spine
Os pedidos vão para o fórum e são movidos para um quadro público do Trello (não acessível: robots.txt). Do fórum: a prioridade mais alta historicamente é a estabilidade do editor e as atualizações de runtime. O plano da era do Kickstarter (malhas → FFD → skinning → IK pinning) está agora totalmente entregue, de modo que a malha/FFD/skinning do OSATURA estão em paridade com aquela lista antiga; IK pinning ✅ (restrição Pin). Títulos de pedidos recentes no fórum: suporte ao three.js / React Three Fiber mais recente; suporte ao Spine 4.3 no Defold.

## Rive
Roadmap público = "canal Feature Requests" com rótulos de status (em andamento / planejado / lançado); exige login para leitura. Nada recuperável além das notas de lançamento (rodada 4).

## OpenToonz, TVPaint, Pencil2D
As páginas do GitHub (OpenToonz/Synfig) estão bloqueadas pelo robots.txt; nenhum quadro do TVPaint legível encontrado; existe um tópico "Feature requests to add" do Pencil2D, mas não foi aberto (fora do escopo).

## Adições ao roadmap a partir das listas de desejos
47. Camada auto-animate / idle procedural (respiração, piscar, balanço da cabeça com raridade e intensidade) — desejo n.º 20 do CA; baseia-se na oscilação existente
48. Profundidade de campo da câmera e filtro de blur — CA
49. Luzes (spot/ambiente) com sombras projetadas para camadas de imagem — CA
50. Operações de alinhar / distribuir (op do editor) — CA
51. Codificação por cor dos canais e visibilidade de canais "shy" no studio — CA
52. Várias cenas por projeto + cortes de cena — CA
53. Auto-mesh de densidade variável; suavização de vertex-flow — Live2D
54. Parâmetro de centro de rotação do warp deformer; redimensionamento proporcional — Live2D
55. Importação de física com merge, sem sobrescrever grupos — Live2D
56. Travamento de parâmetros por track durante a edição de loop — Live2D
57. Layout do studio acoplável/personalizado — Moho, CA
58. Cache de parâmetros entre quadros (avaliação incremental) — Synfig

## Fontes (rodada 6)
- Reallusion Feedback Tracker: "Wish list for new features for future updates"; quadro do Cartoon Animator 5 (reallusion.com/FeedBackTracker)
- community.live2d.com — pedidos de recursos do Cubism Editor (página 1 de 7)
- Lista de ideias do Synfig GSoC 2022 (synfig-docs-dev.readthedocs.io)
- Índice do fórum da Lost Marble (lostmarble.net/forum); tópicos 2250 e 8033 do fórum do Spine; tópico 1501 do Kickstarter
- FAQ do Rive: "Does Rive have a public roadmap?"

---

# Adendo — Projetos de código aberto em plataformas git (rodada 7, 2026-10-19)

**Acesso:** a API de busca do GitHub não está disponível neste ambiente ("sessions are bound to their configured repositories"), então os projetos foram encontrados por busca na web e lidos a partir de seus READMEs públicos. As APIs do GitLab e do Codeberg estão bloqueadas pelo proxy; páginas do Codeberg/GitLab apareceram apenas em resultados de busca (o Friction estava legível). Não foi possível obter contagens de estrelas. Vários projetos são muito novos/iniciais — os recursos listados são os alegados nos READMEs, não verificados por execução.

## Ferramentas de código aberto semelhantes de rigging / animação
| Projeto | Plataforma | Recursos notáveis | Comparação com o OSATURA |
|---|---|---|---|
| **Rigby2D** (jaisurya-dev-art/rigby2d) | GitHub, MIT/Apache | Editor no estilo Spine (importação PSD, malha + pintura de pesos, malhas vinculadas, restrições IK/transform/path/physics/**slider**, skins com ossos específicos de skin, eventos, attachments de bounding-box/clipping/point/path, dopesheet + editor de curvas), exportação Spine JSON 3.8/4.2/4.3 + .skel, atlas, runtimes JS/Rust/Godot, **servidor MCP integrado** para assistentes de IA, desktop Tauri | O OSATURA não tem GUI de editor, atlas, runtimes, skins, eventos, restrições slider/path; tem avaliador de função pura, FX de líquido/booleanas/emissor, lip-sync, gerador de caminhada |
| **SkelForm** (Retropaint/SkelForm, Rust) | GitHub | IK FABRIK, wrapper nativo + web + mobile | núcleo de rig semelhante ✅; sem GUI ❌ |
| **Ankhimate editor** | GitHub, pré-alfa | Modos Setup/Animate, slots + skins, FABRIK de qualquer comprimento, restrições de caminho/física, mesh tracing da silhueta a partir do alfa, pincel de pesos com modos de mesclagem + travamento de ossos, **timelines de FFD**, eventos com áudio, marcadores, deslocamentos de track por osso, **editor de formato de exportação baseado em templates**, runtime de referência em Rust | ❌ em grande parte GUI; ideias: templates de exportação, deslocamentos de track por osso |
| **rig2d-rs** (tanhieuno41) | GitHub, Rust | Port da física do Spine 4.2, IK com **mix/softness/bend/compress/stretch/uniform**, pesos de auto-bind por queda de distância, suavizar/normalizar/limitar a 4 pesos, packer de atlas em prateleiras, ghosting de onion skin, importação/exportação Spine JSON | ❌ IK soft/stretch, ferramentas de pesos, atlas, interoperabilidade com Spine |
| **Boner2D**, **Skelly**, **omarvision 2d-bones-IK** | GitHub | Ferramentas esqueléticas para Unity / designers de jogos | básico |
| **Skeletal-Animator** (ostad-ai) | GitHub, Python | ossos + FABRIK, formas retângulo/cápsula/círculo, 4 easings, onion skin, janela de recorte da câmera com keys, **faixas de visibilidade por osso**, spritesheet PNG+JSON, **MP4/WebM/MOV/GIF transparentes** | ❌ faixas de visibilidade, exportação de vídeo/spritesheet transparente, onion skin |
| **Stretchy Studio** (MangoLion) | GitHub, MIT | Importação PSD/PNG, **auto-rigging por IA** (detecção de pose DWPose), wizard de configuração, animação de vértices por malha de parte, skinning de vértices para membros, **auto eye clipping** (íris dentro do olho), **shape keys no estilo Blender** com sliders de influência, faixas de áudio na timeline, **exportação Spine 4.0 JSON** | ✅ morphs ≈ shape keys; ❌ auto-rig, eye clipping, áudio |
| **Iki** (zeikar/iki) | GitHub, MIT | Motor web no estilo Live2D: quads guiados por parâmetros, deformadores warp-mesh/grade, máscaras de stencil, física de mola/cadeia, importação PSD, **auto-rigger a partir de camadas com nomes de papéis** (piscar, olhar, lip-sync, giro de cabeça, aceno, inclinação, sobrancelhas, respiração, física de cabelo), esquema JSON aberto com **IDs StandardParameter**, servidor MCP + plugin do Claude Code | ✅ esquema JSON, lip-sync; ❌ IDs de parâmetros padrão, auto-rigger, máscaras |
| **Umamo** (umamoorg/umamo) | GitHub, Java | Editor compatível com Live2D: lê/grava CMO3/MOC3 (até o Cubism 5.4), importação PSD/KRA/CLIP, paleta de comandos, presets de atalhos de teclado, **CLI de diagnóstico** (dump/extração/conversão/diff de modelos, presets de qualidade de auto-mesh); animação ainda não suportada | ❌ CLI de diff de modelos, paleta de comandos |
| **Ayatsuri2D** | GitHub | Modelo `.aom` em MessagePack (superconjunto do moc3), exportação moc3 v3–6, ABI C, fuzzing em CI, runtime sem panic, benchmarks de latência p99.9 (inicial, "not usable yet") | ❌ formato binário, ABI C, fuzzing |
| **Ayagami** (AyagamiDev) | GitHub, Rust/wgpu | MOC3 até o SDK 5.0 incl. glue e parâmetros de blendshape, recomputação parcial, mesclagem linear/sRGB, mesclagem avançada/off-screen do SDK 5.3, componente para Godot | ❌ renderizador GPU, MOC3 |
| **kasane-2d**, **img2live**, **claude-live-2d**, **minivt** | GitHub | Modelagem no estilo Live2D / pesquisa de imagem única para puppet | n/a |
| **Glaxnimate** (KDE) | espelho do GitHub / KDE | animação vetorial; as dependências de build mostram vetorização bitmap com Potrace, libav, scripting em Python, libarchive; foco em Lottie | ❌ vetorização, scripting |
| **Friction** (friction/friction, Codeberg) | Codeberg, GPL-3 | animação vetorial + raster para web/vídeo, motion graphics, SVG, Skia + FFmpeg + GLSL (baseado no enve) | ❌ animação SVG, shaders |
| **Tahoma2D** | GitHub | 2D + stop-motion, baseado na versão Studio Ghibli do Toonz | ❌ |
| **Pencil2D** | GitHub, GPL-2 | bitmap + vetor desenhado à mão, onion skin | ❌ |
| **q0idmation** (q0s0id) | GitHub | editor + player + **linguagem de scripting** | ❌ scripting |
| **svgMotion** (michaelsboost) | GitHub | animação vetorial no navegador | ❌ |
| **kinetic** | GitHub | timeline de keyframes SVG → exportação CSS/GSAP | ❌ exportação CSS/GSAP |
| **dotlottie-rs** (LottieFiles) | GitHub, Rust | **máquinas de estados** (guards/transições/ações), theming/**slots** (substituições em runtime de cor/escalar/texto/vetor), contêiner zip `.lottie` multianimação, API C + WASM, renderizador ThorVG | 🟡 exportação Lottie ✅; ❌ slots/máquinas de estados/contêiner dotLottie |
| **rive-runtime** | GitHub | Runtime C++: renderizador vetorial GPU (Metal, Vulkan, D3D11/12, GL/WebGL), encapsulado para Apple/Android/Flutter/Unity/Unreal/web | ❌ |
| **spine-runtimes** | GitHub | runtimes para Unity, Godot, Unreal, libGDX, MonoGame, XNA, Flutter, Haxe, C/C++/C#/TS, Android, iOS, GLFW, SDL, SFML; política de um branch por versão do editor | ❌ |
| **Repositório de ferramentas do Moho** (wanderjimenezRD/Moho-Pro-14.4-2D-Animation-Tools) | GitHub | apenas listagem; a descrição não traz recursos — não avaliado | — |

Surgiram repositórios de software pirata (p. ex. "activate-free" do Adobe Animate): não foram abertos nem usados.

## Ideias que vale adotar (novos candidatos ao roadmap)
59. **Servidor MCP / API de agente** sobre o editor de patch ops para que assistentes de IA possam fazer rig e animar (Rigby2D, Iki) — combina com a arquitetura de JSON-patch do OSATURA
60. **IDs de parâmetros padrão** (ParamAngleX, EyeOpen, MouthOpen…) para que qualquer host possa guiar um modelo (Iki)
61. **Auto-rigger a partir de camadas com nomes de papéis**: piscar, olhar, giro/aceno/inclinação da cabeça, sobrancelhas, respiração, física de cabelo (Iki)
62. **Auto eye clipping** (íris mascarada pela forma do olho) (Stretchy Studio) — via camada Boolean
63. **Restrição slider** e **restrição de caminho** (Rigby2D, Spine)
64. Modos de IK **soft / stretchy / compress / uniform** (rig2d)
65. **Ferramentas de pesos**: suavizar, normalizar, limitar a N, copiar, travamento de ossos, modos de mesclagem do pincel (rig2d, Ankhimate)
66. **Mesh tracing da silhueta a partir do alfa** (Ankhimate)
67. **Faixas de visibilidade por osso** (Skeletal-Animator)
68. **Exportação de vídeo com fundo transparente (WebM/MOV com alfa) e spritesheet PNG+JSON** (Skeletal-Animator)
69. **Motor de templates de exportação** (formatos de saída definidos pelo usuário) (Ankhimate)
70. **Contêiner dotLottie `.lottie` + slots/theming em runtime + máquinas de estados** (dotlottie-rs)
71. **CLI de diff / dump / conversão de modelos** (`osatura diff a.json b.json`) (Umamo)
72. **Paleta de comandos + atalhos reconfiguráveis** no studio (Umamo)
73. **Loader com fuzzing, caminho de runtime sem panic, benchmarks de latência** (Ayatsuri2D) — reforço de robustez de `Engine::new`
74. **Importação/exportação Spine JSON** para interoperabilidade (Rigby2D, rig2d, Stretchy Studio)
75. **Packer de atlas em prateleiras** para camadas de imagem (rig2d, Rigby2D)
76. **Eventos com valores com keys e dicas de áudio** na timeline (Rigby2D, Ankhimate)
77. **Deslocamentos de track por osso / time-shift** (Ankhimate)

## Fontes (rodada 7)
- github.com/jaisurya-dev-art/rigby2d; Retropaint/SkelForm; Ankhimate/editor; tanhieuno41/rig2d-rs (release v0.1.0)
- github.com/zeikar/iki; MangoLion/stretchystudio; umamoorg/umamo; Yoru-KenomoVT/ayatsuri_2d; AyagamiDev/ayagami
- github.com/ostad-ai/Skeletal-Animator; KDE/glaxnimate; tahoma2d/tahoma2d; pencil2d/pencil
- github.com/rive-app/rive-runtime; EsotericSoftware/spine-runtimes; LottieFiles/dotlottie-rs
- github.com/topics/animation-editor, /topics/vector-animation
- codeberg.org/friction/friction

---

# Adendo — Outras fontes públicas (rodada 8, 2026-10-19)

Fontes: itch.io, Steam, AlternativeTo, Wikipedia, resultados de busca acadêmica/de patentes, análises de ferramentas do After Effects. As afirmações vêm de páginas de lojas/projetos, não de testes práticos.

## Ferramentas de rigging indie / de nicho (itch.io, Steam)
| Ferramenta | Recursos notáveis | OSATURA |
|---|---|---|
| **Inochi Creator / Inochi2D** (Kitsunebi Games, código aberto, beta) | rigging de puppets 2D aberto voltado a VTubers + runtime em tempo real; formato próprio (não compatível com Live2D/Spine); exportação GIF/sequência de imagens; UI multilíngue | 🟡 mesmo nicho (formato de rig aberto) |
| **Keyframe.it** (navegador, gratuito) | ossos + IK + malhas com pesos + movimento de mola; **clipes de símbolos de jogo** (idle, land, anticipation, win); editor de imagens integrado + importação PSD; exporta GIF, sprite sheet, vídeo, player HTML autônomo, **pacotes Spine 3.8/4.2**; salvamento automático no navegador; conector de agente de IA **keyframe-mcp**; assistente de IA integrado opcional | ✅ player HTML/GIF; ❌ sprite sheet, pacote Spine, MCP, editor no navegador |
| **RigBone 2D** (Godot) | pose de ossos, pose-como-quadro, onion skin, recorte + fundo transparente, sequência de quadros PNG (sem necessidade de runtime) | ❌ onion skin, sequência PNG com recorte de alfa |
| **SpineBones** | slots + attachments de imagem, grupos, IK, **prévia de áudio na timeline**, imagem de referência de fundo, exportação JSON / sprite sheet / WebM | ❌ |
| **PySpine, Tangent (sprite animator)** | listados no itch.io; nenhum texto de recursos lido | — |
| **Mocap2D / FBX2SPINE** (Steam) | importação de mocap FBX/BVH; **Auto Match** de esqueleto por nomes + hierarquia; **Smart Retargeting** (projeção, bom para vista lateral); **Direct Retargeting** por eixo; **conversão automática in-place** para loops de locomoção; **IK de contato com o solo** (foot lock); pré-visualização ao vivo; importação/exportação de FBX, BVH, esqueletos Spine, Unity 2D Animation | 🟡 retarget por nome ✅; ❌ importação de mocap, IK foot-lock, conversão in-place |
| **VTube Studio** (Steam) | rastreamento por webcam (OpenSeeFace), **boca guiada por voz**, mouse/caneta como entrada de rastreamento, mapeamento de qualquer entrada → qualquer parâmetro do modelo, atalhos para expressões/animações, "itens" PNG fixados que seguem o modelo; somente modelos Live2D | 🟡 canais de parâmetros ✅; ❌ mapeamento de entrada ao vivo, atalhos, itens fixados |
| VBridger, Vitamins | Ferramentas do Steam em torno de VTubing; não avaliadas | — |

## Ecossistema de rigging do After Effects (análise da School of Motion)
| Recurso | Fonte | OSATURA |
|---|---|---|
| **Controlador joystick** (5 poses-chave: origem/direita/esquerda/cima/baixo) interpoladas em 2D | Joysticks 'n Sliders | ❌ — forte candidato |
| **Sliders** que misturam poses ilimitadas | JnS, Duik | 🟡 canais de morph |
| Estruturas (ossos independentes da arte, reutilizáveis), autorig | Duik Bassel | ✅ |
| Controladores: slider, slider 2D, ângulo | Duik | ❌ |
| **Connector** (slider/joystick/rotação ligando qualquer propriedade a outra), controle de propriedade mestre | Duik | 🟡 expressões? (motor de expressões ✅ nos canais) |
| Keys de pose intermediária entre estados de pose | Duik | ✅ |
| Animar caminhos de máscara | JnS | ✅ morphs de caminho |

## Aplicativos concorrentes / alternativos (AlternativeTo, lista da Wikipedia em 2026)
Tahoma2D (fork do OpenToonz, stop-motion), TupiTube, enve (descontinuado; tornou-se Friction), Expressive Animator (SVG interativo), Cavalry, Jitter (motion design web), Blazy (Figma → animação, exportação CSS/React), Animation Desk, Pivot Animator, FlipaClip, animação do Krita, Blender (mar 2026), Clip Studio Paint (animação), Adobe Character Animator (puppeteering por webcam; página "what's new" bloqueada pelo robots.txt), Adobe Animate (última versão listada: out 2023), CrazyTalk, DigiCel FlipBook, Apple Motion, RETAS. Nenhum acrescenta recursos de rigging além do que as rodadas anteriores cobrem; ideias novas: **presets de ferramenta de design (Figma) → animação com exportação CSS/React** (Blazy) e **puppeteering por webcam** (Character Animator, VTube Studio).

## Literatura acadêmica e de patentes (somente títulos/resumos)
- Jacobson et al. 2011, **Bounded Biharmonic Weights** para deformação em tempo real — pesos de skinning suaves automáticos a partir de uma cage/esqueleto ✅ ideia: melhorar `auto_skin` (atualmente baseado em distância).
- Patentes da Adobe US9865073 / US10078910, **"Computer animation of artwork using fast 2-dimensional deformation solving"** (o mesh warp do Character Animator). *Nota clean-room:* o OSATURA usa FFD de Bernstein e skinning com pesos, não esse solver patenteado; qualquer futuro solver no estilo ARAP deve ser revisado com um advogado. Isto não é aconselhamento jurídico.
- Auto-rigging em arXiv 2025–26: **ASMR** (esqueleto + skin de malha 3D via priors generativos 2D), UniRig, "How to Train Your Dragon" (rigging por difusão), HumanRig, SPRig, UniMate — todos 3D/ML; relevantes apenas como um futuro recurso de "sugestão de auto-rig".
- Stony Brook TVC 2017 **retargeting automático de pesos de skinning** — copiar pesos entre malhas (✅ roadmap: cópia de pesos).

## Novos candidatos ao roadmap desta rodada
78. **Controlador joystick**: ponto de controle 2D que mistura 5 poses-chave (e variantes de N poses) → guia quaisquer canais — JnS / Duik / parâmetros do Live2D
79. Importação de mocap (BVH) + esqueleto de auto-match + conversão de loop in-place + IK foot-lock — Mocap2D
80. Mapeamento de entrada ao vivo: qualquer entrada (mouse, MIDI, nível de áudio, landmarks de webcam) → parâmetro, mais atalhos para expressões — VTube Studio
81. Itens/props fixados que seguem um osso/âncora com deslocamento próprio — VTube Studio
82. Presets de clipes de símbolos de jogo (idle / land / anticipation / win) — Keyframe.it
83. Exportação de spritesheet (PNG + JSON) e de sequência PNG transparente com recorte — Keyframe.it, RigBone
84. Prévia de áudio na timeline com forma de onda — SpineBones
85. Camada de imagem de referência (fundo não renderizado) — SpineBones
86. Pesos de auto-skin biharmônicos limitados ou por difusão de calor (substituir a queda por distância) — artigo BBW
87. Presets de ferramenta de design → exportação CSS/React — Blazy
88. Puppeteering por rastreamento facial com webcam via studio (WebRTC + modelo de landmarks) — Character Animator, VTube Studio

## Fontes (rodada 8)
- itch.io: kitsunebi-games/inochi-creator, keyframeit.itch.io, itch.io/blog/1586091 (RigBone 2.0), itch.io/t/6177139 (SpineBones)
- Steam: FBX2SPINE / Mocap2D (app 2467960), VTube Studio (app 1325860)
- schoolofmotion.com — Joysticks 'n Sliders vs Duik Bassel
- alternativeto.net (alternativas ao Moho, ao Expressive Animator); en.wikipedia.org/wiki/List_of_2D_animation_software
- Jacobson 2011 BBW (homes.cs.washington.edu/~jovan/papers/jacobson-2011-bbw.pdf); patentes US9865073B2 / US10078910; arXiv 2503.13579, 2503.15586, 2504.12451; retargeting de pesos Stony Brook TVC 2017

---

# Adendo — Adobe Character Animator, conjunto completo de recursos (rodada 9, 2026-10-19)

Fontes: tabela de versões da Wikipedia (incl. 26.0 de jan 2026), página de produto da Adobe, página "Behaviors" da Ajuda da Adobe e notas de lançamento (out 2018, out 2020/fev 2021, out/dez 2021, 2023). O índice "what's new" da própria Adobe retorna apenas 2023 e 2026; 24.x e 25.x são listadas pela Wikipedia como versões somente de estabilidade.

## Conceito
Arte em camadas do Photoshop/Illustrator (também EPS/SVG desde out 2018) torna-se um **puppet** com rig automático a partir de **nomes/tags de camadas**; a performance é capturada ao vivo (webcam facial/corporal, microfone, teclado, mouse, MIDI) como **takes** em uma timeline; as saídas vão para sequências PNG, WAV, Media Encoder (incl. **vídeo com alfa**), Dynamic Link para After Effects / Premiere e Syphon / Mercury Transmit ao vivo. O modo Starter é gratuito; o Pro exige Creative Cloud.

## Behaviors (lista completa da Ajuda da Adobe)
| Behavior | O que faz | OSATURA |
|---|---|---|
| **Face** | cabeça, olhos, sobrancelhas, nariz, boca a partir da webcam; parallax e intensidade da posição da cabeça | ❌ (sem rastreamento ao vivo) — roadmap n.º 88 |
| **Body** (Body Tracker, Sensei AI, 22.0) | braços, tronco, pernas a partir da webcam | ❌ |
| **Lip Sync** | visemas da boca a partir da voz; precisão assistida por transcrição (22.1); engine 3.4 melhor em F, W-Oo, menos L/M | 🟡 WAV → 5 visemas ✅; alinhamento de transcrição ❌ |
| **Nutcracker Jaw** | mandíbula inferior a partir do rosto/voz | ❌ |
| **Eye Gaze** | pupilas se movem de forma independente | ✅ restrições aim/canais |
| **Head & Body Turner** | troca grupos de camadas (frontal, três-quartos, lateral) conforme a cabeça/corpo giram | 🟡 camadas Switch ✅; sem troca automática guiada por ângulo |
| **Walk** | ciclo de caminhada, corrida, ênfase de pose, balanço de ombros/quadris, **Pin Feet When Standing** (3.4/3.5) | ✅ gerador de caminhada; ❌ pin-feet |
| **Limb IK** (antes Arm IK) | direção de dobra + alongamento para braços e pernas | 🟡 IK de 2 ossos ✅; alongamento ❌ |
| **Leader/Follower** (22.5) | molda como os membros dobram/alongam | ❌ |
| **Dragger** | mouse/toque move uma região | ❌ (o studio não é interativo no canvas) |
| **Physics / Dangle** | balanço, colisão, gravidade, vento; squash & stretch, amortecimento | 🟡 molas ✅; colisão/vento ❌ |
| **Magnets** | alças se encaixam por proximidade | ❌ |
| **Particles** | emite objetos físicos (neve, tiros de canhão) | ✅ emissor |
| **Motion Lines** | linhas de rastro atrás de objetos rápidos | ❌ |
| **Wiggler** | oscilação em arte vetorial | ✅ wobble/boil |
| **Breath** | movimento de respiração | 🟡 canal manual; roadmap n.º 61 |
| **Auto Blink** | piscadas automáticas / alternância de camadas | ❌ (roadmap n.º 61) |
| **Motion Library** (23.0, mais de 350 movimentos, parâmetro de intensidade 23.1) | aplicar animações de corpo inteiro | 🟡 apenas caminhada; ❌ biblioteca |
| **Cycle Layers** | percorrer as camadas de um grupo | 🟡 camadas Switch ✅; ciclo de quadros ❌ |
| **Layer Picker / Motion Trigger / Fader** | mostrar uma camada; trocar pela direção do movimento; fade ao acionar | 🟡 |
| **Transform** | mover/escalar/girar/opacidade de alças | ✅ |
| **Handle Fixer** | travar partes de uma malha | ✅ pins |
| **Triggers** | tecla pressionada mostra camadas/gestos; **painel Controls** com sliders, dials de ângulo, botões, mapeamento MIDI | ❌ — roadmap n.º 80 |
| **Collide, Dynamic, Cycle Pause, Auto-swap (22.5)** | tags/behaviors para física e trocas do tipo quadro | ❌ |

## Recursos de timeline, gravação e fluxo de trabalho
| Recurso | Versão | OSATURA |
|---|---|---|
| Takes, replays (gravações reutilizáveis acionáveis), smart replays, replays editáveis | 2.0 / 22.0 / 23.6 | 🟡 clipes ✅; replays ❌ |
| Keyframes, esticamento de tempo de keyframes, câmeras de cena, áudio acionável | 3.0 / 3.2 | ✅ keys/câmera; ❌ áudio acionável |
| Speech-Aware Animation (movimento de cabeça/sobrancelhas a partir da voz, Sensei) | 3.4 | ❌ — roadmap n.º 61 |
| Mesclar/sequenciar takes; linhas de take coloridas; filtros de busca (`kind:` `shared:` `armed:`); linhas shy; recolher takes | 3.4 | ❌ |
| Snap a quadros / objetos; snapping na timeline | 3.5 | ❌ |
| Operações na área de trabalho: ripple delete, inserir tempo, loop, aparar, exportar | 2.0 | 🟡 operações de clipe ✅ |
| Painel de problemas de rigging; busca de tags e triggers | 3.0 | ✅ a validação coleta todos os erros |
| Marcadores de histórico; recuperação automática de projeto danificado (4.2); novo formato de arquivo compacto/resistente a corrupção (4.0) | | 🟡 desfazer ✅; recuperação ❌ |
| Puppet Maker (templates: cabelo, pele, acessórios) | 22.1.1 | ❌ |
| Characterizer (transformar uma foto/pintura em puppet) | 2.0 | ❌ |
| Quick Export para compartilhamento em redes sociais | 22.5 | 🟡 GIF/MP4 ✅ |
| Snapshots de cena, contagem regressiva de gravação, arte personalizada de botões de controle | 1.5 | ❌ |
| Seleção de câmera, multimonitor, HiDPI | 2.0 | n/a |
| Saída ao vivo (Syphon, Mercury Transmit), extensão Twitch Character Trigger | 1.1 / 2.1 | ❌ — roadmap n.º 43 |
| MIDI, scripts via V8 (JavaScript) | 1.1 / 2.0 | ❌ — roadmap n.º 31 |
| Compartilhar/publicar puppets, arquivos de puppet compartilháveis | Beta 5 | 🟡 JSON portátil |
| Dynamic Link para AE / Premiere | Beta 5 | ❌ |
| Pontos fracos relatados (análise): curva de aprendizado íngreme, dependência de hardware (webcam/microfone), longo tempo de preparação do personagem, Pro pago, complexidade da configuração de triggers | | oportunidade: pipeline offline determinístico do OSATURA, CLI/JSON |

## O que se destaca para o OSATURA (novos candidatos ao roadmap)
89. **Auto-rig baseado em tags a partir de nomes de camadas** (cabeça, olhos, sobrancelhas, visemas da boca, tags de braço/perna) — ideia central do Character Animator; estende o roadmap n.º 61
90. **Takes e replays**: gravar a performance de parâmetros como clipes reutilizáveis e acionáveis, com easing de mistura
91. **Pin Feet When Standing** e travamento de contato com o solo para o gerador de caminhada
92. Comportamentos de alongamento de membros **Leader/Follower**; **Magnets**; **Motion Lines**
93. Formato de **Motion Library** (clipes de movimento nomeados com intensidade de 0 a 100%) 
94. **Head & Body Turner**: troca automática de grupos de vista a partir de um canal de ângulo yaw
95. **Lip-sync assistido por transcrição** (alinhamento forçado de texto com os fonemas do WAV)
96. **Movimento de cabeça/sobrancelhas sensível à fala** a partir da energia e das pausas do áudio
97. **Exportação de vídeo com alfa** (ProRes 4444 / WebM VP9 com alfa) — também no Skeletal-Animator (roadmap n.º 68)
98. **Áudio acionável** e parâmetros reativos ao áudio

## Fontes (rodada 9)
- en.wikipedia.org/wiki/Adobe_Character_Animator (lista de recursos e tabela de versões até a 26.0, jan 2026)
- adobe.com/products/character-animator.html
- helpx.adobe.com — Behaviors in Character Animator; resumos de recursos out/dez 2021, out 2020/fev 2021; What's new (2023–2026)
- community.adobe.com — "Character Animator October 2018 detailed changes"
- manytools.com/review/adobe-character-animator (pontos fortes/fracos)
