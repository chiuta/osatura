// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
import { t } from './i18n.js';
const $ = (s) => document.getElementById(s);
const stage = $('stage'), play = $('play'), scrub = $('scrub'), bones = $('bones'), info = $('info'), err = $('err'), perf = $('perf');
let meta = null, f = 0, running = false, showBones = false, raf = 0, t0 = 0, f0 = 0, busy = false, dirty = false, lastDraw = 0;
const canvas = document.createElement('canvas');
canvas.setAttribute('role', 'img');
const svgBox = document.createElement('div'); svgBox.className = 'svg'; svgBox.setAttribute('role', 'img');
const reduce = matchMedia('(prefers-reduced-motion: reduce)').matches;

// the engine lives in a worker: rendering a 640×480 frame takes tens of milliseconds
const worker = new Worker(new URL('./worker.js', import.meta.url), { type: 'module' });
let seq = 0; const pending = new Map();
worker.onmessage = ({ data: { id, ok, r, e } }) => { const p = pending.get(id); pending.delete(id); ok ? p.res(r) : p.rej(new Error(e)); };
worker.onerror = (e) => fail(e.message || e);
const call = (op, args) => new Promise((res, rej) => { const id = ++seq; pending.set(id, { res, rej }); worker.postMessage({ id, op, args }); });

// rolling render statistics, shown under the player
const times = []; let shown = 0, drawn = 0, since = performance.now();
function stat(ms) {
  times.push(ms); if (times.length > 60) times.shift(); drawn++;
  const now = performance.now();
  if (now - since > 1000) {
    const s = [...times].sort((p, q) => p - q), med = s[s.length >> 1];
    perf.textContent = t('perf', { ms: med.toFixed(0), fps: (drawn * 1000 / (now - since)).toFixed(0), target: meta.fps });
    drawn = 0; since = now; shown++;
  }
}
async function draw() {
  if (busy) { dirty = true; return; }
  busy = true; const at = f;
  try {
    if (showBones) {
      // the SVG comes from our own engine; parse it rather than injecting markup
      const { svg, ms } = await call('svg', { f: at });
      const doc = new DOMParser().parseFromString(svg, 'image/svg+xml');
      svgBox.replaceChildren(document.importNode(doc.documentElement, true));
      if (stage.firstChild !== svgBox) stage.replaceChildren(svgBox);
      stat(ms);
    } else {
      const { width, height, data, ms } = await call('rgba', { f: at });
      if (canvas.width !== width) canvas.width = width;
      if (canvas.height !== height) canvas.height = height;
      canvas.getContext('2d').putImageData(new ImageData(data, width, height), 0, 0);
      if (stage.firstChild !== canvas) stage.replaceChildren(canvas);
      stat(ms);
    }
  } catch (e) { fail(e); } finally { busy = false; }
  if (dirty) { dirty = false; draw(); }
}
function loop(ts) {
  if (!running) return;
  if (!t0) { t0 = ts; f0 = f; }
  f = (f0 + ((ts - t0) / 1000) * meta.fps) % meta.frames;
  scrub.value = f.toFixed(1);
  // never faster than the animation's own fps; drop frames instead of queueing them when rendering is slower
  if (!busy && ts - lastDraw >= 1000 / meta.fps - 2) { lastDraw = ts; draw(); }
  raf = requestAnimationFrame(loop);
}
function setRunning(on) {
  running = on; t0 = 0; cancelAnimationFrame(raf);
  play.textContent = on ? t('pause') : t('play');
  if (on) raf = requestAnimationFrame(loop);
}
function fail(e) { setRunning(false); err.textContent = String(e && e.message ? e.message : e); }
async function open(project, name) {
  err.textContent = '';
  if (!String(project).trim()) { fail(t('empty')); return; }
  try { meta = await call('load', { project }); } catch (e) { fail(e); return; }
  f = 0; scrub.max = String(Math.max(0, meta.frames - 0.1));
  const label = t('info', { name, w: meta.width, h: meta.height, n: meta.frames, fps: meta.fps });
  info.textContent = label; canvas.setAttribute('aria-label', label); svgBox.setAttribute('aria-label', label);
  for (const el of [play, scrub, bones]) el.disabled = false;
  await draw(); setRunning(!reduce);
}
play.onclick = () => setRunning(!running);
scrub.oninput = () => { setRunning(false); f = +scrub.value; draw(); };
bones.onclick = () => { showBones = !showBones; bones.setAttribute('aria-pressed', String(showBones)); bones.textContent = showBones ? t('hide') : t('show'); draw(); };
$('file').onchange = async (ev) => { const file = ev.target.files[0]; if (file) open(await file.text(), file.name); };
try {
  await call('init');
  await open(await (await fetch('demo-person.json')).text(), 'demo-person.json');
} catch (e) { $('loading') && ($('loading').textContent = t('fail')); fail(e); }
