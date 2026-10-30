<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->

# Glossario di OSATURA

Termini di dominio usati nel codice e nella documentazione, in ordine alfabetico (dei termini inglesi). "Vedi" rimanda
al modulo (in `osatura-core/src`, se non indicato altrimenti) o al documento in cui il termine è implementato o
specificato. Documenti: [PROJECT_FORMAT.md](PROJECT_FORMAT.md), [ARCHITECTURE.md](ARCHITECTURE.md).
Le fonti delle tecniche sono elencate in `CLEANROOM.md`.

**Vincolo aim (Aim constraint).** Un vincolo che ruota un osso in modo che il suo asse +x punti verso la testa di un altro osso,
più uno scostamento. Vedi `rig`; PROJECT_FORMAT sezione 4.3.

**Ancora (Anchor).** Un punto di un tracciato di Bézier con una posizione e maniglie opzionali di ingresso e di uscita; può anche
portare pesi degli ossi. Vedi `path`; PROJECT_FORMAT sezione 6.

**Limiti angolari (Angle limits).** Minimo e massimo per osso della rotazione animata, in gradi, applicati dopo
i canali e dopo ogni catena IK. Vedi `rig`, `engine`.

**Versione dell'API (API version).** La versione della superficie Rust stabile (`api` e `prelude`), attualmente 1.4.0,
controllata da un test di snapshot. Non coincide con il `format` del file né con l'`ABI_VERSION` del C. Vedi `api`.

**Asset.** Un file a cui fa riferimento un progetto (immagine, normal map, suono). I percorsi devono essere relativi e restare
all'interno della cartella del progetto, a meno che non venga indicato `--allow-external-assets`. Vedi `assets`; PROJECT_FORMAT
sezione 18.

**Bake.** Calcolare valori nel tempo e memorizzarli come normali canali o chiavi, in modo che non
serva più lo strumento che li calcola. Usato da fisica, clip, magnet, footlock, macchine a stati e altro. Vedi
ARCHITECTURE sezione 4.

**Bend.** Il segno (+1 o -1) che sceglie da che parte si piega una catena IK a due ossa, ad esempio gomito verso l'interno o l'esterno.
Vedi `rig`.

**Curva di Bézier (Bezier curve).** Una curva cubica parametrica definita da due ancore e dalle loro maniglie. Appiattita in segmenti
di retta (suddivisione di de Casteljau) per il riempimento e la misura. Vedi `path`.

**Binding.** Una regola a runtime che associa un valore esterno (una variabile di gioco, uno slider) tramite
un'espressione, un remap e un livellamento a una proprietà o a un input di una macchina a stati. Vedi `binding`.

**Modalità di fusione (Blend mode).** Il modo in cui il colore di un livello si combina con ciò che sta sotto (multiply, screen, overlay e
così via). Le formule seguono W3C Compositing and Blending Level 1. Vedi `raster`, `fraster`.

**Blend tree.** Una fusione monodimensionale di più clip controllata da un input numerico, usata all'interno di una macchina
a stati. Vedi `machine`.

**Blob.** Un cerchio di un livello liquido (metaball). Vedi `fx`.

**Osso (Bone).** Un segmento con nome dello scheletro, con una posizione della testa, una lunghezza, un angolo di riposo e un genitore
opzionale. Le ossa guidano la geometria con skinning e i livelli collegati. Vedi `rig`; PROJECT_FORMAT sezione 4.

**Livello booleano (Boolean layer).** Un livello che combina tracciati vettoriali per unione, intersezione, differenza o xor. Il
primo tracciato è il soggetto e gli altri sono uniti come clip. Vedi `fx`.

**Bounce (easing).** Una curva di easing di una palla che cade e rimbalza con restituzione 0,6 e tre
rimbalzi. Una derivazione originale dalla caduta libera. Vedi `anim`.

**Bounding box.** Il più piccolo rettangolo allineato agli assi che racchiude la geometria; usato per l'allineamento, la selezione e
l'adattamento delle normal map. Vedi `layout`, `pick`.

**Deformatore cage (Cage deformer).** Un deformatore in cui il contenuto all'interno di un poligono chiuso segue i vertici del poligono
usando le mean-value coordinates. Vedi `cage`; vedi anche MVC.

**Spline di Catmull-Rom (Catmull-Rom spline).** Una spline cubica che passa per i suoi punti di controllo. L'easing `smooth` usa
una versione non uniforme attraverso le chiavi vicine. Vedi `anim`, il livellamento `liquid` in `fx`.

**Canale (Channel).** Un valore animato: un bersaglio, una proprietà, una sorgente (chiavi, espressione, rumore o pilotata) e
una modalità di fusione e un peso. I canali vengono eseguiti nell'ordine dell'elenco. Vedi PROJECT_FORMAT sezione 8.

**CIE76.** La differenza di colore originale, la distanza euclidea in CIE L*a*b*. Economica, meno accurata
di CIEDE2000. Vedi `percep`.

**CIEDE2000.** La formula di differenza di colore CIE 2000 (CIE TR 142-2001), implementata come stampata in
Sharma, Wu e Dalal (2005) e verificata con le loro coppie di test pubblicate. Usata dalla CI dell'animazione per
giudicare la variazione visibile. Vedi `percep`, `ci`; `docs/CI.md`.

**Clean room.** La regola di sviluppo per cui il codice è scritto a partire da standard pubblici e metodi da manuale,
senza codice copiato e senza riprodurre i formati o i nomi di altri strumenti. Ogni tecnica è registrata in
`CLEANROOM.md`.

**Clip.** Un take riutilizzabile di tracce di animazione che può essere collocato sulla timeline con velocità, loop,
peso, dissolvenze e una modalità di fusione. Vedi `clipstack`; PROJECT_FORMAT sezione 11.1.

**Cloth.** Un deformatore a griglia i cui vertici si muovono esattamente secondo i propri scostamenti. Anche la simulazione
di position-based dynamics che esegue il bake di quegli scostamenti. Vedi `cloth`.

**Vincolo (Constraint).** Una regola applicata dopo l'IK che limita un osso: aim, copy rotation o pin. Vedi `rig`.

**Contenitore (layout) (Container).** Un gruppo di layout monodimensionale in stile flexbox con spaziatura, padding, giustificazione,
allineamento e a capo automatico, che posiziona livelli e altri contenitori. Vedi `containers`.

**Vincolo copy-rotation (Copy-rotation constraint).** Un vincolo che fa corrispondere un osso alla rotazione nel mondo di un altro osso. Vedi
`rig`.

**CRDT.** Conflict-free replicated data type: una struttura che le repliche possono modificare in modo indipendente e
fondere con lo stesso risultato senza coordinamento. OSATURA usa un CRDT basato su operazioni. Vedi `collab`;
ARCHITECTURE sezione 10. Correlati: registro LWW, RGA, clock di Lamport, vector clock, tombstone.

**Easing di Bézier cubico (Cubic Bezier easing).** Una curva di easing in stile CSS definita da due punti di controllo tra (0,0) e
(1,1). Vedi `anim`.

**Deformatore (Deformer).** Un modificatore di forma applicato a un livello vettoriale o immagine nello spazio di riposo: lattice, spine, pin,
cage o cloth. Vedi PROJECT_FORMAT sezione 7.1.

**Triangolazione di Delaunay (Delaunay triangulation).** Una triangolazione di punti in cui nessun punto cade all'interno della circonferenza circoscritta di alcun triangolo,
il che evita i triangoli sottili. Costruita con l'algoritmo di Bowyer-Watson per le mesh di immagini.
Vedi `meshtrace`.

**Profondità di campo (Depth of field, DoF).** Sfocatura che aumenta con la distanza del `parallax` di un livello dal `focus`
della camera. Applicata solo dai comandi `dof` e `dof-export`. Vedi `dof`.

**Determinismo (Determinism).** La proprietà per cui lo stesso input produce lo stesso output. La valutazione di un fotogramma è pura;
il rendering è identico byte per byte su una stessa macchina e build, non verificato tra piattaforme. Vedi
ARCHITECTURE sezione 6.

**Dope sheet.** Una vista della timeline che mostra le chiavi come segni per canale e le modifica nel tempo. Vedi
`timeline`, `keyops`.

**Chiave pilotata (Driven key).** Un canale il cui input è il valore di un'altra proprietà anziché il tempo, ad esempio dall'angolo del gomito
alla dimensione del rigonfiamento. Vedi `drivers`; PROJECT_FORMAT sezione 8.1.

**Easing.** La funzione di modellazione applicata al segmento che inizia in una chiave. Vedi `anim`; PROJECT_FORMAT
sezione 8.4.

**Editor.** L'oggetto che possiede un progetto e la sua cronologia di annulla e applica le op. Vedi `edit`;
ARCHITECTURE sezione 5.

**Emettitore (Emitter).** Un livello che produce particelle come funzione in forma chiusa del tempo. Vedi `fx`.

**Motore (Engine).** La forma validata e con i nomi risolti di un progetto che valuta i fotogrammi. Vedi `engine`.

**Envelope (collab).** Il messaggio che trasporta le op tra le repliche: id della replica, numero di sequenza,
tempo di Lamport, dipendenze e operazioni. Vedi `collab`.

**Valutazione (Evaluation).** Calcolare la posa, la geometria e la camera per un fotogramma a partire da un progetto. Una funzione pura
del fotogramma. Vedi PROJECT_FORMAT sezione 16.

**FABRIK.** Forward And Backward Reaching Inverse Kinematics: un solver IK iterativo che alternativamente
tira la catena verso il bersaglio dalla punta e poi la riancora alla radice. Usato per catene di
più di due ossa. Vedi `rig`.

**FK.** Cinematica diretta: calcolare le trasformazioni nel mondo delle ossa a partire dalle trasformazioni dei loro genitori e
dai loro angoli e scostamenti. Vedi `rig`, `engine`.

**Versione del formato (Format version).** Il campo intero `format` di un progetto, attualmente 1. I file più recenti vengono rifiutati.
Vedi PROJECT_FORMAT sezione 19.

**Fotogramma (Frame).** (1) Un passo temporale della timeline, `0..frames-1`. (2) La struct `Frame`: il risultato della
valutazione di un passo (ossa, geometria, camera, flag). Vedi `engine`.

**Glue.** Un collegamento che fa sì che i vertici di una mesh seguano i vertici di un'altra (snap, rigido o con scostamento). Vedi
`links`.

**Maniglia del gradiente (Gradient handle).** Un punto di controllo di un gradiente lineare o radiale che può essere animato con
`grad_dx` e `grad_dy`. Vedi PROJECT_FORMAT sezione 8.2.

**Livello gruppo (Group layer).** Un livello che contiene altri livelli e passa loro la propria trasformazione. Vedi `project`.

**Maniglia (Handle).** Un punto di controllo di Bézier collegato a un'ancora. Coordinate assolute nel file. Vedi `path`.

**HOG.** Histogram of oriented gradients, una caratteristica di forma locale (Dalal e Triggs 2005) usata dal rilevatore
di volti. Vedi `facedet`.

**IK.** Cinematica inversa: calcolare gli angoli delle ossa in modo che la punta di una catena raggiunga un bersaglio. Vedi `rig`.
Correlati: FABRIK, soft IK, stretch IK, bend.

**IK mix.** La fusione tra la posa FK (0) e il risultato IK (1) per una catena. Animata con `ik_mix`.
Vedi PROJECT_FORMAT sezione 4.2.

**Mesh di immagine (Image mesh).** Una mesh a griglia o a triangoli liberi su un livello immagine, usata per i pesi delle ossa e i deformatori.
Vedi PROJECT_FORMAT sezione 5.8.

**Chiave (keyframe) (Keyframe, key).** Una tripla (fotogramma, valore, easing) su una traccia. L'easing modella il segmento che inizia
alla chiave. Vedi `anim`.

**L*a*b*.** Spazio colore CIE 1976 progettato in modo che le distanze approssimino le differenze percepite. Vedi
`percep`.

**Clock di Lamport (Lamport clock).** Un contatore logico che ordina gli eventi in un sistema distribuito senza un orologio
condiviso (Lamport 1978). Vedi `collab`.

**Lattice.** Un deformatore con una griglia di punti di controllo su un rettangolo i cui scostamenti piegano il contenuto
tramite fusione di Bézier-Bernstein. Vedi PROJECT_FORMAT sezione 7.1.

**Livello (Layer).** Un nodo dell'albero della scena: vettoriale, gruppo, switch, liquido, booleano, emettitore o immagine. Vedi
PROJECT_FORMAT sezione 5.

**Effetto di livello (Layer effect).** Un filtro per livello (sfocatura, ombra proiettata, tinta, riempimento) renderizzato su un canvas fuori schermo.
Vedi `layerfx`.

**Livello di dettaglio (Level of detail, LOD).** Una versione semplificata di un progetto prodotta entro un limite d'errore dichiarato
in pixel, per un'anteprima rapida o un output piccolo. Vedi `lod`, `osatura lod`.

**Linter.** Il controllore consultivo (`osatura lint`) che segnala problemi di qualità dell'animazione con codici
come L011 e L012. Separato dalla validazione. Vedi `lint`.

**Livello liquido (Liquid layer).** Un livello di metaball il cui contorno è l'isocontorno di un campo a somma di influenze. Vedi
`fx`.

**Registro LWW (LWW register).** Last-writer-wins register: una cella CRDT in cui sopravvive la scrittura con il timestamp logico
più alto. Usato per le chiavi degli oggetti. Vedi `collab`.

**Magnet.** Uno strumento di bake che attira le posizioni verso un punto con decadimento smoothstep. Vedi `magnet`.

**Marching squares.** Un algoritmo che estrae linee di contorno da un campo scalare campionato su una griglia.
Usato per i contorni liquidi e il tracciamento di bitmap. Vedi `fx`, `trace`, `meshtrace`.

**Marcatore (Marker).** Un fotogramma o una regione di fotogrammi con nome sulla timeline. Le macchine a stati possono usare una regione di marcatori come
sorgente. Vedi `keyops`; PROJECT_FORMAT sezione 10.2.

**Maschera (Mask).** Un livello il cui alfa limita la visibilità di un altro livello; può essere invertita. Un livello nascosto può
comunque essere una maschera. Vedi `raster`.

**Merge (CRDT).** Combinare gli stati di due repliche in modo che entrambe convergano allo stesso documento. Vedi
`collab`.

**Mesh lock.** Un collegamento che fa sì che una mesh seguace segua una mesh ospite con una forza 0..1. Vedi `links`.

**Metaball.** Un blob la cui influenza di campo si fonde con quella dei vicini in modo che i blob vicini si uniscano. Vedi livello
liquido.

**Morph (blend shape).** Un insieme alternativo di ancore per un tracciato, applicato in modo additivo con un peso. Vedi
PROJECT_FORMAT sezione 6.1.

**MVC.** Mean-value coordinates (Floater 2003): coordinate baricentriche generalizzate per un punto all'interno di
un poligono, usate dal deformatore cage. Vedi `cage`.

**Non distruttivo (Non-destructive).** Che lascia intatto il lavoro esistente. I baker aggiungono canali anziché riscrivere le chiavi, e
l'editor registra le op inverse. Vedi ARCHITECTURE sezioni 4 e 5.

**Normal map.** Un'immagine che memorizza la direzione della superficie come RGB, usata dall'effetto `relight` e
dall'illuminazione per livello. Vedi `postfx`; PROJECT_FORMAT sezione 7.3.

**Op.** Una modifica espressa come `set`, `insert`, `remove` o `batch` su un puntatore JSON RFC 6901; ha un
inverso per l'annulla. Vedi `edit`.

**OT.** Operational transformation: un'alternativa ai CRDT che riscrive le operazioni concorrenti le une rispetto alle altre,
di solito con un server centrale. OSATURA non la usa; usa un CRDT con un sequenziatore centrale. Vedi ARCHITECTURE sezione 10.

**Parallasse (Parallax).** Un fattore per livello che scala il movimento della camera, in modo che i livelli distanti si muovano meno. Anche il
valore di profondità usato per la profondità di campo. Vedi PROJECT_FORMAT sezione 10.1.

**Particella (Particle).** Un punto di breve durata emesso da un emettitore; la sua posizione è una funzione in forma chiusa del
tempo. Vedi `fx`.

**Percorso (bersaglio) (Path, target).** Il nome separato da barre di un livello annidato, come `parent/child`; i sotto-bersagli usano
`::` come in `layer::path::morph`. Vedi PROJECT_FORMAT sezione 5.

**Tracciato (vettoriale) (Path, vector).** Una sequenza di ancore, aperta o chiusa, con riempimento, contorno e morph opzionali. Vedi
`path`.

**Vincolo pin (Pin constraint).** Un vincolo che fa scorrere la testa di un osso fino alla testa di un altro osso. Vedi `rig`.

**Pin warp.** Un deformatore in cui i pin trascinati muovono la grafica in modo uniforme entro un raggio, usando una
funzione a base radiale a supporto compatto (Wendland 1995). Vedi `pinwarp`.

**Player.** Il wrapper runtime nella facciata che aggiunge macchine a stati e binding a un'animazione.
Vedi `api`.

**Preorder.** L'ordine di visita dell'albero dei livelli: un nodo, poi i suoi figli. La valutazione lo usa in modo che
i genitori vengano calcolati per primi.

**Proxy.** Un'immagine a dettaglio ridotto usata durante lo scrubbing, costruita a partire da un livello di dettaglio. Vedi `scrub`, `lod`.

**QEM.** Quadric error metric (Garland e Heckbert): una misura dell'errore geometrico causato dal
collasso di un vertice, usata per la semplificazione delle mesh. Vedi `lod`.

**Raster.** Un'immagine di pixel, o il modulo che renderizza un fotogramma in pixel con tiny-skia. Vedi `raster`.

**RDP.** Ramer-Douglas-Peucker: semplificazione ricorsiva di linee che mantiene i punti più lontani di una tolleranza
dalla corda. Vedi `trace`, `sizefit`.

**Livello di riferimento (Reference layer).** Un livello contrassegnato come `reference`; nascosto a meno che l'editor mostri i riferimenti. Vedi
PROJECT_FORMAT sezione 5.1.

**Remap.** Una mappatura affine di un intervallo, con una curva e un clamp opzionali, applicata al valore di un binding.
Vedi `binding`.

**Replica (collab).** La copia del documento di un partecipante nel CRDT. Vedi `collab`.

**Posa di riposo (Rest pose).** Le trasformazioni delle ossa senza alcuna animazione applicata; le matrici di skin sono `world * inverse(rest
world)`. Vedi `rig`.

**RGA.** Replicated growable array (Roh et al. 2011): un CRDT di sequenze in cui ogni elemento ha un id univoco,
e le cancellazioni lasciano tombstone. Usato per gli array JSON. Vedi `collab`.

**Rig.** Lo scheletro e i suoi ausiliari: ossa, catene IK, vincoli e molle. Vedi PROJECT_FORMAT
sezione 4.

**Scrub.** Trascinare la testina di riproduzione; lo studio mette in cache i fotogrammi per revisione e usa i proxy. Vedi `scrub`.

**Skew.** Un angolo di distorsione (shear) nella trasformazione di un livello. Vedi PROJECT_FORMAT sezione 5.1.

**Skin (animazione) (Skin, animation).** Una variante con nome di livelli, ossa, vincoli o IK che può essere attivata in un
gruppo, come i costumi. Vedi `skins`.

**Skinning.** Deformare la geometria fondendo le trasformazioni delle ossa, con pesi per ancora o vertice.
OSATURA normalizza i pesi e tratta i pesi vuoti come statici. Vedi `engine`, `weights`.

**Soft IK.** Un'opzione IK che addolcisce l'estensione vicino alla portata massima per evitare di scattare in posizione dritta. Intervallo da 0 a
0,9. Vedi `rig`.

**SPDX.** Software Package Data Exchange: un modo standard per dichiarare la licenza di un file, come in
<!-- REUSE-IgnoreStart -->
`SPDX-License-Identifier: MPL-2.0`. Ogni file sorgente ne porta una. Vedi `LICENSING.md`.
<!-- REUSE-IgnoreEnd -->

**Deformatore spine (Spine deformer).** Un deformatore che piega la geometria lungo una curva proiettando le ancore su una curva
di riposo e riagganciandole alla curva deformata. Vedi PROJECT_FORMAT sezione 7.1.

**Molla (osso) (Spring, bone).** Una molla smorzata del secondo ordine che fa ritardare e superare il bersaglio alla rotazione di un osso; ne viene fatto il bake al
caricamento, così qualsiasi fotogramma può essere valutato. Vedi PROJECT_FORMAT sezione 4.4.

**SSIM.** Structural similarity index (Wang et al. 2004): una misura di qualità dell'immagine ricavata da medie locali,
varianze e covarianza. OSATURA usa una versione a blocchi. Vedi `percep`.

**Stage.** Lo spazio di coordinate dell'immagine di output: pixel, +x a destra, +y in basso.

**Macchina a stati (State machine).** Un grafo di stati di animazione con transizioni guidate da input. Esiste una versione con bake,
basata su log di eventi, e una versione runtime. Vedi `statemachine`, `machine`.

**Easing step (Step easing).** Un easing che mantiene il valore della chiave fino alla chiave successiva. Usato per i livelli switch. Vedi
`anim`.

**Stretch IK.** Un'opzione IK che scala la catena fino a un limite (1..4) per raggiungere un bersaglio lontano. Vedi
`rig`.

**Contorno (Stroke).** Il profilo di un tracciato: vernice, larghezza, estremità, giunzione, tratteggio e un profilo di larghezza opzionale. Vedi
PROJECT_FORMAT sezione 6.3.

**Livello switch (Switch layer).** Un livello che mostra esattamente uno dei suoi figli, scelto da un canale indice. Usato per
l'animazione fotogramma per fotogramma e le forme della bocca. Vedi PROJECT_FORMAT sezione 5.4.

**Take.** Un insieme con nome di tracce che forma il contenuto di una clip. Vedi `clipstack`.

**Tombstone.** Un marcatore lasciato in un array CRDT al posto di un elemento cancellato, in modo che le operazioni concorrenti possano
comunque essere collocate. La compattazione rimuove i tombstone che ogni replica ha visto. Vedi `collab`.

**Traccia (Track).** L'elenco delle chiavi di un canale, con una modalità di estrapolazione. Vedi `anim`.

**Trasformazione (Transform).** Posizione, rotazione, scala, skew e ancora di un livello; la matrice locale è
`T(position) R Shear S T(-anchor)`. Vedi PROJECT_FORMAT sezione 5.1.

**Vector clock.** Un vettore di contatori per replica che dice se un evento è avvenuto prima di un altro o se
sono concorrenti (Fidge, Mattern). Vedi `collab`.

**Pesi (osso) (Weights, bone).** L'influenza di ciascun osso su un'ancora o un vertice. Possono essere dipinti o calcolati
per diffusione. Vedi `weights`.

**Wobble.** Uno spostamento di rumore deterministico aggiunto alla geometria di un livello, con ampiezza, scala, frequenza
e seed. Vedi PROJECT_FORMAT sezione 5.1.

**Assottigliamento di Zhang-Suen (Zhang-Suen thinning).** Un algoritmo parallelo che riduce una forma binaria a uno scheletro di un pixel. Usato
da autotrace per il tracciamento della linea centrale. Vedi `autotrace`.
