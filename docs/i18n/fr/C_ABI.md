# ABI C d'OSATURA

*Version d'ABI 1 (`OSATURA_ABI_VERSION`, `osatura_abi_version()`). Crate `osatura-capi`, en-tête `osatura-capi/osatura.h`. Public : intégrateurs qui veulent charger et lire des projets OSATURA depuis C, C++, Python, Go, C#, Swift ou tout langage disposant d'une interface de fonctions étrangères C. Documentation CC-BY-4.0 ; exemples de code MPL-2.0.*

L'ABI C est une petite enveloppe en C pur autour de la façade Rust stable ([API.md](API.md)). Avec elle, un programme hôte peut ouvrir un projet, lire ses informations et les transformations des os, rendre des images en RGBA et exécuter des **machines à états** (lecture interactive, défilement, niveau de détail). Depuis la façade 1.4, elle permet aussi des **éditions** de base (créer des calques et des os, définir des clés, baker la physique, annuler) et l'**export en mémoire** (GIF, glTF, Lottie, HTML, APNG, PNG, EXR, JSON du projet). Elle ne permet ni le contrôle de version ni l'exécution des outils : utilisez pour cela l'API Rust ou la ligne de commande ([CLI_REFERENCE.md](CLI_REFERENCE.md)).

Sommaire : [Compilation et édition de liens](#compilation-et-édition-de-liens) - [Conventions](#conventions) - [Référence des fonctions](#référence-des-fonctions) - [Propriété et durée de vie](#propriété-et-durée-de-vie) - [Gestion des erreurs](#gestion-des-erreurs) - [Threads](#threads) - [Exemple C complet](#exemple-c-complet) - [Défilement et niveau de détail en C](#défilement-et-niveau-de-détail-en-c) - [Exemple Python (ctypes)](#exemple-python-ctypes) - [Autres langages](#autres-langages) - [Gestion des versions](#gestion-des-versions) - [Particularités connues](#particularités-connues)

## Compilation et édition de liens

La crate produit trois types de bibliothèques (`crate-type = ["cdylib", "staticlib", "rlib"]`) ; elle dépend de `osatura-core` avec `default-features = false`, donc elle ne contient ni moteur de script ni crate tierce MPL.

```sh
cargo build -p osatura-capi             # debug:   target/debug/libosatura_capi.{so,a}
cargo build --release -p osatura-capi   # release: target/release/libosatura_capi.{so,a}
```

| Plateforme | Bibliothèque dynamique | Bibliothèque statique |
|---|---|---|
| Linux | `libosatura_capi.so` | `libosatura_capi.a` |
| macOS | `libosatura_capi.dylib` | `libosatura_capi.a` |
| Windows | `osatura_capi.dll` (+ bibliothèque d'import) | `osatura_capi.lib` |

**Testé :** Linux x86-64, gcc, avec la bibliothèque partagée comme la statique (ci-dessous). macOS et Windows correspondent aux noms de sortie standard de Rust pour ces cibles, mais n'ont **pas été compilés ni exécutés** par l'auteur. Un `.so` de debug pèse environ 150 Mo (informations de débogage) ; compilez en `--release` pour tout ce que vous livrez (la taille de la bibliothèque release n'a pas été mesurée ici).

Compiler et lier avec la bibliothèque partagée :

```sh
cc -std=c99 -Wall -Wextra -Iosatura-capi -o example example.c -Ltarget/debug -losatura_capi
LD_LIBRARY_PATH=target/debug ./example project.json frame.ppm      # or install the .so and use rpath / ldconfig
```

Liaison statique (vérifiée sous Linux ; la bibliothèque standard de Rust a besoin de ces bibliothèques système) :

```sh
cc -std=c99 -Iosatura-capi -o example example.c target/debug/libosatura_capi.a -lpthread -ldl -lm
```

`osatura.h` comporte des gardes `extern "C"` pour C++, n'a besoin que de `<stddef.h>` et `<stdint.h>`, et c'est le seul fichier à inclure. Copiez-le à côté de vos sources. Il n'y a ni fichier pkg-config, ni cible d'installation, ni SONAME versionné ; embarquez la bibliothèque avec votre application.

Les paniques sont interceptées à chaque point d'entrée et converties en codes d'erreur, ce qui suppose que la bibliothèque soit compilée avec le réglage par défaut `panic = "unwind"`. Ne la compilez pas avec `panic = "abort"` (un bug du moteur interromprait alors votre processus au lieu de renvoyer -1). Le workspace ne définit pas cette option de profil.

## Conventions

* Les chaînes sont en UTF-8. Les chaînes d'entrée (`const char *`) se terminent par NUL, sauf le JSON du projet, qui est passé sous forme de pointeur plus longueur et n'a pas besoin de terminateur.
* Les fonctions qui renvoient un `int` donnent **0 en cas de succès et une valeur négative en cas d'échec**. `-1` est une erreur générale ; `-2` signifie en outre « votre tampon de sortie est trop petit » et ne se produit que dans les deux fonctions de rendu.
* Les fonctions qui remplissent un tampon de caractères prennent `(char *out, size_t cap)` et renvoient la **longueur complète** du texte (NUL exclu). Le texte est tronqué à `cap - 1` octets et terminé par NUL lorsque `cap > 0`. Passez `out == NULL` (avec n'importe quel `cap`) pour demander seulement la longueur. Un retour négatif est une erreur (mauvais index, nom inconnu).
* Les arguments flottants doivent être finis ; les images hors de la timeline sont bornées ; `scale` doit être compris entre 0,05 et 4.
* Tout argument de handle peut être `NULL` pour les accesseurs (ils renvoient 0 et définissent le texte d'erreur) et constitue une erreur pour les autres fonctions.
* Le RGBA de sortie est à **alpha droit** (non prémultiplié), 8 bits par canal, ligne par ligne, rangée du haut en premier, `width * height * 4` octets.
* Les transformations monde des os sont de six flottants par os `[a b c d tx ty]` ; un point local à l'os `(x, y)` est envoyé sur `(a*x + c*y + tx, b*x + d*y + ty)`.

## Référence des fonctions

53 fonctions. `E` = `OsaturaEngine *`, `P` = `OsaturaPlayer *`.

### Bibliothèque

| Fonction | Description |
|---|---|
| `const char *osatura_version(void)` | chaîne de version de la crate `osatura-capi` (actuellement `"0.1.0"`) ; statique, jamais libérée |
| `uint32_t osatura_abi_version(void)` | version d'ABI de la bibliothèque chargée ; à comparer avec `OSATURA_ABI_VERSION` de l'en-tête avec lequel vous avez compilé |
| `const char *osatura_last_error(void)` | texte de la dernière erreur **sur ce thread** ; chaîne vide s'il n'y en a pas encore ; voir [Gestion des erreurs](#gestion-des-erreurs) |

### Moteur : ouvrir, interroger, rendre

| Fonction | Description |
|---|---|
| `E *osatura_open(const char *json, size_t len)` | analyse et valide un projet (UTF-8, `len` octets, de 1 à 256 Mio) ; renvoie `NULL` en cas d'erreur. La bibliothèque copie ce dont elle a besoin ; vous pouvez libérer `json` immédiatement. Les fichiers image relatifs du projet sont recherchés dans le répertoire courant du processus, les chemins absolus et `..` sont refusés (voir [SECURITY.md](SECURITY.md)) ; les images en URI de données fonctionnent toujours. |
| `void osatura_close(E *)` | libère le moteur ; `NULL` est autorisé ; à appeler exactement une fois par handle |
| `uint32_t osatura_frame_count(const E *)` | nombre d'images |
| `float osatura_fps(const E *)` | images par seconde |
| `uint32_t osatura_width(const E *)`, `osatura_height(const E *)` | taille de la scène en unités de scène, arrondie à l'entier |
| `uint32_t osatura_bone_count(const E *)` | nombre d'os |
| `int osatura_bone_name(const E *, uint32_t index, char *out, size_t cap)` | nom de l'os `index` (ordre du rig) ; la longueur, ou -1 pour un mauvais index |
| `int osatura_bone_world(const E *, float frame, float *out, size_t cap_floats)` | matrices monde de chaque os ; `cap_floats` doit valoir au moins `6 * bone_count`, sinon -1 |
| `int osatura_render_rgba(const E *, float frame, float scale, uint8_t *out, size_t cap, uint32_t *w, uint32_t *h)` | rend une image ; taille de sortie `round(width*scale) x round(height*scale)` ; renvoie 0, **-2 si `cap` est inférieur à `w*h*4`** (et `*w`, `*h` sont renseignés, vous pouvez donc allouer et rappeler ; passez `out == NULL` pour interroger), -1 pour les autres erreurs. `w` et `h` peuvent être `NULL`. |

### Player : machines à états

Un projet peut contenir des `machines` (machines à états runtime) et des `bindings` (valeurs externes qui pilotent des propriétés). Un player exécute une machine : définissez les entrées par leur nom, faites avancer le temps, lisez les états et les événements, rendez la pose courante. Voir [API.md](API.md) pour la sémantique (ce sont les méthodes de `Player`).

| Fonction | Description |
|---|---|
| `uint32_t osatura_machine_count(const E *)` | machines du projet |
| `int osatura_machine_name(const E *, uint32_t index, char *out, size_t cap)` | nom de la machine `index` (règles des tampons de caractères) |
| `P *osatura_player_open(const E *, const char *machine)` | démarre la machine au temps 0 ; `NULL` en cas d'erreur (machine inconnue, ...). Le player possède une **copie** du projet : il reste valide après `osatura_close` du moteur. |
| `void osatura_player_close(P *)` | libère le player ; `NULL` autorisé |
| `int osatura_player_set_bool(P *, const char *name, int value)` | entrée booléenne (`value != 0`) |
| `int osatura_player_set_number(P *, const char *name, double value)` | entrée numérique (finie) |
| `int osatura_player_fire(P *, const char *trigger)` | déclenche un trigger ; il est consommé par la transition qui le teste |
| `int osatura_player_set_value(P *, const char *path, double value)` | valeur externe pour les data bindings, p. ex. `"player.health"` ; pilote aussi une région de défilement dont l'`input` est ce chemin (pixels le long de son axe principal) |
| `int osatura_player_advance(P *, double dt)` | avance de `dt` secondes, `0..=10` ; déterministe (les longs pas sont découpés en morceaux d'au plus 1/60 s) |
| `int osatura_player_state(P *, const char *layer, char *out, size_t cap)` | nom de l'état courant d'un calque de machine ; la longueur, ou -1 pour un calque inconnu |
| `int osatura_player_events(P *, char *out, size_t cap)` | événements en attente sous forme de tableau JSON `[{"time":..,"layer":..,"kind":"entered"\|"exited","state":..}]`. Renvoie la longueur complète. **Les événements ne sont consommés que si tout le texte tient (`cap` > longueur)** ; sinon ils sont conservés et vous rappelez avec un tampon plus grand. `out == NULL` interroge la taille et ne consomme rien. |
| `int osatura_player_render_rgba(P *, float scale, uint8_t *out, size_t cap, uint32_t *w, uint32_t *h)` | même contrat que `osatura_render_rgba`, pour la pose courante ; la taille suit le canevas du player |

### Player : mise en page, défilement, niveau de détail

| Fonction | Description |
|---|---|
| `int osatura_player_set_canvas(P *, float width, float height)` | mise en page adaptative : taille du canevas en unités de scène (finie, positive, au plus 32768) ; les conteneurs sont recalculés, les rendus prennent cette taille |
| `int osatura_player_canvas_size(P *, float *w, float *h)` | taille courante du canevas ; l'un ou l'autre pointeur peut être `NULL` |
| `uint32_t osatura_player_scroll_count(P *)` | régions de défilement du projet |
| `int osatura_player_scroll_id(P *, uint32_t index, char *out, size_t cap)` | identifiant de la région `index` (règles des tampons de caractères) |
| `int osatura_player_scroll_drag(P *, const char *id, float x, float y)` | décalage brut du pointeur en pixels (contenu déplacé vers le haut / la gauche) ; au-delà des extrémités, il est borné ou élastique |
| `int osatura_player_scroll_release(P *, const char *id, float vx, float vy)` | vitesse de relâchement en px/s ; `osatura_player_advance` lance alors l'inertie, aimante et ramène en ressort |
| `int osatura_player_scroll_offset(P *, const char *id, float *x, float *y)` | décalage affiché ; -1 pour un identifiant inconnu ; les pointeurs peuvent être `NULL` |
| `int osatura_player_scroll_max(P *, const char *id, float *x, float *y)` | plus grand décalage utile à la taille de canevas courante |
| `int osatura_player_scroll_settled(P *, const char *id)` | 1 au repos, 0 en mouvement, -1 identifiant inconnu |
| `int osatura_player_set_screen(P *, float width_px, float height_px, float max_error_px)` | indique au player la taille à laquelle il est affiché ; il rend le plus grossier des niveaux de détail préconstruits dont la borne d'erreur déclarée reste dans `max_error_px` pixels. Les niveaux sont construits à la première utilisation (cet appel peut prendre un moment sur un gros projet, longtemps dans un build de debug). |
| `int osatura_player_lod_level(P *)` | niveau utilisé (0 = original, y compris sans taille d'écran) ; négatif pour un player `NULL` |

### Édition et export en mémoire (additif, façade 1.4)

Les éditions sont validées et annulables ; une édition qui échoue ne change rien, renvoie -1 et définit le texte d'erreur. Elles prennent un `E *` non constant (un seul thread à la fois, comme toujours). Les couleurs s'écrivent `#rgb`, `#rrggbb` ou `#rrggbbaa`. `parent` peut être `NULL`.

| Fonction | Description |
|---|---|
| `int osatura_add_rect_layer(E *, name, x, y, w, h, fill, parent)` | ajoute un calque rectangle rempli ; `parent` nomme un calque groupe de niveau supérieur, `NULL` ajoute au niveau supérieur |
| `int osatura_add_ellipse_layer(E *, name, cx, cy, rx, ry, fill, parent)` | idem pour une ellipse |
| `int osatura_add_group_layer(E *, name)` | ajoute un calque groupe vide |
| `int osatura_add_bone(E *, name, parent, length, x, y, angle)` | ajoute un os ; le parent doit exister (`NULL` = racine) ; angle en degrés |
| `int osatura_set_key(E *, target, prop, frame, value, interpolation)` | une clé sur `target.prop` ; interpolation 0 linéaire, 1 palier, 2 ease-in, 3 ease-out, 4 ease-in-out, 5 lissée |
| `int osatura_bake_physics(E *, chain, preset, overwrite)` | simule une chaîne d'os séparés par des virgules (racine en premier) avec `hair`, `tail`, `rope`, `antenna` ou `cape` ; renvoie le nombre de canaux écrits (0 lorsque des os déjà animés ont été conservés) |
| `int osatura_undo(E *)`, `int osatura_redo(E *)` | 1 fait, 0 rien à faire, négatif en cas d'erreur |
| `uint32_t osatura_layer_count(const E *)`, `int osatura_layer_name(const E *, index, out, cap)` | chemins de calques (`group/child`), règles des tampons de caractères |
| `B *osatura_export(const E *, format, scale)` | export de toute la timeline, `B` = `OsaturaBlob *` ; format 0 GIF, 1 glTF binaire, 2 JSON Lottie, 3 lecteur HTML, 4 APNG ; `scale` de 0,05 à 4 pour GIF et APNG ; `NULL` en cas d'erreur |
| `B *osatura_render_png(const E *, frame, scale)` | une image en PNG |
| `B *osatura_render_exr(const E *, frame, scale, float32)` | une image en OpenEXR (linéaire, prémultiplié ; un `float32` non nul stocke en FLOAT, sinon HALF) |
| `B *osatura_to_json(const E *)` | le projet (édité) en JSON UTF-8, sans terminateur |
| `const uint8_t *osatura_blob_data(const B *)`, `size_t osatura_blob_size(const B *)` | les octets d'un blob, valides jusqu'à sa libération ; `NULL` / 0 pour un blob `NULL` |
| `void osatura_blob_free(B *)` | libère un blob exactement une fois ; `NULL` est autorisé |

## Propriété et durée de vie

* **Les chaînes et tableaux sont copiés dans des tampons que vous fournissez ; les résultats d'export sont des blobs que vous libérez.** Chaque résultat de type chaîne ou tableau est copié dans un tampon que *vous* fournissez. La seule mémoire appartenant à la bibliothèque est un `OsaturaBlob` issu de `osatura_export`, `osatura_render_png`, `osatura_render_exr` ou `osatura_to_json` : lisez-le avec `osatura_blob_data` / `osatura_blob_size`, puis libérez-le une fois avec `osatura_blob_free`. Il n'existe pas de `osatura_free` générique. Les seuls pointeurs que renvoie la bibliothèque sont `osatura_version()` (statique) et `osatura_last_error()` (voir plus bas), ainsi que les deux handles opaques.
* **Handles.** `osatura_open` renvoie un handle de moteur ; `osatura_player_open` renvoie un handle de player. Chacun doit être fermé exactement une fois avec son propre `_close` ; fermer `NULL` est sans danger ; utiliser un handle après l'avoir fermé, ou le fermer deux fois, est un comportement indéfini.
* **Le moteur et les players sont indépendants.** Un player possède une copie du projet, donc fermer d'abord le moteur est permis et courant (l'exemple le fait). Plusieurs players peuvent être ouverts à partir d'un même moteur.
* **Vos tampons.** Les pointeurs d'entrée n'ont besoin d'être valides que pendant la durée de l'appel ; la bibliothèque n'en conserve aucun. Les tampons de sortie doivent réellement avoir la capacité que vous indiquez ; la bibliothèque écrit au plus `cap` octets (chaînes) ou `cap_floats` flottants.
* **`osatura_last_error()`** renvoie un pointeur vers une chaîne locale au thread. Il reste valide jusqu'au prochain appel *en échec* sur le même thread (cet appel l'écrase) ou jusqu'à la fin du thread. Copiez-la si vous en avez besoin plus longtemps. Ne la libérez pas.

## Gestion des erreurs

1. Testez toujours la valeur de retour (`NULL` pour les deux fonctions `_open` et `osatura_player_open`, un `int` négatif dans les autres cas, `0` pour un accesseur à qui l'on passe un handle `NULL`).
2. Lisez ensuite `osatura_last_error()` sur le même thread pour connaître la raison : par exemple `invalid project JSON: ...`, `output buffer needs 1228800 bytes`, `invalid argument: input 'excited' is not a number`.
3. **Le texte d'erreur n'est défini que par les échecs et n'est jamais effacé par un succès.** Après un appel ultérieur réussi, il contient toujours l'ancien message. N'utilisez jamais « la chaîne n'est pas vide » comme test d'échec.
4. Les textes d'erreur proviennent de la façade Rust (`Display` de `ApiError`) ; ils s'adressent aux humains, ils ne constituent pas un contrat. L'ABI C n'expose pas le *type* d'erreur.
5. Une panique à l'intérieur du moteur est interceptée et signalée par `internal error (panic) inside osatura` avec le retour d'échec habituel ; le handle reste en principe utilisable, mais considérez-le comme un bug et signalez-le.

## Threads

* Chaque handle peut être utilisé depuis **un seul thread à la fois** (le contrat de l'en-tête). Vous pouvez déplacer un handle d'un thread à l'autre tant que les appels ne se chevauchent jamais ; des handles différents peuvent s'exécuter en parallèle sur des threads différents (les types Rust sous-jacents sont `Send + Sync`, mais le contrat C reste prudent).
* La chaîne de dernière erreur est **par thread** : lisez-la sur le thread qui a fait l'appel en échec.
* Il n'y a pas de callbacks et la bibliothèque ne démarre aucun thread de son propre chef.

## Exemple C complet

Charge un projet, affiche des informations et les positions des os, rend une image dans un fichier PPM (en demandant d'abord la taille avec `NULL`, puis en allouant exactement), exécute la première machine à états du projet, lit ses événements avec le motif d'interrogation de taille et montre le chemin d'erreur. Vérifié avec gcc, y compris sous AddressSanitizer/UBSan (aucune anomalie), avec la bibliothèque partagée comme avec la statique.

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

Compilé et exécuté avec le projet `osatura demo` étendu d'une machine à états (`hero`, entrée `excited`, états `idle` et `wave`) :

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

Le dépôt fournit aussi `osatura-capi/tests/smoke.c` (ouverture, rendu, somme de contrôle) et les tests Rust `osatura-capi/tests/abi.rs` (pointeurs nuls, mauvais index, tailles de tampon, sûreté face aux paniques).

## Défilement et niveau de détail en C

Un projet avec un conteneur de mise en page et une région de défilement (le projet de test de défilement des tests Rust : dix lignes de 20 unités dans une colonne, un viewport épinglé aux bords gauche, haut et bas). Vérifié :

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

Sortie : `scroll 'list': max=(0,80) dragged offset=(0,80) settled=1` (glisser jusqu'à 500 borne au maximum lorsque l'élasticité est désactivée), puis `set_screen rc=0, lod level=7` (avec jusqu'à 4 px d'erreur acceptés, le niveau préconstruit le plus grossier convient sur ce minuscule projet ; le nombre dépend du projet) et `render size 100x120`. Notez la variable `rc` explicite dans le source : l'ordre dans lequel C évalue les arguments d'une fonction n'est pas spécifié, donc ne mettez jamais dans un même `printf` l'appel qui modifie le player et l'appel qui le lit.

## Exemple Python (ctypes)

Bibliothèque standard uniquement. Déclarez `argtypes` et `restype` pour chaque fonction que vous appelez : ctypes suppose `int` sinon et tronque les pointeurs 64 bits. Vérifié sur Python 3 avec la bibliothèque de debug ; sa sortie PPM est identique octet pour octet à celle de l'exemple C.

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

## Autres langages

Tout ce qui sait appeler une fonction C fonctionne de la même manière : déclarez les 53 fonctions, conservez les pointeurs opaques sous forme de `void *` et suivez les conventions de tampon ci-dessus. Remarques propres aux hôtes courants (non testées ici, simples conséquences du contrat ci-dessus) : en C#/.NET, utilisez `[DllImport]` avec `CallingConvention.Cdecl` et transmettez les chaînes sous forme de tableaux d'octets UTF-8 ; en Go avec cgo, ne passez pas de pointeurs Go aux appels, sauf pour des tampons qui vivent le temps de l'appel ; en Rust, préférez la façade directement plutôt que cette ABI.

## Gestion des versions

* `OSATURA_ABI_VERSION` / `osatura_abi_version()` vaut actuellement **1**. Elle ne change que lorsqu'une fonction existante est supprimée ou que sa signification change. Tous les ajouts jusqu'ici (les fonctions de player dans la façade 1.1, la mise en page, le défilement et le niveau de détail dans la 1.2) étaient additifs et l'ont laissée à 1 : un programme compilé avec un ancien en-tête continue donc de fonctionner avec une bibliothèque plus récente, et un en-tête plus récent avec une bibliothèque plus ancienne échoue à l'édition de liens ou au chargement faute du symbole manquant. Vérifiez `osatura_abi_version()` au démarrage et comparez-la avec la macro.
* `osatura_version()` est la version de la crate `osatura-capi` (0.1.0), pas celle de la façade. Aucune fonction C ne renvoie la version de la façade (`API_VERSION`).
* La façade 1.4 a ajouté les fonctions d'édition de base et d'export en mémoire ci-dessus (additif, version d'ABI toujours 1). Non exposés dans l'ABI C : le rendu SVG, la suppression de clés, le contrôle de version, la CI d'animation, le tracé, le mocap, le fuzzing, l'export à taille cible, les scripts. Ils sont disponibles en Rust et via la CLI.

## Particularités connues

Les commentaires de l'en-tête ont été corrigés pour correspondre au comportement (ils divergeaient auparavant) ; les points à connaître :

* `osatura_render_rgba` écrit `*w` / `*h` en cas de succès et en cas de -2 (tampon trop petit), ainsi que lorsqu'une erreur de rendu ultérieure survient après que `frame` et `scale` ont été acceptés. Elle ne les écrit **pas** lorsque le moteur est nul ou lorsque `frame` / `scale` sont rejetés (-1). Test de régression : `documented_size_outputs_and_last_error_semantics` dans `osatura-capi/tests/abi.rs`.
* Le texte de dernière erreur est local au thread, n'est écrasé que par le prochain appel *en échec* et n'est jamais effacé par un succès. Copiez-le si vous en avez besoin plus longtemps.
* `osatura_width` / `osatura_height` arrondissent à l'entier la taille flottante de la scène ; la taille de rendu réelle est `round(size * scale)`, utilisez donc les `w` / `h` renvoyés par l'appel de rendu.
* La note de l'en-tête sur les régions de défilement qui suivent `osatura_player_set_value` se trouve entre les déclarations du défilement et celles du niveau de détail ; elle décrit `osatura_player_set_value`.
