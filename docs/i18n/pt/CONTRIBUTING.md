# Contribuindo para o OSATURA

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/CONTRIBUTING.md) · [Română](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/ro/CONTRIBUTING.md) · [Français](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/fr/CONTRIBUTING.md) · [Italiano](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/it/CONTRIBUTING.md) · [Español](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/es/CONTRIBUTING.md) · [Português](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/pt/CONTRIBUTING.md) · [Deutsch](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/de/CONTRIBUTING.md) · [Magyar](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/hu/CONTRIBUTING.md)

* **Regra de implementação independente.** Contribua apenas com código, matemática e formatos que você possa legalmente compartilhar. Não copie, não descompile nem faça engenharia reversa de ferramentas, arquivos ou interfaces proprietários, e não cole código de projetos cuja licença não seja compatível (veja CLEANROOM.md). Não leia nem cole o código-fonte de ferramentas concorrentes enquanto escreve uma funcionalidade. Cite a norma pública ou o livro-texto a partir do qual você implementou.
* **Licença das contribuições (entrada = saída).** Uma contribuição é aceita sob a licença da parte que ela modifica: MPL-2.0 para `osatura-core`, `-capi`, `-wasm`, `-gpu` e `scripts/`; AGPL-3.0-or-later para `osatura-cli`; CC-BY-4.0 para a documentação e os assets sintéticos. Todo arquivo de código-fonte traz uma linha SPDX que deve ser igual à `license` do seu crate (verificado por `osatura-cli/tests/spdx_audit.rs`).
* **O sign-off não concede nenhum direito de relicenciamento.** O sign-off abaixo certifica apenas a origem. **Uma licença comercial ou dupla (por exemplo, uma licença proprietária para quem incorpora o código) NÃO é atualmente possível para o código contribuído**: ela exigiria um Contributor Licence Agreement separado ou uma concessão explícita de saída de cada contribuidor, e nenhum está em vigor. Enquanto não existir um, ninguém (nem mesmo o mantenedor) pode afirmar ser capaz de relicenciar as contribuições de outras pessoas. O projeto não oferece nenhuma licença comercial ou dupla ([CODE_SIGNING_POLICY.md](../../../CODE_SIGNING_POLICY.md)).
* **Revisão.** Todo pull request de quem não tem acesso de escrita é revisado por um committer antes de ser incorporado; os builds de release com assinatura de código vêm apenas de código revisado em `main` ([CODE_SIGNING_POLICY.md](../../../CODE_SIGNING_POLICY.md), papéis da equipe).
* **Sign-off (DCO).** Todo commit deve conter `Signed-off-by: Your Name <email>` (`git commit -s`), certificando o Developer Certificate of Origin 1.1 reproduzido abaixo.
* Adicione testes; execute `cargo test --workspace` e `scripts/licence-audit.sh` antes de abrir um pull request.

## Developer Certificate of Origin 1.1 (verbatim)

*Reproduzido sem alterações de https://developercertificate.org/ (Copyright (C) 2004, 2006 The Linux Foundation and its contributors); seus próprios termos permitem cópias literais. Este texto não é coberto pela CC-BY-4.0 do restante deste arquivo.*

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.


Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```
