// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
// Runs the engine off the main thread so the page stays responsive while frames render.
import { Osatura } from './osatura.js';
let x = null, a = null;
const ops = {
  async init() { x = await Osatura.instantiate(await fetch('osatura_wasm.wasm')); return {}; },
  load({ project }) {
    if (a) a.close();
    a = new Osatura(x, 0); a.load(project);
    return { width: a.width, height: a.height, frames: a.frames, fps: a.fps };
  },
  rgba({ f }) {
    const t = performance.now();
    const { width, height, data } = a.rgba(f, 1);
    const copy = new Uint8ClampedArray(data); // detach from wasm memory so it can be transferred
    return { width, height, data: copy, ms: performance.now() - t, transfer: [copy.buffer] };
  },
  svg({ f }) { const t = performance.now(); const svg = a.svg(f, true); return { svg, ms: performance.now() - t }; },
};
self.onmessage = async ({ data: { id, op, args } }) => {
  try {
    const r = await ops[op](args || {}); const tr = r.transfer || []; delete r.transfer;
    self.postMessage({ id, ok: true, r }, tr);
  } catch (e) { self.postMessage({ id, ok: false, e: String(e && e.message ? e.message : e) }); }
};
