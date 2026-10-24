#!/usr/bin/env node
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
/* scripts/build-person-demo.mjs — demo-ul „pas cu pas” al persoanei care merge, din randările REALE ale motorului.
 *
 *   node scripts/build-person-demo.mjs --bin /cale/osatura [--template site/index.template.html]
 *   python3 scripts/build-site.py            # apoi regenerează index.html
 *
 * Etapele (scripts/lib/person-stages.mjs) sunt proiectul `osatura demo-person`, tăiat după fiecare pas; aici fiecare etapă e randată
 * cu binarul real (`render` / `frames`). Textele pașilor vin din scripts/lib/demo-person-i18n.json (aceleași ca în turul din Studio)
 * și înlocuiesc cheile dS, dD și dP, dH, dNote din blocul demo:i18n al șablonului.
 */
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import zlib from 'node:zlib';
import {fileURLToPath} from 'node:url';
import {personStages} from './lib/person-stages.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '..');
const arg = (n, d) => { const i = process.argv.indexOf(n); return i >= 0 ? process.argv[i + 1] : d; };
const BIN = path.resolve(arg('--bin', 'osatura'));
const TPL = path.resolve(arg('--template', path.join(ROOT, 'site', 'index.template.html')));
const WORK = fs.mkdtempSync(path.join(os.tmpdir(), 'person-'));
const gunzipSrc = (() => { const l = fs.readFileSync(path.join(HERE, 'lib', 'gunzip.js'), 'utf8').split('\n'); let i = 0; while (l[i].startsWith('//')) i++; return l.slice(i).join('\n').trim(); })();
const injectGunzip = (text) => text.replace(/\/\*gunzip:begin\*\/[\s\S]*?\/\*gunzip:end\*\//, () => '/*gunzip:begin*/\n' + gunzipSrc + '\n/*gunzip:end*/');
const cleanup = () => fs.rmSync(WORK, {recursive: true, force: true});

try {
  const {run, read, full, steps: raw, engine} = personStages({bin: BIN, work: WORK});
  const AR = `${full.width} / ${full.height}`;
  const svgs = (dir) => fs.readdirSync(path.join(WORK, dir)).filter((f) => f.endsWith('.svg')).sort().map((f) => read(path.join(dir, f)));
  const still = (file, extra = []) => { run(['render', file, '0', 'still.svg', ...extra]); return read('still.svg'); };
  const frames = (file, dir) => { fs.rmSync(path.join(WORK, dir), {recursive: true, force: true}); run(['frames', file, dir]); return svgs(dir); };

  const steps = raw.map((st) => {
    const {project, file, anim, bones, ...s} = st;
    const out = {...s, ar: AR};
    if (s.same) return out;                                 // verificare și export arată animația etapei „ground”
    if (anim) { const fr = frames(file, 'fr-' + s.id); out.still = fr[0]; out.count = fr.length; out.zf = zlib.gzipSync(Buffer.from(JSON.stringify(fr)), {level: 9}).toString('base64'); }
    else out.still = still(file, bones ? ['--bones'] : []);
    return out;
  });
  const sizes = {};
  for (const ext of ['gif', 'html', 'json']) sizes[ext] = fs.statSync(path.join(WORK, `person.${ext}`)).size;
  steps[steps.length - 1].sizes = sizes;
  const data = {version: 2, generator: 'scripts/build-person-demo.mjs', engine, width: full.width, height: full.height, fps: full.fps, frames: full.frames, steps};

  // ---- injectare în șablon ----
  let tpl = injectGunzip(fs.readFileSync(TPL, 'utf8'));
  const json = JSON.stringify(data).replace(/</g, '\\u003c');
  const dRe = /(<script type="application\/json" id="demo-data">)[\s\S]*?(<\/script>)/;
  if (!dRe.test(tpl)) throw new Error('nu găsesc #demo-data în șablon');
  tpl = tpl.replace(dRe, (_, a, b) => a + json + b);

  const iRe = /\/\*demo:i18n\*\/Object\.entries\(([\s\S]*?)\)\.forEach/;
  const mm = tpl.match(iRe);
  if (!mm) throw new Error('nu găsesc blocul demo:i18n');
  const tr = JSON.parse(mm[1]);
  const add = JSON.parse(fs.readFileSync(path.join(HERE, 'lib', 'demo-person-i18n.json'), 'utf8'));
  for (const l of Object.keys(add)) {
    if (!tr[l]) throw new Error('limbă nouă fără bloc: ' + l);
    for (let n = 1; n <= steps.length; n++) if (!add[l]['dS' + n] || !add[l]['dD' + n]) throw new Error(`lipsește dS${n}/dD${n} în ${l}`);
  }
  for (const l of Object.keys(tr)) {
    for (const k of Object.keys(tr[l])) if (/^d[SD]\d+$/.test(k)) delete tr[l][k];
    Object.assign(tr[l], add[l] || add.en);
  }
  tpl = tpl.replace(iRe, () => '/*demo:i18n*/Object.entries(' + JSON.stringify(tr) + ').forEach');
  fs.writeFileSync(TPL, tpl);
  const kb = (s) => (Buffer.byteLength(s) / 1024).toFixed(0);
  console.log(`ok: ${steps.length} pași, date ${kb(json)} KiB → ${TPL}`);
  for (const s of steps) console.log(' ', s.id.padEnd(9), s.zf ? s.count + ' cadre, gzip ' + kb(s.zf) + ' KiB' : s.still ? 'still ' + kb(s.still) + ' KiB' : '-');
} finally { cleanup(); }
