# C ABI do OSATURA

*Versão da ABI 1 (`OSATURA_ABI_VERSION`, `osatura_abi_version()`). Crate `osatura-capi`, header `osatura-capi/osatura.h`. Público-alvo: integradores que querem carregar e reproduzir projetos do OSATURA a partir de C, C++, Python, Go, C#, Swift ou qualquer linguagem com uma interface de funções externas (FFI) para C. Documentação CC-BY-4.0; exemplos de código MPL-2.0.*

A C ABI é um wrapper pequeno, em C puro, sobre a fachada Rust estável ([API.md](API.md)). Com ela, um programa hospedeiro pode abrir um projeto, ler seus dados e as transformações dos ossos, renderizar quadros em RGBA e executar **máquinas de estados** (reprodução interativa, rolagem, nível de detalhe). Desde a fachada 1.4 ela também pode fazer **edições** básicas (criar camadas e ossos, definir chaves, fazer baking de física, desfazer) e **exportar para a memória** (GIF, glTF, Lottie, HTML, APNG, PNG, EXR, JSON do projeto). Ela não consegue fazer controle de versão nem executar as ferramentas: use a API Rust ou a linha de comando ([CLI_REFERENCE.md](CLI_REFERENCE.md)) para isso.

Conteúdo: [Compilar e vincular](#compilar-e-vincular) - [Convenções](#convenções) - [Referência de funções](#referência-de-funções) - [Propriedade e tempo de vida](#propriedade-e-tempo-de-vida) - [Tratamento de erros](#tratamento-de-erros) - [Threads](#threads) - [Exemplo completo em C](#exemplo-completo-em-c) - [Rolagem e nível de detalhe em C](#rolagem-e-nível-de-detalhe-em-c) - [Exemplo em Python (ctypes)](#exemplo-em-python-ctypes) - [Outras linguagens](#outras-linguagens) - [Versionamento](#versionamento) - [Particularidades conhecidas](#particularidades-conhecidas)

## Compilar e vincular

O crate gera três tipos de biblioteca (`crate-type = ["cdylib", "staticlib", "rlib"]`); ele depende de `osatura-core` com `default-features = false`, portanto não contém motor de scripting nem crate de terceiros MPL.

```sh
cargo build -p osatura-capi             # debug:   target/debug/libosatura_capi.{so,a}
cargo build --release -p osatura-capi   # release: target/release/libosatura_capi.{so,a}
```

| Plataforma | Biblioteca dinâmica | Biblioteca estática |
|---|---|---|
| Linux | `libosatura_capi.so` | `libosatura_capi.a` |
| macOS | `libosatura_capi.dylib` | `libosatura_capi.a` |
| Windows | `osatura_capi.dll` (+ biblioteca de importação) | `osatura_capi.lib` |

**Testado:** Linux x86-64, gcc, tanto a biblioteca compartilhada quanto a estática (abaixo). macOS e Windows são os nomes de saída padrão do Rust para esses alvos, mas **não foram compilados nem executados** pelo autor. Um `.so` de debug tem cerca de 150 MB (informações de depuração); compile com `--release` para qualquer coisa que você distribuir (o tamanho da biblioteca release não foi medido aqui).

Compile e vincule com a biblioteca compartilhada:

```sh
cc -std=c99 -Wall -Wextra -Iosatura-capi -o example example.c -Ltarget/debug -losatura_capi
LD_LIBRARY_PATH=target/debug ./example project.json frame.ppm      # or install the .so and use rpath / ldconfig
```

Vinculação estática (verificada no Linux; a biblioteca padrão do Rust precisa destas bibliotecas do sistema):

```sh
cc -std=c99 -Iosatura-capi -o example example.c target/debug/libosatura_capi.a -lpthread -ldl -lm
```

`osatura.h` tem proteções `extern "C"` para C++, precisa apenas de `<stddef.h>` e `<stdint.h>` e é o único arquivo que você inclui. Copie-o para junto dos seus fontes. Não há arquivo pkg-config, nem alvo de instalação, nem SONAME versionado; distribua a biblioteca junto com a sua aplicação.

Os panics são capturados em todos os pontos de entrada e convertidos em códigos de erro, o que depende de a biblioteca ser compilada com o padrão `panic = "unwind"`. Não a compile com `panic = "abort"` (nesse caso, um bug no engine abortaria o seu processo em vez de retornar -1). O workspace não define essa opção de perfil.

## Convenções

* As strings são UTF-8. As strings de entrada (`const char *`) terminam em NUL, exceto o JSON do projeto, que é passado como ponteiro mais comprimento e não precisa de terminador.
* As funções que retornam `int` dão **0 em caso de sucesso e um valor negativo em caso de falha**. `-1` é um erro geral; `-2` significa, além disso, "o seu buffer de saída é pequeno demais" e ocorre apenas nas duas funções de render.
* As funções que preenchem um buffer de caracteres recebem `(char *out, size_t cap)` e retornam o **comprimento total** do texto (sem o NUL). O texto é truncado em `cap - 1` bytes e terminado com NUL quando `cap > 0`. Passe `out == NULL` (com qualquer `cap`) para pedir apenas o comprimento. Um retorno negativo é um erro (índice inválido, nome desconhecido).
* Os argumentos float devem ser finitos; quadros fora da timeline são limitados (clamped); `scale` deve estar entre 0.05 e 4.
* Todo argumento de handle pode ser `NULL` nos getters (eles retornam 0 e definem o texto de erro) e é um erro nas demais funções.
* O RGBA de saída tem **alfa direto** (straight, não pré-multiplicado), 8 bits por canal, em ordem de linhas, linha superior primeiro, `width * height * 4` bytes.
* As transformações de mundo dos ossos são seis floats por osso `[a b c d tx ty]`; um ponto local do osso `(x, y)` é mapeado para `(a*x + c*y + tx, b*x + d*y + ty)`.

## Referência de funções

53 funções. `E` = `OsaturaEngine *`, `P` = `OsaturaPlayer *`.

### Biblioteca

| Função | Descrição |
|---|---|
| `const char *osatura_version(void)` | string de versão do crate `osatura-capi` (atualmente `"0.1.0"`); estática, nunca é liberada |
| `uint32_t osatura_abi_version(void)` | versão da ABI da biblioteca carregada; compare com `OSATURA_ABI_VERSION` do header contra o qual você compilou |
| `const char *osatura_last_error(void)` | texto do último erro **nesta thread**; string vazia se ainda não houve nenhum; veja [Tratamento de erros](#tratamento-de-erros) |

### Engine: abrir, consultar, renderizar

| Função | Descrição |
|---|---|
| `E *osatura_open(const char *json, size_t len)` | interpreta e valida um projeto (UTF-8, `len` bytes, 1 a 256 MiB); retorna `NULL` em caso de erro. A biblioteca copia o que precisa; você pode liberar `json` imediatamente. Os arquivos de imagem relativos do projeto são procurados no diretório atual do processo, caminhos absolutos e `..` são recusados (veja [SECURITY.md](SECURITY.md)); imagens em data URI sempre funcionam. |
| `void osatura_close(E *)` | libera o engine; `NULL` é permitido; chame exatamente uma vez por handle |
| `uint32_t osatura_frame_count(const E *)` | número de quadros |
| `float osatura_fps(const E *)` | quadros por segundo |
| `uint32_t osatura_width(const E *)`, `osatura_height(const E *)` | tamanho do palco em unidades do palco, arredondado para inteiro |
| `uint32_t osatura_bone_count(const E *)` | número de ossos |
| `int osatura_bone_name(const E *, uint32_t index, char *out, size_t cap)` | nome do osso `index` (ordem do rig); o comprimento, ou -1 para um índice inválido |
| `int osatura_bone_world(const E *, float frame, float *out, size_t cap_floats)` | matrizes de mundo de todos os ossos; `cap_floats` deve ser no mínimo `6 * bone_count`, caso contrário -1 |
| `int osatura_render_rgba(const E *, float frame, float scale, uint8_t *out, size_t cap, uint32_t *w, uint32_t *h)` | renderiza um quadro; tamanho da saída `round(width*scale) x round(height*scale)`; retorna 0, **-2 se `cap` for menor que `w*h*4`** (e `*w`, `*h` são definidos, de modo que você pode alocar e chamar de novo; passe `out == NULL` para perguntar), -1 nos demais erros. `w` e `h` podem ser `NULL`. |

### Player: máquinas de estados

Um projeto pode conter `machines` (máquinas de estados de runtime) e `bindings` (valores externos que controlam propriedades). Um player executa uma máquina: defina entradas por nome, avance o tempo, leia estados e eventos, renderize a pose atual. Veja [API.md](API.md) para a semântica (são os métodos de `Player`).

| Função | Descrição |
|---|---|
| `uint32_t osatura_machine_count(const E *)` | máquinas no projeto |
| `int osatura_machine_name(const E *, uint32_t index, char *out, size_t cap)` | nome da máquina `index` (regras do buffer de caracteres) |
| `P *osatura_player_open(const E *, const char *machine)` | inicia a máquina no tempo 0; `NULL` em caso de erro (máquina desconhecida, ...). O player possui uma **cópia** do projeto: permanece válido após `osatura_close` do engine. |
| `void osatura_player_close(P *)` | libera o player; `NULL` é permitido |
| `int osatura_player_set_bool(P *, const char *name, int value)` | entrada bool (`value != 0`) |
| `int osatura_player_set_number(P *, const char *name, double value)` | entrada number (finita) |
| `int osatura_player_fire(P *, const char *trigger)` | dispara um trigger; ele é consumido pela transição que o testa |
| `int osatura_player_set_value(P *, const char *path, double value)` | valor externo para data bindings, por exemplo `"player.health"`; também controla uma região de rolagem cujo `input` é este caminho (pixels ao longo de seu eixo principal) |
| `int osatura_player_advance(P *, double dt)` | avança `dt` segundos, `0..=10`; determinístico (passos longos são divididos em pedaços de no máximo 1/60 s) |
| `int osatura_player_state(P *, const char *layer, char *out, size_t cap)` | nome do estado atual de uma camada de máquina; o comprimento, ou -1 para uma camada desconhecida |
| `int osatura_player_events(P *, char *out, size_t cap)` | eventos pendentes como um array JSON `[{"time":..,"layer":..,"kind":"entered"\|"exited","state":..}]`. Retorna o comprimento total. **Os eventos só são consumidos se o texto inteiro couber (`cap` > comprimento)**; caso contrário são mantidos, e você chama de novo com um buffer maior. `out == NULL` consulta o tamanho e não consome nada. |
| `int osatura_player_render_rgba(P *, float scale, uint8_t *out, size_t cap, uint32_t *w, uint32_t *h)` | mesmo contrato de `osatura_render_rgba`, para a pose atual; o tamanho acompanha o canvas do player |

### Player: layout, rolagem, nível de detalhe

| Função | Descrição |
|---|---|
| `int osatura_player_set_canvas(P *, float width, float height)` | layout responsivo: tamanho do canvas em unidades do palco (finito, positivo, no máximo 32768); os contêineres são resolvidos de novo, os renders passam a ter este tamanho |
| `int osatura_player_canvas_size(P *, float *w, float *h)` | tamanho atual do canvas; qualquer um dos ponteiros pode ser `NULL` |
| `uint32_t osatura_player_scroll_count(P *)` | regiões de rolagem do projeto |
| `int osatura_player_scroll_id(P *, uint32_t index, char *out, size_t cap)` | id da região `index` (regras do buffer de caracteres) |
| `int osatura_player_scroll_drag(P *, const char *id, float x, float y)` | deslocamento bruto do ponteiro em pixels (conteúdo movido para cima / para a esquerda); além das extremidades é limitado (clamp) ou elástico (rubber-band) |
| `int osatura_player_scroll_release(P *, const char *id, float vx, float vy)` | velocidade de soltura em px/s; `osatura_player_advance` então lança, ajusta e faz voltar com elasticidade |
| `int osatura_player_scroll_offset(P *, const char *id, float *x, float *y)` | deslocamento exibido; -1 para um id desconhecido; os ponteiros podem ser `NULL` |
| `int osatura_player_scroll_max(P *, const char *id, float *x, float *y)` | maior deslocamento útil no tamanho atual do canvas |
| `int osatura_player_scroll_settled(P *, const char *id)` | 1 em repouso, 0 em movimento, -1 id desconhecido |
| `int osatura_player_set_screen(P *, float width_px, float height_px, float max_error_px)` | informa ao player o tamanho em que ele é exibido; ele renderiza o nível de detalhe pré-construído mais grosseiro cujo limite de erro declarado permaneça dentro de `max_error_px` pixels. Os níveis são construídos no primeiro uso (esta chamada pode levar um momento em um projeto grande, e muito tempo em uma compilação de debug). |
| `int osatura_player_lod_level(P *)` | nível em uso (0 = original, também sem tamanho de tela); negativo para um player `NULL` |

### Edição e exportação para a memória (aditivo, fachada 1.4)

As edições são validadas e desfazíveis; uma edição que falha não altera nada, retorna -1 e define o texto de erro. Elas recebem um `E *` não const (uma thread por vez, como sempre). As cores são `#rgb`, `#rrggbb` ou `#rrggbbaa`. `parent` pode ser `NULL`.

| Função | Descrição |
|---|---|
| `int osatura_add_rect_layer(E *, name, x, y, w, h, fill, parent)` | acrescenta uma camada de retângulo preenchido; `parent` nomeia uma camada de grupo de nível superior, `NULL` acrescenta no nível superior |
| `int osatura_add_ellipse_layer(E *, name, cx, cy, rx, ry, fill, parent)` | o mesmo para uma elipse |
| `int osatura_add_group_layer(E *, name)` | acrescenta uma camada de grupo vazia |
| `int osatura_add_bone(E *, name, parent, length, x, y, angle)` | acrescenta um osso; o pai deve existir (`NULL` = raiz); ângulo em graus |
| `int osatura_set_key(E *, target, prop, frame, value, interpolation)` | uma chave em `target.prop`; interpolação 0 linear, 1 step, 2 ease-in, 3 ease-out, 4 ease-in-out, 5 smooth |
| `int osatura_bake_physics(E *, chain, preset, overwrite)` | simula uma cadeia de ossos separados por vírgula (raiz primeiro) com `hair`, `tail`, `rope`, `antenna` ou `cape`; retorna os canais gravados (0 quando os ossos com chaves existentes foram mantidos) |
| `int osatura_undo(E *)`, `int osatura_redo(E *)` | 1 feito, 0 nada a fazer, negativo em caso de erro |
| `uint32_t osatura_layer_count(const E *)`, `int osatura_layer_name(const E *, index, out, cap)` | caminhos de camada (`group/child`), regras do buffer de caracteres |
| `B *osatura_export(const E *, format, scale)` | exportação da timeline inteira, `B` = `OsaturaBlob *`; formato 0 GIF, 1 glTF binário, 2 Lottie JSON, 3 player HTML, 4 APNG; `scale` 0.05 a 4 para GIF e APNG; `NULL` em caso de erro |
| `B *osatura_render_png(const E *, frame, scale)` | um quadro como PNG |
| `B *osatura_render_exr(const E *, frame, scale, float32)` | um quadro como OpenEXR (linear, pré-multiplicado; um `float32` diferente de zero grava FLOAT, caso contrário HALF) |
| `B *osatura_to_json(const E *)` | o projeto (editado) como JSON UTF-8, sem terminador |
| `const uint8_t *osatura_blob_data(const B *)`, `size_t osatura_blob_size(const B *)` | os bytes de um blob, válidos até ele ser liberado; `NULL` / 0 para um blob `NULL` |
| `void osatura_blob_free(B *)` | libera um blob exatamente uma vez; `NULL` é permitido |

## Propriedade e tempo de vida

* **Strings e arrays são copiados para buffers que você fornece; os resultados de exportação são blobs que você libera.** Todo resultado em string ou array é copiado para um buffer que *você* fornece. A única memória pertencente à biblioteca é um `OsaturaBlob` vindo de `osatura_export`, `osatura_render_png`, `osatura_render_exr` ou `osatura_to_json`: leia-o com `osatura_blob_data` / `osatura_blob_size` e depois libere-o uma vez com `osatura_blob_free`. Não existe um `osatura_free` genérico. Os únicos ponteiros que a biblioteca retorna são `osatura_version()` (estático) e `osatura_last_error()` (veja abaixo), além dos dois handles opacos.
* **Handles.** `osatura_open` retorna um handle de engine; `osatura_player_open` retorna um handle de player. Cada um deve ser fechado exatamente uma vez com o seu próprio `_close`; fechar `NULL` é aceitável; usar um handle depois de fechá-lo, ou fechá-lo duas vezes, é comportamento indefinido.
* **O engine e os players são independentes.** Um player possui uma cópia do projeto, portanto fechar o engine primeiro é permitido e comum (o exemplo faz isso). Vários players podem ser abertos a partir de um mesmo engine.
* **Seus buffers.** Os ponteiros de entrada só precisam ser válidos durante a chamada; a biblioteca não guarda nenhum ponteiro para eles. Os buffers de saída devem realmente ter a capacidade que você informa; a biblioteca escreve no máximo `cap` bytes (strings) ou `cap_floats` floats.
* **`osatura_last_error()`** retorna um ponteiro para uma string thread-local. Ele permanece válido até a próxima chamada na mesma thread que *falhar* (essa chamada o sobrescreve) ou até a thread terminar. Copie-o se precisar dele por mais tempo. Não o libere.

## Tratamento de erros

1. Teste sempre o valor de retorno (`NULL` nas duas funções `_open` e em `osatura_player_open`, um `int` negativo nas demais, `0` de um getter que recebeu um handle `NULL`).
2. Depois leia `osatura_last_error()` na mesma thread para saber o motivo: por exemplo `invalid project JSON: ...`, `output buffer needs 1228800 bytes`, `invalid argument: input 'excited' is not a number`.
3. **O texto de erro só é definido por falhas e nunca é limpo por um sucesso.** Depois de uma chamada bem-sucedida posterior, ele ainda contém a mensagem antiga. Nunca use "a string não está vazia" como teste de falha.
4. Os textos de erro vêm da fachada Rust (`Display` de `ApiError`); são para pessoas, não um contrato. A C ABI não expõe o *tipo* do erro.
5. Um panic dentro do engine é capturado e relatado como `internal error (panic) inside osatura` com o retorno de falha habitual; o handle continua utilizável em princípio, mas trate isso como um bug e relate-o.

## Threads

* Cada handle pode ser usado a partir de **uma thread por vez** (o contrato do header). Você pode mover um handle entre threads, desde que as chamadas nunca se sobreponham; handles diferentes podem ser executados em paralelo em threads diferentes (os tipos Rust subjacentes são `Send + Sync`, mas o contrato de C permanece conservador).
* A string de último erro é **por thread**: leia-a na thread que fez a chamada que falhou.
* Não há callbacks e a biblioteca não inicia threads próprias.

## Exemplo completo em C

Carrega um projeto, imprime os dados e as posições dos ossos, renderiza um quadro em um arquivo PPM (pedindo primeiro o tamanho com `NULL` e depois alocando exatamente), executa a primeira máquina de estados do projeto, lê seus eventos com o padrão de consulta de tamanho e mostra o caminho de erro. Verificado com gcc, também sob AddressSanitizer/UBSan (sem achados), contra a biblioteca compartilhada e a estática.

```c
/* OSATURA C ABI example: load, inspect, render a PPM, run a state machine.
 * Build:  cc -std=c99 -Wall -Wextra -Iosatura-capi -o example example.c -Ltarget/debug -losatura_capi
 * Run:    LD_LIBRARY_PATH=target/debug ./example project.json frame.ppm [machine-name]
 */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "osatura.h"

static char *read_file(const char *path, size_t *len) {
    FILE *f = fopen(path, "rb");
    if (!f) return NULL;
    fseek(f, 0, SEEK_END);
    long n = ftell(f);
    fseek(f, 0, SEEK_SET);
    char *buf = malloc(n > 0 ? (size_t)n : 1);
    if (!buf || fread(buf, 1, (size_t)n, f) != (size_t)n) { fclose(f); free(buf); return NULL; }
    fclose(f);
    *len = (size_t)n;
    return buf;
}

int main(int argc, char **argv) {
    if (argc < 3) { fprintf(stderr, "usage: %s project.json out.ppm [machine]\n", argv[0]); return 2; }
    if (osatura_abi_version() != OSATURA_ABI_VERSION) { fprintf(stderr, "header and library disagree\n"); return 2; }
    printf("osatura %s, ABI %u\n", osatura_version(), osatura_abi_version());

    size_t len = 0;
    char *json = read_file(argv[1], &len);
    if (!json) { fprintf(stderr, "cannot read %s\n", argv[1]); return 3; }
    OsaturaEngine *e = osatura_open(json, len);
    free(json);                                   /* the engine keeps its own copy */
    if (!e) { fprintf(stderr, "open failed: %s\n", osatura_last_error()); return 4; }
    printf("%ux%u, %u frames @ %.1f fps\n", osatura_width(e), osatura_height(e), osatura_frame_count(e), osatura_fps(e));

    /* bones: copy the name, then the world matrices of frame 10 */
    uint32_t nb = osatura_bone_count(e);
    float *world = malloc(sizeof(float) * 6 * (nb ? nb : 1));
    if (osatura_bone_world(e, 10.0f, world, 6 * (size_t)nb) != 0) { fprintf(stderr, "%s\n", osatura_last_error()); return 5; }
    for (uint32_t i = 0; i < nb; i++) {
        char name[64];
        osatura_bone_name(e, i, name, sizeof name);
        printf("  bone %-10s tx=%7.2f ty=%7.2f\n", name, world[i * 6 + 4], world[i * 6 + 5]);
    }
    free(world);

    /* render: ask for the size first (out == NULL), then allocate exactly */
    uint32_t w = 0, h = 0;
    int rc = osatura_render_rgba(e, 10.0f, 0.5f, NULL, 0, &w, &h);
    if (rc != -2) { fprintf(stderr, "unexpected rc %d: %s\n", rc, osatura_last_error()); return 6; }
    size_t need = (size_t)w * h * 4;
    uint8_t *px = malloc(need);
    if (osatura_render_rgba(e, 10.0f, 0.5f, px, need, &w, &h) != 0) { fprintf(stderr, "%s\n", osatura_last_error()); return 6; }
    FILE *out = fopen(argv[2], "wb");
    if (!out) return 7;
    fprintf(out, "P6\n%u %u\n255\n", w, h);       /* RGBA straight alpha -> RGB over white */
    for (size_t i = 0; i < (size_t)w * h; i++) {
        unsigned a = px[i * 4 + 3];
        for (int c = 0; c < 3; c++) fputc((px[i * 4 + c] * a + 255 * (255 - a)) / 255, out);
    }
    fclose(out);
    free(px);
    printf("wrote %s (%ux%u)\n", argv[2], w, h);

    /* state machine (only when the project has one) */
    if (osatura_machine_count(e) > 0) {
        char mname[64];
        osatura_machine_name(e, 0, mname, sizeof mname);
        const char *wanted = argc > 3 ? argv[3] : mname;
        OsaturaPlayer *p = osatura_player_open(e, wanted);
        if (!p) { fprintf(stderr, "player: %s\n", osatura_last_error()); return 8; }
        osatura_close(e);                        /* allowed: the player owns a copy of the project */
        osatura_player_set_bool(p, "excited", 1);
        for (int i = 0; i < 30; i++) osatura_player_advance(p, 1.0 / 30.0);
        char state[64];
        osatura_player_state(p, "body", state, sizeof state);
        printf("machine %s: layer body is in state '%s'\n", wanted, state);
        int need_ev = osatura_player_events(p, NULL, 0);   /* query the JSON size; nothing consumed */
        char *ev = malloc((size_t)need_ev + 1);
        osatura_player_events(p, ev, (size_t)need_ev + 1); /* fits: events are consumed now */
        printf("events: %s\n", ev);
        free(ev);
        /* a wrong call reports through osatura_last_error() */
        if (osatura_player_set_number(p, "excited", 1.0) != 0) printf("expected error: %s\n", osatura_last_error());
        osatura_player_close(p);
    } else {
        osatura_close(e);
    }
    return 0;
}
```

Compilado e executado contra o projeto de `osatura demo` estendido com uma máquina de estados (`hero`, entrada `excited`, estados `idle` e `wave`):

```text
osatura 0.1.0, ABI 1
640x480, 48 frames @ 24.0 fps
  bone arm_up     tx= 150.00 ty= 330.00
  ...
wrote frame.ppm (320x240)
machine hero: layer body is in state 'wave'
events: [{"time":0.0,"layer":"body","kind":"entered","state":"idle"},{"time":0.016666666666666666,"layer":"body","kind":"exited","state":"idle"},{"time":0.016666666666666666,"layer":"body","kind":"entered","state":"wave"}]
expected error: invalid argument: input 'excited' is not a number
```

O repositório também inclui `osatura-capi/tests/smoke.c` (abrir, renderizar, checksum) e os testes Rust `osatura-capi/tests/abi.rs` (ponteiros nulos, índices inválidos, tamanhos de buffer, segurança contra panics).

## Rolagem e nível de detalhe em C

Um projeto com um contêiner de layout e uma região de rolagem (o projeto de teste de rolagem dos testes Rust: dez linhas de 20 unidades em uma coluna, um viewport fixado às bordas esquerda, superior e inferior). Verificado:

```c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "osatura.h"
int main(int argc, char **argv) {
    FILE *f = fopen(argv[1], "rb"); fseek(f, 0, SEEK_END); long n = ftell(f); fseek(f, 0, SEEK_SET);
    char *json = malloc(n); if (fread(json, 1, n, f) != (size_t)n) return 1; fclose(f);
    OsaturaEngine *e = osatura_open(json, n); free(json);
    if (!e) { printf("%s\n", osatura_last_error()); return 2; }
    OsaturaPlayer *p = osatura_player_open(e, "hero");
    osatura_close(e);
    if (!p) { printf("%s\n", osatura_last_error()); return 3; }
    osatura_player_set_canvas(p, 100.0f, 120.0f);               /* containers re-solve for this size */
    char id[32];
    for (uint32_t i = 0; i < osatura_player_scroll_count(p); i++) {
        osatura_player_scroll_id(p, i, id, sizeof id);
        float mx, my, ox, oy;
        osatura_player_scroll_max(p, id, &mx, &my);
        osatura_player_scroll_drag(p, id, 0.0f, 500.0f);        /* far past the end: rubber band / clamp */
        osatura_player_scroll_offset(p, id, &ox, &oy);
        printf("scroll '%s': max=(%g,%g) dragged offset=(%g,%g) settled=%d\n", id, mx, my, ox, oy, osatura_player_scroll_settled(p, id));
        osatura_player_scroll_release(p, id, 0.0f, 0.0f);
        for (int k = 0; k < 60; k++) osatura_player_advance(p, 1.0 / 60.0);
        osatura_player_scroll_offset(p, id, &ox, &oy);
        printf("  after release: offset=(%g,%g) settled=%d\n", ox, oy, osatura_player_scroll_settled(p, id));
    }
    int rc = osatura_player_set_screen(p, 100.0f, 120.0f, 4.0f);   /* accept up to 4 px of error */
    printf("set_screen rc=%d, lod level=%d\n", rc, osatura_player_lod_level(p));
    uint32_t w, h; osatura_player_render_rgba(p, 1.0f, NULL, 0, &w, &h);
    printf("render size %ux%u\n", w, h);
    osatura_player_close(p);
    return 0;
}
```

Saída: `scroll 'list': max=(0,80) dragged offset=(0,80) settled=1` (arrastar até 500 limita no máximo quando a elasticidade está desativada), depois `set_screen rc=0, lod level=7` (com até 4 px de erro aceitos, o nível pré-construído mais grosseiro se qualifica neste projeto minúsculo; o número depende do projeto) e `render size 100x120`. Observe a variável `rc` explícita no código-fonte: a ordem em que C avalia os argumentos de uma função não é especificada, portanto nunca coloque a chamada que altera o player e a chamada que o lê em um único `printf`.

## Exemplo em Python (ctypes)

Apenas a biblioteca padrão. Declare `argtypes` e `restype` para toda função que você chamar: caso contrário o ctypes assume `int` e trunca ponteiros de 64 bits. Verificado em Python 3 contra a biblioteca de debug; sua saída PPM é idêntica, byte a byte, à do exemplo em C.

```python
#!/usr/bin/env python3
"""OSATURA C ABI from Python with ctypes (standard library only).

    python3 example.py libosatura_capi.so project.json out.ppm
"""
import ctypes as C
import sys

lib = C.CDLL(sys.argv[1])

# --- declare every signature you use (ctypes defaults to int and would truncate pointers) ---
lib.osatura_version.restype = C.c_char_p
lib.osatura_abi_version.restype = C.c_uint32
lib.osatura_last_error.restype = C.c_char_p
lib.osatura_open.argtypes = [C.c_char_p, C.c_size_t]
lib.osatura_open.restype = C.c_void_p                      # opaque handle
lib.osatura_close.argtypes = [C.c_void_p]
for name, res in (("frame_count", C.c_uint32), ("fps", C.c_float), ("width", C.c_uint32),
                  ("height", C.c_uint32), ("bone_count", C.c_uint32), ("machine_count", C.c_uint32)):
    f = getattr(lib, "osatura_" + name)
    f.argtypes = [C.c_void_p]
    f.restype = res
lib.osatura_bone_name.argtypes = [C.c_void_p, C.c_uint32, C.c_char_p, C.c_size_t]
lib.osatura_bone_name.restype = C.c_int
lib.osatura_bone_world.argtypes = [C.c_void_p, C.c_float, C.POINTER(C.c_float), C.c_size_t]
lib.osatura_bone_world.restype = C.c_int
lib.osatura_render_rgba.argtypes = [C.c_void_p, C.c_float, C.c_float, C.c_void_p, C.c_size_t,
                                     C.POINTER(C.c_uint32), C.POINTER(C.c_uint32)]
lib.osatura_render_rgba.restype = C.c_int


def last_error() -> str:
    return (lib.osatura_last_error() or b"").decode("utf-8", "replace")


class Engine:
    """Owns one handle; use as a context manager so osatura_close runs exactly once."""

    def __init__(self, json_bytes: bytes):
        self.h = lib.osatura_open(json_bytes, len(json_bytes))
        if not self.h:
            raise ValueError(last_error())

    def close(self):
        if self.h:
            lib.osatura_close(self.h)
            self.h = None

    __enter__ = lambda self: self
    __exit__ = lambda self, *a: self.close()

    def bones(self):
        out = []
        for i in range(lib.osatura_bone_count(self.h)):
            buf = C.create_string_buffer(128)
            lib.osatura_bone_name(self.h, i, buf, len(buf))
            out.append(buf.value.decode())
        return out

    def bone_world(self, frame: float):
        n = lib.osatura_bone_count(self.h)
        arr = (C.c_float * (6 * n))()
        if lib.osatura_bone_world(self.h, frame, arr, len(arr)) != 0:
            raise RuntimeError(last_error())
        return [tuple(arr[i * 6:i * 6 + 6]) for i in range(n)]

    def render(self, frame: float, scale: float = 1.0):
        w, h = C.c_uint32(), C.c_uint32()
        rc = lib.osatura_render_rgba(self.h, frame, scale, None, 0, C.byref(w), C.byref(h))
        if rc != -2:                                       # -2 = "buffer too small", and w/h are filled in
            raise RuntimeError(last_error())
        buf = (C.c_uint8 * (w.value * h.value * 4))()
        if lib.osatura_render_rgba(self.h, frame, scale, buf, len(buf), C.byref(w), C.byref(h)) != 0:
            raise RuntimeError(last_error())
        return w.value, h.value, bytes(buf)


if __name__ == "__main__":
    print("osatura", lib.osatura_version().decode(), "ABI", lib.osatura_abi_version())
    data = open(sys.argv[2], "rb").read()
    try:
        Engine(b"{ not json")
    except ValueError as e:
        print("expected error:", e)
    with Engine(data) as e:
        print(lib.osatura_width(e.h), "x", lib.osatura_height(e.h), lib.osatura_frame_count(e.h), "frames")
        print("bones:", e.bones())
        print("arm_up world @10:", [round(v, 2) for v in e.bone_world(10.0)[0]])
        w, h, rgba = e.render(10.0, 0.5)
        with open(sys.argv[3], "wb") as f:
            f.write(b"P6\n%d %d\n255\n" % (w, h))
            for i in range(0, len(rgba), 4):
                r, g, b, a = rgba[i:i + 4]
                f.write(bytes((c * a + 255 * (255 - a)) // 255 for c in (r, g, b)))
        print("wrote", sys.argv[3], w, h)
```

## Outras linguagens

Qualquer coisa que consiga chamar uma função C funciona da mesma forma: declare as 53 funções, mantenha os ponteiros opacos como `void *` e siga as convenções de buffer acima. Observações específicas de hospedeiros comuns (não testadas aqui, apenas consequências do contrato acima): em C#/.NET use `[DllImport]` com `CallingConvention.Cdecl` e faça o marshaling das strings como arrays de bytes UTF-8; em Go com cgo, mantenha ponteiros de Go fora das chamadas, exceto para buffers que vivam durante a chamada; em Rust, prefira usar a fachada diretamente em vez desta ABI.

## Versionamento

* `OSATURA_ABI_VERSION` / `osatura_abi_version()` vale atualmente **1**. Ela só muda quando uma função existente é removida ou seu significado muda. Todas as adições até agora (as funções do player na fachada 1.1, layout, rolagem e nível de detalhe na 1.2) foram aditivas e a mantiveram em 1, de modo que um programa compilado com um header mais antigo continua funcionando com uma biblioteca mais nova, e um header mais novo com uma biblioteca mais antiga falha na vinculação ou no carregamento por causa do símbolo ausente. Verifique `osatura_abi_version()` na inicialização e compare-a com a macro.
* `osatura_version()` é a versão do crate `osatura-capi` (0.1.0), não a versão da fachada. Não existe função C que informe a versão da fachada (`API_VERSION`).
* A fachada 1.4 adicionou os fundamentos de edição e as funções de exportação para a memória descritos acima (aditivas, versão da ABI ainda 1). Não expostos na C ABI: renderização SVG, exclusão de chaves, controle de versão, CI de animação, tracing, mocap, fuzzing, exportação com tamanho-alvo, scripts. Eles estão disponíveis em Rust e pela CLI.

## Particularidades conhecidas

Os comentários do header foram corrigidos para corresponder ao comportamento (antes eles divergiam); os pontos que vale conhecer:

* `osatura_render_rgba` grava `*w` / `*h` em caso de sucesso e em -2 (buffer pequeno demais), e também quando ocorre um erro de renderização posterior depois de `frame` e `scale` terem sido aceitos. Ela **não** os grava quando o engine é nulo ou quando `frame` / `scale` são rejeitados (-1). Teste de regressão: `documented_size_outputs_and_last_error_semantics` em `osatura-capi/tests/abi.rs`.
* O texto do último erro é thread-local, sobrescrito apenas pela próxima chamada que *falhar*, e nunca é limpo por um sucesso. Copie-o se precisar dele por mais tempo.
* `osatura_width` / `osatura_height` arredondam para inteiro o tamanho do palco em ponto flutuante; o tamanho real da renderização é `round(size * scale)`, portanto use os `w` / `h` retornados pela chamada de renderização.
* A observação do header sobre as regiões de rolagem que acompanham `osatura_player_set_value` fica entre as declarações de rolagem e de nível de detalhe; ela descreve `osatura_player_set_value`.
