# OSATURA pour WebAssembly

*Version d'ABI 1 (`arm_abi_version()`). Crate `osatura-wasm`, wrapper JavaScript `osatura-wasm/js/osatura.js`, page de démonstration `osatura-wasm/index.html`, script de compilation `osatura-wasm/build-wasm.sh`. Public : développeurs web qui veulent lire des projets OSATURA dans un navigateur ou dans node, et la première personne qui exécute le module dans un navigateur. Documentation CC-BY-4.0 ; exemples de code MPL-2.0.*

> ## STATUT : COMPILÉ ET TESTÉ EN CI, EXÉCUTÉ SEULEMENT DANS CHROMIUM SANS INTERFACE
>
> **Au 24 oct. 2026, le module est compilé sur GitHub Actions** (`wasm32-unknown-unknown`, build de débogage dans `ci.yml`, build release dans `release.yml`) et **le test du wrapper réussit sur le vrai module sous Node 20** (`node osatura-wasm/js/test.mjs demo.json`, y compris sa partie avec le vrai module). Le build release est joint à la préversion en brouillon `v0.1.0-alpha.3` sous le nom `osatura-web.tar.gz` (environ 0,9 Mo compressé, avec `osatura.js` et la page de démonstration). **Il a été chargé une fois dans Chromium 141 sans interface sous Linux, sans GPU** (résultats à la fin de cette page, produits par `osatura-wasm/js/browser-check.js`) ; Firefox, Safari, les téléphones et les vrais GPU n'ont pas été essayés, de sorte que les affirmations de cette page à leur sujet ne sont pas un comportement observé, et l'environnement de développement lui-même ne peut toujours pas installer la cible (le téléchargement de sa bibliothèque standard y est bloqué).
>
> Ce qui a été testé, et jusqu'où cela va :
>
> | Couche | Test | Prouve | Ne prouve pas |
> |---|---|---|---|
> | Les fonctions ABI Rust (`arm_*`) | `cargo test -p osatura-wasm` : 8 tests unitaires sur l'**hôte**, qui appellent directement les fonctions exportées | la vérification des arguments, les codes d'erreur, le protocole de tampon de sortie, les machines à états, les appels de mise en page / défilement / niveau de détail, le refus d'une sortie surdimensionnée avant allocation | le comportement du module compilé |
> | Le module compilé | CI : `cargo build --target wasm32-unknown-unknown`, puis `node osatura-wasm/js/test.mjs demo.json` avec le vrai `.wasm` | que les dépendances compilent pour wasm32, que le module exporte ce que le wrapper attend, et que le chargement, le rendu et les appels de machine à états fonctionnent dans le moteur WebAssembly de Node | les navigateurs, la vitesse et la mémoire en release, les paniques (un trap), la page de démonstration |
> | Le wrapper JavaScript | le même script face à un **mock JavaScript** des exports (trois sections de mock : rendu, machine à états, mise en page) | les séquences d'appels du wrapper, le passage des chaînes, la traduction des erreurs, la copie des résultats | - |
> | La page de démonstration | chargée dans Chromium sans interface **sans** le fichier `.wasm` | les chemins d'import du module se résolvent et la page indique « wasm not found » au lieu de planter | tout rendu dans un navigateur |
> | Dépendances | `cargo tree -p osatura-wasm` : ni `rhai` ni `smartstring` dans l'arbre | le moteur de script ne fait pas partie du module | - |
>
> **Risques restants :** (1) *Résolu en CI :* la compilation de `zip`, `tiny-skia`, `png`, `gif`, `jpeg-decoder`, `i_overlay` et de leurs dépendances pour `wasm32-unknown-unknown`. (2) `std::time::Instant` / `SystemTime`, `process::id` et `thread::spawn` **paniquent** sur `wasm32-unknown-unknown` : depuis la vague 7, le core ne les appelle plus directement. Tout usage passe par `osatura_core::clock` (un type d'horodatage, `unix_seconds`, `process_id` ; le constructeur de proxys LOD se rabat sur le thread appelant lorsqu'un thread ne peut pas être créé), qui sur cette cible lit une horloge fournie par l'hôte (l'export `arm_set_clock(mono_ms, unix_ms)`, alimenté par le wrapper JavaScript avec `performance.now()` et `Date.now()` avant `load`, `svg` et `rgba` ; un hôte qui ne l'appelle jamais obtient des durées de 0, de sorte que les budgets de temps n'expirent jamais) et ne panique jamais. Un test (`osatura-core/tests/wasm_safety.rs`) échoue lorsqu'un nouvel usage direct apparaît en dehors de `clock.rs` ; le test Node en CI exerce les chemins qu'il atteint, pas tous. Rhai (qui exigerait aussi sa feature `no_time`) ne fait pas partie du build. (3) Les modules liés au système de fichiers et aux processus compilent mais échouent à l'exécution : `audio.rs` (ouverture de fichier), `soundtrack.rs` (ffprobe), `snapshots.rs`, `open_formats.rs` (répertoires d'export), `agent.rs` (enregistrement), ainsi que le chargement des images et des normal maps à chemin relatif dans `engine.rs`, `meshtrace.rs` et `postfx.rs` ; ils renvoient des erreurs au lieu de fonctionner dans un navigateur, et aucune feature cargo ne les protège. (4) Les fonctions en virgule flottante (`sin`, `cos`, `powf`, ...) proviennent d'une autre bibliothèque mathématique sur wasm32, de sorte que les pixels rendus peuvent différer légèrement de ceux du natif (l'étape 6 de la liste de vérification le mesure). (5) Sur `wasm32-unknown-unknown`, le réglage par défaut est `panic = "abort"`, `catch_unwind` ne peut rien intercepter, et une panique Rust interrompt toute l'instance (`WebAssembly.RuntimeError: unreachable`). Voir la [liste de vérification](#liste-de-vérification-pour-la-première-personne-en-mesure-de-lexécuter-dans-un-navigateur) à la fin.

Sommaire : [De quoi il s'agit](#de-quoi-il-sagit) - [Compilation](#compilation) - [Modèle mémoire et ABI](#modèle-mémoire-et-abi) - [Référence des exports](#référence-des-exports) - [Le wrapper JavaScript](#le-wrapper-javascript) - [La page de démonstration](#la-page-de-démonstration) - [Différences avec l'ABI C](#différences-avec-labi-c) - [Aspérités connues](#aspérités-connues) - [Liste de vérification](#liste-de-vérification-pour-la-première-personne-en-mesure-de-lexécuter-dans-un-navigateur)

## De quoi il s'agit

`osatura-wasm` est une `cdylib` de **fonctions `extern "C"` simples** (sans `wasm-bindgen`, sans code de liaison généré) au-dessus de la façade Rust stable ([API.md](API.md)). Compilée pour `wasm32-unknown-unknown`, elle donne un module capable d'ouvrir un projet, de rendre des images sous forme de texte SVG ou de pixels RGBA et d'exécuter des machines à états avec mise en page, défilement et niveau de détail. Elle ne peut ni éditer, ni exporter, ni versionner, ni exécuter des outils ; elle n'a ni système de fichiers, ni réseau, ni audio, ni threads. `osatura.js` est un module ES sans dépendance (navigateurs et node >= 18) qui masque la gestion des pointeurs.

Licence : MPL-2.0 (crate et wrapper). Le module dépend de `osatura-core` avec `default-features = false` : pas de moteur de script ni de crate tierce sous licence MPL.

## Compilation

Vous n'avez pas à le compiler : le brouillon de version contient `osatura-web.tar.gz` (module, wrapper, page de démonstration). Pour le compiler vous-même, il faut une machine où la cible peut être installée (et, facultativement, `wasm-opt` de Binaryen pour réduire la taille). La CI le compile avec `cargo build --release --target wasm32-unknown-unknown -p osatura-wasm --locked`.

```sh
rustup target add wasm32-unknown-unknown
./osatura-wasm/build-wasm.sh
```

Le script (`osatura-wasm/build-wasm.sh`, un script shell POSIX) fait exactement ceci, depuis la racine du dépôt :

1. `cargo build --target wasm32-unknown-unknown --release -p osatura-wasm`
2. copie `target/wasm32-unknown-unknown/release/osatura_wasm.wasm` vers `osatura-wasm/osatura_wasm.wasm` ;
3. si `wasm-opt` est dans le `PATH`, réécrit ce fichier avec `wasm-opt -Oz` ;
4. affiche la taille du résultat et une suggestion d'exécuter le test node ou de servir la démo.

**Le script lui-même n'est pas ce que la CI exécute** (la CI appelle directement `cargo build` et copie le module). Sa suggestion finale indique `node osatura-wasm/js/test.mjs` ; avec un vrai `.wasm` présent, le test a en plus besoin d'un chemin de projet comme premier argument (`node osatura-wasm/js/test.mjs project.json`), sinon il s'arrête avec « pass a project JSON path as argv[2] ». Le module release du brouillon pèse environ 0,9 Mo en archive compressée ; les tailles brute et après `wasm-opt -Oz` ne sont pas consignées ici.

Servez ensuite le dossier en HTTP (les navigateurs ne chargent ni modules ni `.wasm` depuis `file://`) et ouvrez la page de démonstration :

```sh
cd osatura-wasm && python3 -m http.server 8000      # then open http://localhost:8000/index.html
```

Si vous utilisez `WebAssembly.instantiateStreaming` (le wrapper le fait lorsque vous lui passez une `Response`), le serveur doit envoyer `Content-Type: application/wasm` ; le chemin par URL du wrapper utilise `fetch(...).arrayBuffer()` et n'a pas cette exigence. Si votre page définit une Content-Security-Policy, elle doit autoriser la compilation WebAssembly (`'wasm-unsafe-eval'` dans `script-src`).

Une autre cible, `wasm32-wasip1`, se compile avec `-C panic=unwind` afin que `catch_unwind` fonctionne (le source le mentionne) ; elle n'a pas été essayée et nécessite un runtime ou une couche de compatibilité WASI.

## Modèle mémoire et ABI

Le module possède une **mémoire linéaire** (exportée sous le nom `memory`). L'hôte ne partage jamais de pointeurs avec le moteur, sauf selon ces règles :

* **Entrée.** Pour transmettre des octets (le JSON du projet, ou un nom), l'hôte appelle `arm_alloc(len)`, qui renvoie un pointeur vers `len` octets **mis à zéro** (nul lorsque `len` vaut 0 ou dépasse 256 Mio). L'hôte écrit dans `new Uint8Array(memory.buffer, ptr, len)`, appelle la fonction avec `(ptr, len)`, puis libère le bloc avec `arm_free(ptr, len)` en utilisant le **même** pointeur et la même longueur. Le moteur copie ce dont il a besoin ; il ne conserve aucun pointeur vers votre bloc.
* **Sortie.** Les résultats sont produits dans un tampon **à l'intérieur du handle** : après un appel qui renvoie une longueur `n >= 0`, `arm_out_ptr(h)` / `arm_out_len(h)` donnent les octets (`n` est égal à la longueur) et, pour les images, `arm_out_width(h)` / `arm_out_height(h)` donnent la taille (0 après des résultats SVG ou texte). Le pointeur reste valide **jusqu'au prochain appel qui produit une sortie, ou jusqu'à `arm_close`**. L'hôte doit copier les octets immédiatement (le wrapper le fait : `.slice()`). Ne conservez jamais une vue `Uint8Array` d'un appel à l'autre : outre qu'elle est écrasée, tout appel peut agrandir la mémoire du module, ce qui détache `memory.buffer` et invalide toutes ses vues. Relisez `memory.buffer` après chaque appel.
* **Erreurs.** Une fonction qui échoue renvoie `-1` (ou un handle nul) et mémorise le message ; `arm_error_ptr()` / `arm_error_len()` donnent ses octets UTF-8 (sans terminateur NUL). L'emplacement est global au module (qui est mono-thread), est écrasé par l'échec suivant et **n'est pas effacé par les succès** : lisez-le donc uniquement juste après un échec.
* **Chaînes en entrée.** `(ptr, len)` en UTF-8, `len` entre 1 et 4096 (une chaîne vide ou plus longue est une erreur).
* **Handles.** `arm_open` renvoie un pointeur opaque vers un handle (nul en cas d'erreur). Un handle contient l'animation, le player facultatif et le tampon de sortie. Fermez-le avec `arm_close` exactement une fois ; `null` est autorisé.
* **Limites.** JSON du projet de 1 octet à 256 Mio ; `arm_render_rgba` refuse avant d'allouer si l'image dépasse 256 Mio (`output would be WxH pixels (N bytes), more than the limit of 268435456 bytes: lower the scale`) ; échelle comprise entre 0,05 et 4 ; les images hors de la timeline sont bornées ; longueurs renvoyées en `i32`. Le rendu nécessite de la mémoire pour l'image plus des tampons de travail ; l'espace d'adressage 32 bits (4 Gio, moins dans certains navigateurs) est le véritable plafond.
* **Pas de threads, pas de promesse sur les paniques.** Le module est mono-thread. Les paniques ne sont interceptées que sur les cibles qui déroulent la pile ; sur `wasm32-unknown-unknown`, une panique interrompt l'instance et l'hôte doit l'abandonner et en créer une nouvelle. Le moteur est écrit pour ne pas paniquer sur une entrée utilisateur et vérifie les tailles avant d'allouer, mais cette couche ne peut pas promettre de survivre à un bug.

## Référence des exports

37 exports plus `memory`. `ptr` et `len` sont des `i32` dans la signature wasm (`usize` en Rust). `h` est un handle. Le retour `-1` signifie une erreur, sauf indication contraire.

| Export | Signature (vue Rust) | Résultat |
|---|---|---|
| `arm_abi_version` | `() -> u32` | 1 |
| `arm_set_clock` | `(mono_ms: f64, unix_ms: f64)` | facultatif : donne l'heure au module (`performance.now()`, `Date.now()`) ; les valeurs invalides sont ignorées. Ajouté après la fixation de la version d'ABI 1 ; le wrapper l'appelle lorsqu'il est présent |
| `arm_alloc` | `(len) -> *mut u8` | bloc mis à zéro, nul si `len` vaut 0 ou dépasse 256 Mio |
| `arm_free` | `(ptr, len)` | libère un bloc issu de `arm_alloc` (nul / longueur zéro ignorés) |
| `arm_open` | `(json_ptr, json_len) -> *mut Handle` | handle ou nul ; valide le projet |
| `arm_close` | `(h)` | libère le handle |
| `arm_frame_count`, `arm_fps`, `arm_width`, `arm_height` | `(h) -> u32 / f32 / f32 / f32` | `width` / `height` sont la taille de la scène en unités de scène (non arrondie) ; 0 et une erreur pour un handle nul |
| `arm_render_svg` | `(h, frame: f32, show_bones: i32) -> i32` | texte SVG dans le tampon de sortie ; sa longueur en octets |
| `arm_render_rgba` | `(h, frame: f32, scale: f32) -> i32` | RGBA8 à alpha droit dans le tampon de sortie ; longueur en octets (`w*h*4`) ; taille via `arm_out_width/height` |
| `arm_out_ptr`, `arm_out_len` | `(h) -> *const u8 / usize` | le tampon de sortie (nul / 0 pour un handle nul) |
| `arm_out_width`, `arm_out_height` | `(h) -> u32` | taille de l'image du dernier rendu RGBA (0 après du texte) |
| `arm_machine_count` | `(h) -> u32` | machines du projet |
| `arm_machine_name` | `(h, index) -> i32` | nom dans le tampon de sortie ; sa longueur ; -1 pour un mauvais index |
| `arm_player_start` | `(h, name_ptr, name_len) -> i32` | démarre (ou redémarre) la machine ; 0 |
| `arm_player_stop` | `(h)` | abandonne le player |
| `arm_player_set_bool` | `(h, ptr, len, value: i32) -> i32` | 0 |
| `arm_player_set_number` | `(h, ptr, len, value: f64) -> i32` | 0 |
| `arm_player_fire` | `(h, ptr, len) -> i32` | 0 |
| `arm_player_set_value` | `(h, ptr, len, value: f64) -> i32` | 0 ; valeur externe pour les bindings / l'entrée de défilement |
| `arm_player_advance` | `(h, dt: f64) -> i32` | 0 ; `dt` dans `0..=10` secondes |
| `arm_player_state` | `(h, layer_ptr, layer_len) -> i32` | nom de l'état dans le tampon de sortie ; longueur ; -1 calque inconnu |
| `arm_player_events` | `(h) -> i32` | événements en attente en JSON dans le tampon de sortie, **consommés** ; longueur |
| `arm_player_render_svg` | `(h, show_bones: i32) -> i32` | SVG de la pose courante dans le tampon de sortie |
| `arm_player_render_rgba` | `(h, scale: f32) -> i32` | RGBA de la pose courante ; taille via `arm_out_width/height` |
| `arm_player_set_canvas` | `(h, w: f32, h: f32) -> i32` | 0 |
| `arm_player_layout` | `(h) -> i32` | JSON `{"canvas":[w,h],"scrolls":[ids],"lod":level}` dans le tampon de sortie |
| `arm_player_scroll_drag` | `(h, id_ptr, id_len, x: f32, y: f32) -> i32` | 0 |
| `arm_player_scroll_release` | `(h, id_ptr, id_len, vx: f32, vy: f32) -> i32` | 0 |
| `arm_player_scroll_info` | `(h, id_ptr, id_len) -> i32` | JSON `{"offset":[x,y],"max":[x,y],"settled":bool}` dans le tampon de sortie |
| `arm_player_set_screen` | `(h, w_px: f32, h_px: f32, max_error_px: f32) -> i32` | 0 ; choisit le niveau de détail |
| `arm_error_ptr`, `arm_error_len` | `() -> *const u8 / usize` | texte de la dernière erreur |

Tous les appels `arm_player_*` sauf `arm_player_start` exigent un player démarré (« no player started (call arm_player_start) » sinon). Les appels de player sont les méthodes `Player` de la façade ([API.md](API.md)).

## Le wrapper JavaScript

`osatura-wasm/js/osatura.js` exporte `class Osatura` (aussi comme export par défaut). Chaque méthode qui modifie quelque chose renvoie `this`, ce qui permet d'enchaîner les appels. Les erreurs sont levées sous forme d'`Error` avec le message du moteur.

| Membre | Description |
|---|---|
| `static async instantiate(source, imports = {})` | renvoie l'objet brut des exports. `source` : un `ArrayBuffer` ou tableau typé (octets), une `Response` (utilise `instantiateStreaming` ; nécessite le MIME `application/wasm`), ou une chaîne URL / `URL` (utilise `fetch` + `arrayBuffer`). Le module n'a aucun import, donc `imports` reste `{}`. |
| `static async create(source, project)` | `source` est soit des exports déjà instanciés (un objet avec `arm_open`), soit tout ce qu'accepte `instantiate` ; `project` est du texte JSON ou un objet simple (`JSON.stringify` est appliqué). Renvoie un `Osatura`. |
| `load(project)` | charge ou remplace le projet ; lève une erreur en cas d'échec et **conserve alors utilisable le projet précédemment chargé**. Renseigne `frames`, `fps`, `width`, `height`. |
| `frames`, `fps`, `width`, `height` | informations issues du dernier `load` (`width` / `height` en unités de scène, flottants) |
| `svg(n, showBones = false)` | `string` : document SVG pour l'image `n` (images fractionnaires permises, bornées) |
| `rgba(n, scale = 1)` | `{ width, height, data }` : `data` est un `Uint8ClampedArray` (RGBA à alpha droit, une copie qui vous appartient) |
| `frame(n, { kind = 'image', scale = 1, bones = false } = {})` | `kind: 'svg'` donne `{ svg }` ; sinon `{ image: ImageData }` dans un navigateur ou `{ width, height, data }` là où `ImageData` n'existe pas (node) |
| `machines()` | `string[]` noms des machines |
| `startPlayer(name)`, `stopPlayer()` | démarre (redémarre) / abandonne le player ; les entrées reviennent à leurs valeurs par défaut |
| `setBool(name, v)`, `setNumber(name, v)`, `fire(name)`, `setValue(path, v)` | entrées et valeurs externes ; un nom vide lève `empty string`, un nom de plus de 4096 octets lève une erreur |
| `advance(dt)` | fait avancer la machine et les régions de défilement de `dt` secondes (`0..=10`) |
| `state(layer)` | `string` état courant d'un calque de machine |
| `events()` | `{ time, layer, kind: 'entered' \| 'exited', state }[]`, **consommés** par l'appel |
| `setCanvas(w, h)` | mise en page adaptative pour une taille de canevas en unités de scène |
| `layout()` | `{ canvas: [w, h], scrolls: string[], lod: number }` |
| `scrollDrag(id, x, y)`, `scrollRelease(id, vx = 0, vy = 0)` | glissement du pointeur / vitesse de relâchement en px et px/s |
| `scrollInfo(id)` | `{ offset: [x, y], max: [x, y], settled: boolean }` |
| `setScreen(widthPx, heightPx, maxErrorPx = 0.5)` | choisit le niveau de détail pour une taille d'écran |
| `playerSvg(showBones = false)`, `playerRgba(scale = 1)` | la pose courante du player |
| `play(canvas, { scale = 1, loop = true, onFrame } = {})` | dessine la timeline sur un canevas 2D avec `requestAnimationFrame` (se rabat sur `setTimeout` hors d'un navigateur) ; renvoie une fonction d'arrêt |
| `playMachine(canvas, { scale = 1, onTick } = {})` | fait avancer la machine démarrée selon le temps réel écoulé (au plus 0,25 s par tick) et la dessine ; `onTick(dt)` s'exécute avant chaque avance, c'est donc là qu'il faut définir les entrées ; renvoie une fonction d'arrêt |
| `stop()` | arrête `play` / `playMachine` |
| `close()` | arrête et libère le handle ; à appeler une fois terminé |

Le wrapper effectue les recherches de `x.memory.buffer` à l'intérieur de chaque appel et copie les résultats avec `.slice()`, de sorte qu'un agrandissement de la mémoire ne peut pas lui laisser une vue périmée. Chaque objet `Osatura` possède un handle ; créez plusieurs objets (ils peuvent partager un même `exports`) pour conserver plusieurs projets. Ne partagez pas un même objet entre des tâches asynchrones concurrentes qui entrelacent les appels et les lectures de résultats : le résultat de chaque appel reste dans le tampon de sortie du handle jusqu'à l'appel suivant.

### Exemple (séquence d'appels)

Voici la séquence d'appels exacte que le test node exerce. **Elle a été exécutée contre le mock JavaScript** (la doublure de `test.mjs` avec des réponses préenregistrées : machine `hero`, calque `body`, entrée booléenne `go`, région de défilement `list`), pas contre un vrai module.

```js
import { Osatura } from './js/osatura.js';

// 1. Instantiate the module once (a URL in the browser; in node pass the bytes: readFileSync('osatura_wasm.wasm')).
const exports = await Osatura.instantiate('./osatura_wasm.wasm');

// 2. Load a project (JSON text or a plain object). Throws Error(<engine message>) when it is invalid.
const anim = await Osatura.create(exports, projectJsonText);
console.log(`${anim.width}x${anim.height} stage units, ${anim.frames} frames @ ${anim.fps} fps`);

// 3. Render. Frames may be fractional and are clamped to the timeline.
const svg = anim.svg(10, /* showBones */ false);        // string
const { width, height, data } = anim.rgba(10, 0.5);     // data: Uint8ClampedArray, straight-alpha RGBA, a copy you own

// 4. Interactive playback with a state machine.
console.log(anim.machines());                           // ['hero']
anim.startPlayer('hero').setBool('go', true).advance(0.1);
console.log(anim.state('body'), anim.events());         // 'b', [{time, layer, kind, state}, ...] (consumed)
const poseSvg = anim.playerSvg();
const pose = anim.playerRgba(0.5);

// 5. Layout and scrolling.
anim.setCanvas(100, 120);
anim.scrollDrag('list', 0, 50).scrollRelease('list', 0, 100);
console.log(anim.scrollInfo('list'));                   // { offset: [0, 50], max: [0, 80], settled: false }
anim.setScreen(100, 120, 4);                            // pick a level of detail within 4 px
console.log(anim.layout());                             // { canvas: [100, 120], scrolls: ['list'], lod: 2 }

// 6. Always release the handle (it frees the project inside the module's memory).
anim.close();
```

Dans une page de navigateur :

```html
<canvas id="c"></canvas>
<script type="module">
  import { Osatura } from './js/osatura.js';
  const x = await Osatura.instantiate('./osatura_wasm.wasm');
  const text = await (await fetch('./project.json')).text();
  const anim = await Osatura.create(x, text);

  // Timeline playback: requestAnimationFrame loop, loops at the end, returns a stop function.
  const stop = anim.play(document.getElementById('c'), { scale: 1, loop: true });

  // Or run a state machine in real time; onTick(dt) runs before each advance, so set inputs there.
  // anim.startPlayer('hero');
  // const stopMachine = anim.playMachine(canvas, { onTick: () => anim.setBool('go', keyIsDown) });
</script>
```

## La page de démonstration

`osatura-wasm/index.html` est une page minimale : elle importe le wrapper, instancie `./osatura_wasm.wasm`, et propose un champ de fichier (projet `.json`) et un bouton Stop. Choisir un fichier crée un `Osatura`, affiche `WIDTHxHEIGHT, N frames @ FPS fps` et lance `play(canvas)` ; les erreurs du moteur s'affichent dans le texte à côté des boutons. Si le module ne peut pas être récupéré ou compilé, elle affiche `wasm not found (not built yet): <reason>`. Testée uniquement dans cet état « non compilé », dans Chromium sans interface : la page se charge, l'import du module se résout, le message apparaît (avec la page 404 du serveur en guise de « module », la raison est `expected magic word 00 61 73 6d, found 3c 21 44 4f`, voir plus bas). Tout le reste n'est pas vérifié.

## Différences avec l'ABI C

| | ABI C ([C_ABI.md](C_ABI.md)) | WebAssembly |
|---|---|---|
| Chaînes en entrée | terminées par NUL | `(ptr, len)` copiées avec `arm_alloc` |
| Résultats en sortie | tampons fournis par l'appelant | le tampon de sortie du handle (`arm_out_ptr/len`), valide jusqu'au prochain appel produisant une sortie |
| Interrogation de la taille | un tampon `NULL` renvoie la taille | inutile : l'appel renvoie la longueur et les données sont déjà dans le tampon de sortie |
| Texte d'erreur | chaîne NUL `osatura_last_error()`, par thread | octets `arm_error_ptr/len`, global |
| Matrices d'os | `osatura_bone_world` | non exporté |
| SVG | non exporté | `arm_render_svg`, `arm_player_render_svg` |
| Événements | motif d'interrogation de taille, conservés si le tampon est trop petit | toujours consommés |
| Taille de la scène | entiers arrondis | flottants |

## Aspérités connues

Relevées à la lecture du code ; aucune n'a été vue en échec :

* `Osatura.instantiate(url)` vérifie `response.ok` et les erreurs de fetch et signale l'URL et le statut HTTP (tests avec fetch simulé dans `osatura-wasm/js/test.mjs` ; exécutés sous node, et en CI face au vrai module).
* `frame()` avec `kind: 'svg'` renvoie `{ svg }`, et non une chaîne comme `svg()`.
* La suggestion affichée par `build-wasm.sh` inclut désormais l'argument `<project.json>` dont `test.mjs` a besoin.
* `play()` dessine en convertissant chaque image en RGBA sur le CPU puis en appelant `putImageData` ; il n'y a ni audio, ni pause, ni recherche de position.
* Le wrapper n'exporte pas de déclarations TypeScript.

## Liste de vérification pour la première personne en mesure de l'exécuter dans un navigateur

Les étapes 1 et 5 ont été faites par la CI le 24 oct. 2026. Les étapes 3, 4, 6, 8, 9, 11 et 13 ont été faites le même jour dans **Chromium 141 sans interface (Linux, sans GPU)** avec `osatura-wasm/js/browser-check.js` ; les chiffres figurent dans le tableau ci-dessous. **Toujours ouvertes :** Firefox et Safari (étapes 11 et 12 là-bas), l'étape 12 sur la CSP et le type de contenu, la profondeur de pile de l'étape 10, et les étapes 7 et 14 du lecteur. Faites ceci dans l'ordre ; arrêtez-vous au premier échec et consignez le message exact. Les éléments marqués (risque) correspondent aux risques ci-dessus. Lorsque vous avez des résultats, mettez à jour [LIMITATIONS.md](LIMITATIONS.md) (section 1), la ligne R-090 de la section 9 de ROADMAP.md et le cadre de statut de cette page.

1. **Installer la cible et compiler.** `rustup target add wasm32-unknown-unknown && ./osatura-wasm/build-wasm.sh`. (risque) Vous verrez si `zip`, `tiny-skia`, `png`, `gif`, `jpeg-decoder` et `i_overlay` compilent. Consignez le compilateur et `rustc --version`.
2. **Pas de moteur de script.** `cargo tree --target wasm32-unknown-unknown -p osatura-wasm | grep -ci 'rhai\|smartstring'` doit afficher 0.
3. **Examiner les exports.** Dans node : `WebAssembly.Module.exports(new WebAssembly.Module(fs.readFileSync('osatura-wasm/osatura_wasm.wasm')))`. Attendez-vous à `memory` et aux 37 fonctions du [tableau des exports](#référence-des-exports), et à **aucun import** (`WebAssembly.Module.imports(...)` vide). Si `memory` manque ou si des imports apparaissent (par exemple `env.*` issus d'une feature de std), le wrapper doit être adapté.
4. **Taille.** Consignez la taille avant et après `wasm-opt -Oz`, ainsi que les tailles gzip / brotli. Mesurée une fois lors de l'exécution dans Chromium : voir le tableau des résultats à la fin de cette page.
5. **Lancer le test node avec un vrai projet.** `osatura demo demo.json && node osatura-wasm/js/test.mjs demo.json` doit afficher `real wasm tests: ok` après les trois sections de mock.
6. **Comparer avec le moteur de rendu natif.** Pour les images 0, 12 et 47 de la démo aux échelles 0,5 et 1, comparez `Osatura.rgba` avec `osatura render demo.json N out.png --scale S` décodé en RGBA. (risque) Attendez-vous à de petites différences sur les bords anticrénelés (bibliothèque mathématique différente) ; indiquez la différence absolue maximale et moyenne, ainsi que la fraction de pixels au-dessus de 24/255. Une grande différence signale un vrai bug.
7. **Exercer le player.** Construisez le projet de l'exemple de l'ABI C (demo + `take-record` + `clip add` + `machine add`, voir [C_ABI.md](C_ABI.md)), pilotez `startPlayer`, `setBool`, `advance`, `state`, `events`, `playerRgba` ; et le projet avec un conteneur et une région de défilement pour `setCanvas`, `scrollDrag`, `scrollRelease`, `scrollInfo`, `setScreen` (ce dernier construit des niveaux de détail : chronométrez-le, il est lent sans optimisation et n'a pas été chronométré sur wasm).
8. **Chemins à sécurité intégrée.** Fournissez un projet invalide, une chaîne vide, un fichier aléatoire de 5 Mo, un projet avec un chemin d'image `source.file` et un projet avec une piste sonore. Chaque cas doit produire une `Error` levée avec un message, pas un trap. (risque) Un trap (`RuntimeError: unreachable`) sur l'un d'eux est un bug à signaler ; les candidats les plus probables sont `Instant` / `SystemTime` ou des appels au système de fichiers atteints par un chemin inattendu.
9. **Gros rendu.** `rgba(0, 4)` sur la démo (2560 x 1920) et une scène de 12000 x 12000 à l'échelle 1 (doit être refusée avec le message « more than the limit », sans épuiser la mémoire). Surveillez la croissance de la mémoire et confirmez que le wrapper fonctionne encore après l'agrandissement de la mémoire (les vues sont relues à chaque appel).
10. **Profondeur de pile.** `wasm32-unknown-unknown` donne au thread principal une petite pile par défaut (1 Mio dans les chaînes d'outils actuelles). Ouvrez un projet avec des groupes profondément imbriqués et une longue chaîne d'expressions (voir `osatura-cli/tests/robust.rs` pour les entrées hostiles) et confirmez l'absence de trap par débordement de pile.
11. **Exécution dans un navigateur.** Servez le dossier, ouvrez `index.html` dans Chromium et Firefox, choisissez un projet, confirmez la lecture, redimensionnez, fermez et rouvrez plusieurs projets (contrôle des fuites : la mémoire ne doit pas croître sans limite ; `close()` libère le handle).
12. **`instantiateStreaming` et CSP.** Servez avec et sans `Content-Type: application/wasm`, et avec une CSP qui omet `'wasm-unsafe-eval'` ; documentez ce qui se passe.
13. **Performance.** Chronométrez `rgba(n, 1)` pour la démo 640 x 480 (natif release : environ 14 ms par image) et un projet lourd ; consignez le nombre d'images par seconde de `play()` dans le navigateur.
14. **Facultatif : paniques.** Décidez si le comportement de trap sur panique est acceptable pour votre usage. Sinon, essayez la voie `wasm32-wasip1` + `-C panic=unwind` mentionnée dans le source.

### Résultats de l'exécution Chromium (24 oct. 2026)

| Étape | Résultat |
|---|---|
| 4. Taille | module release 2 802 534 octets ; 853 804 en gzip ; 615 267 en brotli (non passé par `wasm-opt`) |
| 6. Comparaison avec le natif | image 12 de la démo, 640x480 : **0 octet différent** sur 1 228 800 (différence absolue moyenne et maximale 0) |
| 8. Sécurité en cas d'échec | JSON vide, absurde, de mauvaise forme, 5 Mo aléatoires et « no frames » lèvent chacun une `Error` avec un message ; aucun trap |
| 9. Gros rendu | l'échelle 4 (2560x1920) fonctionne ; une scène de 12000x12000 est refusée avec le message de limite et le module fonctionne encore ensuite |
| 11. Fuite | 60 cycles d'ouverture et de fermeture : mémoire linéaire de 54 591 488 octets avant et après |
| 13. Vitesse | `rgba` de la démo 640x480 : médiane 72,5 ms, p95 96,5 ms, max 125,4 ms (environ 14 i/s ; natif release environ 14 ms) |

Relancez-le avec `node osatura-wasm/js/browser-check.js <dossier osatura-web décompressé> <project.json>` (nécessite Playwright). La page n'a levé aucune erreur pendant l'exécution.
