#!/usr/bin/env node
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
/* scripts/build-studio-tour.mjs — turul ghidat „Demo” din Studio: persoana care merge, în 13 etape.
 *
 *   node scripts/build-studio-tour.mjs --bin target/release/osatura [--studio osatura-cli/src/studio.html]
 *
 * `osatura demo-person` scrie proiectul terminat; etapele sunt ACELAȘI proiect, tăiat după fiecare pas. Fiecare etapă e verificată
 * cu binarul real (`check`, `timeline`, `lint`, `stats`, `export`), iar turul o încarcă în editor prin POST /api/op (Set pe calea ""
 * = înlocuiește tot proiectul, un singur pas de Undo). Nu scrie nimic pe disc; Save e dezactivat cât timp demo-ul e încărcat.
 * Idempotent: rulat din nou, înlocuiește blocurile dintre marcaje. Sursele: scripts/studio-tour/{tour.html,tour.css,tour.js}.
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
const STUDIO = path.resolve(arg('--studio', path.join(ROOT, 'osatura-cli', 'src', 'studio.html')));
const gunzipSrc = (() => { const l = fs.readFileSync(path.join(HERE, 'lib', 'gunzip.js'), 'utf8').split('\n'); let i = 0; while (l[i].startsWith('//')) i++; return l.slice(i).join('\n').trim(); })();
const injectGunzip = (text) => text.replace(/\/\*gunzip:begin\*\/[\s\S]*?\/\*gunzip:end\*\//, () => '/*gunzip:begin*/\n' + gunzipSrc + '\n/*gunzip:end*/');
const WORK = fs.mkdtempSync(path.join(os.tmpdir(), 'person-'));
let built;
try { built = personStages({bin: BIN, work: WORK}); } catch (e) { fs.rmSync(WORK, {recursive: true, force: true}); throw e; }
const {steps, engine} = built;

// ---- textele (engleză, din scripts/lib/demo-person-i18n.json, aceleași ca pe site) ----
const TEXTS = JSON.parse(fs.readFileSync(path.join(HERE, 'lib', 'demo-person-i18n.json'), 'utf8')).en;
const IDS = steps.map((s) => s.id);
const dmText = Object.fromEntries(IDS.map((id, k) => [id, [TEXTS['dS' + (k + 1)], TEXTS['dD' + (k + 1)]]]));

// ---- injectare în studio.html ----
const td = (f) => fs.readFileSync(path.join(HERE, 'studio-tour', f), 'utf8');
let s = fs.readFileSync(STUDIO, 'utf8');
const must = (c, w) => { if (!c) throw new Error('nu găsesc în studio.html: ' + w); };
const strip = (open, close) => { const a = s.indexOf(open); if (a < 0) return; const b = s.indexOf(close, a); must(b > 0, close); s = s.slice(0, a) + s.slice(b + close.length).replace(/^\n/, ''); };
strip('/*demo:css*/', '/*/demo:css*/');
strip('<!--demo:panel-->', '<!--/demo:panel-->');
strip('<!--demo:script-->', '<!--/demo:script-->');
const cssAt = s.indexOf('</style>'); must(cssAt > 0, '</style>');
s = s.slice(0, cssAt) + '/*demo:css*/\n' + td('tour.css') + '/*/demo:css*/\n' + s.slice(cssAt);
const panelAt = s.indexOf('<div id="sgp"'); must(panelAt > 0, '<div id="sgp"');
s = s.slice(0, panelAt) + '<!--demo:panel-->\n' + td('tour.html') + '<!--/demo:panel-->\n' + s.slice(panelAt);
// proiectele se stochează gzip+base64 (decomprimate în browser la „Start the demo”)
fs.rmSync(WORK, {recursive: true, force: true});
const packed = {engine, projectName: built.full.name, steps: steps.map((st) => { const {project, ...r} = st; return st.same ? r : {...r, zp: zlib.gzipSync(Buffer.from(JSON.stringify(project)), {level: 9}).toString('base64')}; })};
const data = JSON.stringify(packed).replace(/</g, '\\u003c');
const tourJs = injectGunzip(td('tour.js')).replace(/^\/\/ SPDX.*\n\/\/ Copyright.*\n/, '').replace('/*DM_TEXT*/{}', JSON.stringify(dmText)).replace(/`/g, '');
const bodyEnd = s.lastIndexOf('</body>'); must(bodyEnd > 0, '</body>');
s = s.slice(0, bodyEnd) + '<!--demo:script-->\n<script type="application/json" id="demo-tour">' + data + '</script>\n<script>\n' + tourJs + '</script>\n<!--/demo:script-->\n' + s.slice(bodyEnd);
const btn = '<button id="sgbtn"';
if (!s.includes('id="demobtn"')) { must(s.includes(btn), 'butonul Suggest'); s = s.replace(btn, '<button id="demobtn" title="guided step-by-step demo: build a walking person with the real commands">Demo</button>' + btn); }
fs.writeFileSync(STUDIO, s);
console.log(`ok: ${steps.length} pași, date ${(data.length / 1024).toFixed(0)} KiB → ${STUDIO} (${(s.length / 1024).toFixed(0)} KiB)`);
