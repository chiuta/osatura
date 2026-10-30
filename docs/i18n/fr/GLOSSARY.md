<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->

# Glossaire d'OSATURA

Termes de domaine utilisés dans le code et la documentation, par ordre alphabétique de l'original anglais. « Voir » renvoie au
module (dans `osatura-core/src` sauf indication contraire) ou au document où le terme est implémenté ou
spécifié. Documents : [PROJECT_FORMAT.md](PROJECT_FORMAT.md), [ARCHITECTURE.md](ARCHITECTURE.md).
Les sources des techniques sont listées dans `CLEANROOM.md`.

**Contrainte d'aim (Aim constraint).** Une contrainte qui fait tourner un os pour que son axe +x pointe vers la tête d'un autre os,
avec un décalage en plus. Voir `rig` ; PROJECT_FORMAT section 4.3.

**Ancre (Anchor).** Un point d'un tracé de Bézier avec une position et des poignées d'entrée et de sortie facultatives ; elle peut aussi
porter des poids d'os. Voir `path` ; PROJECT_FORMAT section 6.

**Limites d'angle (Angle limits).** Minimum et maximum, par os, de la rotation animée, en degrés, appliqués après
les canaux et après chaque chaîne d'IK. Voir `rig`, `engine`.

**Version de l'API (API version).** La version de la surface Rust stable (`api` et `prelude`), actuellement 1.4.0,
vérifiée par un test d'instantané. À ne pas confondre avec le `format` du fichier ni avec l'`ABI_VERSION` C. Voir `api`.

**Asset.** Un fichier référencé par un projet (image, normal map, son). Les chemins doivent être relatifs et rester
dans le répertoire du projet, sauf si `--allow-external-assets` est donné. Voir `assets` ; PROJECT_FORMAT
section 18.

**Bake.** Calculer des valeurs au fil du temps et les stocker sous forme de canaux ou de clés ordinaires, afin qu'elles n'aient plus
besoin de l'outil qui les calcule. Utilisé par la physique, les clips, magnet, footlock, les machines à états et d'autres. Voir
ARCHITECTURE section 4.

**Flexion (Bend).** Le signe (+1 ou -1) qui détermine dans quel sens plie une chaîne d'IK à deux os, par exemple coude vers l'intérieur ou vers l'extérieur.
Voir `rig`.

**Courbe de Bézier (Bezier curve).** Une courbe cubique paramétrique définie par deux ancres et leurs poignées. Aplatie en segments de
droite (subdivision de de Casteljau) pour le remplissage et la mesure. Voir `path`.

**Liaison (Binding).** Une règle d'exécution qui applique une valeur externe (une variable de jeu, un curseur) via une
expression, un remap et un lissage à une propriété ou à une entrée de machine à états. Voir `binding`.

**Mode de fusion (Blend mode).** La façon dont la couleur d'un calque se combine avec ce qui se trouve en dessous (multiplication, écran, incrustation, etc.).
Les formules suivent W3C Compositing and Blending Level 1. Voir `raster`, `fraster`.

**Blend tree.** Un mélange 1D de plusieurs clips contrôlé par une entrée numérique, utilisé au sein d'une machine à états.
Voir `machine`.

**Blob.** Un cercle d'un calque liquide (metaball). Voir `fx`.

**Os (Bone).** Un segment nommé du squelette avec une position de tête, une longueur, un angle de repos et un parent
facultatif. Les os pilotent la géométrie avec skinning et les calques rattachés. Voir `rig` ; PROJECT_FORMAT section 4.

**Calque booléen (Boolean layer).** Un calque qui combine des tracés vectoriels par union, intersection, différence ou xor. Le
premier tracé est le sujet et les autres sont réunis pour former le clip. Voir `fx`.

**Rebond (interpolation) (Bounce (easing)).** Une courbe d'interpolation d'une balle qui tombe et rebondit, avec une restitution de 0,6 et trois
rebonds. Une dérivation originale à partir de la chute libre. Voir `anim`.

**Boîte englobante (Bounding box).** Le plus petit rectangle aligné sur les axes autour d'une géométrie ; utilisée pour l'alignement, la sélection et
l'ajustement des normal maps. Voir `layout`, `pick`.

**Déformateur cage (Cage deformer).** Un déformateur dans lequel le contenu situé à l'intérieur d'un polygone fermé suit les sommets du polygone
grâce aux coordonnées à valeur moyenne. Voir `cage` ; voir aussi MVC.

**Spline de Catmull-Rom (Catmull-Rom spline).** Une spline cubique qui passe par ses points de contrôle. L'interpolation `smooth` utilise
une version non uniforme passant par les clés voisines. Voir `anim`, lissage de `liquid` dans `fx`.

**Canal (Channel).** Une valeur animée : une cible, une propriété, une source (clés, expression, bruit ou piloté) et
un mode de fusion et un poids. Les canaux s'exécutent dans l'ordre de la liste. Voir PROJECT_FORMAT section 8.

**CIE76.** La différence de couleur d'origine, la distance euclidienne dans CIE L*a*b*. Peu coûteuse, moins précise
que CIEDE2000. Voir `percep`.

**CIEDE2000.** La formule de différence de couleur CIE 2000 (CIE TR 142-2001), implémentée telle qu'imprimée dans
Sharma, Wu et Dalal (2005) et vérifiée sur leurs paires de test publiées. Utilisée par la CI d'animation pour
juger d'un changement visible. Voir `percep`, `ci` ; `docs/CI.md`.

**Clean room.** La règle de développement selon laquelle le code est écrit à partir de normes publiques et de méthodes de manuel,
sans code copié et sans reproduction des formats ou des noms d'autres outils. Chaque technique est consignée dans
`CLEANROOM.md`.

**Clip.** Un take réutilisable de pistes d'animation qui peut être placé sur la timeline avec une vitesse, des boucles, un
poids, des fondus et un mode de fusion. Voir `clipstack` ; PROJECT_FORMAT section 11.1.

**Tissu (Cloth).** Un déformateur en grille dont les sommets se déplacent exactement de leurs propres décalages. Désigne aussi la simulation
de dynamique à base de positions qui fait le bake de ces décalages. Voir `cloth`.

**Contrainte (Constraint).** Une règle appliquée après l'IK qui limite un os : aim, copy rotation ou pin. Voir `rig`.

**Conteneur (mise en page) (Container (layout)).** Un groupe de mise en page unidimensionnel de type flexbox avec espacement, marge intérieure, justification,
alignement et retour à la ligne, qui positionne des calques et d'autres conteneurs. Voir `containers`.

**Contrainte copy-rotation (Copy-rotation constraint).** Une contrainte qui fait correspondre un os à la rotation globale d'un autre os. Voir
`rig`.

**CRDT.** Type de données répliqué sans conflit (conflict-free replicated data type) : une structure que les répliques peuvent modifier indépendamment et
fusionner vers le même résultat sans coordination. OSATURA utilise un CRDT fondé sur les opérations. Voir `collab` ;
ARCHITECTURE section 10. Termes liés : registre LWW, RGA, horloge de Lamport, horloge vectorielle, tombstone.

**Interpolation de Bézier cubique (Cubic Bezier easing).** Une courbe d'interpolation de style CSS définie par deux points de contrôle entre (0,0) et
(1,1). Voir `anim`.

**Déformateur (Deformer).** Un modificateur de forme appliqué à un calque vectoriel ou image dans l'espace de repos : lattice, spine, pin,
cage ou cloth. Voir PROJECT_FORMAT section 7.1.

**Triangulation de Delaunay (Delaunay triangulation).** Une triangulation de points dans laquelle aucun point ne se trouve à l'intérieur du cercle circonscrit d'un triangle,
ce qui évite les triangles minces. Construite avec l'algorithme de Bowyer-Watson pour les maillages d'image.
Voir `meshtrace`.

**Profondeur de champ (DoF) (Depth of field (DoF)).** Un flou qui augmente avec la distance entre le `parallax` d'un calque et le `focus`
de la caméra. Appliqué uniquement par les commandes `dof` et `dof-export`. Voir `dof`.

**Déterminisme (Determinism).** La propriété selon laquelle une même entrée donne une même sortie. L'évaluation d'une image est pure ;
le rendu est identique à l'octet près sur une même machine et un même build, sans vérification entre plateformes. Voir
ARCHITECTURE section 6.

**Dope sheet.** Une vue de la timeline qui montre les clés sous forme de marques par canal et les modifie dans le temps. Voir
`timeline`, `keyops`.

**Clé pilotée (Driven key).** Un canal dont l'entrée est la valeur d'une autre propriété plutôt que le temps, par exemple l'angle du coude
vers la taille d'un renflement. Voir `drivers` ; PROJECT_FORMAT section 8.1.

**Interpolation (Easing).** La fonction de mise en forme appliquée au segment qui commence à une clé. Voir `anim` ; PROJECT_FORMAT
section 8.4.

**Éditeur (Editor).** L'objet qui possède un projet et son historique d'annulation et qui applique les ops. Voir `edit` ;
ARCHITECTURE section 5.

**Émetteur (Emitter).** Un calque qui produit des particules comme fonction de forme close du temps. Voir `fx`.

**Moteur (Engine).** La forme validée, aux noms résolus, d'un projet qui évalue les images. Voir `engine`.

**Enveloppe (collab) (Envelope (collab)).** Le message qui transporte les ops entre les répliques : identifiant de réplique, numéro de séquence,
temps de Lamport, dépendances et opérations. Voir `collab`.

**Évaluation (Evaluation).** Le calcul de la pose, de la géométrie et de la caméra pour une image à partir d'un projet. Une fonction pure
de l'image. Voir PROJECT_FORMAT section 16.

**FABRIK.** Forward And Backward Reaching Inverse Kinematics : un solveur d'IK itératif qui tire alternativement la chaîne vers la cible depuis
l'extrémité, puis la réancre à la racine. Utilisé pour les chaînes de plus de deux os. Voir `rig`.

**FK.** Cinématique directe (forward kinematics) : calcul des transformations globales des os à partir des transformations de leurs parents et de
leurs propres angles et décalages. Voir `rig`, `engine`.

**Version du format (Format version).** Le champ entier `format` d'un projet, actuellement 1. Les fichiers plus récents sont rejetés.
Voir PROJECT_FORMAT section 19.

**Image (Frame).** (1) Un pas de temps de la timeline, `0..frames-1`. (2) La structure `Frame` : le résultat de
l'évaluation d'un pas (os, géométrie, caméra, indicateurs). Voir `engine`.

**Glue.** Un lien qui fait suivre à des sommets d'un maillage ceux d'un autre (accrochage, rigide ou avec décalage). Voir
`links`.

**Poignée de dégradé (Gradient handle).** Un point de contrôle d'un dégradé linéaire ou radial qui peut être animé avec
`grad_dx` et `grad_dy`. Voir PROJECT_FORMAT section 8.2.

**Calque de groupe (Group layer).** Un calque qui contient d'autres calques et leur transmet sa transformation. Voir `project`.

**Poignée (Handle).** Un point de contrôle de Bézier rattaché à une ancre. Coordonnées absolues dans le fichier. Voir `path`.

**HOG.** Histogramme de gradients orientés (histogram of oriented gradients), une caractéristique de forme locale (Dalal et Triggs 2005) utilisée par le détecteur
de visage. Voir `facedet`.

**IK.** Cinématique inverse (inverse kinematics) : calcul des angles des os pour que l'extrémité d'une chaîne atteigne une cible. Voir `rig`.
Termes liés : FABRIK, IK souple, IK étirable, flexion.

**Mélange IK (IK mix).** Le mélange entre la pose FK (0) et le résultat de l'IK (1) pour une chaîne. Animé avec `ik_mix`.
Voir PROJECT_FORMAT section 4.2.

**Maillage d'image (Image mesh).** Un maillage en grille ou en triangles libres sur un calque d'image, utilisé pour les poids d'os et les déformateurs.
Voir PROJECT_FORMAT section 5.8.

**Image clé (clé) (Keyframe (key)).** Un triplet (image, valeur, interpolation) sur une piste. L'interpolation façonne le segment qui commence
à la clé. Voir `anim`.

**L*a*b*.** L'espace colorimétrique CIE 1976 conçu pour que les distances approximent les différences perçues. Voir
`percep`.

**Horloge de Lamport (Lamport clock).** Un compteur logique qui ordonne les événements dans un système distribué sans horloge
partagée (Lamport 1978). Voir `collab`.

**Lattice.** Un déformateur avec une grille de points de contrôle sur un rectangle dont les décalages plient le contenu
par mélange de Bézier-Bernstein. Voir PROJECT_FORMAT section 7.1.

**Calque (Layer).** Un nœud de l'arbre de la scène : vectoriel, groupe, commutation, liquide, booléen, émetteur ou image. Voir
PROJECT_FORMAT section 5.

**Effet de calque (Layer effect).** Un filtre par calque (flou, ombre portée, teinte, remplissage) rendu sur un canevas hors écran.
Voir `layerfx`.

**Niveau de détail (LOD) (Level of detail (LOD)).** Une version simplifiée d'un projet produite sous une borne d'erreur déclarée
en pixels, pour un aperçu rapide ou une petite sortie. Voir `lod`, `osatura lod`.

**Linter.** Le vérificateur consultatif (`osatura lint`) qui signale des problèmes de qualité d'animation avec des codes
tels que L011 et L012. Distinct de la validation. Voir `lint`.

**Calque liquide (Liquid layer).** Un calque de metaballs dont le contour est l'isocontour d'un champ de somme d'influences. Voir
`fx`.

**Registre LWW (LWW register).** Registre last-writer-wins (le dernier écrivain gagne) : une cellule de CRDT où l'écriture dotée de l'horodatage logique le plus élevé survit. Utilisé pour les clés d'objet. Voir `collab`.

**Magnet.** Un outil de bake qui attire des positions vers un point avec une décroissance en smoothstep. Voir `magnet`.

**Marching squares.** Un algorithme qui extrait des lignes de contour d'un champ scalaire échantillonné sur une grille.
Utilisé pour les contours liquides et le tracé de bitmaps. Voir `fx`, `trace`, `meshtrace`.

**Marqueur (Marker).** Une image nommée ou une région d'images nommée sur la timeline. Les machines à états peuvent utiliser une région de marqueur comme
source. Voir `keyops` ; PROJECT_FORMAT section 10.2.

**Masque (Mask).** Un calque dont l'alpha limite la visibilité d'un autre calque ; il peut être inversé. Un calque masqué peut
quand même servir de masque. Voir `raster`.

**Fusion (CRDT) (Merge (CRDT)).** La combinaison des états de deux répliques pour que toutes deux convergent vers le même document. Voir
`collab`.

**Verrou de maillage (Mesh lock).** Un lien qui fait suivre à un maillage suiveur un maillage hôte avec une intensité 0..1. Voir `links`.

**Metaball.** Un blob dont l'influence de champ se mélange à celle de ses voisins, de sorte que des blobs proches fusionnent. Voir calque
liquide.

**Morph (blend shape).** Un jeu d'ancres alternatif pour un tracé, appliqué de façon additive avec un poids. Voir
PROJECT_FORMAT section 6.1.

**MVC.** Coordonnées à valeur moyenne (mean-value coordinates, Floater 2003) : coordonnées barycentriques généralisées pour un point à l'intérieur
d'un polygone, utilisées par le déformateur cage. Voir `cage`.

**Non destructif (Non-destructive).** Qui laisse le travail existant intact. Les bakers ajoutent des canaux au lieu de réécrire des clés, et
l'éditeur enregistre les ops inverses. Voir ARCHITECTURE sections 4 et 5.

**Normal map.** Une image qui stocke la direction de la surface en RVB, utilisée par l'effet `relight` et par
l'éclairage par calque. Voir `postfx` ; PROJECT_FORMAT section 7.3.

**Op.** Une modification exprimée sous la forme `set`, `insert`, `remove` ou `batch` sur un pointeur JSON RFC 6901 ; elle a un
inverse pour l'annulation. Voir `edit`.

**OT.** Transformation opérationnelle (operational transformation) : une alternative aux CRDT qui réécrit les opérations concurrentes les unes par rapport aux autres,
généralement avec un serveur central. OSATURA ne l'utilise pas ; il utilise un CRDT avec un séquenceur central. Voir
ARCHITECTURE section 10.

**Parallaxe (Parallax).** Un facteur par calque qui met à l'échelle le mouvement de la caméra, de sorte que les calques éloignés bougent moins. C'est aussi
la valeur de profondeur utilisée pour la profondeur de champ. Voir PROJECT_FORMAT section 10.1.

**Particule (Particle).** Un point à courte durée de vie émis par un émetteur ; sa position est une fonction de forme close du
temps. Voir `fx`.

**Chemin (cible) (Path (target)).** Le nom, séparé par des barres obliques, d'un calque imbriqué, tel que `parent/child` ; les sous-cibles utilisent
`::`, comme dans `layer::path::morph`. Voir PROJECT_FORMAT section 5.

**Tracé (vectoriel) (Path (vector)).** Une suite d'ancres, ouverte ou fermée, avec remplissage, contour et morphs facultatifs. Voir
`path`.

**Contrainte pin (Pin constraint).** Une contrainte qui fait glisser la tête d'un os vers la tête d'un autre os. Voir `rig`.

**Pin warp.** Un déformateur dans lequel des pins déplacés déplacent le dessin en douceur dans un rayon, à l'aide d'une fonction à base radiale
à support compact (Wendland 1995). Voir `pinwarp`.

**Player.** L'enveloppe d'exécution de la façade qui ajoute des machines à états et des liaisons à une animation.
Voir `api`.

**Preorder.** L'ordre de parcours de l'arbre des calques : un nœud, puis ses enfants. L'évaluation l'utilise pour que les
parents soient calculés en premier.

**Proxy.** Une image à détail réduit utilisée pendant le scrubbing, construite à partir d'un niveau de détail. Voir `scrub`, `lod`.

**QEM.** Métrique d'erreur quadrique (quadric error metric, Garland et Heckbert) : une mesure de l'erreur géométrique causée par
l'effondrement d'un sommet, utilisée pour la simplification de maillage. Voir `lod`.

**Raster.** Une image en pixels, ou le module qui rend une image en pixels avec tiny-skia. Voir `raster`.

**RDP.** Ramer-Douglas-Peucker : simplification récursive de ligne qui conserve les points situés à plus d'une tolérance
de la corde. Voir `trace`, `sizefit`.

**Calque de référence (Reference layer).** Un calque marqué `reference` ; masqué sauf si l'éditeur affiche les références. Voir
PROJECT_FORMAT section 5.1.

**Remap.** Une application affine d'une plage, avec une courbe et un écrêtage facultatifs, appliquée à la valeur d'une liaison.
Voir `binding`.

**Réplique (collab) (Replica (collab)).** La copie du document d'un participant dans le CRDT. Voir `collab`.

**Pose de repos (Rest pose).** Les transformations des os sans aucune animation appliquée ; les matrices de skin valent `world * inverse(rest
world)`. Voir `rig`.

**RGA.** Tableau croissant répliqué (replicated growable array, Roh et al. 2011) : un CRDT de séquence où chaque élément a un identifiant
unique, et où les suppressions laissent des tombstones. Utilisé pour les tableaux JSON. Voir `collab`.

**Rig.** Le squelette et ses auxiliaires : os, chaînes d'IK, contraintes et ressorts. Voir PROJECT_FORMAT
section 4.

**Scrub.** Faire glisser la tête de lecture ; le studio met en cache les images par révision et utilise des proxies. Voir `scrub`.

**Inclinaison (Skew).** Un angle de cisaillement dans la transformation d'un calque. Voir PROJECT_FORMAT section 5.1.

**Skin (animation).** Une variante nommée de calques, d'os, de contraintes ou d'IK qui peut être activée au sein d'un
groupe, comme des costumes. Voir `skins`.

**Skinning.** Déformation de la géométrie par mélange des transformations des os, pondéré par ancre ou par sommet.
OSATURA normalise les poids et traite les poids vides comme statiques. Voir `engine`, `weights`.

**IK souple (Soft IK).** Une option d'IK qui adoucit l'extension près de la portée maximale pour éviter un redressement brusque. Plage 0 à
0,9. Voir `rig`.

**SPDX.** Software Package Data Exchange : une façon normalisée d'indiquer la licence d'un fichier, comme dans
<!-- REUSE-IgnoreStart -->
`SPDX-License-Identifier: MPL-2.0`. Chaque fichier source en porte une. Voir `LICENSING.md`.
<!-- REUSE-IgnoreEnd -->

**Déformateur spine (Spine deformer).** Un déformateur qui plie la géométrie le long d'une courbe en projetant les ancres sur une courbe de
repos et en les rattachant à la courbe déformée. Voir PROJECT_FORMAT section 7.1.

**Ressort (os) (Spring (bone)).** Un ressort amorti du second ordre qui fait traîner et dépasser la rotation d'un os ; calculé en bake au
chargement pour que n'importe quelle image puisse être évaluée. Voir PROJECT_FORMAT section 4.4.

**SSIM.** Indice de similarité structurelle (structural similarity index, Wang et al. 2004) : une mesure de qualité d'image à partir des moyennes locales,
des variances et de la covariance. OSATURA utilise une version par blocs. Voir `percep`.

**Scène (Stage).** L'espace de coordonnées de l'image de sortie : pixels, +x vers la droite, +y vers le bas.

**Machine à états (State machine).** Un graphe d'états d'animation avec des transitions pilotées par des entrées. Il existe une version bakée,
avec journal d'événements, et une version d'exécution. Voir `statemachine`, `machine`.

**Interpolation en pas (Step easing).** Une interpolation qui maintient la valeur de la clé jusqu'à la clé suivante. Utilisée pour les calques de commutation. Voir
`anim`.

**IK étirable (Stretch IK).** Une option d'IK qui met la chaîne à l'échelle jusqu'à une limite (1..4) pour atteindre une cible éloignée. Voir
`rig`.

**Contour (Stroke).** Le pourtour d'un tracé : peinture, largeur, terminaison, jonction, tirets et un profil de largeur facultatif. Voir
PROJECT_FORMAT section 6.3.

**Calque de commutation (Switch layer).** Un calque qui affiche exactement un de ses enfants, choisi par un canal d'index. Utilisé pour l'animation image par image
et les formes de bouche. Voir PROJECT_FORMAT section 5.4.

**Take.** Un ensemble nommé de pistes qui forme le contenu d'un clip. Voir `clipstack`.

**Tombstone.** Un marqueur laissé dans un tableau CRDT à la place d'un élément supprimé pour que les opérations concurrentes puissent
toujours être placées. Le compactage supprime les tombstones que toutes les répliques ont vues. Voir `collab`.

**Piste (Track).** La liste des clés d'un canal, avec un mode d'extrapolation. Voir `anim`.

**Transformation (Transform).** Position, rotation, échelle, inclinaison et ancre d'un calque ; la matrice locale est
`T(position) R Shear S T(-anchor)`. Voir PROJECT_FORMAT section 5.1.

**Horloge vectorielle (Vector clock).** Un vecteur de compteurs par réplique qui indique si un événement s'est produit avant un autre ou
s'ils sont concurrents (Fidge, Mattern). Voir `collab`.

**Poids (os) (Weights (bone)).** L'influence de chaque os sur une ancre ou un sommet. Ils peuvent être peints ou calculés
par diffusion. Voir `weights`.

**Wobble.** Un déplacement par bruit déterministe ajouté à la géométrie d'un calque, avec amplitude, échelle, fréquence
et seed. Voir PROJECT_FORMAT section 5.1.

**Amincissement de Zhang-Suen (Zhang-Suen thinning).** Un algorithme parallèle qui réduit une forme binaire à un squelette d'un pixel. Utilisé
par autotrace pour le tracé de ligne médiane. Voir `autotrace`.
