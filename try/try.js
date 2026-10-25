// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
import { Osatura } from './osatura.js';
const $ = (s) => document.getElementById(s);
const stage = $('stage'), play = $('play'), scrub = $('scrub'), bones = $('bones'), info = $('info'), err = $('err');
let x = null, a = null, f = 0, running = false, showBones = false, raf = 0, t0 = 0, f0 = 0;
const canvas = document.createElement('canvas');
canvas.setAttribute('role', 'img');
const svgBox = document.createElement('div'); svgBox.className = 'svg'; svgBox.setAttribute('role', 'img');
const reduce = matchMedia('(prefers-reduced-motion: reduce)').matches;

function draw() {
  if (showBones) {
    // the SVG comes from our own engine; parse it rather than injecting markup
    const doc = new DOMParser().parseFromString(a.svg(f, true), 'image/svg+xml');
    svgBox.replaceChildren(document.importNode(doc.documentElement, true));
    if (stage.firstChild !== svgBox) stage.replaceChildren(svgBox);
  } else {
    const { width, height, data } = a.rgba(f, 1);
    if (canvas.width !== width) canvas.width = width;
    if (canvas.height !== height) canvas.height = height;
    canvas.getContext('2d').putImageData(new ImageData(data, width, height), 0, 0);
    if (stage.firstChild !== canvas) stage.replaceChildren(canvas);
  }
  scrub.value = f.toFixed(1);
}
function loop(ts) {
  if (!running) return;
  if (!t0) { t0 = ts; f0 = f; }
  f = (f0 + ((ts - t0) / 1000) * a.fps) % a.frames;
  try { draw(); } catch (e) { fail(e); return; }
  raf = requestAnimationFrame(loop);
}
function setRunning(on) {
  running = on; t0 = 0; cancelAnimationFrame(raf);
  play.textContent = on ? 'Pause' : 'Play';
  if (on) raf = requestAnimationFrame(loop);
}
function fail(e) { setRunning(false); err.textContent = String(e && e.message ? e.message : e); }
function open(project, name) {
  err.textContent = '';
  try {
    if (a) a.close();
    a = new Osatura(x, 0); a.load(project);
  } catch (e) { fail(e); return; }
  f = 0; scrub.max = String(Math.max(0, a.frames - 0.1));
  const label = `${name}: ${a.width}×${a.height}, ${a.frames} frames at ${a.fps} fps`;
  info.textContent = label; canvas.setAttribute('aria-label', label); svgBox.setAttribute('aria-label', label);
  for (const el of [play, scrub, bones]) el.disabled = false;
  draw(); setRunning(!reduce);
}
play.onclick = () => setRunning(!running);
scrub.oninput = () => { setRunning(false); f = +scrub.value; try { draw(); } catch (e) { fail(e); } };
bones.onclick = () => { showBones = !showBones; bones.setAttribute('aria-pressed', String(showBones)); bones.textContent = showBones ? 'Hide bones' : 'Show bones'; try { draw(); } catch (e) { fail(e); } };
$('file').onchange = async (ev) => { const file = ev.target.files[0]; if (file) open(await file.text(), file.name); };
try {
  x = await Osatura.instantiate(await fetch('osatura_wasm.wasm'));
  open(await (await fetch('demo-person.json')).text(), 'demo-person.json');
} catch (e) { $('loading') && ($('loading').textContent = 'The engine could not start in this browser.'); fail(e); }
