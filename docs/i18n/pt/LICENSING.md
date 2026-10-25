# OSATURA — licenciamento (DECIDIDO: Variante A, aplicada em 22 de out. de 2026; veja COPYRIGHT e LICENSES/)

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/LICENSING.md) · [Română](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/ro/LICENSING.md) · [Français](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/fr/LICENSING.md) · [Italiano](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/it/LICENSING.md) · [Español](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/es/LICENSING.md) · [Português](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/pt/LICENSING.md) · [Deutsch](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/de/LICENSING.md) · [Magyar](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/hu/LICENSING.md)

**Situação:** a Variante A abaixo foi escolhida e aplicada: MPL-2.0 para as bibliotecas (core, capi, wasm, gpu), AGPL-3.0-or-later para `osatura-cli` (CLI + studio) e `osatura-gui` (o estúdio em uma janela nativa), CC-BY-4.0 para a documentação e para os assets e modelos sintéticos, política de marca em TRADEMARK.md, DCO em CONTRIBUTING.md, cabeçalhos SPDX em todos os arquivos .rs/.js/.mjs/.sh e REUSE.toml para os demais, oferta de código-fonte da seção 13 da AGPL (`/source`, `--source-url`), campos `license` do Cargo definidos. Ações restantes do proprietário: revisão por um advogado, busca de marca (EUIPO/OSIM), triagem de patentes (docs/PATENT_NOTES.md), um CLA caso se queira licenciamento duplo, incluir os avisos de terceiros nas versões binárias.


*Isto não é aconselhamento jurídico. A licença está DECIDIDA (Variante A, veja COPYRIGHT); as seções abaixo são mantidas como justificativa e como registro das alternativas consideradas. Peça a um advogado (de preferência alguém que conheça o direito autoral romeno/da UE) que confirme antes da primeira versão pública. Contribuições: não existe direito de relicenciamento nem de licenciamento duplo sobre contribuições de terceiros enquanto não houver um CLA (CONTRIBUTING.md).*

## 1. O que a licença precisa fazer por você
Sua renda vem de **doações (Buy Me a Coffee)**, não de taxas de licença. Portanto, a licença não cria o dinheiro — **boa vontade, visibilidade e confiança criam**. Ela tem três funções:
1. **Maximizar os usuários** (mais usuários → mais apoiadores): ela deve ser livre para uso, inclusive em trabalho comercial.
2. **Impedir que uma empresa pegue o código, o feche e o venda** (ou o hospede como serviço pago) sem dar nada em troca.
3. **Preservar seu direito de vender exceções** (licença comercial, suporte) e de defender o nome.

Ela também deve se adequar às suas convicções **trade-free / TROM**: o software não deve ser limitado de propósito, financiado por anúncios nem atrelado à coleta de dados. (Licenças que proíbem o uso comercial, como CC-NC ou Polyform-NC, *não* são de código aberto e não são o mesmo que "trade-free"; verifique a redação do próprio TROM antes de adotar qualquer uma delas.)

## 2. Constatações que restringem a escolha
- **As dependências não são um obstáculo.** Auditoria de todos os crates de terceiros (363 na regeneração de 24 de out. de 2026 por `scripts/licence-audit.sh`, contados em todas as plataformas juntas depois que o aplicativo de janela foi adicionado; eram 160 em 22 de out.; o número muda, veja docs/THIRD_PARTY.md): todos são MIT, Apache-2.0, BSD, Zlib, ISC, Unlicense, CC0, Unicode ou licenças duplas dessas, exceto seis crates MPL-2.0 de copyleft fraco, com copyleft no nível de arquivo: smartstring (MPL-2.0+, alcançado apenas pelo recurso `scripting`, que vem ATIVADO por padrão em `osatura-core` e `osatura-cli`) e `cssparser`, `cssparser-macros`, `dtoa-short`, `selectors` e `option-ext` (alcançados apenas pelo aplicativo de janela `osatura-gui`); uma compilação das bibliotecas com `--no-default-features` não tem nenhum crate MPL de terceiros. Eles podem ser combinados com AGPL, MPL ou uma licença proprietária; o crate MPL exige que seu código-fonte/aviso seja preservado.
- **Titularidade dos direitos autorais.** Boa parte do código foi escrita com o Claude. O direito autoral geralmente se vincula à contribuição criativa *humana* (orientação do US Copyright Office; a lei da UE/romena exige igualmente a criação intelectual própria de um autor humano). Consequências práticas: (a) você pode e deve licenciar o projeto como seu autor/compilador humano, mas (b) não prometa exclusividade sobre cada linha, (c) mantenha no repositório suas próprias notas de design, seleções e edições como prova de autoria, (d) um DCO apenas certifica a origem; para relicenciar contribuições mais tarde (por exemplo, licenciamento duplo), é necessário um CLA ou uma concessão de saída explícita, e isso NÃO existe hoje.
- **Nenhuma licença pode forçar doações.** Nunca condicione o código a uma doação; um fork simplesmente removeria a trava. Venda *conveniência e extras*, não permissão.

## 3. Pacote recomendado — "Biblioteca de núcleo aberto, aplicativo protegido" (todas as variantes abaixo estão prontas para aplicação)

### ★ Variante A (recomendada): biblioteca MPL-2.0 + aplicativos AGPL-3.0 + marca
| Parte | Licença | Por quê |
|---|---|---|
| `osatura-core` (motor, avaliador, formatos, exportadores) e qualquer futuro runtime/WASM/C-ABI | **MPL-2.0** | Copyleft no nível de arquivo: qualquer pessoa pode incorporá-lo em jogos/aplicativos (fechados ou abertos), mas as alterações em *nossos arquivos* devem ser compartilhadas. Inclui uma **concessão de patente** explícita e rescisão em caso de ação judicial de patente. Maximiza a adoção, que é o que as doações exigem. |
| `osatura-cli`, `osatura-gui`, studio (navegador/janela) | **AGPL-3.0-or-later** | Impede forks fechados e clones de "OSATURA como serviço web pago"; os *projetos e renderizações* dos próprios usuários continuam inteiramente deles. |
| Nome "OSATURA", logotipo | **Política de marca** (arquivo separado) | Os forks devem mudar de nome; protege os apoiadores contra imitações. Verifique a disponibilidade (EUIPO/OSIM/WIPO) antes do lançamento. |
| Rigs de exemplo, arte, áudio que você criar | **CC BY 4.0** (ou CC BY-SA) | Livres para reutilização com crédito; a arte de terceiros mantém sua própria licença. |
| Documentação | **CC BY 4.0** | |
<!-- REUSE-IgnoreStart -->
Cargo: `license = "MPL-2.0"` (core), `license = "AGPL-3.0-or-later"` (cli). SPDX header per file: `// SPDX-License-Identifier: MPL-2.0`.
<!-- REUSE-IgnoreEnd -->
**Licenciamento duplo:** possível apenas para código escrito somente pelo proprietário, ou após um CLA/concessão de saída de todos os contribuidores. NÃO é possível hoje para código contribuído (CONTRIBUTING.md).

### Variante B: AGPL-3.0-or-later em tudo (+ licença comercial sob solicitação)
Posição mais forte contra apropriação indevida e mais próxima do ethos trade-free. Custo: empresas e motores de jogos evitarão incorporar o runtime, resultando em menos usuários e menos apoiadores indiretos. Escolha se a proteção importar mais que o alcance.

### Variante C: Apache-2.0 em tudo (não escolhida; o workspace aplica a Variante A: bibliotecas MPL-2.0 e `license = "AGPL-3.0-or-later"` para `osatura-cli`)
Alcance máximo e nenhum atrito; concessão de patente incluída. Custo: qualquer pessoa — inclusive uma grande empresa — pode fechá-lo e vendê-lo sem nenhuma obrigação. As doações passam a depender puramente da boa vontade. Escolha apenas se quiser ser um serviço de utilidade pública.

### Variante D: Código disponível (BUSL-1.1 ou Polyform Shield/NC com data de abertura adiada)
Impede a concorrência comercial por N anos. Custo: **não é código aberto**; entra em conflito com a história de clean-room/formato aberto, com as contribuições da comunidade e com os valores trade-free; muitos usuários e distribuições a recusarão. **Não recomendada.**

| | Alcance | Proteção contra apropriação | Concessão de patente | Receita de licenciamento duplo | Adequação ao trade-free |
|---|---|---|---|---|---|
| **A núcleo MPL + aplicativos AGPL** | alto | boa | sim | sim | sim |
| B AGPL em tudo | médio | a mais forte | sim (v3) | sim | sim |
| C Apache em tudo | o mais alto | nenhuma | sim | não | em parte |
| D código disponível | baixo | forte | n/a | sim | não |

## 4. Como realmente ganhar com doações (tudo compatível com a Variante A)
1. **Página do Buy Me a Coffee** (`buymeacoffee.com/echo.of.the.strings`) com link no README, no texto de `--version` da CLI, no rodapé do studio e na linha de crédito do player HTML (um crédito *removível* — nunca obrigatório).
2. **Assinaturas, não paywalls:** níveis mensais para versões antecipadas, votação do próximo item do roadmap, nome em `SUPPORTERS.md`, chat privado. Todos recebem o mesmo código; os membros recebem *tempo e voz*.
3. **Extras pagos que não são o código:** binários/instaladores assinados e prontos para executar, pacotes de assets (rigs, cenários, bibliotecas de ciclos de caminhada) sob uma licença que permita o uso em filmes/jogos, mas não a revenda do próprio pacote, videoaulas, workshops.
4. **Encomendas e suporte:** serviços de rigging, exportadores sob medida, consultoria de integração; a licença comercial dupla para quem incorpora o código.
5. **Níveis de patrocínio para organizações** (logotipo no README) e candidaturas a fundos de código aberto/criativos (NLnet, programas do tipo Sovereign Tech, fundos de cultura) — eles pagam exatamente por esse tipo de infraestrutura aberta.
6. **Página de transparência:** publique a receita e o que ela financiou; isso aumenta de forma mensurável o apoio recorrente.
7. Mantenha o **Patreon** (`patreon.com/c/alexio_tf`) e o BMC ambos visíveis, mas dê a cada um uma finalidade distinta (BMC para agradecimentos pontuais; Patreon para apoio recorrente) para que os apoiadores não fiquem confusos.

## 5. Checklist para aplicar a Variante A (≈ 1 hora de trabalho depois de escolhida)
- [x] `LICENSES/MPL-2.0.txt`, `LICENSES/AGPL-3.0-or-later.txt`, `LICENSES/CC-BY-4.0.txt`, `LICENSES/LicenseRef-third-party.txt` (estrutura REUSE) mais o resumo `COPYRIGHT`; `REUSE.toml` cobre os arquivos que não são código-fonte.
- [x] `license =` definido em todos os seis arquivos `Cargo.toml`; cabeçalhos SPDX em todos os arquivos .rs/.js/.mjs/.sh, verificados por `osatura-cli/tests/spdx_audit.rs`.
- [x] `docs/THIRD_PARTY.md` e `docs/THIRD_PARTY_NOTICES.md`, ambos regenerados por `scripts/licence-audit.sh`.
- [x] `CONTRIBUTING.md` com o texto do DCO 1.1. Ainda é necessário um CLA antes de qualquer licença dupla.
- [x] `TRADEMARK.md` (permitido: "compatible with OSATURA", redistribuição sem modificações; não permitido: compilações modificadas sob o mesmo nome).
- [x] `.github/FUNDING.yml` (`buy_me_a_coffee: echo.of.the.strings`) para que a plataforma mostre um botão Sponsor.
- [~] Busca de marca: o primeiro nome, ARMATURA, colidiu com uma marca nas classes 9/42 (EUIPO, 24 out 2026), por isso o projeto foi renomeado para OSATURA; a busca no EUIPO por "osatura" não encontrou conflito (um resultado sem relação, expirado, na classe 10). Os nomes de pacote `osatura` no crates.io, npm e PyPI estão livres (verificado em 24 out 2026), mas existe uma marca próxima, OSATRA (Open-Xchange AG, registro US 7459236, classes 9/38/42). As buscas no OSIM / TMview / OMPI e de domínio e um parecer de advogado continuam em aberto, veja [docs/LIMITATIONS.md](LIMITATIONS.md) L-78.
