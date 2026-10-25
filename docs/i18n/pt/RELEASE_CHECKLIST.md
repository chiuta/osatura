<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Checklist de lançamento (lado do proprietário)

*Etapas que somente o proprietário pode realizar, em ordem. Situação em 24 out 2026: o repositório <https://github.com/chiuta/osatura> existe, o GitHub Actions está verde (testes em Linux, macOS e Windows, clippy, wasm32, cargo-deny, REUSE) e o pré-lançamento em rascunho `v0.1.0-alpha.3` foi compilado (arquivos para Windows, macOS e Linux, o módulo web e `SHA256SUMS.txt`); **nada foi publicado ainda**. As etapas abaixo que o CI já cumpriu estão marcadas; as demais precisam de uma pessoa, de um advogado ou de uma conta. Isto não é aconselhamento jurídico. Os itens marcados **[blocker]** devem ser feitos antes de um anúncio público.*

## 1. Antes de publicar qualquer coisa

- [ ] **[blocker]** Confirme ou substitua o contato de segurança (`alexio@trom.tf` é um placeholder em `docs/SECURITY.md`, em `--version` e nos documentos de licença).
- [ ] **[blocker]** Pesquise o nome "OSATURA" nos registros de marcas de seu interesse (EUIPO, USPTO, OSIM) e em lojas de aplicativos / registros de pacotes. Veja `TRADEMARK.md`. Se houver conflito, mude o nome antes da primeira versão (o nome aparece no binário, nos crates e na documentação).
- [ ] **[blocker]** Peça a um advogado que revise `LICENSING.md`, a divisão MPL-2.0 / AGPL-3.0, `docs/THIRD_PARTY.md` e `docs/PATENT_NOTES.md`. Decida se você quer licenciamento duplo; se sim, é necessário um CLA *antes* de aceitar contribuições externas (hoje: apenas DCO).
- [ ] Verifique os termos da referência de ajuste de curvas do Graphics Gems (citada de memória em `CLEANROOM.md`); mantenha a redação de implementação independente somente se ela for precisa.
- [x] `reuse lint` e `cargo deny check` passam no CI (o job `licences`); execute-os também uma vez em sua máquina.
- [ ] Verifique se não há dados pessoais, caminhos locais ou tokens na árvore (`grep -r "/home/" .`, `grep -ri token`), e se o modelo incluído e os assets de demonstração são todos autogerados.

## 2. Repositório

- [x] O repositório no GitHub existe e a árvore foi enviada sem `target/` e `out/` (`.gitignore`).
- [ ] Mantenha `COPYRIGHT`, `LICENSES/`, `REUSE.toml`, `.github/FUNDING.yml` (Buy Me a Coffee `echo.of.the.strings`, Patreon `alexio_tf`).
- [ ] Ative o relato privado de vulnerabilidades (aba Security) e aponte `docs/SECURITY.md` para ele.
- [ ] Passe a URL do repositório com `--source-url` ao executar o studio (oferta de código-fonte da AGPL, rota `/source`) e teste a oferta a partir de uma instalação nova.

## 3. Primeira execução do CI (concluída em 2026-10-24)

- [x] `.github/workflows/ci.yml` está verde: testes em Linux, Windows e macOS e `clippy -D warnings`; o job de build wasm32; o job de cargo-deny e REUSE.
- [x] As falhas da primeira execução no Windows foram corrigidas (tamanho da pilha, testes exclusivos de POSIX, acesso ao log de colaboração) e registradas em `docs/LIMITATIONS.md` (L-04).
- [x] O job wasm32 passa (build e teste do módulo real no Node). **Em aberto:** execute a lista de verificação de `docs/WASM.md` em um navegador real e atualize L-01 / L-02.

## 4. Testes com pessoas

- [ ] Dê o studio a 3 a 5 animadores por uma semana, com uma tarefa curta (fazer o rig de um personagem, criar um ciclo de caminhada, exportar um GIF). Registre onde eles travam; abra cada caso como uma issue.
- [ ] Teste em navegadores reais (Chrome, Firefox, Safari), em um dispositivo de toque e em uma sessão apenas com teclado.
- [ ] Em um **Windows 11** real e em um **Mac** real: descompacte o arquivo, dê um duplo clique em `osatura.exe` (Windows) e em `osatura-studio`, faça o rig e exporte um projeto, e relate o que falhar (L-74 a L-76).
- [ ] Experimente os recursos de detector de rostos / mocap / voz com dados reais; informe com honestidade que eles são limitados (L-12, L-13).

## 5. Segurança da implantação (se você hospedar o studio ou a colaboração)

- [ ] Coloque-o atrás de um proxy reverso que termine o TLS (receita: `docs/SECURITY.md` seção 12); nunca exponha a porta HTTP simples à internet.
- [ ] Use tokens por participante (`collab-token`), defina expiração e faça o bind em localhost atrás do proxy.
- [ ] Contrate uma revisão de segurança independente antes de oferecer colaboração hospedada a estranhos.

## 6. Artefatos de lançamento

- [x] Os binários de release são compilados em runners hospedados pelo GitHub (`release.yml`) e `SHA256SUMS.txt` é gerado no rascunho. **Em aberto:** revise o rascunho `v0.1.0-alpha.3`, marque "pre-release", publique-o e crie a tag do commit (a tag é criada junto com o lançamento).
- [ ] Decida sobre a assinatura de código (certificado Authenticode do Windows, Apple Developer ID e notarização); até lá, os downloads são sinalizados como não assinados (L-74).
- [ ] Anexe o código-fonte correspondente das partes AGPL (a própria tag basta; mantenha-o disponível enquanto você distribuir binários).
- [x] `docs/THIRD_PARTY_NOTICES.md` foi regenerado (24 out 2026) e cada arquivo o inclui; regenere de novo com `scripts/licence-audit.sh` antes de cada lançamento posterior.
- [ ] Crates.io / npm / registros de pacotes: verifique se os nomes estão livres e se os campos de licença correspondem a `LICENSING.md`.

## 7. Anúncio e doações

- [ ] Primeira tela do README: o que funciona, o que não funciona (link para `docs/LIMITATIONS.md`); não afirme "classe Moho" sem ressalvas.
- [ ] Mostre as capturas de tela em `docs/img/` e um GIF curto feito com o projeto de demonstração.
- [ ] Adicione os links de doação (Buy Me a Coffee, Patreon) ao README e à caixa Sobre do studio; não prometa recursos em troca de doações, a menos que você consiga cumprir a promessa.
- [ ] Divulgue onde os animadores estão (comunidades de rigging 2D / animação indie); peça relatos de bugs, não elogios.
- [ ] Reserve tempo para responder rapidamente às primeiras issues; as duas primeiras semanas decidem se os contribuidores ficam.
