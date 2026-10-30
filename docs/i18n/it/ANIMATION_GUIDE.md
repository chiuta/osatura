# Guida all'animazione di OSATURA

Come far muovere le cose, riutilizzare il movimento, pilotarlo da audio, volti, mocap e input dal vivo, verificarlo e
distribuirlo in formato compatto. Ogni comando qui riportato è stato eseguito con il binario reale (`osatura 0.1.0`, build di debug, Linux);
gli output sono copiati da quelle esecuzioni. Il rigging si trova in [RIGGING_GUIDE.md](RIGGING_GUIDE.md); il primo progetto in
[GETTING_STARTED.md](GETTING_STARTED.md); ogni flag in [CLI_REFERENCE.md](CLI_REFERENCE.md).

Licenza di questo documento: CC-BY-4.0.

Gli esempi usano il bipede a 11 ossa (`biped.json`, 400x360, 24 fps, 24 fotogrammi) della guida al rigging: rettangoli
e cerchi separati attaccati alle ossa, un livello switch `mouth` con cinque figli, costruito con una sola chiamata a `script`.
Funziona qualsiasi rig con ossa dai nomi simili.

![Sei fotogrammi (0, 4, 8, 12, 16, 20) del ciclo di camminata generato di un semplice bipede fatto di rettangoli: le gambe oscillano in opposizione, le braccia in controfase, il bacino sale e scende.](../../img/walk-cycle.png)

## 1. Canali e chiavi

Tutto ciò che si muove è un **canale**: un target (osso, livello, camera, punto di un deformatore), una proprietà, una
modalità e una sorgente.

```
{"target":"hand_ctrl","prop":"bone_offset_y","source":"keys",
 "track":{"keys":[{"frame":0,"value":0,"ease":{"type":"ease_in_out"}}, ...],"extrapolate":"hold"},
 "mode":"replace","weight":1.0}
```

* **Sorgenti:** `keys` (chiavi), `expr` (un'espressione), `noise` (rumore frattale di valori in `[-amp, amp]`, con `freq` in
  cicli al secondo) e `driven` (una curva che mappa il valore di un'altra proprietà su questa). I comandi che eseguono il bake
  (physics, walk, clip, lip-sync...) scrivono normali `keys`. `osatura timeline` mostra la sorgente di ogni traccia:
  `#1 bone_offset_y (noise)`, `#0 bone_angle (expr)`, `#0 layer_switch (driven)`, oppure l'elenco dei fotogrammi chiave.
* **Modalità:** `replace`, `add` (sopra a ciò che c'è sotto) e `multiply`. I canali si combinano nell'ordine della lista.
* **Extrapolate** (cosa fa una traccia fuori dalla sua prima e dalla sua ultima chiave): `hold` (predefinito), `linear` (continua la
  pendenza finale), `loop`, `ping_pong` e `loop_offset` (ripete, aggiungendo a ogni ciclo la variazione tra la prima e l'ultima chiave, cosa che si adatta a un
  offset x di camminata). Impostalo con `keyedit ... extrapolate --channels 0 --extrapolate loop_offset`; un nome sbagliato elenca quelli validi.
* Le proprietà delle ossa sono **delta rispetto alla posa di riposo**: `bone_angle`, `bone_offset_x`, `bone_offset_y`,
  `bone_scale`. Le proprietà dei livelli comprendono `layer_x`, `layer_y`, `layer_rotation`, `layer_scale_x`, `layer_scale_y`,
  `layer_opacity` e `layer_switch`. I deformatori usano `cage_dx`... come nella guida al rigging.

Crea le chiavi da uno script (`set_key(target, prop, frame, value[, ease])`), dalla CLI con `keyedit add`, dallo
studio oppure nel JSON.

```
$ osatura keyedit walk.json k3.json add --target torso --prop bone_angle --at 12 --value 6 --ease ease_in_out
add: 1 keys changed, wrote k3.json
```

`osatura timeline <project> [--json]` elenca ogni traccia per osso/livello con i suoi fotogrammi chiave; `osatura explain <project> <bone|layer> <frame>`
mostra come è stato calcolato un singolo fotogramma.

## 2. Interpolazione ed easing

Ogni chiave memorizza l'ease del segmento che **inizia** in quella chiave. I nomi di easing che non richiedono parametri sono
`linear`, `step` (mantiene il valore fino alla chiave successiva), `ease_in`, `ease_out`, `ease_in_out`, `smooth` (una spline
Catmull-Rom attraverso le chiavi vicine: morbida, senza controllo dell'overshoot) e `bounce`. Nel JSON un ease è un oggetto:

```
{"type":"ease_in_out"}
{"type":"cubic_bezier","x1":0.25,"y1":0.1,"x2":0.25,"y2":1}    // CSS-style curve through (0,0) and (1,1)
{"type":"spring","bounces":3,"decay":4}                          // damped-spring settle that lands exactly on the target
{"type":"back","overshoot":0.6}                                  // ease out with overshoot
```

`keyedit ... ease --sel SEL --ease E` accetta un nome semplice, `bezier:x1,y1,x2,y2` (memorizzato come `cubic_bezier`) o un oggetto JSON per
`spring` e `back`; tutte e quattro le forme sono state applicate alle 17 chiavi dell'anca del ciclo di camminata senza errori. Nella CLI del dope sheet la
sintassi di selezione `SEL` è `target:prop@first..last`, `target:prop` oppure `all`.

```
$ osatura keyedit walk.json k2.json ease --sel all --ease smooth
ease: 119 keys changed, wrote k2.json
```

Aggiungi `-` come output per trasformare qualsiasi `keyedit` in una **prova a vuoto** che stampa il piano come JSON senza scrivere nulla:

```
$ osatura keyedit walk.json - move --sel 'thigh_l:bone_angle@0..6' --dt 2
{ "op": { "op": "batch", "ops": [ { "op": "set", "path": "/channels/0/track", "value": { "extrapolate": "loop", "keys": [ ... ] } } ] }, ... }
```

**Insidia.** `keyedit` rifiuta le modifiche che lascerebbero l'intervallo di fotogrammi del progetto, a meno che tu non passi `--free`:
`scale: frame 27 is outside the project range 0..=24` (lo stesso comando con `--free` ha stampato `scale: 17 keys changed`, e le
chiavi in più si trovano poi oltre l'ultimo fotogramma, dove `lint` segnala `L007`). Vengono rifiutati anche i canali bloccati:
`channel hand_ctrl.BoneOffsetY is locked` (vedi `chan-meta ... lock=on`).

Il graph editor dello studio disegna le stesse curve, con tangenti trascinabili (`handle`, `smooth`; vedi
[STUDIO.md](STUDIO.md)). Le chiavi hanno maniglie tangenti di ingresso/uscita facoltative (`free`, `aligned` o `auto`).

## 3. Il dope sheet dalla riga di comando: `keyedit`

Tutto ciò che può fare la timeline dello studio esiste come sottocomando di `keyedit`, applicato come un unico batch sicuro per l'annullamento:

| Sottocomando | Effetto |
|---|---|
| `move --sel SEL [--dt N] [--dv N] [--collision replace\|merge\|refuse] [--no-snap] [--free]` | sposta le chiavi nel tempo e/o nel valore |
| `scale --sel SEL --pivot F --factor X` | cambia la temporizzazione attorno a un fotogramma |
| `dup --sel SEL --offset N` | duplica |
| `delete --sel SEL` | elimina |
| `ease --sel SEL --ease E` / `value --sel SEL --value V` / `smooth --sel SEL [--kind monotone\|catmull_rom]` | imposta ease, valore o auto-smooth |
| `copy --sel SEL --clip clip.json` / `paste --clip clip.json --at F [--remap a:p=b:q]` | appunti tra progetti |
| `add --target T --prop P --at F [--value V] [--ease E]` | inserisce una chiave |
| `ripple --at F --by N --mode insert\|delete [--scope 0,1] [--adjust-frames]` | inserisce o rimuove tempo |
| `offset --channels 0,1 --dt N` | fa scorrere interi canali |
| `extrapolate --channels 0,1 --extrapolate loop` / `loop --channels 0,1 [--end F]` | ripete dopo l'ultima chiave |
| `handle --sel SEL --side in\|out --t F --v V [--mode free\|aligned\|auto]` | maniglia tangente del grafico |
| `lock --target T [--prop P] [--value on\|off]` | blocca un canale |

## 4. Loop

Un progetto di N fotogrammi riproduce ripetutamente i fotogrammi da 0 a N-1, quindi **il fotogramma N deve essere la stessa posa del fotogramma 0 e non viene mai mostrato
due volte**. Tre comandi aiutano:

* `keyedit ... loop --channels 0,1` imposta `extrapolate: loop` sui canali (le chiavi restano intatte):

```
$ osatura keyedit s4.json s6.json loop --channels 0,1
loop: 2 keys changed, wrote s6.json
```

* `loop-solve <in> <out> [--window 24] [--inclusive]` rende i canali con chiavi perfettamente ciclici e riporta la giunzione:

```
$ osatura loop-solve 04-keys.json buddy-final.json
wrote buddy-final.json: 2 channel(s) adjusted, seam 0.000 -> 0.000
```

* `make-loop <in> <out> [targets|all] [props] [--inclusive]` **rimuove lo spostamento** in modo che il movimento funzioni sul posto:

```
$ osatura make-loop t1.json t2.json
ball LayerX: removed drift 280.00
wrote t2.json
```

Quella palla si spostava di 280 unità verso destra in 23 fotogrammi. `make-loop` l'ha trasformata in un loop perfetto sul posto... di una palla che non si
muove più (a quel punto lint segnala `Info L005 [ball:LayerX] all keys have the same value; the channel never changes anything`).
Questo è il significato di «rimuove lo spostamento»: usalo per cicli di camminata che avvengono sul posto, poi sposta l'intero personaggio con un
percorso `follow` o con una camera (sezione 9). `loop-solve` non corregge lo spostamento: sulla stessa palla riporta `seam 280.000 -> 280.000`.
Se invece vuoi che lo spostamento **continui** attraverso i cicli, dai al canale l'extrapolate `loop_offset` (sezione 1): `explain` riporta allora
`keys → 121.739 (2 key(s), LoopOffset outside range)` per la palla al fotogramma 10.

`--inclusive` seleziona la convenzione più vecchia, in cui l'ultimo fotogramma mostrato coincide con il primo.

**Insidia: il noise non fa loop.** Un canale `noise` non è periodico. Per un'oscillazione ciclica usa un'espressione di `t`
(0..1 sull'intera clip) come `8 * sin(t * 2 * pi)`, oppure interrompi il loop di proposito.

## 5. Espressioni, noise e driver

Un canale **espressione** valuta una piccola formula a ogni fotogramma. Variabili: `frame`, `time` (secondi), `fps`, `frames`,
`pi` e **`t` = 0..1 sulla clip**. Operatori `+ - * / % ^`, confronti, `&& || !`, `?:`. Tra le funzioni ci sono
`sin`, `abs`, `min`, `clamp(x, lo, hi)`, `lerp(a, b, t)`, `wiggle(...)` (noise; due o tre argomenti, per esempio `wiggle(3, 5, 7)`)
e `get("target","prop")`, che legge il valore corrente di un altro canale (i canali vengono eseguiti nell'ordine della lista e quelli successivi vedono i risultati dei precedenti).

```
cat > ex.rhai <<'EOF'
add_channel_expr("head", "bone_angle", "8 * sin(t * 2 * pi)", "add");
add_noise("hips", "bone_offset_y", 1.5, 2.0, 7, "add");
EOF
osatura script biped.json ex1.json ex.rhai
```
```
2 request(s) applied; wrote ex1.json
```

Risultato di `explain ex1.json head <frame>`: l'angolo mondo della testa è -90.0 ai fotogrammi 0, 12 e 24, **-82.0 al fotogramma 6** e
-98.0 al fotogramma 18 (un'intera sinusoide sulla clip; l'angolo di riposo della testa è -90 gradi mondo). Il noise sul bacino
sposta `hips` di una frazione di unità: la sua testa si trova a y 169.7 al fotogramma 0 e a 170.5 al fotogramma 24 (170.0 senza il noise), quindi non fa loop (vedi sopra).

Gli errori vengono intercettati prima che venga scritto qualsiasi cosa:

```
$ osatura script biped.json ex2.json ex2.rhai       # "8 * sin(tt * 2 * pi)"
request 1 (channel head.BoneAngle) was rejected, nothing changed: channel 0 ('head'): expression error: unknown variable 'tt'
$ osatura script biped.json ex3.json ex3.rhai       # "8 * sin("
request 1 (channel head.BoneAngle) was rejected, nothing changed: channel 0 ('head'): expression error: unexpected end of expression
```

**Insidia.** In un'espressione `t` è normalizzato (da 0 a 1), non sono secondi né fotogrammi. `sin(t * 6.2832 / 24)` sembra giusto
ed è sbagliato: si muove appena. Usa `time` per i secondi oppure `frame`.

### Driver, slider e data binding

* `slider` e `view-swap` (guida al rigging) creano ossa controller più canali `driven`.
* `bind` collega un percorso di valore esterno (da un gioco, un'interfaccia o un test) a una proprietà o a un input di macchina a stati, con remap,
  smoothing e un'espressione facoltativa:

```
$ osatura bind add m1.json b1.json look player.look head:bone_angle --remap 0,1,-20,20 --smooth 0.1
added binding 'look' (1 bindings); wrote b1.json
$ osatura bind add b1.json b2.json run player.running @hero:walking
$ osatura bind list b2.json
look  player.look -> head:BoneAngle remap 0..1 -> -20..20 clamp smooth 0.1s
run  player.running -> @hero:walking
$ echo '{"player.look":0.75,"player.running":0}' > vals.json
$ osatura bind-eval b2.json vals.json
look = 10
run = 0
```

`0.75` sull'intervallo 0..1 mappato su -20..20 dà 10, come mostrato. `bind-eval --data out.json` scrive gli stessi numeri come JSON.

## 6. Cicli di camminata e libreria di movimenti

`walk` genera una camminata procedurale: richiede i nomi delle ossa per le gambe e (facoltativamente) per braccia e bacino.

```
$ osatura walk biped.json walk.json --thigh-l thigh_l --thigh-r thigh_r --shin-l shin_l --shin-r shin_r \
    --arm-l arm_upper_l --arm-r arm_upper_r --hips hips --period 24 --stride 28 --knee 40 --bob 4 --swing 25
wrote walk.json (+7 channels)
```

`--period` è la lunghezza del ciclo in fotogrammi, `--stride` l'oscillazione della coscia in gradi, `--knee` la piegatura del ginocchio, `--bob`
il movimento verticale del bacino in pixel e `--swing` l'oscillazione delle braccia. I sette canali hanno una chiave ogni 1,5 fotogrammi, fanno loop
(`extrapolate: loop`) e superano `lint` con 0 segnalazioni. Poiché il ciclo è generato sul posto, non c'è alcuno spostamento
da rimuovere.

### Take e clip

Un **take** è un pezzo di animazione registrato e memorizzato in un file di **libreria** che puoi riutilizzare tra i progetti:

```
$ osatura take-record walk.json lib.json walk 0 24 thigh,shin,arm,hips
recorded 7 tracks into lib.json
$ osatura take-list lib.json
walk (24 frames, 7 tracks)
$ osatura take-apply biped.json ta.json lib.json walk 0
applied 7 tracks; wrote ta.json
```

Gli ultimi argomenti di `take-record` sono prefissi di nomi di target da includere. `take-apply` scrive il take a partire da un fotogramma,
con rinominazioni facoltative `from=to`, una `speed` e `--blend FRAMES` per fare una dissolvenza incrociata con l'animazione esistente.

### Lo stack di clip: livelli e pesi

Una **clip** è un take posto sulla timeline con una modalità, un peso e maschere. Le clip si sovrappongono in ordine; quelle successive stanno sopra
alle precedenti. Esempio: una camminata, più un braccio destro che saluta con dissolvenza in entrata e in uscita.

```
$ osatura clip add biped.json c1.json lib.json walk walkclip --at 0 --loops 1 --mode replace --weight 1
added clip 'walkclip' (1 clips); run clip bake to generate channels
$ osatura clip add c1.json c2.json lib.json wave waveclip --at 0 --mode replace --weight 1 \
    --include 'arm_*_r' --fade-in 4 --fade-out 4
$ osatura clip list c2.json
1. walkclip take 'walk' frames 0..24 x1 speed 1 Replace weight 1
2. waveclip take 'wave' frames 0..24 x1 speed 1 Replace weight 1 include arm_*_r
$ osatura clip bake c2.json c3.json
baked 2 clips into 18 channels (0 replaced); wrote c3.json
$ osatura clip set-weight c3.json c4.json waveclip --keys 0:0,6:1,18:1,24:0
updated clip 'waveclip'; run clip bake to update channels
```

Lo stack non tocca mai i canali creati a mano né i canali bloccati. `--include`/`--exclude` accettano pattern di nomi,
`--track-weight arm_*=1,leg_*=0.2` pesa le tracce singolarmente, `--mode additive|multiply` sovrappone invece di sostituire.

**Insidia.** È `clip bake` a scrivere i canali: dopo aver modificato le clip eseguilo di nuovo (il comando lo dice).

## 7. Macchine a stati e binding

Una **macchina a stati** sceglie tra le clip a runtime in base agli input (booleani, numeri, trigger). È descritta in JSON e
viene verificata al momento dell'aggiunta:

```
{"name":"hero","inputs":[{"name":"greet","kind":"trigger"},{"name":"walking","kind":"bool","default":1}],
 "layers":[{"name":"base","entry":"walk",
  "states":[{"name":"walk","source":{"kind":"clip","clip":"walkclip"},"mode":"loop"},
            {"name":"wave","source":{"kind":"clip","clip":"waveclip"},"mode":"once","blend_in":0.15}],
  "transitions":[{"from":"walk","to":"wave","when":[{"input":"greet"}],"duration":0.15},
                 {"from":"wave","to":"walk","exit_time":1.0,"duration":0.2}]}]}
```

Gli stati riproducono una clip, un `segment` di fotogrammi, una `region` di marker o un `blend` 1D di più sorgenti; le transizioni scattano quando tutte le
condizioni `when` (`input op value`, AND) sono vere, eventualmente solo dopo `exit_time` (da 0 a 1 della lunghezza dello stato); vince la prima
transizione corrispondente in ordine; `"from":"*"` significa qualsiasi stato.

```
$ osatura machine add c2.json m1.json machine.json
added machine 'hero' (1 machines); wrote m1.json
$ osatura machine validate m1.json
1 machines, 0 bindings: ok
$ echo '{"duration":3.0,"events":[{"t":1.0,"trigger":["greet"]}]}' > script.json
$ osatura machine-sim m1.json script.json simout --fps 12
   0.000s  base     Entered walk
   1.000s  base     Exited walk
   1.000s  base     Entered wave
   2.000s  base     Exited wave
   2.000s  base     Entered walk
37 ticks at 12 fps, 5 events, 12 svg frames; wrote simout/sim.json
```

`machine-sim` riproduce eventi di input temporizzati e scrive un log degli stati (`sim.json`) più fotogrammi SVG, così le macchine a stati sono testabili
e confrontabili con diff senza un runtime. Una macchina a stati è una funzione pura del suo log di input: lo stesso script dà sempre lo stesso
risultato.

## 8. Lip-sync e parlato

Tutti gli input audio devono essere **WAV**; converti prima gli altri formati con ffmpeg. Un audio più lungo del progetto viene **troncato** alla
lunghezza del progetto; allunga prima il progetto (`frames` nel file), altrimenti ottieni meno fotogrammi di quanti ne abbia il suono.

```
$ osatura lipsync biped.json speech.wav mouth ls.json
wrote ls.json (3.00s of audio → 24 frames)       # the project has 24 frames = 1 s: the last 2 s are cut
```

Su una copia del progetto di 72 fotogrammi (3 s):

```
$ osatura lipsync talk0.json speech.wav mouth ls.json
wrote ls.json (3.00s of audio → 72 frames)
$ osatura lipsync-text talk0.json speech.wav transcript.txt mouth lt.json --lang en --words-out words.json
wrote lt.json (7 words, 18 exposure changes; energy + text alignment, not phoneme recognition)
$ osatura speech-motion lt.json speech.wav transcript.txt sm.json --head head --nod 4 --fall 3 --tilt 2 --seed 5 --lang en
wrote sm.json (1 additive channel(s) from 7 words)
```

* `lipsync` sceglie le forme della bocca dalla sola **sonorità** dell'audio.
* `lipsync-text` allinea la tua trascrizione (`Hello there, nice to meet you today`) all'energia dell'audio e usa il testo per
  le forme. **Non è riconoscimento dei fonemi**; lo strumento lo dichiara ogni volta. `--words-out` scrive i tempi di parole e sillabe
  come JSON o SRT: `00:00:00,220 --> 00:00:02,690  Hello there, nice to meet you today`. Lingue: `--lang ro|en`.
* `speech-motion` aggiunge cenni del capo, abbassamenti e inclinazioni temporizzati sulle parole (additivo).
* L'ordine delle forme è rest, small, open, round, wide (`--map` rimappa sul tuo switch).

### Cue creati a mano: `mouth`

Se hai cue provenienti da un altro strumento, `mouth <project> <cues.txt> <switch_layer> <out>` accetta sia righe
**numerate per fotogramma** (stile switch di Moho: `frame PHONEME`) sia **secondi** (TSV di Rhubarb): un tempo **scritto con il punto decimale
è in secondi, un intero semplice è un fotogramma**.

```
printf '0.00 X\n0.20 C\n0.35 B\n0.60 D\n1.00 A\n' > cues.txt
osatura mouth talk0.json cues.txt mouth mc.json                 # wrote mc.json (5 cues)
osatura mouth talk0.json cues.txt mouth mc.json --rhubarb       # also wrote mc.json (5 cues)
```

Le lettere delle forme corrispondono alle cinque classi di bocca: `X A MBP rest` a rest; `B G H L FV etc` (e le lettere sconosciute) a small; `D AI` a open;
`F O U WQ` a round; `C E` a wide. `--rhubarb` cambia una sola cosa: la lettera `E` viene allora trattata come la forma arrotondata di Rhubarb (round) anziché come quella larga di
Papagayo (wide). Qui `X C B D A` ha dato rest, wide, small, open, rest.

**Insidia.** `0.00 X` è il secondo 0; `0 X` sarebbe il fotogramma 0. Mescolarli in un unico file è lecito e fonte di confusione.

### Reattività all'audio e beat

```
osatura audio-react biped.json ar.json speech.wav head bone_angle gain=10 add=on   # loudness drives a property
osatura beats biped.json bt.json speech.wav as=markers name=syl                    # 2 beats at frames [5, 14]
osatura sound add biped.json s1.json voice speech.wav 0 1.0
osatura sound mix s1.json mix.wav                                                  # wrote mix.wav (1.00 s)
```

Anche `sound mix` tronca alla lunghezza del progetto (un suono di 3 s è diventato 1.00 s di un progetto di 1 s). L'`export` in `.html` incorpora il suono
mixato come mp3 (la pagina di questo progetto è passata da 50 KB a 131 KB), cosa che richiede `ffmpeg`; senza di esso la pagina viene scritta senza suono. Anche `.mp4`,
`.webm` e il mix mp3/ogg richiedono `ffmpeg`.

## 9. Muovere oggetti lungo percorsi, follow-through, foot-lock

* **Path follow:** disegna un livello linea, poi `follow <in> <out> <layer|bone:NAME> <path_layer> <start> <end> [no-orient]` inserisce chiavi su posizione
  e (a meno di `no-orient`) rotazione lungo di essa:

```
cat > path.rhai <<'EOF'
add_line("route", [40, 300, 120, 260, 200, 290, 300, 240, 360, 280], "#bbbbbb", 2, false);
add_circle("ball", 0, 0, 10, "#c1121f");
EOF
osatura script biped.json p0.json path.rhai
osatura follow p0.json p1.json ball route 0 23         # 3 channels: layer_x, layer_y, layer_rotation
```

(`add_line` richiede tre o più punti: `points need an even number of coordinates: 3..1000 points`.)

* **Follow-through di catena:** `follow <in> <out> --target SRC --prop P --chain a,b,c [--delay F --decay X]` copia il movimento di un driver su una
  catena di follower, ciascuno più in ritardo e più debole (azione sovrapposta, code, antenne):

```
$ osatura follow walk.json fo.json --target hips --prop bone_offset_y --chain torso,head --delay 2 --decay 0.6
wrote fo.json (+2 channels)
```

  Le chiavi dei follower vengono spostate in avanti del ritardo, quindi alcune cadono oltre l'ultimo fotogramma (`lint`: `Info L007 [head:BoneOffsetY] key at
  frame 25 lies outside the 0..24 timeline`) e si affidano al loop copiato.
* **Foot-lock:** mantiene un piede appoggiato mentre il corpo si muove. Richiede un **controller radice** (un osso senza genitore) e
  un goal IK del piede:

```
cat > ctrl.rhai <<'EOF'
add_bone("root_ctrl", "", 10, 200, 312, 0);
add_bone("foot_goal_l", "", 10, 212, 312, 0);
add_ik("leg_ik_l", ["thigh_l", "shin_l"], "foot_goal_l", -1);
EOF
osatura script biped.json fl0.json ctrl.rhai
cat > fl1.rhai <<'EOF'
set_key("foot_goal_l", "bone_offset_x", 0, 0);
set_key("foot_goal_l", "bone_offset_x", 12, -40);
set_key("foot_goal_l", "bone_offset_x", 24, 0);
set_key("foot_goal_l", "bone_offset_y", 0, 0);
set_key("foot_goal_l", "bone_offset_y", 6, -15);
set_key("foot_goal_l", "bone_offset_y", 12, 0);
set_key("foot_goal_l", "bone_offset_y", 24, 0);
EOF
osatura script fl0.json fl2.json fl1.rhai
$ osatura foot-lock fl2.json fl3.json foot_goal_l 312 1 2
contact frames 12..23 pinned at (172.0, 312.0)
wrote fl3.json
```

  `foot-lock <in> <out> <ctrl_bone> <ground_y> [tolerance] [min_frames]` trova i fotogrammi in cui il controller tocca il terreno
  e aggiunge canali di compensazione (modalità `add`) in modo che non scivoli. Senza un controller privo di genitore si ferma:
  `foot lock needs a root controller bone (no parent)`.
* **Bake correlati:** `stretch <in> <out> <follower> <leader>` (il follower mira e si allunga verso il leader), `magnet <in> <out> <ctrl>
  <bone:NAME|x,y> <radius> <strength>` (attrazione di prossimità), `bone-vis` (visibilità negli overlay).

## 10. Face tracking e suoi limiti

**Sii onesto su ciò che è.** OSATURA contiene un proprio piccolo rilevatore di volti e un modello di landmark (23 landmark, layout
`osatura23`). È integrato nel binario, funziona in locale, non richiede download ed è stato **addestrato solo su volti generati
dal generatore sintetico di questo stesso progetto**: teste stilizzate e da cartone animato. **Non è validato su fotografie, webcam o video
reali.** Le cifre misurate (`osatura face-eval --eval-n 50 --seed 9`) sono:

| Misura | Risultato |
|---|---|
| Volti sintetici esclusi dall'addestramento, tasso di rilevamento | 1.0 (50 su 50) |
| Errore sui landmark (NME) media / mediana / 90º percentile | 0.028 / 0.026 / 0.046 |
| Falsi positivi su 50 scene senza volto | 2 |
| Disegni cartoon, NME dove rilevato | round-emoji 0.164, manga-oval 0.166, line-drawing 0.105 |
| Disegni cartoon, rilevati | round-emoji 6/6, manga-oval 6/6, line-drawing 6/6, **boxy-robot 0/6** |

Queste cifre valgono per *il suo stile sintetico*. Non leggerle come accuratezza su una persona davanti a una videocamera. Se
ti serve il tracking di volti reali, produci i landmark con uno strumento di cui ti fidi e forniscili come NDJSON (il formato di input di `face-bake`
è aperto); il pannello Face dello studio si aspetta che tu fornisca una funzione `window.osaturaLandmarkProvider` per l'input da videocamera.

La pipeline:

```
osatura face-synth face --frames 60 --size 160 --seed 7        # synthetic head animation: PNG frames, truth.ndjson, params.json
osatura face-track face face/track.ndjson --fps 24             # wrote face/track.ndjson: 60 frames, face found in 60
osatura face-bake biped.json face/track.ndjson facemap.json fb.json --fps 24
```
```
60 landmark frames -> +2 channels (K keys in them), 0 trigger keys set, 2 events; wrote fb.json   # K = number of keys in the 2 new channels
```

con un file di mappatura (qui `facemap.json`):

```
{"face": {"mincutoff": 3.0},
 "params": [
  {"source":"face","id":"head_yaw","target":"head","prop":"bone_angle","out_min":-20,"out_max":20},
  {"source":"face","id":"mouth_open","target":"mouth","prop":"layer_scale_y","out_min":0.6,"out_max":1.8}
 ],
 "triggers": [
  {"source":"face","id":"eye_r_open","edge":"release","threshold":0.5,"action":{"type":"fire_event","name":"blink"}}
 ]}
```

Id dei segnali: `head_yaw`, `head_pitch`, `head_roll`, `eye_l_open`, `eye_r_open`, `mouth_open`, `mouth_wide`, `mouth_round`,
`brow_raise`, `brow_l_raise`, `brow_r_raise`. Un id sbagliato elenca quelli validi: `map 0: unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)`.
`face-bake` calibra il volto neutro dai primi fotogrammi (`--calibrate-frames`, `--calib`).

**Da sapere.** Il bake viene **troncato alla lunghezza del progetto**: 60 fotogrammi di landmark su un progetto di 24 fotogrammi
hanno prodotto chiavi solo da 0 a 23, e un avviso su stderr dice quanti fotogrammi della registrazione non sono stati inclusi nel bake (la lunghezza del progetto non cambia). La riga di riepilogo mostra `(K keys in them)` per i nuovi canali; `N trigger keys set` conta le chiavi scritte in canali preesistenti. `face-detect <image> --overlay o.png` mostra cosa ha trovato il rilevatore su un fermo immagine.

## 11. Mocap e BVH

```
$ osatura mocap-info walk.bvh
11 joints (5 end sites), 36 channels per frame
48 frames at 0.0417 s (24.000 fps), duration 1.958 s
Hips  offset (0.000, 95.000, 0.000)  [Xpos Ypos Zpos Zrot Xrot Yrot]
...
$ osatura mocap-match biped.json walk.bvh
slot         bvh joint                  rig bone                conf  via         len x  note
hips         Hips                       hips                    1.00  name         2.00
head         Head                       head                    1.00  name         4.00
shin.l       LeftLeg                    shin_l                  0.63  name         7.20  limb length ratio is 2.1x off the median
...
$ osatura mocap-retarget biped.json walk.bvh mo.json --view side --smooth 2 --weight 1
note: project length extended from 24 to 48 frames
11 bones, 48 frames at 24 fps, root scale 2.7692, +13 channels; wrote mo.json
```

* Un rig 2D può usare soltanto una proiezione del movimento 3D: `--view front|side|top|yaw=DEG` la sceglie.
* `mocap-match` stampa una tabella di confidenza (le righe `shin` qui sopra hanno una nota perché la lunghezza dei loro arti è 2,1 volte lontana dalla mediana).
  `--map map.json` sostituisce la stima e `--min-conf C` imposta la confidenza sotto la quale un'articolazione non viene usata.
* **`mocap-retarget` allunga il progetto** per adattarlo alla clip (24 sono diventati 48 fotogrammi; una nota lo dice, e `--keep-length` tronca invece la clip). `--in-place` rimuove lo spostamento della radice,
  `--footlock BONE --ground Y` fissa un piede (richiede il controller radice senza genitore, come `foot-lock`: `foot lock needs a root controller bone (no parent)`),
  e `--mirror` specchia il movimento.
* `import-bvh` è la forma più vecchia ed esplicita, con `--joint-map Joint=bone,...`.

## 12. Input dal vivo (MIDI, mouse, gamepad, tasti)

Registra una performance nel pannello Live dello studio, oppure riproduci offline una registrazione o un file MIDI:

```
cat > lmap.json <<'EOF'
{"params":[{"source":"midi_cc","id":"1:1","target":"arm_upper_r","prop":"bone_angle","in_min":0,"in_max":1,"out_min":0,"out_max":160,"smooth":0.2}],
 "triggers":[{"source":"midi_note","id":"1:60","edge":"press","threshold":0.5,"action":{"type":"fire_event","name":"cymbal"}}]}
EOF
osatura live-map biped.json wheel.mid lmap.json lm.json
```
```
27 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 1 events (0 after the last frame skipped); wrote lm.json
```

La rotella di modulazione (CC 1 sul canale 1) ha fatto spazzare al braccio l'intervallo da 0 a 160 gradi attraverso 24 chiavi create dal bake, e la nota attivata a 0,5 s è diventata l'evento del progetto
`cymbal` al fotogramma 12. Le sorgenti sono `midi_cc`, `midi_note`, `midi_pitch_bend`, `mouse_x`, `mouse_y`, `mouse_button`,
`gamepad_axis`, `gamepad_button`, `key` e `face`. Gli id MIDI sono `"<channel>:<number>"` con canali a partire da 1. Il Web MIDI nello
studio richiede un browser che lo supporti (vedi [STUDIO.md](STUDIO.md)); il comando offline richiede solo il file.

## 13. Contenitori di layout e scroll

I contenitori di layout ridispongono i livelli su una diversa dimensione del canvas (animazione di interfacce responsive); le regioni di scroll aggiungono uno scroll
limitato o elastico. I contenitori sono JSON e il solver è deterministico:

```
$ cat cont.json
{"id":"faces","axis":"row","gap":12,"padding":[10,10,10,10],
 "rect":{"kind":"pin","left":10,"right":10,"top":10,"height":100},
 "members":[{"layer":"eye_l"},{"layer":"eye_r","grow":1}]}
$ osatura container add biped.json c1.json cont.json
added container 'faces' (1 containers); wrote c1.json
$ osatura container solve c1.json --size 800x360
canvas 800x360
container faces: x 10.00 y 10.00 w 780.00 h 100.00
layer eye_l: dx -186.000 dy -44.000 scale 1.0000 x 1.0000
layer eye_r: dx -146.000 dy -44.000 scale 1.0000 x 1.0000
$ osatura layout-preview c1.json lp.svg --size 800x360
wrote lp.svg (800x360, 1 containers, 2 layers placed)
```

`dx`/`dy` sono scostamenti applicati sopra l'animazione. I dettagli sui contenitori e su `scroll add|list|remove|eval` (limiti elastici, punti di snap, simulazione
di fling) si trovano in [CLI_REFERENCE.md](CLI_REFERENCE.md#scroll-add-list-remove-eval). Il layout viene applicato dal runtime e dalle anteprime;
**i file esportati contengono il layout risolto per la dimensione propria del progetto.**

## 14. Camera, inquadrature, profondità di campo

```
osatura shot walk.json sh1.json wide 200 180 1          # name x y zoom [rotation]
osatura shot sh1.json sh2.json close 200 100 2.2
osatura camera-seq sh2.json sh3.json 0:wide,12:close,23:wide
osatura parallax sh3.json px.json sky,ground 0.2 1.0
osatura dof px.json 12 dof.png --focus 1.0 --strength 10
osatura dof-export px.json dof.gif --from 0 --to 11 --scale 0.5
```

`timeline sh3.json` mostra 11 canali: i 7 della camminata più quelli della camera. `dof` sfoca ogni livello di primo livello di `|parallax - focus|`
(intensità e raggio sono limitati da `--max-radius`, predefinito 24). La profondità di campo è un render in post-elaborazione; `dof-export` la scrive in una
GIF, una APNG o una cartella.

## 15. Effetti di livello

Gli effetti sono un elenco ordinato su un livello (blur, drop shadow, tint, fill) renderizzato in un passaggio isolato prima della maschera, dell'opacità e della modalità
di fusione del livello, fino a 16 per livello. I loro numeri sono animabili con canali che hanno come target `"<layer>::fx<index>"` e una delle proprietà `fx_blur`, `fx_dx`,
`fx_dy`, `fx_soft`, `fx_amount` (i colori non sono animabili):

```
"effects":[{"type":"drop_shadow","dx":4,"dy":6,"blur":8,"color":"#00000099","opacity":0.8}]      // on layer "head"
{"target":"head::fx0","prop":"fx_dx","source":"keys","track":{"keys":[{"frame":0,"value":-6},{"frame":24,"value":6}]}}
```

`check` passa e l'ombra scorre da -6 a +6 lungo il loop. Una **catena di post-effetti** (outline, posterize, relight con normal map...) renderizza invece un
fotogramma a partire da un elenco JSON:

```
$ echo '[{"type":"outline","width":3,"color":"#222222"},{"type":"posterize","levels":6}]' > chain.json
$ osatura fx biped.json 0 chain.json fxc.png
wrote fxc.png (2 effect(s))
```

Controlli di sicurezza per lampeggiamenti e movimenti di camera (un ausilio tecnico basato sulla definizione pubblica WCAG 2.3.1, **non una certificazione broadcast**):

```
$ osatura safety walk.json
flashes      ok    worst 0 transitions/s (frame 0)
red flashes  ok    worst 0 transitions/s (frame 0)
camera       pan 0.00 widths/s, rotation 0°/s, zoom ×1.00/s
$ osatura reduced-motion walk.json rm.json --factor 0.25
wrote rm.json
```

## 16. Onion skin e scie (overlay)

```
osatura onion p1.json 12 onion.png --before 3 --after 2                  # ghosts of neighbouring frames, raster
osatura onion p1.json onion.svg --frame 12 --before 3 --after 2 --keyed --outline   # SVG; --keyed ghosts key frames
osatura trail p1.json trail.svg --target layer:ball --from 0 --to 23     # wrote trail.svg (24 ticks, 24 key marker(s))
```

`--keyed` disegna i fantasmi nei fotogrammi *chiave* vicini anziché a passo fisso. Gli overlay non modificano mai il progetto e non fanno parte di un'esportazione.
Lo studio li mostra dal vivo (`Alt+N` onion, `Alt+M` scie, `Alt+7` pannello).

## 17. Idle procedurali

```
osatura idle blink  p1.json i1.json --target eye_l --prop layer_scale_y --interval 2 --seed 4 --closed 0.1
osatura idle breath p1.json i2.json --target torso  --prop bone_scale   --amp 0.03 --period 48
osatura idle sway   p1.json i3.json --target head   --prop bone_angle   --amp 3 --freq 0.5 --seed 2
```

Usano un seed, quindi lo stesso comando produce la stessa animazione ovunque. `autorig` aggiunge per te respirazione e ammiccamenti quando i nomi corrispondono.

## 18. Controllo qualità: check, lint, fuzz, CI

* `check` compila il progetto esattamente come farà il renderer. Esce con codice 1 in caso di errore.
* `lint` trova problemi statici (esce con 1 in caso di errori; informazioni e avvisi non causano fallimento): bersagli IK irraggiungibili (`L003`), canali che
  non cambiano mai (`L005`), chiavi fuori dalla timeline (`L007`)...
* `explain`, `coach`, `diff a.json b.json` (`+` aggiunto, `-` rimosso, `~` modificato) e `vcs` (controllo di versione con merge semantico per i file di progetto).
* Il **fuzz** mette il rig in pose casuali e riporta dove si rompe:

```
$ osatura fuzz walk.json --samples 200 --seed 3 --strategy lhs --only thigh_l,shin_l,arm_upper_l
fuzz 'Biped': 200 samples (lhs, seed 3, frame 0), 12 parameters, 22 failures (11.0%), max severity 1.00, baseline 0.00, 0.78 ms/sample
  off_canvas: 22
  extreme-range failure excess  arm_upper_l              28%
  extreme-range failure excess  thigh_l                  9%
  extreme-range failure excess  shin_l                   2%
  strongest interaction: bone:arm_upper_l:scale + bone:arm_upper_l:offx (cell failure rate 53% over 17 samples)
fuzz: failure rate 11.00% exceeds --max-fail-rate 5.00%
```

  **`fuzz` semplice è un rapporto e termina con 0; aggiungi `--gate` (qualsiasi fallimento termina con 1) oppure `--max-fail-rate R` (una frazione) per fargli da gate.** L'intervallo di campionamento è l'intero intervallo di ciascun parametro, quindi un rig con IK
  fallisce in quasi tutti i campioni (`98.0%`, 48 di 50 `ik_unreachable`, sul bipede con foot-lock). Restringi con `--only`, `--props`,
  `--margin` e `--ignore KINDS`, scegli l'intervallo che consideri lecito e imposta la soglia che accetti.
  `--report out.json`, `--heatmap out.svg` e `--minimize` (la pose riproducibile più piccola) aiutano. Strategie: `random`, `lhs`, `halton`, `sweep`, `corners`, `pca`.
* **Golden e CI.** `golden write walk.json gold.json` memorizza le firme dei fotogrammi; `golden check` confronta in modo esatto per impostazione predefinita (esce con 1 a qualsiasi
  modifica; `--tol T` lo rende più permissivo). Controllando con essi un'animazione *diversa* si elencano i fotogrammi che differiscono: `frame 4 differs: pixel hash changed, thumbnail distance 0.013, ...`, ...
  `ci` racchiude questo e altro in un manifest:

```
$ osatura ci init walk.json --out osatura-ci.json
wrote osatura-ci.json with 9 checks:
  validate / render frames / render frames-float / determinism / pose-fuzz / limits / gif-size / loop-seam / render-speed
$ osatura ci run osatura-ci.json
[PASS   ] validate     validate           project compiles; 0 lint finding(s), none at 'error' or above
[PASS   ] render       frames             6 frame(s) match
[PASS   ] render       frames-float       2 frame(s) match
[PASS   ] determinism  determinism        6 render(s) repeated byte-identically
[PASS   ] fuzz         pose-fuzz          44 of 100 pose samples fail (44.00% <= 46.00%)
[PASS   ] limits       limits             within all limits
[PASS   ] export-size  gif-size           gif export is 106945 bytes (budget 160418)
[PASS   ] loop         loop-seam          seam 1.728% vs mean step 1.178%
[INFO   ] perf         render-speed       median 64.52 ms/frame, budget 534 ms (+50% tolerance = 801.00 ms)
9 checks: 8 passed, 0 failed, 0 errors, 1 info, 0 updated, 0 blessed (4.9 s)
```

  `ci run` termina con 0 quando tutto passa, con 1 quando un controllo fallisce e con 2 in caso di errore. `ci bless` accetta i render correnti come nuovi fotogrammi golden,
  `ci diff a.png b.png --metric ssim|de2000|de76|mse` spiega una modifica con mappe di calore. La soglia di pose-fuzz del 46 % nel manifest è stata calibrata sul
  progetto «così com'è ora» (lo strumento stampa quella nota); restringila deliberatamente. I dettagli sono in [CI.md](CI.md).

## 19. Dimensioni e livello di dettaglio

```
$ osatura size-report walk.json | head -9
project file: 17507 bytes (2185 gzip)
  by layer (JSON bytes; embedded image base64 in brackets):
         2658  mouth
          643  mouth/m_small
           ...
  by channel (keys, removable at tol 0.05000000074505806):
         1116  thigh_r.BoneAngle  keys 17 removable 1
```

`size-report` mostra quali livelli, immagini e canali costano byte. Quando una piattaforma ha un budget, `export-fit` trova l'esportazione di migliore qualità che vi rientra:

```
$ osatura export-fit walk.json fit.gif --max-size 60KB --format gif
gif -> fit.gif: 44540 bytes raw, 43822 gzip (budget 60000 raw); SSIM mean 0.9271, worst 0.9057 at frame 3, max CIEDE2000 74.98; 8 candidates built
time budget exhausted (60 s, checked inside every candidate): the search stopped early and the best result so far is kept (exit code 0 if it fits the size budget, 1 if not; ...)
$ osatura export-fit walk.json fit.json --max-size 8KB --format lottie
lottie -> fit.json: 14952 bytes raw, 1429 gzip (budget 8000 raw); SSIM mean 0.9350, worst 0.9060 at frame 10, max CIEDE2000 74.98; 5 candidates built
existing exporter at defaults: 76430 bytes raw, 6654 gzip
budget not met: the smallest export found is 14952 bytes (raw), above the budget of 8000 bytes: every knob is at its floor ...
```

Il secondo comando è terminato con **1** e ha comunque scritto il file più piccolo che ha trovato (14952 byte). I budget vengono verificati per impostazione predefinita sulla dimensione raw;
`--measure gzip` verifica la dimensione compressa (il Lottie qui sopra è di 1429 byte compresso con gzip). La ricerca è limitata da `--max-seconds` (predefinito 60 s di tempo reale);
nella build di debug la ricerca della GIF è stata interrotta da quel limite e ha mantenuto il suo miglior risultato.

Per disegni con molte curve e mesh, il **livello di dettaglio** produce copie semplificate con un limite di errore *dichiarato* in pixel di output:

```
$ osatura lod walk.json --tolerance-px 0.5 --levels 3 --out-dir lods --verify
level  tol_px  anchors  mesh_v  keys  layers    bytes  declared_px  ...  measured_px  ratio
L0      0.000       75       0   119      20    17507       0.0000  ...            -      -
L1      0.500       72       0   100      20    16193       0.1153  ...       0.0895   1.3x
L2      1.000       70       0   100      20    16040       0.3969  ...       0.3597   1.1x
L3      2.000       69       0   100      20    16080       0.5570  ...       0.4997   1.1x
$ osatura lod-pick walk.json --screen-px 200x180 --max-px 0.5
screen 200x180 -> scale 0.5000 px/unit; budget 0.5 px -> level L3 (declared 0.2785 px, 69 anchors, 0 mesh vertices, 100 keys)
```

`--verify` misura la deviazione reale (esce con 1 se supera il limite); nell'esecuzione qui sopra ogni errore misurato era inferiore al suo limite dichiarato. Nota quanto
poco faccia risparmiare un piccolo progetto basato su rettangoli (da 17507 a 16080 byte): il LOD rende su grafica ricca di dettagli, non su questa.

## 20. Limiti onesti

* Il lip-sync è energia più allineamento del testo, non riconoscimento dei fonemi.
* Il rilevatore di volti è solo sintetico (sezione 10): da solo non rende utilizzabile una performance da webcam.
* `make-loop` rimuove lo spostamento; `loop-solve` non lo corregge; il noise non fa loop.
* L'audio viene troncato alla lunghezza del progetto; `mocap-retarget` allunga il progetto.
* Lottie, GIF e HTML non possono trasportare ogni funzione; gli avvisi di esportazione e `--report` elencano ciò che è stato approssimato ([EXPORT_FORMATS.md](EXPORT_FORMATS.md)).
* Lo studio è stato testato solo con Chromium ([STUDIO.md](STUDIO.md)).
