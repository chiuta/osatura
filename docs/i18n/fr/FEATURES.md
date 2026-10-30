# OSATURA — matrice des fonctionnalités + listes de contrôle de référence et de la concurrence

> Formulation des versions : toutes les crates sont en `0.1.0` (les premiers builds sont la préversion en brouillon `v0.1.0-alpha.3`) ; « v0.5 / v0.6 / v0.7 » dans ce dépôt désignent des *niveaux de fonctionnalités* (voir [CHANGELOG.md](CHANGELOG.md)), et la section 9 de `ROADMAP.md` fait foi sur ce qui est construit.

État honnête de ce qui existe. **✅ terminé et testé · 🟡 partiel · ❌ non implémenté.**

> Note de périmètre : OSATURA est un moteur implémenté de manière indépendante (à partir de spécifications et d'articles publics ; les listes de fonctionnalités des concurrents ont été consultées pour définir le périmètre ; aucun code source tiers n'a été copié ; voir CLEANROOM.md pour la déclaration de recherche), et non une ré-implémentation d'un
> produit commercial ; il **n'atteint pas** la parité avec une suite commerciale vieille de 25 ans
> (interface mature, outils de dessin, entrée en direct, 3D, écosystème). Les premiers tableaux sont organisés par
> capacité ; la **seconde moitié de ce fichier** est une liste de contrôle, version par version, des fonctionnalités
> documentées publiquement du produit de référence (sources indiquées à la fin) avec
> l'état d'OSATURA pour chacune. Les « innovations » ne sont revendiquées que par rapport à la technique générique,
> et non comme absentes de tout autre outil.

---

# ★ ÉCHELLE DE CONSTRUCTION — du plus simple au plus complexe, avec le statut de droit d'auteur / propriété intellectuelle (niveau de fonctionnalités v0.7, 20-22 oct. 2026)

Cette section est l'**unique endroit où voir ce qui peut être construit, dans quel ordre, et si c'est sans risque**.
Elle remplace les listes de « candidats n° » dispersées plus bas ; les détails et les identifiants se trouvent dans `ROADMAP.md`
(R-xxx / I-xx), la méthode clean-room dans `CLEANROOM.md`, les licences dans `LICENSING.md`.

**Statut :** ✅ construit et testé · 🟡 en partie construit · ⬜ prévu (non commencé)
**Statut PI (après les alternatives de ROADMAP §8) :**
🟢 sans risque — standard ouvert, mathématiques de manuel, ou conception propre ·
🟠 sans risque *si* une condition est respectée (indiquée dans la dernière colonne) ·
🔴 à ne pas construire.
*Examen d'ingénierie, pas un avis juridique ; un juriste doit confirmer chaque 🟠 avant une publication commerciale.*

## Niveau 1 — Fondations (quelques heures à quelques jours chacune ; aucune exposition en matière de PI)
| Fonctionnalité | Statut | PI | Base sûre / condition |
|---|---|---|---|
| JSON de projet, validation, `evaluate(frame)` pure | ✅ | 🟢 | schéma propre |
| Chemins de Bézier, remplissages, contours, dégradés, tirets, contours à largeur variable | ✅ | 🟢 | géométrie de manuel |
| os FK, limites, IK à 2 os/FABRIK, contraintes | ✅ | 🟢 | cinématique de manuel |
| Clés, interpolation, expressions, bruit | ✅ | 🟢 | conception propre |
| Éditeur d'opérations de patch avec annuler/rétablir exact | ✅ | 🟢 | pointeurs JSON de style RFC 6902 |
| Export SVG, PNG, GIF, lecteur HTML, Lottie | ✅ | 🟢 | spécifications publiques |
| Journal de provenance, `stats` / `diff` sémantique, régression par images de référence (hash exact + tolérance), plafonds de ressources, test de fuzzing par mutation, verrou de version du format de fichier (R-001…R-005) | ✅ | 🟢 | conception propre |
| Vocabulaire de rôles standard propre + étiqueteur par nom (`tag`) (R-006) | ✅ | 🟢 | noms propres, mots d'anatomie courants |
| Export de JSON Schema (schéma R-003) | ⬜ | 🟢 | schéma propre |

## Niveau 2 — Bases de création et sorties ouvertes (quelques jours)
| Fonctionnalité | Statut | PI | Base sûre / condition |
|---|---|---|---|
| APNG, sprite sheet PNG+JSON | ✅ | 🟢 | spécifications PNG/APNG |
| WebM VP9 / AV1 avec alpha (via votre ffmpeg) | ✅ | 🟢 | codecs spécifiés ouvertement ; statut des brevets non évalué |
| Export OCA (Open Cel Animation) | ✅ | 🟢 | spécification publique ; import ⬜ |
| Import OpenRaster (.ora) | ✅ | 🟢 | spécification ouverte, limites anti zip-bomb |
| MP4 / H.264 | ✅ | 🟠 | uniquement via le ffmpeg *de l'utilisateur* ; ne jamais embarquer d'encodeur |
| Décalage/mise à l'échelle/inversion/copier/coller de clés, adoucissement par lot, décalage temporel par canal, marqueurs et régions, timecode SMPTE (R-010, R-011, R-044) | ✅ | 🟢 | édition générique |
| Instantanés de révision avec élagage et récupération après plantage (R-014) ; compositions de calques (R-015) ; événements de timeline avec charges utiles (R-045) ; changement de parent en préservant la pose de repos (partie de R-013) | ✅ | 🟢 | édition générique |
| Alignement/répartition, calque de référence, étiquettes masquer/solo/verrouiller (R-012, R-013, R-016) | ✅ | 🟢 | édition générique |
| Onion skin, trajectoires de mouvement (superpositions jamais exportées) (R-020) | ✅ | 🟢 | conception propre |
| Chaîne de post-effets : flou, ombre portée, lueur, contour, postérisation, pixellisation, niveaux/gamma, fondu (partie de R-022) | ✅ | 🟢 | traitement d'image de manuel |
| Trim paths, filtres par calque, masques, profondeur de champ (R-021, R-024, R-025, R-034) | ✅ (trim paths précalculés ; limites des effets de calque : voir ROADMAP §9) | 🟢 | mathématiques d'image standard |
| Import SVG (chemins, formes, groupes, transformations, remplissages unis ; dégradés/texte/clip signalés comme avertissements) (R-032) | ✅ | 🟢 | grammaire W3C SVG 1.1 |
| Import BVH (un axe de rotation → angles d'os, réduction bornée des clés) (R-036) | ✅ | 🟢 | format texte ouvert |
| Export du conteneur dotLottie (R-03A) | ✅ | 🟢 | spécification publique du conteneur (zip + manifeste) |
| Export SVG animé, import de séquences d'images, sortie OpenEXR/16 bits, export glTF (R-032, R-037…R-039) | 🟡 (glTF : formes de l'image 0 plus os et poids de morph, voir ROADMAP R-039) | 🟢 | formats publics |
| Import des repères de bouche Papagayo (Moho-switch) et Rhubarb (R-03B) | ✅ | 🟢 | lecture de leur seule sortie texte |

## Niveau 3 — Profondeur de rig et d'animation (quelques semaines)
| Fonctionnalité | Statut | PI | Base sûre / condition |
|---|---|---|---|
| Ressorts (précalculés), lattices (FFD), déformateur de spine, calques de commutation, particules, liquide, booléens | ✅ | 🟢 | manuel |
| Lip-sync audio → formes de bouche, générateur de cycle de marche, miroir, retime, clip, fondu enchaîné | ✅ | 🟢 | conception propre |
| **Pin warp** (remplace la déformation de type cage / ARAP) | ✅ | 🟢 | RBF de Wendland 1995 |
| **Poids de skin automatiques par diffusion de chaleur** | ✅ | 🟢 | équation de Laplace |
| **Enregistrement de performance** avec réduction bornée des clés | ✅ | 🟢 | Douglas–Peucker |
| **Mappage d'entrée** (signaux CSV/JSON → paramètres) | ✅ | 🟢 | aucun tracker embarqué |
| IK souple/étirable (R-051), pose drivers : curseur + joystick 2D (R-050), changement de vue piloté par l'angle (R-056), auto-rig par étiquettes (R-058) | ✅ | 🟢 | rigging générique ; mathématiques de manuel, conception propre |
| Contraintes de chemin, IK à verrouillage de pied, clés de changement de parent, skins, aimants (R-040…R-049, R-052…R-055, R-057) | 🟡 (construits : R-040…R-043, R-044, R-047, R-048, R-052…R-055, R-057 ; voir ROADMAP section 9 pour les parties marquées NOT done) | 🟢 | rigging générique |
| Générateurs d'inactivité (clignement, respiration, balancement) et chaînes de follow-through (R-047, R-048) | ✅ | 🟢 | à seed, déterministes |
| Vent, gravité, collisions, mèches/bandes de tissu (R-070…R-072) | 🟡 (R-070, R-071 terminés ; les bandes R-072 ne sont que des os, le tissu est une nappe plane) | 🟢 | manuels masses-ressorts / PBD |

## Niveau 4 — Outils intelligents (quelques semaines à quelques mois)
| Fonctionnalité | Statut | PI | Base sûre / condition |
|---|---|---|---|
| **Vectorisation image → vecteur** (remplace l'auto-trace par ML) | ✅ | 🟢 | marching squares + RDP |
| **Machine à états à journal d'événements** + bake vers des canaux (remplace les clones de machines à états de runtime) | ✅ | 🟢 | sémantique propre — conserver nos noms |
| **Retargeting par portée de l'effecteur** | ✅ | 🟢 | méthode propre |
| **Solveur de correspondance de pose** (stratégie d'évolution, tenant compte des limites d'articulation) | ✅ | 🟢 | Beyer & Schwefel |
| **Contrôle des flashs WCAG 2.3.1 / flashs rouges + rapport de mouvement de caméra + jumeau à mouvement réduit** | ✅ | 🟠 | définitions publiques uniquement ; dire « aide d'ingénierie », jamais « certifié » |
| **Linter de rig**, **contrats de mouvement**, **solveur de boucle sans raccord**, **coach des principes** (I-02, I-04, I-05, I-14) | ✅ | 🟢 | idées propres |
| **Ajustement de ressort par dessin du dépassement** (I-08) — ajuste fréquence/amortissement avec l'intégrateur propre au moteur | ✅ | 🟢 | idée propre |
| **Explain-this-pixel** pour les os et les calques (I-07) | ✅ | 🟢 | idée propre |
| Before | after | image de revue des différences (première étape de I-06 ; la métrique perceptuelle est `ci diff`, SSIM et CIEDE2000) | ✅ | 🟢 | idée propre |
| Éclairage par normal map, parallaxe, profondeur de champ (R-064, R-065, R-024) | ✅ | 🟢 | manuel |
| Lip-sync aligné sur la transcription (R-081) | ✅ (aligneur propre, `speech.rs`) | 🟠 | utiliser un aligneur à licence permissive ; vérifier la licence de son modèle |
| Suivi du visage/du corps par webcam alimentant le mappeur d'entrée (R-082) | 🟡 (détecteur propre entraîné uniquement sur des visages synthétiques ; webcams réelles et photographies non testées) | 🟠 | l'utilisateur apporte un tracker permissif ; n'embarquer aucun poids de modèle sauf si la licence le permet |
| Retargeting complet de mocap BVH avec appariement automatique par hiérarchie (R-083) | ✅ | 🟢 | s'appuie sur l'import BVH |
| Import PSD (R-035) | ⬜ | 🟠 | lecture seule, spécification publiée, aucune marque Adobe ; ou utiliser simplement ORA |

## Niveau 5 — Plateforme (quelques mois)
| Fonctionnalité | Statut | PI | Base sûre / condition |
|---|---|---|---|
| Studio dans le navigateur avec annuler/rétablir | 🟡 | 🟢 | interface propre ; aucune copie de dispositions ou de noms d'éditeurs |
| Runtime WASM, API Rust stable + ABI C, scripts en bac à sable (R-090…R-093) | 🟡 (R-091, R-092, R-093 terminés ; R-090 : le module wasm se compile et passe le test du wrapper sous Node en CI, et n'a été exécuté que dans Chromium sans interface) | 🟢 | API propre |
| **Serveur agent / MCP** au-dessus des opérations de patch : inspecter, éditer (atomique, annulable), linter, sécurité, rendu, enregistrement dans son propre fichier uniquement (R-094) | ✅ | 🟢 | protocole JSON-RPC ouvert |
| Interface complète : gizmos, timeline, éditeur de courbes (R-100…R-103) | 🟡 (R-101, R-102 terminés ; les pages R-100 et R-103 existent mais n'ont jamais été exécutées dans un vrai navigateur par l'auteur) | 🟢 | widgets génériques ; apparence propre |
| Contrôle de version d'animation avec fusion sémantique, diff perceptuel en CI, explain-this-pixel (I-01, I-06, I-07) | ✅ (I-01 `osatura vcs`, I-06 `osatura ci`, I-07 `explain`) | 🟢 | idées propres |
| Fuzzing de poses avec cartes de chaleur de contrainte, export à taille cible, niveau de détail déterministe avec borne d'erreur déclarée (I-03, I-10, I-11) | 🟡 (les trois sont construits ; la borne de I-11 ne couvre pas les effets, les chemins skinnés, les valeurs pilotées par IK ni les poses de machine à états) | 🟢 | idées propres, mathématiques de manuel |

## Niveau 6 — Frontière (niveau recherche)
| Fonctionnalité | Statut | PI | Base sûre / condition |
|---|---|---|---|
| Moteur de rendu GPU avec tests de parité CPU (R-110) | 🟡 (crate `osatura-gpu` construite et testée ; exécution matérielle limitée, voir ROADMAP) | 🟢 | wgpu (permissive) |
| Édition multi-utilisateur en temps réel au-dessus du journal d'opérations (R-111) | 🟡 (réalisée avec un journal de session persistant optionnel `--collab-log` ; les participants et l'historique d'annulation ne sont pas persistés ; HTTP simple, jeton partagé) | 🟢 | littérature CRDT |
| Runtime de liaison de données / mise en page (R-091, R-095) | ✅ (conception propre ; examen des brevets non effectué, docs/PATENT_NOTES.md) | 🟠 | examen des brevets avant diffusion |
| Outils assistés par ML (R-112) | 🟡 | 🟠 | méthodes classiques uniquement, aucun poids de modèle : `autotrace` (k-means, ajustement de Bézier de Schneider, mode ligne Zhang-Suen) et suggestions apprises à partir de vos poses (PCA, noyaux, petit MLP optionnel dans un format de poids propre) ; pas d'apprentissage profond |

## À ne jamais construire (🔴)
- Lire ou écrire des fichiers de projet natifs **Live2D, Spine, Moho, Harmony, Rive, Cartoon Animator** (propriétaires ; exposition au CLUF et à la rétro-ingénierie) — utiliser plutôt Lottie, OCA, ORA, APNG, SVG.
- Les **solveurs rapides de déformation d'illustrations de type ARAP** tels que décrits dans les brevets d'Adobe — le pin warp, la FFD et le skinning couvrent le cas d'usage.
- Copier la **disposition d'interface, les icônes, les noms, le contenu d'exemple ou les nombres magiques et la terminologie de format de fichier** d'un éditeur quelconque.
- Imiter l'outil de photosensibilité **Harding** ou revendiquer une conformité à la diffusion télévisée.
- Embarquer des encodeurs **H.264/H.265/AAC**.

## Conditions applicables à chaque ligne 🟢
Journal de provenance tenu (`CLEANROOM.md` ; implémentation indépendante, et non clean-room stricte) ; uniquement des dépendances permissives (à régénérer avec `scripts/licence-audit.sh` ; actuellement 363 crates tierces toutes plateformes confondues, toutes de classe MIT / Apache-2.0 / BSD / Zlib / Unlicense sauf six crates MPL-2.0 (smartstring via la fonctionnalité `scripting`, activée par défaut, et cinq via l'application à fenêtre `osatura-gui`) ; voir docs/THIRD_PARTY.md) ; aucune illustration ou audio tiers livré sans son propre fichier de licence ; le nom **OSATURA** validé par une recherche de marque avant le lancement.


# Tableaux détaillés des capacités
## Dessin et formes
| Fonctionnalité | Statut | Notes |
|---|---|---|
| Chemins de Bézier cubiques, ouverts/fermés, poignées | ✅ | |
| Remplissage : uni, dégradé linéaire et radial | ✅ | les dégradés suivent la transformation du calque |
| Contour : largeur, extrémités, jonctions, tirets | ✅ | |
| Contour à largeur variable (profil de largeur) | ✅ | contour polygonal ; chemins fermés → deux anneaux |
| Lissage Catmull-Rom, aides rectangle/cercle/ellipse/polygone | ✅ | |
| Booléens de polygones (union/intersection/différence/xor) | ✅ | calque `boolean` ; trous reliés en un seul polygone ; le résultat est polygonal (aplati) |
| Calques metaball / liquide | ✅ | marching squares, les os peuvent piloter les blobs |
| Calques d'image (PNG), intégrés ou en fichier | ✅ | |
| Pinceaux bitmap/vectoriels, remplissages texturés, texte/polices | ❌ | |
| Dessin à main levée, outils d'édition de points, vectorisation d'image | ❌ | pas d'interface de dessin |

## Rigging
| Fonctionnalité | Statut | Notes |
|---|---|---|
| Hiérarchie d'os, FK, pose de repos | ✅ | |
| Limites d'angle / d'échelle | ✅ | |
| IK : analytique à 2 os (flexion ±) | ✅ | |
| IK : chaînes de N os (FABRIK) | ✅ | |
| Mélange FK↔IK (animable) | ✅ | |
| Contraintes : aim, copy-rotation, pin (poids animable) | ✅ | |
| Skinning par mélange linéaire des ancres + poignées, auto-skin | ✅ | |
| Calques attachés à un os, parentage / groupes de calques | ✅ | |
| Mouvement secondaire : ressorts amortis précalculés | ✅ | déterministe, à accès aléatoire (précalculé au chargement) |
| Canaux pilotés par la pose (« driven keys ») | 🟡 | une courbe fait correspondre n'importe quelle propriété à une autre ; pas d'interface ni d'actions dédiées de type smart-bone |
| Physique de corps rigides, tissu, corps mous | ❌ | |

## Déformation
| Fonctionnalité | Statut | Notes |
|---|---|---|
| Décalages par point (animables) | ✅ | adressés par `layer::path::index` |
| Morph targets / blend shapes (poids additifs) | ✅ | |
| Lattice de déformation libre (FFD de Bernstein) | ✅ | points de contrôle animables |
| Déformateur de flexion de spine / courbe | ✅ | rayon doux optionnel |
| Ondulation « boil » (tremblement dessiné à la main) | ✅ | bruit cohérent, déterministe |
| Déformation de maillage d'image avec poids d'os | ✅ | maillage en grille ; pas d'éditeur de maillage |

## Animation
| Fonctionnalité | Statut | Notes |
|---|---|---|
| Clés + interpolation (linear/step/ease/cubic-bezier) | ✅ | |
| Extrapolation : hold/linear/loop/ping-pong/loop-offset | ✅ | |
| Canaux d'expression | ✅ | petit langage d'expressions sûr ; peut lire d'autres canaux |
| Canaux de bruit, mélange add/multiply/replace, poids | ✅ | |
| Calques de commutation + exposition (image par image, échanges de bouche) | ✅ | |
| Retime/décalage de clip, retarget, miroir G↔D, bake de fondu enchaîné | ✅ | fonctions + CLI `mirror` ; pas d'interface NLA |
| Cycle de marche procédural | ✅ | générateur sinusoïdal simple, pas un outil de marche professionnel |
| Lip-sync audio à partir de WAV | 🟡 | heuristique volume/brillance → 5 formes de bouche, **pas** de la reconnaissance de phonèmes |
| Émetteur de particules déterministe | ✅ | forme close : n'importe quelle image est évaluée directement |
| Onion skin, trajectoires de mouvement | ✅ | CLI `onion` et `trail`, superpositions du studio ; jamais exportées |
| Lecture audio / défilement de forme d'onde | 🟡 | commande `waveform` et lecture audio du studio ; aucune prétention d'interface de défilement de forme d'onde |

## Caméra et compositing
| Fonctionnalité | Statut | Notes |
|---|---|---|
| Caméra 2D panoramique/zoom/rotation (animable) | ✅ | |
| Parallaxe multiplan par calque de premier niveau | ✅ | |
| Opacité, modes de fusion, masques alpha (+inversion) | ✅ | |
| Vrais calques / caméra 3D, stéréo | ❌ | |

## Sortie et interopérabilité
| Fonctionnalité | Statut | Notes |
|---|---|---|
| Images SVG | ✅ | |
| Images PNG (rastérisation tiny-skia) | ✅ | |
| GIF animé | ✅ | |
| MP4 | ✅ | nécessite `ffmpeg` dans le PATH |
| Lecteur HTML autonome | ✅ | toutes les images en ligne, curseur de défilement |
| Lottie JSON | 🟡 | géométrie précalculée par image ; dégradés → première couleur ; pas de masques/modes de fusion/images ; **non validé dans un vrai lecteur Lottie** |
| Import de fichiers d'autres outils, PSD, FBX, etc. | ❌ | (volontairement : pas de formats propriétaires) |
| API de script / de plugin | 🟡 | `script` Rhai en bac à sable via des requêtes validées, plus les expressions ; pas d'API de plugin |

## Édition
| Fonctionnalité | Statut | Notes |
|---|---|---|
| Modèle d'édition par JSON-patch avec annuler/rétablir | ✅ | chaque édition est validée ; les éditions invalides sont annulées |
| Aides pour définir/supprimer des clés | ✅ | |
| Studio web (navigateur) | 🟡 | aperçu, curseur de défilement, pose de clés d'angle par os, éditeur JSON, annuler/rétablir, enregistrement, exports. **Pas de dessin, pas d'édition de points/os sur le canevas, pas d'éditeur de courbes.** |

## Points qui sont de modestes « améliorations » par rapport à la base de référence des manuels
(sans prétendre qu'ils sont absents ailleurs)
- Chaque édition de projet est un JSON patch validé et inversible → l'annulation ne peut jamais corrompre un projet.
- Toute simulation (ressorts, particules) est déterministe à accès aléatoire : faire défiler ou rendre n'importe quelle image dans n'importe quel ordre donne des pixels identiques.
- Toute propriété — y compris les points de contrôle de déformateurs et les poids de morph — est adressable par un unique chemin `layer::sub::id`, de sorte que les expressions et les courbes pilotées peuvent relier n'importe quoi à n'importe quoi.
- La validation signale *tous* les problèmes en une seule passe avec les noms fautifs.
- Un seul moteur → SVG, raster, GIF, MP4, HTML, Lottie ; le même `evaluate(frame)` alimente tout.

## Tests
`cargo test` — 142 tests unitaires/d'intégration (mathématiques, pistes, expressions, solveurs de rig, déformateurs,
pixels de rendu, exporteurs, annuler/rétablir de l'éditeur, audio, particules, booléens, metaballs).

---

# Liste de contrôle par rapport au produit de référence, version par version

Référence : Moho (anciennement Anime Studio), 1999 → 14.5 « SHOWTIME » (12 oct. 2026). Les listes de fonctionnalités
proviennent des pages de publication de l'éditeur et de la presse spécialisée (voir Sources). Lacunes du dossier : Anime Studio 6–8,
12.5 et 14.1 n'avaient pas de liste publique détaillée que j'ai pu trouver ; ils ne sont donc couverts que par
l'historique général. Le statut est celui d'OSATURA : ✅ un équivalent existe · 🟡 partiel / approche différente · ❌ manquant.

## Premières versions (1999 – Anime Studio 8, historique général uniquement)
| Fonctionnalité | OSATURA |
|---|---|
| Particules | ✅ calque émetteur déterministe |
| Masques de calque | ✅ masques alpha + inversion |
| Onion skins | ✅ (CLI `onion`, superposition du studio) |
| Contraintes d'os | ✅ aim / copy-rotation / pin, limites d'angle et d'échelle |
| Scripts Lua | 🟡 (Rhai, pas Lua ; canaux d'expression aussi) |
| Lip-sync | 🟡 heuristique d'amplitude à partir de WAV |
| Physique | 🟡 ressorts + gravité des particules ; pas de corps rigides |
| Création / import 3D | ❌ |
| Import Photoshop | ❌ |
| Prise en charge Wacom | ❌ (pas d'interface de dessin) |
| Smart bones, IK | 🟡 canaux pilotés / ✅ 2 os + FABRIK |

## Anime Studio 9 (2013)
| Fonctionnalité | OSATURA |
|---|---|
| 64 bits, accélération GPU | 🟡 Rust 64 bits natif ; rastériseur CPU uniquement (pas de GPU) |
| Smart Bones (cadrans de contrôle de flexion d'articulation) | 🟡 les canaux pilotés font correspondre n'importe quelle valeur à n'importe quelle autre ; pas d'interface/actions à cadrans |
| Graphes de mouvement éditables | ❌ (les courbes sont des données avec interpolation cubic-bezier ; pas d'éditeur de courbes) |
| Timeline remaniée | ❌ (le studio n'a qu'un curseur de défilement + des repères de clés) |
| Multi-touch Wacom | ❌ |

## Anime Studio 10 (2014)
| Fonctionnalité | OSATURA |
|---|---|
| Contraintes d'angle indépendantes (globales) | 🟡 une copy-rotation vers un os de contrôle en approche l'effet |
| Squash & stretch | 🟡 l'échelle d'un os est animable ; pas de squash/stretch automatique |
| Cadrans / configuration de Smart Bone | 🟡 voir ci-dessus |
| Interpolation bounce / elastic / stagger | 🟡 bounce, spring (de type élastique), back, cubic-bezier ✅ ; stagger ❌ |
| Pot de peinture, pinceau blob, gomme, réduction de points | ❌ |
| Personnages préchargés / contenu de bibliothèque | ❌ |
| Import d'objets 3D | ❌ |

## Anime Studio 11 (2015)
| Fonctionnalité | OSATURA |
|---|---|
| Animation image par image | ✅ calques de commutation + canaux d'exposition |
| Référencement de calques (doublons liés) | ❌ |
| Ordonnancement animé des formes | ❌ (l'ordre de dessin est fixe par calque ; les calques de commutation peuvent l'échanger) |
| Courbes de mouvement éditables | 🟡 données de courbes oui, pas d'interface de courbes |
| Import PSD (y compris mise à jour en direct) | ❌ |
| Export FBX / Unity | ❌ |
| Import/enregistrement audio + lip-sync automatique | 🟡 import WAV + lip-sync heuristique ; pas d'enregistrement |
| Fusion de contours, outil Color Points | ❌ |

## Moho 12 (2016) et 12.x
| Fonctionnalité | OSATURA |
|---|---|
| Poignées de Bézier | ✅ |
| Outils à main levée améliorés | ❌ |
| Smart Warp (maillages personnalisés) | 🟡 maillage en grille d'image avec poids d'os ; déformateurs lattice / spine pour les vecteurs ; pas d'éditeur de maillage |
| Flou de mouvement | ❌ |
| Pin bones | 🟡 contrainte pin, pas de pin bones à point unique |
| Import/export d'actions entre rigs similaires | 🟡 fonctions `clip`, `retarget`, `mirror` sur des ensembles de canaux ; pas d'interface de bibliothèque d'actions |
| Animer plusieurs calques à la fois | ✅ un nombre quelconque de canaux par timeline |
| Interface repensée, ancrage d'os, auto-freeze de position | ❌ |

## Moho 13 (2019)
| Fonctionnalité | OSATURA |
|---|---|
| Image par image bitmap + onion skin | 🟡 calques d'image PNG dans des calques de commutation ; pas d'onion skin, pas de dessin bitmap |
| Outils de dessin bitmap / pinceaux personnalisés | ❌ |
| Outils de storyboard et d'animatique | ❌ |
| Fenêtre d'actions | ❌ (clips au niveau des données uniquement) |
| Lumières 3D, matériaux, toon shading | ❌ |
| Davantage de préréglages de sortie | 🟡 formats d'export en CLI, pas de gestionnaire de préréglages |
| Personnages fournis | ❌ |

## Moho 13.5 (2021) et 13.5.x
| Fonctionnalité | OSATURA |
|---|---|
| Vitruvian Bones (échange des dessins *et* des ensembles d'os) | 🟡 les calques de commutation échangent les dessins ; pas d'échange d'ensembles d'os |
| Quad meshes (perspective) | 🟡 maillage en grille + lattice donnent des déformations libres ; pas de quad projectif |
| Dynamique du vent (force, direction, turbulence) | ❌ (les ressorts n'ont pas de vent ; les canaux de bruit peuvent le simuler) |
| Pose de clés relative | ❌ |
| Variantes de visèmes de lip-sync choisies au hasard | ❌ (le mappage peut fusionner des formes, pas de variantes aléatoires) |
| Export audio en WAV | ❌ |
| Copier/coller de clés profondes, ajustement du pivot, marqueurs de timeline | ❌ (pas d'interface) |
| Licences flottantes | n/a |

## Moho 14 (2023)
| Fonctionnalité | OSATURA |
|---|---|
| Nouveau moteur graphique, effets en temps réel (flou, ombre, bruit) | ❌ pas de filtres flou/ombre/bruit ; modes de fusion ✅ |
| Liquid Shapes (fusion/soustraction/intersection/mélange) | 🟡 liquide metaball ✅ + booléens de polygones ✅ ; pas le même modèle de forme |
| Curvers / Compressible Curvers | 🟡 le déformateur de flexion de spine est l'analogue le plus proche |
| Import SVG amélioré | ❌ (**export** SVG uniquement) |
| Améliorations du main levée, nouveaux styles, fusion de formes | ❌ / ❌ / 🟡 (booléens) |
| Multi-Stroke Fill, Connect & Create | ❌ |
| Smart Line Boil, options de pinceau | 🟡 ondulation « boil » à cadence réglable ✅ ; options de pinceau ❌ |
| Couleurs de points, Add Point amélioré | ❌ |
| Dynamic Bones (+ gravité) | 🟡 os à ressort amorti ✅ ; pas de terme de gravité |
| Contraintes retardées | 🟡 les ressorts donnent un retard sur une cible ; pas de contrainte de délai explicite |
| Nouvelles options de timeline | ❌ |
| Améliorations de l'image par image | 🟡 |
| Mise à jour de l'interface de script (Lua 5.4) | ❌ |

## Moho 14.1 – 14.2 (2023-11, 2024-06)
| Fonctionnalité | OSATURA |
|---|---|
| (14.1 : aucun détail public trouvé) | — |
| Décalage animé du profil de courbe | ❌ (les profils de largeur sont statiques) |
| Smart boil pour le bruit vectoriel | 🟡 l'ondulation évolue à une cadence fixée ; pas de mode « uniquement en mouvement » |
| Outil Transform Bone remanié | ❌ (pas d'outils de canevas) |
| Affichage en pixellisation | ❌ |
| Mode de fusion Colour Burn | ✅ |
| Espacement minimal de pinceau réduit, calques écrêtés PSD | ❌ |
| Numérotation de timeline basée sur la cadence d'images | ❌ |
| Option d'affichage « désactiver les calques masqués » | ❌ |
| Meilleur lip-sync à basse cadence d'images | 🟡 |
| Noms multilingues pour os/calques/marqueurs | ✅ (noms en UTF-8) |
| Anti-crénelage du masque Exclude-Strokes, onion skins colorés sur les images | ❌ |
| Smart Bones contrôlant la dynamique des os | ❌ (les réglages de ressort ne sont pas animables) |
| Scripts Lua au démarrage ; option de console Windows | ❌ / n/a |
| Export ProRes alpha | ❌ (GIF/PNG/MP4 uniquement) |

## Moho 14.3 (2024-12)
| Fonctionnalité | OSATURA |
|---|---|
| Opacité de pinceau par point (pilotée par la pression) | ❌ |
| Dérive de couleur selon la pression du stylet | ❌ |
| Contrôles d'outil main levée / largeur de ligne | ❌ |

## Moho 14.4 « SIDE QUEST » (2025-11)
| Fonctionnalité | OSATURA |
|---|---|
| Export vers Unreal / Unity / Godot | ❌ |
| Export glTF vers Blender / outils 3D | 🟡 (`.glb` : formes de l'image 0, os, poids de morph) |
| Smart Bones à 360° | ❌ |
| Create Style from Shape | ❌ |
| Options de particules (réplication le long d'une courbe, délai, émission ordonnée, distribution) | 🟡 délai de départ ✅, émission ordonnée déterministe ✅ ; réplication le long d'une courbe ❌ |
| Layer Selector, décroissance de Magnet, import PSD, lissage à main levée, ajouts Lua | ❌ |
| Création de cycles plus facile | ✅ extrapolation en boucle + générateur de cycle de marche |

## Moho 14.5 « SHOWTIME » (2026-10-12)
| Fonctionnalité | OSATURA |
|---|---|
| Live Playback (webcam / manette / micro, webcam virtuelle, OBS) | ❌ |
| Suivi du visage → os / Smart Bones / maillage | 🟡 (détecteur entraîné sur données synthétiques, `face-bake` ; non validé sur de vrais visages) |
| Enregistrement en direct dans la timeline, contrôle de plusieurs personnages | ❌ |
| Suivi vidéo (séquences d'images) | ❌ |
| Algorithmes de main levée, lignes droites, suppression des queues | ❌ |
| Pot de peinture refait (Vector / Trace / Recolor) | ❌ |
| Déplacement inverse de Magnet, sélection de points par forme (Select Points) | ❌ |
| Catégories de pinceaux personnalisés | ❌ |
| Couleur/opacité de l'onion skin, Shift & Trace, onion skins magnétiques | ❌ |
| Filtres / multi-sélection de la fenêtre des calques | ❌ |
| Performances : annuler/rétablir plus rapide, fichiers plus légers | 🟡 l'annulation est un inverse de JSON-patch peu coûteux ; les fichiers sont du JSON brut |
| Export MP4/H.264 | ✅ via ffmpeg |
| Profils colorimétriques PNG, import/export EXR | ❌ |
| Exporteur par lot, options CLI `-exr` / `-pngprofile` | 🟡 CLI `frames` / `export`, pas de file d'attente par lot |
| Cadence d'images avec 3 décimales (par ex. 23.976) | ✅ |
| L'export SVG gère les contours à largeur variable et les formes liquides | ✅ (les contours deviennent des silhouettes, les liquides deviennent des chemins) |
| Liquid hide-edge, formes liquides en contour seul, exposition de contour | ❌ |
| Linux via Wine/Proton | ✅ Rust natif (Linux/macOS/Windows) ; archives précompilées Windows x86-64, macOS arm64 et Linux x86-64 dans la préversion en brouillon ; testé par la CI sur les trois, par une personne seulement sous Linux |

## Bilan (produit de référence uniquement ; les concurrents suivent plus bas)
Parmi les fonctionnalités documentées du produit de référence, OSATURA couvre le noyau **rigging, déformation,
animation procédurale, mathématiques de compositing et export** (en gros : squelette/IK/contraintes,
déformation par maillage et lattice, clés et expressions, masques/mélanges/dégradés, particules,
metaballs/booléens, GIF/MP4/HTML/Lottie), mais n'a **ni outils de dessin, ni entrée en direct, ni 3D,
ni E/S PSD/FBX** (au moment de la première étude ; l'export glTF, la sortie EXR, les scripts Rhai, un studio avec courbes/timeline, l'onion skin et les effets de flou ont été ajoutés depuis, voir `ROADMAP.md` section 9).
Ce sont les blocs importants, pour la plupart liés à l'interface, du produit, et ils constituent l'écart réel.

## Sources
- Annonce Moho 14.5 SHOWTIME, moho.lostmarble.com (blog, 12 oct. 2026)
- Page des fonctionnalités de Moho Pro, mohoanimation.com/pages/features
- Appcast de Moho 14.4 SIDE QUEST, lostmarble.com/appcasts/Moho14.4Update.html
- Moho 14.3 : Digital Production, 27 déc. 2024
- Article de publication de Moho 14.2, moho.lostmarble.com ; liste de tutoriels « Moho 14: What is new? »
- Nouveautés de Moho Pro 13.5, moho.lostmarble.com ; Lester Banks (avr. 2021) ; notes de la 13.5.3
- Moho 13 : Lester Banks (juin 2019)
- Moho Pro 12 : 80.lv ; Digital Production (nov. 2016)
- Anime Studio 11 : Architosh (mai 2015) ; 10 : Architosh (mars 2014) ; 9 : Renderosity
- Dates des versions : mohoscripting.com/moho_versions ; historique général : Wikipédia « Moho (software) »

---

# Addendum : historique de Moho / Anime Studio, versions 1 – 11 (complété)

Source : le PDF d'historique des versions hébergé par l'éditeur (lostmarble.com, « Anime Studio history », 2015) ainsi que les
pages de presse d'Anime Studio 7 et 8. Il remplace les lignes « historique général uniquement » situées vers le début de la
liste de contrôle de référence. Introuvables même ici : les détails de 8.x, 10.1.x, 11.1, 12.5, 14.1.

| Version | Fonctionnalités documentées | OSATURA |
|---|---|---|
| Moho 1 (1999) | BeOS, portage Windows, sortie Flash | ❌ Flash/SWF (obsolète) |
| 2.x | E/S QuickTime, rendus jusqu'à 4096², ombrage de calque, calques de commutation, nuanciers de couleurs dans les styles, liaison souple des os, import Illustrator amélioré | ✅ calques de commutation, 🟡 liaison par poids, ❌ import Illustrator, ✅ grandes échelles de raster (≤16×) |
| 3.x | Particules, enroulement d'image, masques de calque, onion skins, caméra virtuelle avec profondeur, déformation d'image, calques de commutation améliorés, prise en charge des tablettes, timecode SMPTE, masquage des illustrations hors champ, masques réversibles, randomiseur d'épaisseur de ligne, zones de sécurité, cisaillement de calque | ✅ particules, masques (+inversion), caméra, déformation par maillage d'image, inclinaison de calque ; ❌ onion skin, tablette, timecode, zones de sécurité, randomiseur d'épaisseur |
| 4.x | Courbes d'animation graphiques, contraintes d'os, rendu par lot, audio, paramètres animables, animation en cycle, mise à l'échelle des clés, projets redimensionnables, profondeur des particules, foules aléatoires, liaison souple automatique, version Linux | ✅ contraintes, cycles (extrapolation en boucle), paramètres animables, projets redimensionnables, `clip` retime ≈ mise à l'échelle des clés, Linux ; 🟡 foules (particules), 🟡 audio (lip-sync uniquement) ; ❌ interface d'éditeur de courbes |
| 5.x | Nouvelle interface, export SWF, calques 3D et de notes, actions, styles, scripts Lua | ❌ calques 3D / de notes / Lua ; 🟡 actions (fonctions de clip) ; ✅ les styles sont une peinture par chemin |
| Anime Studio 6 | Suivi de mouvement, lip-sync intégré, pinceau de dispersion, Lua 5.1, ombrage automatique, **morph targets**, séquenceur, vidéo HD | ✅ morph targets ; 🟡 lip-sync ; ❌ suivi de mouvement, pinceau de dispersion, ombrage automatique, séquenceur |
| 6.1 / 6.2 | Envoi vers Facebook, import de GIF transparent | ❌ / 🟡 export GIF ✅, import ❌ |
| Anime Studio 7 | Physique simulée (gravité, densité, frottement, élasticité), conception de formes 3D, enregistrement audio, bibliothèque de contenu, import de scènes Poser, **outil de suivi de chemin**, rassemblement des médias | 🟡 ressorts/gravité des particules ; ❌ physique rigide, 3D, enregistrement, bibliothèque, Poser, suivi de chemin |
| Anime Studio 8 | Vectorisation automatique d'image, Character Wizard, mode débutant, connexion aux médias en temps réel, styles de rendu globaux, rigging d'os avancé | ❌ tout (pas de vectorisation, d'assistant, de liaison en direct, de styles globaux) |
| Anime Studio 9 / 9.5 | Graphes de mouvement éditables, Smart Bones, **compositions de calques**, onion skins améliorés, 64 bits + GPU, pose de clés améliorée, timeline colorée ; 9.5 : améliorations des Smart Bones, décalage de profondeur, contrôles de calques imbriqués, flexi-binding, améliorations des calques de commutation | voir le tableau AS 9 ; ❌ compositions de calques, décalage de profondeur |
| Anime Studio 10 / 10.1 | Nouvelles contraintes, configuration des Smart Bones, bounce/elastic/stagger, processus de rendu séparé, outils combinés os+points, aperçu d'animation, multi-document, éditeur de raccourcis, sélection multi-formes, image zéro, masquage de points/os, édition multi-calques, **courbes à largeur variable**, transparence des textures, source de particules réglable, profondeur de champ, flou fractionnaire ; 10.1 : vectorisation automatique d'image, styles globaux, calques de correction, import PSD, médias en temps réel, Character Wizard | ✅ contours à largeur variable, ✅ animation multi-calques ; 🟡 profondeur de champ ❌ flou ❌ ; ❌ calques de correction, autres comme ci-dessus |
| Anime Studio 11 | Image par image, référencement de calques, ordonnancement animé des formes, **cibles d'os animées**, **changement de parent d'os animé**, retournement d'os, regroupement de calques de sélection, échelle de calque normalisée, export par lot, nouveau format de fichier | ✅ image par image, ✅ cible d'IK animée (la cible est simplement un os), ❌ *changement de parent* animé, 🟡 retournement d'os (`mirror`), ✅ lot via CLI, ✅ format de fichier en JSON brut |

**Nouvelles lacunes révélées ici** (à ajouter à la feuille de route) : compositions de calques (états de visibilité de calques enregistrés),
changement de parent d'os animé, calques de correction, décalage de profondeur, calques de suivi de chemin, superpositions de zones de sécurité,
timecode SMPTE, ombrage automatique, pinceau de dispersion, vectorisation automatique d'image.

---

# Fonctionnalités de la concurrence non couvertes par la liste de contrôle ci-dessus

Produits étudiés : Toon Boom Harmony 24, Spine 4.2, Live2D Cubism 5.x, Cartoon Animator 5.3, Rive,
Synfig Studio, OpenToonz, TVPaint Animation 12, Adobe Animate, Cavalry, Procreate Dreams, Duik (After
Effects), Blender Grease Pencil. Les éléments ci-dessous sont ceux qui **ne figurent pas déjà** dans la liste de
contrôle de référence. La qualité des sources varie : la plupart des lignes proviennent de pages de fonctionnalités/de publication d'éditeurs que j'ai récupérées ;
les lignes marquées † proviennent de connaissances générales ou d'un simple titre / sommaire et n'ont pas été confirmées sur une page.
Le statut est celui d'OSATURA : ✅ · 🟡 · ❌.

## Rigging, contraintes, déformation
| Fonctionnalité | Vu dans | OSATURA |
|---|---|---|
| Outils de poids : souder les poids entre maillages, copier/coller les poids entre sommets | Spine 4.2 | ❌ (auto-skin uniquement, pas de peinture) |
| Contraintes de physique d'os (inertie, vent, gravité, mélange) | Spine 4.2 | 🟡 ressorts amortis (fréquence/amortissement) ; pas de vent/gravité/inertie |
| Contrainte de chemin : les os suivent une spline de Bézier composite | Spine, Rive (Follow Path) | ❌ |
| Mélanges de contraintes, duplication avec les os, édition multiple, dossiers | Spine 4.2 | 🟡 mélange IK et poids de contrainte animables ; pas de dossiers/édition multiple |
| Skins (échange d'ensembles d'attachements sur un même squelette) | Spine, solos de Rive | 🟡 calques de commutation |
| « Outil de pose » IK multi-os (glisser l'extrémité d'un membre) | Spine | ❌ (nécessite une interface de canevas) |
| Attachements de boîte englobante / de test de collision | Spine | ❌ (inutile pour une sortie hors ligne) |
| Déformation libre (Free-Form Deformation), poids de maillage | Spine, Harmony | ✅ lattice + maillage d'image pondéré |
| Chaîne de déformateurs : Bone, Game Bone, Curve, Envelope, Free-Form, Mesh Warp | Harmony | 🟡 bone ✅ curve (spine) ✅ free-form ✅ mesh warp ✅ ; envelope/cage ❌ ; game-bone ❌ |
| Shape-Aware Deformation (points, os, cages) | Harmony 24 | ❌ |
| Bake de tous les types de déformation | Harmony 24 | 🟡 ressorts précalculés ; géométrie précalculée par image dans l'export Lottie |
| Contrôleur maître / sélecteur de transformation pour les groupes de déformation | Harmony 24 | ❌ |
| Pegs et hiérarchie de calques | Harmony, Animate | ✅ groupes, calques attachés à un os, parentage |
| Auto-muscle, auto foot-roll, sortie cinématique † | Harmony | ❌ |
| Déformateurs de rotation / warp + paramètres de keyform, glue | Live2D | 🟡 paramètres ≈ canaux pilotés, warp ≈ lattice ; glue ❌ |
| Blend shapes pilotant rotation, opacité, ordre de dessin, couleur multiply/screen | Live2D 5.0 | 🟡 les poids de morph ne pilotent que la géométrie |
| Auto-mesh avec contrôle de densité uniforme pour les formes fines | Live2D 5.0 | ❌ (grille régulière uniquement) |
| Le déformateur warp se redimensionne automatiquement pour contenir le keyform enfant | Live2D 5.3 | ❌ |
| Pinceau de déformation (formes carrée/ligne/expansion) | Live2D 5.3 | ❌ |
| Déformateur de visage assisté par IA / génération de mouvements faciaux | Live2D 5.0 | ❌ |
| Joysticks (contrôle 2D associé à des animations) | Rive | 🟡 un canal piloté fait correspondre une entrée ; pas de joystick à 2 entrées |
| Contraintes entre objets quelconques (translation/rotation/échelle/distance) | Rive | 🟡 aim, copy-rotation, pin uniquement |
| Rigging bitmap avec intervalles automatiques | TVPaint 12 | 🟡 maillage d'image ✅ ; intervalles automatiques ❌ |
| Outil Plastic (rigging par maillage) | OpenToonz | 🟡 maillage d'image + poids |
| Auto-rigging, contrôleurs, dynamique de ressorts | Duik † | 🟡 ressorts ✅ ; auto-rig ❌ |

## Animation, timeline, interactivité
| Fonctionnalité | Vu dans | OSATURA |
|---|---|---|
| Mixage d'animations : mélange/fondu enchaîné d'animations simultanées à l'exécution | Spine | 🟡 modes Add/Multiply + poids ; `crossfade` précalculé ; pas de transitions à l'exécution |
| Dopesheet + éditeur de courbes + aperçu runtime | Spine, Rive, OpenToonz | ❌ (les données le permettent ; pas d'interface) |
| Machine à états : entrées, écouteurs, transitions conditionnelles, états de mélange, calques | Rive | ❌ |
| Événements, déclencheurs audio, ressources dynamiques | Rive | ❌ |
| Liaison de données (éléments de conception ↔ données en direct) | Rive | ❌ (les expressions peuvent lire des canaux, pas des données externes) |
| Scripts intégrés (Luau, JavaScript, ToonzScript) | Rive, Cavalry, OpenToonz | ❌ |
| Mises en page réactives / composants / bibliothèques | Rive | ❌ |
| Trim Paths, adoucissement vectoriel, N-slicing | Rive | ❌ |
| Synchronisation indépendante de la résolution, interpolation vectorielle | Synfig | ✅ |
| Liaison de paramètres entre calques via des expressions | Synfig | ✅ |
| Graphe de nœuds procédural (générateurs, transformateurs, effecteurs), duplicateurs | Cavalry | ❌ (pas de graphe de nœuds ; pas d'instanciation de calques) |
| Animation pilotée par des données JSON/CSV | Cavalry | ❌ |
| Performing : enregistrer des gestes en direct sous forme de clés | Procreate Dreams | ❌ |
| Raccourcis clavier de déclenchement sur la scène de marionnette, motion pilot (marches pilotées par le curseur) | Cartoon Animator 5.3 | ❌ |
| Sélecteur de caméra : caméras enregistrées, coupes au raccourci, piste de changement de caméra | Cartoon Animator 5.3 | 🟡 une seule caméra ; les coupes peuvent être simulées avec des clés step |
| Mouvements cinématographiques (travelling compensé, suivi d'une cible) | Cartoon Animator 5.3 | 🟡 caméra animable ; pas de suivi de cible |
| Mocap faciale (webcam / iPhone), corps piloté par la tête, volume du micro | Cartoon Animator, Live2D | ❌ (volume du micro uniquement via WAV hors ligne) |
| Mode livestream (OBS/Twitch, mappage Stream Deck) | Cartoon Animator 5.3 | ❌ |
| Lip-sync Motion Sync générant des clés de timeline | Live2D 5.0 | 🟡 WAV heuristique → clés d'exposition |
| Intégration du lip-sync Rhubarb | OpenToonz | ❌ (heuristique uniquement) |
| Extraction du mouvement de racine (compensation du delta) | Spine 4.2 | 🟡 extrapolation loop-offset |
| Synchronisation des poignées de courbe en boucle | Spine 4.2 | ✅ extrapolation en boucle (pas d'interface de poignées) |
| Morphing avec indications de morphing | Harmony | 🟡 morph targets + interpolation de chemin ; pas d'indications |
| Caméra 2D à clés avec minutage sur la timeline | TVPaint 12 | ✅ |

## Dessin, peinture, outils bitmap
| Fonctionnalité | Vu dans | OSATURA |
|---|---|---|
| Pinceau : randomisation (nombre de tampons, décalage), papiers doubles, pinceaux multitextures | Harmony 24, TVPaint 12 | ❌ |
| Opacité du crayon selon pression/inclinaison/vitesse/effilement/fondu | Harmony 24 | ❌ |
| Guides de symétrie (vertical, horizontal, double) | Harmony 24 | ❌ |
| Outil de découpe (lasso, rognage), éditeur de contour, fermeture de trous | Harmony | ❌ |
| Guides de dessin, guides de perspective/règle † | Harmony | ❌ |
| Animation sans papier : onion skin, feuilletage, table lumineuse | Harmony, Procreate Dreams, TVPaint | 🟡 (onion skin oui ; pas d'outils de dessin ni de table lumineuse) |
| Nettoyage de dessins numérisés, numérisation, capture par caméra, stop-motion | OpenToonz | ❌ |
| Pinceau de sélection, bibliothèque d'images | TVPaint 12 | ❌ |
| Netteté lors de la transformation | TVPaint 12 | n/a |
| Sensibilité à la pression du stylet | Synfig | ❌ |
| Ombrage en dégradé basé sur un chemin | Synfig | ❌ |
| Texte à contenu variable/dynamique à l'exécution, dans n'importe quelle langue | Rive, Procreate Dreams | ❌ |
| Polices personnalisées, titres, génériques | Procreate Dreams | ❌ |
| Gestion des palettes et des styles | OpenToonz | ❌ |
| Plus de 50 types de calques (fractales, filtres, distorsions) | Synfig | ❌ (≈7 types de calques) |

## Effets et compositing
| Fonctionnalité | Vu dans | OSATURA |
|---|---|---|
| Réseau de compositing en vue de nœuds avec ports nommés | Harmony 24 | ❌ (arbre de calques + masques/mélanges) |
| Mattes, groupes sous composite | Harmony | ✅ masques, groupes isolés |
| Lecture/écriture CryptoMatte | Harmony 24 | ❌ |
| Toon shader (contour, postérisation) | Harmony 24 | ❌ |
| Masque flou (unsharp mask), échange de canaux | Harmony 24 | ❌ |
| Famille de flous (radial, rotatif, directionnel), bokeh, bruit fractal, tuilage | OpenToonz | ❌ |
| FX de particules | OpenToonz | ✅ calque émetteur |
| 15 modes de fusion RGB + 5 alpha, dessin hors écran par partie | Live2D 5.3 | 🟡 13 modes de fusion ; rendu de groupes isolés ✅ |
| Écrêtage (identifiants de clip par partie) | Live2D, Rive | ✅ masques alpha |
| Imagerie HDR / HDRI | Synfig | ❌ |
| Import vidéo, ProRes, rotoscopie sur séquence filmée | Procreate Dreams | ❌ |
| Moteur audio (voix off, musique, effets sonores) | Procreate Dreams | ❌ |

## 3D, pipeline, interopérabilité, runtimes
| Fonctionnalité | Vu dans | OSATURA |
|---|---|---|
| Scène 3D, pose d'os FBX, graphe de scène Vulkan, dessin sur plan en perspective | Harmony 24 | ❌ |
| Rendu externe via Maya / Blender | Harmony 24 | ❌ |
| Import vectoriel : PDF / AI / SVG ; EXR multi-calques ; alpha DPX ; OpenColorIO (ACES) | Harmony 24 | ❌ |
| Import PSD sans Photoshop | Spine 4.2, Rive | ❌ |
| Import par collage Lottie / Figma | Rive | ❌ (**export** Lottie uniquement) |
| Texture packer / atlas | Spine | ❌ |
| Export de squelette JSON/binaire + 19 runtimes officiels / plus de 40 communautaires | Spine | 🟡 JSON propre ; pas de runtimes |
| Runtimes open source (Web, Apple, Android, React, Flutter, C++, Unity, Unreal) | Rive | ❌ |
| Mode Pixels (aligner les sprites sur les pixels de l'écran) | Spine 4.2 | ❌ |
| Export : WebM avec transparence | Live2D 5.3 | ❌ |
| Export : SVG/PDF, séquences PNG/EXR, profondeur 16/32 bits, lot, ProRes | Cavalry | 🟡 SVG, PNG ✅ ; PDF/EXR/ProRes/grande profondeur de bits ❌ |
| Rendu dans le cloud, liens de partage, intégrations | Rive | ❌ |
| Rendu distribué (Toonz Farm), contrôle de version | OpenToonz | ❌ |
| Export PDF / JSON TVPaint | OpenToonz | ❌ |
| Collaboration d'équipe en temps réel, remix communautaire | Rive | ❌ |
| Export HTML5 Canvas / web interactif | Animate † | 🟡 lecteur HTML (pré-rendu, non interactif) |
| Linux/macOS/Windows natifs | Synfig, OpenToonz | ✅ |
| Licence open source | Synfig, OpenToonz | ✅ MIT/Apache-2.0 |
| Vue du parentage des calques, symboles/instances de bibliothèque † | Animate | 🟡 parentage ✅ ; pas de bibliothèque d'instances |
| Interface modulaire détachable, espaces de travail personnalisés, thème sombre/clair | TVPaint 12, Live2D 5.0 | ❌ |

## Candidats pour la feuille de route (classés par valeur ÷ effort, sans interface d'abord)
1. Trim paths + suivi de chemin (os/calques le long d'une Bézier) — Rive, Spine
2. Onion skin + superpositions de trajectoires de mouvement — presque tous les outils 2D
3. Import SVG (chemins, dégradés) — Moho 14, Harmony, Synfig
4. Termes de vent / gravité pour les ressorts, dynamique d'os pilotée par des entrées de type Smart Bone
5. Flou, ombre portée, lueur, contour, postérisation (filtres de calque)
6. Instanciation de calques / duplicateurs (cloner N le long d'un chemin, avec décalages par copie)
7. Machine à états interactive dans le runtime HTML/JSON (entrées, transitions, mélange)
8. Déformateur à enveloppe (cage), outils de copie/soudure/lissage de poids, auto-mesh à densité uniforme
9. Coupes de caméra / caméras enregistrées, caméra à suivi de cible
10. Export glTF du rig ; export EXR/PDF/WebM-alpha
11. Calques de texte (nécessite le chargement de polices) avec suivi de chemin
12. Interface du studio : édition de points/os sur le canevas, éditeur de courbes, onion skin

## Sources supplémentaires (concurrents)
- Notes de version de Toon Boom Harmony 24 et Premium Getting Started Guide (docs.toonboom.com)
- Spine 4.2 « The physics revolution » et « What is Spine? » (esotericsoftware.com)
- Live2D Cubism 5.0 et 5.3 « New Features » (docs.live2d.com)
- Points forts de Cartoon Animator 5.3 (discussions.reallusion.com)
- Page des fonctionnalités de Rive (rive.app/features)
- Fonctionnalités de Synfig Studio (zoftwarehub.com) ; documentation « What's New » d'OpenToonz
- TVPaint Animation 12 (tvpaint.com) ; test de Cavalry (superrendersfarm.com)
- Adobe Animate « Layer parenting » (adobe.com/learn) ; Procreate Dreams (Cartoon Brew)
- Duik : page d'étiquette de Lester Banks (titre uniquement †) ; Blender Grease Pencil : page du manuel non récupérable (non couvert)

---

# Addendum — Historiques de versions de la concurrence (2e tour, 2026-10-19)

Légende comme ci-dessus : ✅ fait, 🟡 partiel, ❌ pas encore. Seules les fonctionnalités confirmées sur des pages d'éditeurs/de projets ou dans la presse sont listées. Aucun site pirate n'a été utilisé ; web.archive.org n'est pas accessible depuis cet environnement, de sorte que certaines versions plus anciennes restent non documentées (voir « Lacunes connues »).

## Toon Boom Harmony 20 (notes de version de l'éditeur)
| Fonctionnalité | OSATURA |
|---|---|
| Éditeur de ligne centrale : suppression de points, option de lissage | ❌ |
| Lignes de crayon texturées → traits de pinceau texturés ; pipette de texture de crayon | ❌ |
| Outil Ligne courbe / en S ; pinceau Auto-Fill ; pinceau Repaint | ❌ |
| Guides d'alignement, magnétisme limité aux calques sélectionnés | ❌ |
| Quick Ease / Ease In / Ease Out sur les clés sélectionnées | 🟡 interpolation par clé ✅ ; commandes d'adoucissement par lot ❌ |
| Marqueurs de scène (sans durée, ripple edit) | ❌ |
| Vue de substitution de dessins (renommer/filtrer) | 🟡 calques de commutation ✅ |
| Nœuds d'ombrage de surface : occlusion ambiante, ombre portée, surface map/normale | ❌ |
| **Nœud Weighted Deform** | 🟡 maillage d'image pondéré ✅ |
| Nœud Point Kinematic Output (échantillonnage à 1/2/3 points) | ❌ |
| Nœud de cache de pré-rendu | 🟡 cache de bake des ressorts ✅ |
| Nœuds de fondu de couleur / courbes / niveaux / gamma | ❌ |
| Forme de particules en région plane à partir d'un dessin vectoriel | 🟡 émetteur ✅ ; forme personnalisée ❌ |
| Contrôleurs maîtres : déformateur sur déformateur, Function Wizard, Slider Wizard | 🟡 canaux ✅ ; interfaces d'assistant ❌ |
| Scripts Python (PythonManager) | 🟡 opérations JSON patch + CLI ; pas de scripts intégrés |
| Gestion de l'espace colorimétrique, rendu 16 bits, post-traitement scripté de film | ❌ (post-traitement ffmpeg à la main) |
| Control Center : copie/import de scènes par lot/exécution de scripts, file d'attente de rendu | ❌ |

## OpenToonz 1.7
| Fonctionnalité | OSATURA |
|---|---|
| Mode ligne droite pour le pinceau | ❌ |
| Prise en charge des pinceaux GIMP + MyPaint | ❌ |
| Mode de remplissage Selection+Freehand | ❌ |
| Collage d'image depuis le presse-papiers | ❌ |
| Tableau de bord de pré-production / storyboard | ❌ |
| Panneaux personnalisés, fenêtre principale translucide, rotation/retournement du visualiseur | ❌ |
| Échelle de l'Xsheet + navigation par étiquettes | 🟡 données de timeline ✅ ; pas d'interface |
| Export avec informations de mouvement de caméra | 🟡 caméra dans Frame ✅ |
| **Export OCA (Open Cel Animation)** | ❌ |
| Export MOV et APNG | 🟡 MP4/GIF ✅ ; APNG/MOV ❌ |
| Conversion TZP→TLV par lot | ❌ |
| Floor Bump Iwa, Flow Fx ; rendu en virgule flottante + couleur linéaire | ❌ |
| Prise en charge de Haiku OS | n/a |

## Synfig 1.4.2 (et 1.4.0 du 1er tour)
| Fonctionnalité | OSATURA |
|---|---|
| Export Lottie incluant contour avancé et calques de flou, gamma | 🟡 export Lottie ✅ (formes/transformation) ; flou ❌ |
| Éditeur de raccourcis clavier, raccourcis d'outils à touche unique | ❌ |
| TimeLoop éditable dans l'espace de travail | 🟡 clips en boucle ✅ |
| Duplication des calques Skeleton / Skeleton Deformation avec liens d'os | ✅ (les os sont des données de projet) |
| Importeur SVG gérant les attributs de style | ❌ (pas d'import SVG) |

## Éléments du 1er tour (pages d'éditeurs, abrégés)
- **Spine 4.2/4.3** : contraintes de physique, curseurs, mélange de contraintes, alignement sur les pixels — ressorts ✅ (précalculés), curseurs/mélange 🟡 (IK `mix` ✅), mode Pixels ❌.
- **Live2D Cubism 4.0–5.3** : ArtPath, onion skin, export WebM-alpha — ArtPath ❌ ; onion skin et WebM-alpha ✅.
- **Harmony 21/22** : déformateurs pondérés, textures de line-boil — 🟡 / ✅ (ondulation).
- **Cartoon Animator 5** — voir les tableaux de la concurrence ci-dessus.

## Nouveaux candidats pour la feuille de route issus de ce tour
13. Export OCA (Open Cel Animation) et APNG — OpenToonz 1.7
14. Commandes d'adoucissement par lot / « quick ease » dans l'éditeur — Harmony 20
15. Filtres de correction colorimétrique (fondu, niveaux, courbes, gamma) — Harmony 20
16. Ombrage de surface (ombre portée / AO) pour les calques vectoriels — Harmony 20
17. Scripts intégrés (Python/Lua) au-dessus de l'API d'opérations de patch — Harmony 20
18. Nœud de cache de pré-rendu / évaluation en cache — Harmony 20
19. Marqueurs de scène + ripple editing sur la timeline — Harmony 20
20. Compositions de calques, changement de parent d'os animé, calques de correction, suivi de chemin, timecode SMPTE, zones de sécurité, vectorisation automatique, pinceau de dispersion (lacunes trouvées dans l'étude de Moho)

## Lacunes connues (non récupérables depuis les sources accessibles)
Anime Studio 8.x, 10.1.x, 11.1 ; détails de Moho 12.5 et 14.1 ; Spine 3.0–4.1 (le blog 3.7 n'est qu'une page vidéo) ; Harmony 17, 23, 25–27 (pages de documentation en erreur) ; Synfig 1.4.1 ; TVPaint 11 ; Cartoon Animator 4/5.1/5.2 ; journal des modifications de Rive.

## Sources (2e tour)
- docs.toonboom.com — notes de version de Harmony 20
- en.ubunlog.com — article sur OpenToonz 1.7
- synfig.readthedocs.io — notes de version de Synfig 1.4.2
- esotericsoftware.com — présentation des fonctionnalités de Spine 3.7 (page vidéo, sans liste)

---

# Addendum — Sources YouTube (3e tour, 2026-10-19)

Une recherche web restreinte à youtube.com renvoie des titres de vidéos ; WebFetch renvoie la **description / liste de chapitres** de chaque vidéo (pas de transcriptions). Seules les fonctionnalités qui y sont nommées sont listées ; « nouveau dans la version X » est tel qu'indiqué par la personne qui a mis en ligne (souvent l'éditeur).

## Moho 14.1 (description de vidéo)
| Fonctionnalité | OSATURA |
|---|---|
| Afficher/masquer les poignées dans l'outil Curvature | ❌ (pas d'interface de canevas) |
| Effets de forme Shaded et Halo visibles en mode édition | 🟡 dégradés ✅ ; effets halo/shade ❌ |
| Effet de pixellisation dans les Live Project Settings | ❌ |
| Preview Animation : résolution supérieure, bordure de sortie masquée | 🟡 aperçu raster ✅ |
| Boîtes de dialogue non modales pour les scripts Lua | ❌ (pas de scripts) |
| Mode de fusion PSD « Linear Dodge (Add) » ; modes de fusion mis à jour | 🟡 vérifier la liste des modes de fusion ; add/dodge ❌ |
| Indicateur de document non enregistré dans la barre de titre | ✅ le studio suit l'état d'annulation |
| Corrections : évaluation des smart bones, flou de mouvement sous-image + dynamique d'os, poignées de Bézier de longueur nulle | n/a |

## Moho 12.5
Les pages de titre/description n'ont donné aucune liste de fonctionnalités (vidéo obsolète ; téléchargement gratuit de la 12.5 retiré). Les titres associés mentionnent des tutoriels Smart Bones, des rigs 2.5D Smart Warp et l'installation de « Mynd Tools » — Smart Bones 🟡 (canaux/morphs), Smart Warp 🟡 (lattice FFD ✅).

## Spine
| Version | Fonctionnalité | OSATURA |
|---|---|---|
| 3.7 | Prise en charge de l'audio | ❌ (WAV analysé uniquement pour le lip-sync) |
| 3.7 | Exports remaniés | 🟡 |
| 3.7 | Saisie pour rechercher | ❌ |
| 3.7 | IK étirable / compressé / uniforme | ❌ (l'IK conserve les longueurs) |
| 3.7 | Combinaison de skins, duplication de skins, espaces réservés | 🟡 calques de commutation |
| 3.7 | Copier/coller de sommets | ❌ |
| 3.7 | Rendu sur grille de pixels | ❌ |
| 3.7 | Mélange d'animations additif | 🟡 morphs additifs uniquement |
| 3.7 | Runtime spine-cpp ; lecteur web | ❌ / 🟡 lecteur HTML |
| 3.8 | Traçage de maillage | 🟡 maillage en grille automatique ; pas de vectorisation d'image |
| 3.8 | Empaquetage de textures polygonal | ❌ |
| 3.8 | **Soft IK** | ❌ |
| 3.8 | Os et contraintes de skin | ❌ |
| 3.8 | Projet mix-and-match | ❌ |
| 3.8 | Dossiers, règles, export PSD amélioré, écran d'accueil | ❌ |
| 3.8 | .json/.skel plus petits ; chargement binaire dans spine-ts | ❌ (JSON uniquement) |
| 4.0 bêta | **Éditeur de graphes / courbes** | 🟡 interpolation par clé ✅ ; pas d'éditeur |
| 4.1 bêta | Fonction Sequence (attachement de séquence d'images) | ❌ |

## Toon Boom Harmony 25 / 25.2
| Fonctionnalité | OSATURA |
|---|---|
| Outil Pencil Retouch | ❌ |
| Import de pinceaux Photoshop | ❌ |
| **Breakdown Assistant pour l'animation cut-out** | ❌ (le générateur de cycle de marche est apparenté) |
| Verrouillage de la règle de timeline ; préférences de modèles d'actions | ❌ |
| Meilleure prise en charge des fichiers image, 3D et USDZ | ❌ |
| Art Layer Manager (échanger, effacer, fusionner, dupliquer des calques d'art ; 25.2+) | ❌ (duplication de calque via les opérations de patch ✅) |
| Master Controller Grid Wizard | ❌ |

## Série Cartoon Animator 5
| Fonctionnalité | OSATURA |
|---|---|
| 5.1 Édition aller-retour PSD et vectorielle | ❌ |
| 5.1 Import de séquences d'images et de GIF | ❌ |
| 5.2 **Motion Pilot** (manipulation de marionnette par glissement) | ❌ |
| 5.2 Animation sur Motion Path / édition de chemin | ❌ (feuille de route n° 1) |
| Dynamique de ressorts, FFD, animation vectorielle (existants) | ✅ / ✅ / ✅ |

## Nouveaux candidats pour la feuille de route issus de ce tour
21. Soft IK + modes d'IK étirable — Spine 3.8/3.7
22. Calque de séquence d'images (attachement par liste d'images) et import GIF/APNG — Spine 4.1, CA 5.1
23. Filtre de sortie de pixellisation / alignement sur les pixels — Moho 14.1, Spine 4.2
24. Enregistrement de type Motion Pilot : capturer le glissement en direct d'un os/d'une poignée en clés — CA 5.2
25. Assistant de breakdown (intervalles automatiques avec arcs pour les rigs cut-out) — Harmony 25
26. Traçage de maillage à partir du contour alpha — Spine 3.8
27. Ensembles de skins (échanger ensemble des groupes d'attachements) — Spine 3.7/3.8
28. Piste audio dans le projet + synchronisation de lecture (pas seulement l'analyse) — Spine 3.7
29. Points d'accroche de scripts Lua/Python — Moho, Harmony 20

## Sources (3e tour, YouTube)
- Moho 14.1: A Complete Breakdown — youtube.com/watch?v=QT4AiV1wO1k ; Moho 14.1 Update — watch?v=FCiBKBjDaNg
- Présentation des fonctionnalités de Spine 3.7 — watch?v=Fi0ks2LeYSM ; présentation des fonctionnalités de Spine 3.8 — watch?v=kU26B5465SQ ; éditeur de courbes de Spine v4-beta — watch?v=37CeFRzrjqc
- Changements de Harmony 25 — watch?v=kJ58C2DNIk8 ; Art Layer Manager 25.2+ — watch?v=AZc-8zHucOo
- Cartoon Animator 5.2 Motion Pilot — watch?v=Eu7vO2uGGN4 ; import de séquences/GIF de la 5.1 — watch?v=O8EGwVwqgfs

---

# Addendum — Recherche exhaustive des lacunes (4e tour, 2026-10-19)

Sources : notes de version officielles de Synfig/Rive, titres/descriptions YouTube. Seules les fonctionnalités nommées sont listées.

## Anime Studio 8 (série YouTube « Pro Feature », titres + descriptions)
| Fonctionnalité | OSATURA |
|---|---|
| Vectorisation automatique d'image (bitmap → vecteur éditable) | ❌ (feuille de route n° 20) |
| Character Wizard (création guidée de personnage) | 🟡 rigs de démonstration/générateur de marche ; pas d'assistant |
| Import de fichiers Photoshop (PSD) | ❌ |
| Real-Time Media Connection (liaison en direct avec des médias externes) | ❌ |
| Outil Scale Points | 🟡 opérations de patch sur les points |
| Préréglages de projet HD 1080p | ✅ toute taille |
Anime Studio 10.1 / 11.1 : aucune source accessible trouvée.

## Synfig 1.4.1 – 1.4.4
| Version | Fonctionnalité | OSATURA |
|---|---|---|
| 1.4.1 | Calques de son mixés dans la vidéo exportée | ❌ (pas de piste audio) |
| 1.4.1 | Dupliquer un calque copie tous les calques sélectionnés | ✅ (opérations par lot) |
| 1.4.3 | Invite de liaison d'os : lier la transformation plutôt que l'origine | ✅ |
| 1.4.3 | Import Papagayo (.pgo) ; corrections de l'import SVG ; cadence d'images des séquences d'images | ❌ / ❌ / ❌ |
| 1.4.3 | Corrections du rendu GIF, compatibilité FFmpeg 3.x | ✅ GIF/MP4 |
| 1.4.3 | Enregistrement automatique (jusqu'à 15 s) | ❌ (enregistrement explicite) |
| 1.4.4 | Correction de l'export de la dernière image | ✅ couvert par les tests |

## Rive (notes de version officielles, sept.–oct. 2026) — le concurrent le plus actif
| Fonctionnalité | OSATURA |
|---|---|
| Convertisseur String Template (texte à partir de valeurs de view-model, formatage des nombres) | ❌ |
| Contours en tirets (Dash Effect), importés depuis SVG | ❌ |
| Maintenir Q/W/E + glisser pour déplacer/pivoter/mettre à l'échelle | ❌ (pas d'interface) |
| Formes de pinceau animables en mode Animate | ❌ |
| Copier/coller de clés de contrainte ; les convertisseurs conservent les formules | 🟡 contraintes ✅, copie ❌ |
| Liaison de données relative ; view models ; propriétés Blob | ❌ |
| Scripts Luau (fetch, mise en forme de texte, canevas GPU, shaders/WGSL, maillages d'image instanciés) | ❌ |
| États de mélange / Mix-by-view-model sur la timeline | 🟡 bake de fondu enchaîné |
| Écouteurs clavier, focus, manette ; saisie de texte ; sémantique d'accessibilité | ❌ |
| Mises en page Grid et Stack, listes virtualisées, contraintes de défilement et physique de défilement | ❌ |
| Set Parent (Alt+P) conservant position et ordre de dessin | 🟡 changement de parent via opérations de patch ; conservation de la position dans le monde ❌ |
| Align to Key Object ; décalage d'os racine ; Scroll to selection | ❌ |
| Force additive pour remplissages/contours ; canaux RGBA séparés animables/liables | 🟡 morphs additifs ; RGBA animé ✅ |
| Contours intérieurs/extérieurs (accès anticipé) | ❌ |
| Auto Trace (meilleur pour le pixel art) | ❌ |
| Conversion Flash `.fla`/XFL → Rive | ❌ |
| Préréglages de rendu : machine à états → vidéo | 🟡 clip → MP4/GIF |
| Export `.riv` du forfait gratuit ; CLI (`rive push/publish/ls/docs`), extension VS Code, simulation d'entrées headless | 🟡 CLI ✅ (commandes propres) |
| Agent IA dans l'éditeur | ❌ |
| Aperçu Android ; runtimes React Native / Apple / Android / Flutter | ❌ |
| Historique des révisions et restauration, synchronisation hors ligne, annulation qui conserve le zoom | 🟡 annuler/rétablir ✅ |

## TVPaint Animation 11 (descriptions YouTube)
Création de raccourcis, sous-dessins amovibles, lissage de ligne, enregistrement du dessin, clic-glisser du point de pivot de KeyFramer (11.7.1), Sprite Generator, Inlay Texture FX, repères, vidéo « end of update releases » — tout ❌ (outils de peinture raster ; OSATURA est vectoriel/rig).

## Toon Boom Harmony 17 (YouTube)
Free Form Deformer 🟡 (lattice FFD ✅), Master Controllers renforcés 🟡, RGB Difference Keyer ❌, Motion Blur ❌ (feuille de route n° 5).

## Cartoon Animator 4 / Spine 3.6 / Harmony 22–23
Vidéos trouvées (CA 4 : plugin de mocap Rokoko, mise à jour du mouvement 3D vers 2D, normal map et éclairage dans la 4.5 ; chemins de Spine 3.4 ; vidéo des fonctionnalités de Spine 3.6 ; outils de dessin de Harmony 22) mais les descriptions ne listent aucune fonctionnalité → rien n'est revendiqué. Notables d'après le seul titre : **retargeting de motion capture** (CA 4) ❌, **normal maps/éclairage** (CA 4.5) ❌, **contraintes de chemin** (Spine 3.4) ❌.

## Nouveaux candidats pour la feuille de route issus de ce tour
30. Dash effect + contours intérieurs/extérieurs
31. Runtime de scripts intégré (Lua/Luau) avec une API sans GPU
32. Liaison de données / view models + entrées de machine à états (clavier/pointeur/focus) pour le lecteur HTML
33. Contraintes de défilement / conteneurs de mise en page (grille, pile) pour l'animation de type interface
34. Import de lip-sync Papagayo (.pgo) ; import de type Rhubarb
35. Set Parent conservant la transformation dans le monde
36. Enregistrement automatique + historique des révisions dans le studio
37. Filtres Motion Blur et RGB keyer
38. Contraintes de chemin (os le long d'une Bézier) — Spine 3.4, fusionne avec la feuille de route n° 1
39. Retargeting de motion capture vers un rig 2D (BVH/JSON) — CA 4
40. Éclairage par normal map pour les calques d'image — CA 4.5
41. Piste sonore mixée dans l'export MP4 — Synfig 1.4.1
42. Import .fla / XFL et PSD

## Toujours non récupérable
Anime Studio 10.1 / 11.1, notes détaillées de Moho 12.5, pages de détail de Spine 3.0–3.6, Harmony 23, notes détaillées de Cartoon Animator 4.x, texte du journal des modifications de TVPaint 11.

## Sources (4e tour)
- synfig.readthedocs.io — notes de version 1.4.1, 1.4.3, 1.4.4
- releasebot.io/updates/rive — Rive Editor 0.9–0.9.272, CLI 1.2–1.6
- YouTube : série Anime Studio 8 Pro Feature (q_HH2zOE-Yk, CUemDIlPH6Y, 408CLy652mU, l9eXLUO9Dus) ; démo TVPaint 11 (MoqNrCAUy5k) ; Harmony 17 (1CU_3RJBcl4) ; playlist Cartoon Animator 4

---

# Addendum — Réseaux sociaux et canaux communautaires (5e tour, 2026-10-19)

**Réalité de l'accès :** les pages X/Twitter ne peuvent pas être récupérées à cause de robots.txt (seuls les extraits/titres des résultats de recherche sont visibles) ; Facebook de même ; les recherches Reddit avec filtre de domaine ont été rejetées par le proxy de recherche et les recherches sans filtre n'ont renvoyé aucun fil Reddit ; le forum de Lost Marble sert une coquille de redirection JavaScript sans contenu lisible. Rien n'a été contourné. Ce qui était lisible :

## Canaux officiels de Moho (actualités moho.lostmarble.com, extraits X)
| Élément | Notes | OSATURA |
|---|---|---|
| Moho 13.5.3 (avr. 2022) | lip-sync automatique amélioré, meilleure interface, os moins intrusifs | ✅ lip-sync (WAV heuristique) |
| Moho 13.5.5 (X, mai 2022) | **Relative Paths** pour les ressources liées | ✅ le répertoire de ressources est relatif |
| Moho 14.3 « Mantis Shrimp » (déc. 2024) | mise à jour centrée sur les pinceaux | ❌ |
| Moho 14.4 « Side Quest » (nov. 2025) | export vers des moteurs de jeu et des logiciels 3D | 🟡 Lottie/JSON propre ; glTF/FBX ❌ |
| Moho 14.5 « Showtime » (oct. 2026) | onion skins magnétiques, Shift & Trace, options de Liquid Shapes, EXR/PNG avec profils colorimétriques, mode live-playback pour diffuser/contrôler des personnages en temps réel | voir la matrice principale ; mode direct ❌ |
| Programme Ambassadeurs (juin 2026) ; position officielle : aucun outil d'IA prévu (mars 2025) | politique | n/a — OSATURA n'a pas non plus de fonctionnalités d'IA |
| Webinaires (Víctor Paredes, etc.) | tutoriels | n/a |

## Comparaisons de la communauté (forum Spine, « Spine vs Moho 12 »)
- Les utilisateurs apprécient dans Moho : la physique intégrée, le masquage de calques, les smart bones, l'édition vectorielle.
- Les utilisateurs critiquent : une FFD jugée faible ; **pas de vrai runtime**, export de jeu limité à FBX.
- Creature est loué pour sa motion capture intégrée (FFD critiquée).
- Les contraintes de physique de Spine 4.2 suscitent des réactions enthousiastes sur X (messages d'utilisateurs).
OSATURA : physique ✅ (ressorts précalculés), masquage 🟡 (clip via booléens), smart bones 🟡, édition vectorielle ✅, runtime ❌, mocap ❌.

## Titres de listes de souhaits de forum vus (contenu illisible)
« Concentrate on Timeline/Workflow improvements » (2024), « Road map for Moho », « Help Shape the Future of Anime Studio », « 13.5.2 performance issue » — suggère que la timeline/le flux de travail et les performances sont les domaines les plus demandés ; ce n'est pas une liste de fonctionnalités vérifiée.

## Ajouts à la feuille de route
43. Mode live-playback / streaming en temps réel (pose en entrée par WebSocket → images rendues) — Moho 14.5
44. Bibliothèque runtime dans un petit langage portable (build WASM de l'évaluateur) — faiblesse de Moho la plus citée
45. Masquage de calques comme type de calque à part entière (pas seulement des opérations booléennes)
46. Ergonomie de la timeline/du flux de travail dans le studio (éditeur de courbes, sélection multiple de clés) — listes de souhaits de la communauté

## Sources (5e tour)
- moho.lostmarble.com/blogs/news (17 articles, déc. 2021 – oct. 2026)
- Extraits de recherche X : messages de @MohoAnimation (Relative Paths de la 13.5.5 ; tarification de Moho 14), messages d'utilisateurs sur la physique de Spine 4.2
- Fil du forum Spine « 8033 spine vs moho 12 » (de.esotericsoftware.com)
- Titres de fils du forum Lost Marble (contenu illisible)

---

# Addendum — Listes de souhaits et tableaux de demandes de fonctionnalités (6e tour, 2026-10-19)

Seules les demandes réellement lisibles sont listées. La colonne « OSATURA » indique si la demande est déjà couverte par le moteur.

## Moho (forum Lost Marble)
Lisible : uniquement l'index du forum. Les sections « Feature Request Discussions » et « Unofficial Bug Reports and Feature Requests » existent ; titre de demande le plus récent visible : **« Dock Windows – Custom Interface »** (18 oct. 2026). Le long fil « Help Shape the Future of Anime Studio » (828 réponses, 2011–2021) et d'autres fils de souhaits (« Road map for Moho », « My wish for Moho in 2024 – Concentrate on Timeline/Workflow improvements », « Request for top 10 Anime Studio Pro 6 features ») sont des coquilles de redirection JavaScript → contenu illisible. Comparaisons précises de forum à forum (forum Spine) : physique, masquage, smart bones souhaités dans Spine ; Moho critiqué pour sa FFD faible et l'absence de runtime.
| Demande | OSATURA |
|---|---|
| Fenêtres ancrables / interface personnalisée | ❌ (disposition fixe du studio web) |
| Améliorations de la timeline / du flux de travail | 🟡 |
| Vrai runtime pour les jeux | ❌ |
| FFD plus puissante | ✅ lattice de Bernstein + maillage |

## Cartoon Animator (Reallusion Feedback Tracker + fil de liste de souhaits)
| Demande (votes lorsqu'ils sont indiqués) | OSATURA |
|---|---|
| Courbe de clé par défaut dans les préférences | ✅ interpolation par clé |
| Édition de clés multicanal | 🟡 opérations de patch par lot |
| Paramètres de panneau persistants | ❌ |
| Marqueurs et canaux codés par couleur (gauche rouge / droite vert) | ❌ (miroir G↔D ✅) |
| « Uber key » pour afficher tous les canaux animés | ❌ |
| Prise en charge de SVG (Affinity, Inkscape) ; coller des chemins vectoriels | ❌ |
| Outil/panneau d'alignement | ❌ |
| Profondeur de champ / filtre de flou | ❌ |
| Import GIF/APNG/vidéo par glisser-déposer placé sur la timeline | ❌ |
| Lumières spot/ambiantes avec ombres | ❌ |
| Défilement automatique de la timeline | n/a |
| Meilleur texte : crénage/interlignage animables | ❌ |
| Couleur SVG animable | ✅ |
| Masquage global / canaux « shy » | ❌ |
| Panneaux ancrés | ❌ |
| Plusieurs scènes par projet | ❌ |
| **Panneau Auto Animate (mouvement d'inactivité procédural sur les os)** | 🟡 ondulation à bruit cohérent ✅ |
| Calque de dessin en direct / accessoires dynamiques éditables (18) | ❌ |
| Hitbox et physique (14) | 🟡 ressorts |
| Import MOV transparent (14) | ❌ |
| Panorama à 360° (5) ; idées d'intégration d'IA (6) | ❌ |

## Tableau de demandes de fonctionnalités de Live2D Cubism Editor (30 fils, 2023–2026)
| Demande | OSATURA |
|---|---|
| Prise en charge de Linux | ✅ |
| Raccourcis clavier dans l'édition manuelle de maillage | n/a |
| Limite de blendshape jusqu'à −100 % | ✅ morphs non bornés |
| Redimensionnement proportionnel ; outil de gravité ; verrouillage maillage à maillage | ❌ / ❌ / ❌ |
| Glue pour les déformateurs + poids de glue précis | 🟡 poids ✅ |
| Importer la physique sans écraser les groupes existants | ❌ |
| Verrouiller les paramètres dans l'animateur ; édition de boucle vs répétition | ❌ |
| Options « Expand Warp based on Child » ; changer le centre de rotation du warp | ❌ |
| Outil de type « Set Vertex Flow », sélection d'arêtes en boucle, mappage de maillage à densité variable par trait | ❌ (auto-mesh uniforme) |
| Prise en charge du pixel art | ❌ |
| Opacité des sommets ; filtrage des parties et affichage plus propre | ❌ |
| Déplacer facilement les clés | ✅ opérations de patch |
| Purger de la mémoire l'atlas en cache | n/a |
| Fenêtre de physique : % de sortie maximal, édition en masse des plages de pendule | 🟡 |
| RISC-V / Cubism Core embarqué | ❌ |
| Drawables facultatifs dans la spécification CDI3 | n/a |

## Toon Boom Harmony
Le forum des utilisateurs est fermé et redirige vers Discord (non consultable par recherche) ; les anciens fils du « Feature Requests board » (« One can only dream: A Feature List », « Software Upgrade Suggestions Part 2 », « Bitmap Layer Overhaul LONG OVERDUE », « Various Quality of Life suggestions ») n'affichent que l'avis de fermeture. Aucune demande récupérable.

## Synfig (liste d'idées GSoC 2019–2022, numéros de tickets GitHub liés)
| Demande | OSATURA |
|---|---|
| Copier/coller des waypoints (#2139) | ✅ clés via opérations |
| Changement d'onglet par Ctrl+Tab, raccourcis de fenêtres, sélection de la zone de lecture, recherche dans les propriétés, bascule des règles, modification de paramètres à la souris | ❌ (interface) |
| Performances : rendu/export plus rapide ; mise en cache des paramètres de calque ; arbre de tâches de rendu incrémental | ✅ evaluate en fonction pure ; pas de cache ❌ |
| Portage GTK 4, builds Conan/vcpkg | n/a |

## Spine
Les demandes vont sur le forum et sont déplacées vers un tableau Trello public (non récupérable : robots.txt). D'après le forum : la plus haute priorité historique est la stabilité de l'éditeur et les mises à jour des runtimes. Le plan de l'époque Kickstarter (maillages → FFD → skinning → IK pinning) est désormais entièrement livré ; les maillages/FFD/skinning d'OSATURA sont donc à parité avec cette ancienne liste ; IK pinning ✅ (contrainte Pin). Titres de demandes récentes du forum : prise en charge des dernières versions de three.js / React Three Fiber ; prise en charge de Spine 4.3 dans Defold.

## Rive
Feuille de route publique = « canal Feature Requests » avec étiquettes de statut (en cours / prévu / lancé) ; une connexion est nécessaire pour la lire. Rien de récupérable au-delà des notes de version (4e tour).

## OpenToonz, TVPaint, Pencil2D
Les pages GitHub (OpenToonz/Synfig) sont bloquées par robots.txt ; aucun tableau TVPaint lisible trouvé ; un fil Pencil2D « Feature requests to add » existe mais n'a pas été ouvert (hors périmètre).

## Ajouts à la feuille de route issus des listes de souhaits
47. Calque auto-animate / d'inactivité procédural (respiration, clignement, balancement de la tête avec rareté et intensité) — souhait CA n° 20 ; s'appuie sur l'ondulation existante
48. Profondeur de champ de la caméra et filtre de flou — CA
49. Lumières (spot/ambiante) avec ombres portées pour les calques d'image — CA
50. Opérations d'alignement / répartition (opération d'éditeur) — CA
51. Codage couleur des canaux et visibilité « shy » des canaux dans le studio — CA
52. Plusieurs scènes par projet + coupes de scènes — CA
53. Auto-mesh à densité variable ; lissage de flux de sommets — Live2D
54. Paramètre de centre de rotation du déformateur warp ; redimensionnement proportionnel — Live2D
55. Import-fusion de la physique sans écraser les groupes — Live2D
56. Verrouillage des paramètres par piste pendant l'édition de boucle — Live2D
57. Disposition du studio ancrable/personnalisée — Moho, CA
58. Cache des paramètres entre les images (évaluation incrémentale) — Synfig

## Sources (6e tour)
- Reallusion Feedback Tracker : « Wish list for new features for future updates » ; tableau Cartoon Animator 5 (reallusion.com/FeedBackTracker)
- community.live2d.com — demandes de fonctionnalités de Cubism Editor (page 1 sur 7)
- Liste d'idées GSoC 2022 de Synfig (synfig-docs-dev.readthedocs.io)
- Index du forum Lost Marble (lostmarble.net/forum) ; fils du forum Spine 2250, 8033 ; fil Kickstarter 1501
- FAQ de Rive : « Does Rive have a public roadmap? »

---

# Addendum — Projets open source sur des plateformes git (7e tour, 2026-10-19)

**Accès :** l'API de recherche de GitHub n'est pas disponible dans cet environnement (« sessions are bound to their configured repositories »), les projets ont donc été trouvés par recherche web et lus dans leurs README publics. Les API GitLab et Codeberg sont bloquées par le proxy ; les pages Codeberg/GitLab n'apparaissaient que dans les résultats de recherche (Friction était lisible). Le nombre d'étoiles n'a pas pu être récupéré. Plusieurs projets sont très récents / à un stade précoce — les fonctionnalités listées sont celles annoncées dans les README, non vérifiées par exécution.

## Outils open source similaires de rigging / d'animation
| Projet | Plateforme | Fonctionnalités notables | Comparaison avec OSATURA |
|---|---|---|---|
| **Rigby2D** (jaisurya-dev-art/rigby2d) | GitHub, MIT/Apache | Éditeur de style Spine (import PSD, maillage + peinture de poids, maillages liés, contraintes IK/transform/path/physics/**slider**, skins avec os propres au skin, événements, attachements bounding-box/clipping/point/path, dopesheet + éditeur de courbes), export Spine JSON 3.8/4.2/4.3 + .skel, atlas, runtimes JS/Rust/Godot, **serveur MCP intégré** pour assistants IA, bureau Tauri | OSATURA n'a ni interface d'éditeur, ni atlas, ni runtimes, ni skins, ni événements, ni contraintes slider/path ; il a un évaluateur en fonction pure, des FX liquide/booléens/émetteur, le lip-sync, un générateur de marche |
| **SkelForm** (Retropaint/SkelForm, Rust) | GitHub | IK FABRIK, natif + web + enveloppe mobile | cœur de rig similaire ✅ ; pas d'interface ❌ |
| **Ankhimate editor** | GitHub, pré-alpha | Modes Setup/Animate, slots + skins, FABRIK de longueur quelconque, contraintes path/physics, traçage de silhouette de maillage à partir de l'alpha, pinceau de poids avec modes de fusion + verrouillage d'os, **timelines FFD**, événements avec audio, marqueurs, décalages de piste par os, **éditeur de format d'export à base de modèles**, runtime de référence en Rust | ❌ surtout interface ; idées : modèles d'export, décalages de piste par os |
| **rig2d-rs** (tanhieuno41) | GitHub, Rust | Portage de la physique de Spine 4.2, IK avec **mix/softness/bend/compress/stretch/uniform**, liaison automatique des poids par décroissance de distance, lissage/normalisation/limitation à 4 des poids, packer d'atlas par étagères, ghosting d'onion skin, import/export Spine JSON | ❌ IK souple/étirable, outils de poids, atlas, interopérabilité Spine |
| **Boner2D**, **Skelly**, **omarvision 2d-bones-IK** | GitHub | Outils squelettiques Unity / pour concepteurs de jeux | basique |
| **Skeletal-Animator** (ostad-ai) | GitHub, Python | os + FABRIK, formes rect/capsule/cercle, 4 interpolations, onion skin, fenêtre de découpe de caméra animée par clés, **plages de visibilité par os**, spritesheet PNG+JSON, **MP4/WebM/MOV/GIF transparents** | ❌ plages de visibilité, export de vidéo/spritesheet transparent, onion skin |
| **Stretchy Studio** (MangoLion) | GitHub, MIT | Import PSD/PNG, **auto-rigging par IA** (détection de pose DWPose), assistant de configuration, animation de sommets par maillage de partie, skinning de sommets pour les membres, **écrêtage automatique des yeux** (iris à l'intérieur de l'œil), **shape keys de style Blender** avec curseurs d'influence, pistes audio sur la timeline, **export Spine 4.0 JSON** | ✅ morphs ≈ shape keys ; ❌ auto-rig, écrêtage des yeux, audio |
| **Iki** (zeikar/iki) | GitHub, MIT | Moteur web de style Live2D : quads pilotés par paramètres, déformateurs warp-mesh/grille, masques de stencil, physique ressort/chaîne, import PSD, **auto-rigger à partir de calques nommés par rôle** (clignement, regard, lip-sync, rotation de la tête, hochement, inclinaison, sourcils, respiration, physique des cheveux), schéma JSON ouvert avec **identifiants StandardParameter**, serveur MCP + plugin Claude Code | ✅ schéma JSON, lip-sync ; ❌ identifiants de paramètres standard, auto-rigger, masques |
| **Umamo** (umamoorg/umamo) | GitHub, Java | Éditeur compatible Live2D : lecture/écriture CMO3/MOC3 (jusqu'à Cubism 5.4), import PSD/KRA/CLIP, palette de commandes, préréglages de raccourcis, **CLI de diagnostic** (dump/extraction/conversion/diff de modèles, préréglages de qualité d'auto-mesh) ; animation pas encore prise en charge | ❌ CLI de diff de modèles, palette de commandes |
| **Ayatsuri2D** | GitHub | Modèle `.aom` MessagePack (sur-ensemble de moc3), export moc3 v3–6, ABI C, fuzzing en CI, runtime sans panic, benchmarks de latence p99.9 (précoce, « not usable yet ») | ❌ format binaire, ABI C, fuzzing |
| **Ayagami** (AyagamiDev) | GitHub, Rust/wgpu | MOC3 jusqu'au SDK 5.0 avec glue et paramètres de blendshape, recalcul partiel, mélange linéaire/sRGB, mélange avancé / hors écran du SDK 5.3, composant Godot | ❌ moteur de rendu GPU, MOC3 |
| **kasane-2d**, **img2live**, **claude-live-2d**, **minivt** | GitHub | Modélisation de type Live2D / recherche image unique vers marionnette | n/a |
| **Glaxnimate** (KDE) | miroir GitHub / KDE | animation vectorielle ; les dépendances de build montrent la vectorisation bitmap Potrace, libav, scripts Python, libarchive ; accent sur Lottie | ❌ vectorisation, scripts |
| **Friction** (friction/friction, Codeberg) | Codeberg, GPL-3 | animation vectorielle + raster pour le web/la vidéo, motion graphics, SVG, Skia + FFmpeg + GLSL (basé sur enve) | ❌ animation SVG, shaders |
| **Tahoma2D** | GitHub | 2D + stop-motion, basé sur la version Studio Ghibli de Toonz | ❌ |
| **Pencil2D** | GitHub, GPL-2 | dessin à la main bitmap + vectoriel, onion skin | ❌ |
| **q0idmation** (q0s0id) | GitHub | éditeur + lecteur + **langage de script** | ❌ scripts |
| **svgMotion** (michaelsboost) | GitHub | animation vectorielle dans le navigateur | ❌ |
| **kinetic** | GitHub | timeline de clés SVG → export CSS/GSAP | ❌ export CSS/GSAP |
| **dotlottie-rs** (LottieFiles) | GitHub, Rust | **machines à états** (gardes/transitions/actions), thèmes/**slots** (surcharges à l'exécution de couleur/scalaire/texte/vecteur), conteneur zip `.lottie` multi-animations, API C + WASM, moteur de rendu ThorVG | 🟡 export Lottie ✅ ; ❌ slots/machines à états/conteneur dotLottie |
| **rive-runtime** | GitHub | Runtime C++ : moteur de rendu vectoriel GPU (Metal, Vulkan, D3D11/12, GL/WebGL), encapsulé par Apple/Android/Flutter/Unity/Unreal/web | ❌ |
| **spine-runtimes** | GitHub | runtimes pour Unity, Godot, Unreal, libGDX, MonoGame, XNA, Flutter, Haxe, C/C++/C#/TS, Android, iOS, GLFW, SDL, SFML ; politique d'une branche par version de l'éditeur | ❌ |
| **Dépôt d'outils Moho** (wanderjimenezRD/Moho-Pro-14.4-2D-Animation-Tools) | GitHub | simple listage ; la description ne donne aucune fonctionnalité — non évalué | — |

Des dépôts de logiciels piratés sont apparus (par ex. Adobe Animate « activate-free ») : ni ouverts ni utilisés.

## Idées à adopter (nouveaux candidats pour la feuille de route)
59. **Serveur MCP / API d'agent** au-dessus de l'éditeur d'opérations de patch afin que les assistants IA puissent rigger et animer (Rigby2D, Iki) — s'accorde avec l'architecture JSON-patch d'OSATURA
60. **Identifiants de paramètres standard** (ParamAngleX, EyeOpen, MouthOpen…) pour que n'importe quel hôte puisse piloter un modèle (Iki)
61. **Auto-rigger à partir de calques nommés par rôle** : clignement, regard, rotation/hochement/inclinaison de la tête, sourcils, respiration, physique des cheveux (Iki)
62. **Écrêtage automatique des yeux** (iris masqué par la forme de l'œil) (Stretchy Studio) — via un calque Boolean
63. **Contrainte slider** et **contrainte de chemin** (Rigby2D, Spine)
64. Modes d'IK **souple / étirable / compressé / uniforme** (rig2d)
65. **Outils de poids** : lissage, normalisation, limitation à N, copie, verrouillage d'os, modes de fusion du pinceau (rig2d, Ankhimate)
66. **Traçage de silhouette de maillage à partir de l'alpha** (Ankhimate)
67. **Plages de visibilité par os** (Skeletal-Animator)
68. **Export vidéo à fond transparent (WebM/MOV avec alpha) et spritesheet PNG+JSON** (Skeletal-Animator)
69. **Moteur de modèles d'export** (formats de sortie définis par l'utilisateur) (Ankhimate)
70. **Conteneur dotLottie `.lottie` + slots/thèmes à l'exécution + machines à états** (dotlottie-rs)
71. **CLI de diff / dump / conversion de modèles** (`osatura diff a.json b.json`) (Umamo)
72. **Palette de commandes + raccourcis reconfigurables** dans le studio (Umamo)
73. **Chargeur fuzzé, chemin runtime sans panic, benchmarks de latence** (Ayatsuri2D) — durcissement de `Engine::new`
74. **Import/export Spine JSON** pour l'interopérabilité (Rigby2D, rig2d, Stretchy Studio)
75. **Packer d'atlas par étagères** pour les calques d'image (rig2d, Rigby2D)
76. **Événements avec valeurs clés et repères audio** sur la timeline (Rigby2D, Ankhimate)
77. **Décalages / décalage temporel de piste par os** (Ankhimate)

## Sources (7e tour)
- github.com/jaisurya-dev-art/rigby2d ; Retropaint/SkelForm ; Ankhimate/editor ; tanhieuno41/rig2d-rs (version v0.1.0)
- github.com/zeikar/iki ; MangoLion/stretchystudio ; umamoorg/umamo ; Yoru-KenomoVT/ayatsuri_2d ; AyagamiDev/ayagami
- github.com/ostad-ai/Skeletal-Animator ; KDE/glaxnimate ; tahoma2d/tahoma2d ; pencil2d/pencil
- github.com/rive-app/rive-runtime ; EsotericSoftware/spine-runtimes ; LottieFiles/dotlottie-rs
- github.com/topics/animation-editor, /topics/vector-animation
- codeberg.org/friction/friction

---

# Addendum — Autres sources publiques (8e tour, 2026-10-19)

Sources : itch.io, Steam, AlternativeTo, Wikipédia, résultats de recherche académique/de brevets, tests d'outils After Effects. Les affirmations proviennent de pages de boutiques/de projets, pas de tests pratiques.

## Outils de rigging indépendants / de niche (itch.io, Steam)
| Outil | Fonctionnalités notables | OSATURA |
|---|---|---|
| **Inochi Creator / Inochi2D** (Kitsunebi Games, open source, bêta) | rigging de marionnette 2D ouvert orienté VTuber + runtime temps réel ; format propre (non compatible Live2D/Spine) ; export GIF/séquence d'images ; interface multilingue | 🟡 même niche (format de rig ouvert) |
| **Keyframe.it** (navigateur, gratuit) | os + IK + maillages pondérés + mouvement de ressort ; **clips de symboles de jeu** (idle, land, anticipation, win) ; éditeur d'images intégré + import PSD ; export GIF, sprite sheet, vidéo, lecteur HTML autonome, **bundles Spine 3.8/4.2** ; enregistrement automatique dans le navigateur ; connecteur d'agent IA **keyframe-mcp** ; assistant IA intégré optionnel | ✅ lecteur HTML/GIF ; ❌ sprite sheet, bundle Spine, MCP, éditeur dans le navigateur |
| **RigBone 2D** (Godot) | pose d'os, pose-comme-image, onion skin, recadrage + fond transparent, séquence d'images PNG (aucun runtime nécessaire) | ❌ onion skin, séquence PNG avec recadrage alpha |
| **SpineBones** | slots + attachements d'image, groupes, IK, **aperçu audio sur la timeline**, image de référence en arrière-plan, export JSON / sprite sheet / WebM | ❌ |
| **PySpine, Tangent (sprite animator)** | listés sur itch.io ; aucun texte de fonctionnalités lu | — |
| **Mocap2D / FBX2SPINE** (Steam) | import de mocap FBX/BVH ; **Auto Match** du squelette par noms + hiérarchie ; **Smart Retargeting** (projection, bon pour la vue de côté) ; **Direct Retargeting** par axe ; **conversion automatique sur place** pour les boucles de locomotion ; **IK de contact avec le sol** (verrouillage de pied) ; aperçu en direct ; import/export de FBX, BVH, squelettes Spine, Unity 2D Animation | 🟡 retarget par nom ✅ ; ❌ import de mocap, IK à verrouillage de pied, conversion sur place |
| **VTube Studio** (Steam) | suivi par webcam (OpenSeeFace), **bouche pilotée par la voix**, souris/stylet comme entrée de suivi, mappage de n'importe quelle entrée vers n'importe quel paramètre de modèle, raccourcis pour expressions/animations, « items » PNG épinglés qui suivent le modèle ; modèles Live2D uniquement | 🟡 canaux de paramètres ✅ ; ❌ mappage d'entrée en direct, raccourcis, items épinglés |
| VBridger, Vitamins | Outils Steam autour du VTubing ; non évalués | — |

## Écosystème de rigging After Effects (test School of Motion)
| Fonctionnalité | Source | OSATURA |
|---|---|---|
| **Contrôleur joystick** (5 poses clés : origine/droite/gauche/haut/bas) interpolé en 2D | Joysticks 'n Sliders | ❌ — candidat sérieux |
| **Sliders** mélangeant un nombre illimité de poses | JnS, Duik | 🟡 canaux de morph |
| Structures (os indépendants de l'illustration, réutilisables), autorig | Duik Bassel | ✅ |
| Contrôleurs : slider, slider 2D, angle | Duik | ❌ |
| **Connector** (slider/joystick/rotation reliant n'importe quelle propriété à une autre), contrôle de propriété maître | Duik | 🟡 expressions ? (moteur d'expressions ✅ dans les canaux) |
| Clés de pose intermédiaire entre des états de pose | Duik | ✅ |
| Animer les chemins de masque | JnS | ✅ morphs de chemins |

## Applications concurrentes / alternatives (AlternativeTo, liste Wikipédia en 2026)
Tahoma2D (fork d'OpenToonz, stop-motion), TupiTube, enve (abandonné ; devenu Friction), Expressive Animator (SVG interactif), Cavalry, Jitter (motion design web), Blazy (Figma → animation, export CSS/React), Animation Desk, Pivot Animator, FlipaClip, animation Krita, Blender (mars 2026), Clip Studio Paint (animation), Adobe Character Animator (marionnette par webcam ; la page « what's new » est bloquée par robots.txt), Adobe Animate (dernière version listée : oct. 2023), CrazyTalk, DigiCel FlipBook, Apple Motion, RETAS. Aucun n'ajoute de fonctionnalités de rigging au-delà de ce que couvrent les tours précédents ; nouvelles idées : **préréglages outil de conception Figma → animation avec export CSS/React** (Blazy) et **marionnette par webcam** (Character Animator, VTube Studio).

## Littérature académique et de brevets (titres/résumés uniquement)
- Jacobson et al. 2011, **Bounded Biharmonic Weights** pour la déformation en temps réel — poids de skinning lisses automatiques à partir d'une cage/d'un squelette ✅ idée : améliorer `auto_skin` (actuellement basé sur la distance).
- Brevets Adobe US9865073 / US10078910, **« Computer animation of artwork using fast 2-dimensional deformation solving »** (mesh warp de Character Animator). *Note clean-room :* OSATURA utilise la FFD de Bernstein et le skinning pondéré, pas ce solveur breveté ; tout futur solveur de type ARAP devrait être examiné avec un conseil juridique. Ceci n'est pas un avis juridique.
- arXiv 2025–26 sur l'auto-rigging : **ASMR** (squelette + skin de maillage 3D via des a priori génératifs 2D), UniRig, « How to Train Your Dragon » (rigging par diffusion), HumanRig, SPRig, UniMate — tous 3D/ML ; pertinents seulement comme future fonctionnalité de « suggestion d'auto-rig ».
- Stony Brook TVC 2017 **retargeting automatique de poids de skinning** — copier les poids entre maillages (✅ feuille de route : copie de poids).

## Nouveaux candidats pour la feuille de route issus de ce tour
78. **Contrôleur joystick** : point de contrôle 2D mélangeant 5 poses clés (et variantes à N poses) → pilote n'importe quels canaux — JnS / Duik / paramètres Live2D
79. Import de mocap (BVH) + appariement automatique du squelette + conversion de boucle sur place + IK à verrouillage de pied — Mocap2D
80. Mappage d'entrée en direct : n'importe quelle entrée (souris, MIDI, niveau audio, points de repère de webcam) → paramètre, plus raccourcis pour les expressions — VTube Studio
81. Items/accessoires épinglés suivant un os/une ancre avec leur propre décalage — VTube Studio
82. Préréglages de clips de symboles de jeu (idle / land / anticipation / win) — Keyframe.it
83. Export de spritesheet (PNG + JSON) et de séquence PNG transparente avec recadrage — Keyframe.it, RigBone
84. Aperçu audio de la timeline avec forme d'onde — SpineBones
85. Calque d'image de référence (arrière-plan non rendu) — SpineBones
86. Poids d'auto-skin à biharmoniques bornées ou par diffusion de chaleur (remplacer la décroissance de distance) — article BBW
87. Préréglages d'outils de conception → export CSS/React — Blazy
88. Marionnette par suivi du visage à la webcam via le studio (WebRTC + modèle de points de repère) — Character Animator, VTube Studio

## Sources (8e tour)
- itch.io : kitsunebi-games/inochi-creator, keyframeit.itch.io, itch.io/blog/1586091 (RigBone 2.0), itch.io/t/6177139 (SpineBones)
- Steam : FBX2SPINE / Mocap2D (app 2467960), VTube Studio (app 1325860)
- schoolofmotion.com — Joysticks 'n Sliders vs Duik Bassel
- alternativeto.net (alternatives à Moho, Expressive Animator) ; en.wikipedia.org/wiki/List_of_2D_animation_software
- Jacobson 2011 BBW (homes.cs.washington.edu/~jovan/papers/jacobson-2011-bbw.pdf) ; brevets US9865073B2 / US10078910 ; arXiv 2503.13579, 2503.15586, 2504.12451 ; Stony Brook TVC 2017 retargeting de poids

---

# Addendum — Adobe Character Animator, ensemble complet des fonctionnalités (9e tour, 2026-10-19)

Sources : tableau des versions de Wikipédia (y compris la 26.0 de janv. 2026), page produit d'Adobe, page d'aide Adobe « Behaviors » et notes de version (oct. 2018, oct. 2020/févr. 2021, oct./déc. 2021, 2023). L'index « what's new » d'Adobe ne renvoie que 2023 et 2026 ; les versions 24.x et 25.x sont listées par Wikipédia comme des versions de stabilité uniquement.

## Concept
Des illustrations Photoshop/Illustrator en calques (aussi EPS/SVG depuis oct. 2018) deviennent une **marionnette** riggée automatiquement à partir des **noms/étiquettes de calques** ; la performance est capturée en direct (visage/corps par webcam, microphone, clavier, souris, MIDI) sous forme de **takes** sur une timeline ; les sorties vont vers des séquences PNG, WAV, Media Encoder (y compris **vidéo avec alpha**), Dynamic Link vers After Effects / Premiere, et Syphon / Mercury Transmit en direct. Le mode Starter est gratuit ; Pro nécessite Creative Cloud.

## Behaviors (liste complète d'après l'aide Adobe)
| Behavior | Ce qu'il fait | OSATURA |
|---|---|---|
| **Face** | tête, yeux, sourcils, nez, bouche à partir de la webcam ; parallaxe et intensité de position de la tête | ❌ (pas de suivi en direct) — feuille de route n° 88 |
| **Body** (Body Tracker, Sensei AI, 22.0) | bras, torse, jambes à partir de la webcam | ❌ |
| **Lip Sync** | visèmes de bouche à partir de la voix ; précision assistée par transcription (22.1) ; moteur 3.4 meilleur pour F, W-Oo, moins de L/M | 🟡 WAV → 5 visèmes ✅ ; alignement de transcription ❌ |
| **Nutcracker Jaw** | mâchoire inférieure à partir du visage/de la voix | ❌ |
| **Eye Gaze** | les pupilles bougent indépendamment | ✅ contraintes aim/canaux |
| **Head & Body Turner** | échange des groupes de calques (face, trois-quarts, profil) lorsque la tête/le corps tourne | 🟡 calques de commutation ✅ ; pas d'échange automatique piloté par l'angle |
| **Walk** | cycle de marche, course, accentuation de pose, balancement des épaules/hanches, **Pin Feet When Standing** (3.4/3.5) | ✅ générateur de marche ; ❌ pin-feet |
| **Limb IK** (ex Arm IK) | direction de flexion + étirement pour bras et jambes | 🟡 IK à 2 os ✅ ; étirement ❌ |
| **Leader/Follower** (22.5) | façonne la manière dont les membres plient/s'étirent | ❌ |
| **Dragger** | la souris/le toucher déplace une région | ❌ (le studio n'est pas interactif sur le canevas) |
| **Physics / Dangle** | balancement, collision, gravité, vent ; squash & stretch, amortissement | 🟡 ressorts ✅ ; collision/vent ❌ |
| **Magnets** | les poignées s'accrochent par proximité | ❌ |
| **Particles** | émet des objets physiques (neige, tir de canon) | ✅ émetteur |
| **Motion Lines** | lignes de traînée derrière les objets rapides | ❌ |
| **Wiggler** | ondulation sur l'art vectoriel | ✅ ondulation/boil |
| **Breath** | mouvement de respiration | 🟡 canal manuel ; feuille de route n° 61 |
| **Auto Blink** | clignements automatiques / scintillement de calque | ❌ (feuille de route n° 61) |
| **Motion Library** (23.0, plus de 350 mouvements, paramètre d'intensité 23.1) | applique des animations du corps entier | 🟡 marche uniquement ; ❌ bibliothèque |
| **Cycle Layers** | parcourt les calques d'un groupe | 🟡 calques de commutation ✅ ; cyclage d'images ❌ |
| **Layer Picker / Motion Trigger / Fader** | affiche un calque ; commute selon la direction du mouvement ; fondu au déclenchement | 🟡 |
| **Transform** | déplacement/échelle/rotation/opacité des poignées | ✅ |
| **Handle Fixer** | verrouille des parties d'un maillage | ✅ pins |
| **Triggers** | une pression de touche affiche des calques/gestes ; **panneau Controls** avec curseurs, cadrans d'angle, boutons, mappage MIDI | ❌ — feuille de route n° 80 |
| **Collide, Dynamic, Cycle Pause, Auto-swap (22.5)** | étiquettes/behaviors pour la physique et les échanges de type image | ❌ |

## Fonctionnalités de timeline, d'enregistrement et de flux de travail
| Fonctionnalité | Version | OSATURA |
|---|---|---|
| Takes, replays (enregistrements réutilisables déclenchables), replays intelligents, replays éditables | 2.0 / 22.0 / 23.6 | 🟡 clips ✅ ; replays ❌ |
| Clés, étirement temporel des clés, caméras de scène, audio déclenchable | 3.0 / 3.2 | ✅ clés/caméra ; ❌ audio déclenchable |
| Speech-Aware Animation (mouvement de tête/sourcils à partir de la voix, Sensei) | 3.4 | ❌ — feuille de route n° 61 |
| Fusion/séquençage de takes ; lignes de takes colorées ; filtres de recherche (`kind:` `shared:` `armed:`) ; lignes shy ; réduction des takes | 3.4 | ❌ |
| Alignement sur les images / objets ; magnétisme de la timeline | 3.5 | ❌ |
| Opérations de zone de travail : suppression ripple, insertion de temps, boucle, rognage, export | 2.0 | 🟡 opérations de clip ✅ |
| Volet des problèmes de rigging ; recherche d'étiquettes et de triggers | 3.0 | ✅ la validation recueille toutes les erreurs |
| Signets d'historique ; récupération automatique de projet endommagé (4.2) ; nouveau format de fichier compact / résistant à la corruption (4.0) | | 🟡 annulation ✅ ; récupération ❌ |
| Puppet Maker (modèles : cheveux, peau, accessoires) | 22.1.1 | ❌ |
| Characterizer (transformer une photo/peinture en marionnette) | 2.0 | ❌ |
| Quick Export pour le partage sur les réseaux sociaux | 22.5 | 🟡 GIF/MP4 ✅ |
| Instantanés de scène, compte à rebours d'enregistrement, illustrations de boutons de contrôle personnalisées | 1.5 | ❌ |
| Sélection de caméra, multi-écran, HiDPI | 2.0 | n/a |
| Sortie en direct (Syphon, Mercury Transmit), extension Twitch Character Trigger | 1.1 / 2.1 | ❌ — feuille de route n° 43 |
| MIDI, scripts via V8 (JavaScript) | 1.1 / 2.0 | ❌ — feuille de route n° 31 |
| Partage/publication de marionnettes, fichiers de marionnette partageables | Beta 5 | 🟡 JSON portable |
| Dynamic Link vers AE / Premiere | Beta 5 | ❌ |
| Faiblesses rapportées (test) : courbe d'apprentissage abrupte, dépendance matérielle (webcam/micro), long temps de préparation des personnages, Pro payant, complexité de configuration des triggers | | opportunité : pipeline hors ligne déterministe d'OSATURA, CLI/JSON |

## Ce qui ressort pour OSATURA (nouveaux candidats pour la feuille de route)
89. **Auto-rig par étiquettes à partir des noms de calques** (tête, yeux, sourcils, visèmes de bouche, étiquettes bras/jambes) — idée centrale de Character Animator ; étend la feuille de route n° 61
90. **Takes et replays** : enregistrer la performance de paramètres comme clips réutilisables et déclenchables avec interpolation de mélange
91. **Pin Feet When Standing** et verrouillage de contact avec le sol pour le générateur de marche
92. Behaviors d'étirement de membres **Leader/Follower** ; **Magnets** ; **Motion Lines**
93. Format de **Motion Library** (clips de mouvement nommés avec intensité 0-100 %) 
94. **Head & Body Turner** : échange automatique de groupes de vues à partir d'un canal d'angle de lacet (yaw)
95. **Lip-sync assisté par transcription** (alignement forcé du texte sur les phonèmes du WAV)
96. **Mouvement de tête/sourcils sensible à la parole** à partir de l'énergie audio et des pauses
97. **Export vidéo avec alpha** (ProRes 4444 / WebM VP9 alpha) — aussi dans Skeletal-Animator (feuille de route n° 68)
98. **Audio déclenchable** et paramètres réactifs à l'audio

## Sources (9e tour)
- en.wikipedia.org/wiki/Adobe_Character_Animator (liste des fonctionnalités et tableau des versions jusqu'à la 26.0, janv. 2026)
- adobe.com/products/character-animator.html
- helpx.adobe.com — Behaviors in Character Animator ; Feature summaries oct./déc. 2021, oct. 2020/févr. 2021 ; What's new (2023–2026)
- community.adobe.com — « Character Animator October 2018 detailed changes »
- manytools.com/review/adobe-character-animator (forces/faiblesses)
