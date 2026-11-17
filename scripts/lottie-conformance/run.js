// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
// Lottie export conformance against an INDEPENDENT player: the exported .json is played by lottie-web (airbnb, MIT) in headless
// Chromium and compared, frame by frame, with OSATURA's own render of the same frame. It measures how far the Lottie file is from
// what OSATURA draws; it does not prove the file is valid for every player. Findings go to a table (and to --out report.md).
//
//   npm install --prefix /tmp/lw lottie-web@5.12.2
//   cargo build -p osatura-cli && node scripts/lottie-conformance/run.js
//
// Environment: OSATURA_BIN (target/debug/osatura), LOTTIE_WEB (path of lottie.js; default /tmp/lw/node_modules/lottie-web/build/player/lottie.js),
// PLAYWRIGHT_CORE, CHROME as in osatura-cli/tests/studio_texture_smoke.js. MAX_MEAN (default 12) is the largest mean absolute
// channel error (0..255) accepted per frame; MAX_BAD_PCT (default 8) the largest share of pixels off by more than 48.
const { execFileSync, spawnSync } = require('child_process');
const fs = require('fs'), os = require('os'), path = require('path');
const root = path.resolve(__dirname, '..', '..');
const BIN = process.env.OSATURA_BIN || path.join(root, 'target/debug/osatura');
const LOTTIE = process.env.LOTTIE_WEB || '/tmp/lw/node_modules/lottie-web/build/player/lottie.js';
const { chromium } = require(process.env.PLAYWRIGHT_CORE || '/opt/npm-tools/node_modules/playwright-core');
const CHROME = process.env.CHROME || '/opt/pw-browsers/chromium-1243/chrome-linux64/chrome';
const MAX_MEAN = Number(process.env.MAX_MEAN || 12), MAX_BAD = Number(process.env.MAX_BAD_PCT || 8);
const outIdx = process.argv.indexOf('--out');
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'osatura-lottie-'));
process.on('exit', () => { try { fs.rmSync(tmp, { recursive: true, force: true }); } catch (e) { } });
const osa = (...a) => execFileSync(BIN, a, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });

const fixtures = [];
for (const [name, cmd] of [['demo', 'demo'], ['person', 'demo-person'], ['fx', 'demo-fx']]) {
  const p = path.join(tmp, name + '.json');
  osa(cmd, p);
  const d = JSON.parse(fs.readFileSync(p, 'utf8'));
  d.frames = Math.min(d.frames || 24, 24);
  fs.writeFileSync(p, JSON.stringify(d));
  fixtures.push({ name, path: p, frames: d.frames, lossy: name === 'fx' });
}
const FORMAT = JSON.parse(fs.readFileSync(fixtures[0].path, 'utf8')).format;
// hand-made projects that isolate one feature each
const base = (layers, extra) => Object.assign({ format: FORMAT, name: 'x', width: 200, height: 150, fps: 24, frames: 12, background: '#ffffff', camera: {}, bones: [], ik: [], constraints: [], springs: [], layers, channels: [] }, extra || {});
const rect = (x, y, w, h) => ({ name: 'r', closed: true, anchors: [{ pos: [x, y] }, { pos: [x + w, y] }, { pos: [x + w, y + h] }, { pos: [x, y + h] }] });
const vec = (name, paths) => ({ name, type: 'vector', paths });
const solid = c => c;
const hand = {
  'flat-fill': base([vec('a', [Object.assign(rect(30, 30, 100, 60), { fill: solid('#3366cc') })])]),
  'two-fills-opacity': base([Object.assign(vec('a', [Object.assign(rect(20, 20, 100, 80), { fill: solid('#cc3333') })]), { opacity: 0.6 }), vec('b', [Object.assign(rect(60, 50, 100, 80), { fill: solid('#33aa33') })])]),
  'stroke-only': base([vec('a', [Object.assign(rect(40, 40, 100, 60), { stroke: { paint: '#222222', width: 8 } })])]),
  'dashed-round-stroke': base([vec('a', [Object.assign(rect(40, 40, 100, 60), { stroke: { paint: '#aa2288', width: 6, cap: 'round', join: 'round', dash: [12, 8] } })])]),
  'linear-gradient': base([vec('a', [Object.assign(rect(20, 20, 160, 100), { fill: { from: [20, 20], to: [180, 20], stops: [{ offset: 0, color: '#ff0000' }, { offset: 1, color: '#0000ff' }] } })])]),
  'radial-gradient': base([vec('a', [Object.assign(rect(20, 20, 160, 100), { fill: { center: [100, 70], radius: 80, stops: [{ offset: 0, color: '#ffffff' }, { offset: 1, color: '#005500' }] } })])]),
};
for (const [name, doc] of Object.entries(hand)) { const p = path.join(tmp, name + '.json'); fs.writeFileSync(p, JSON.stringify(doc)); fixtures.push({ name, path: p, frames: doc.frames, hand: true }); }

(async () => {
  const browser = await chromium.launch({ executablePath: CHROME });
  const page = await browser.newPage();
  await page.setContent('<canvas id="c"></canvas>');
  await page.addScriptTag({ content: fs.readFileSync(LOTTIE, 'utf8') });
  const rows = []; let bad = 0;
  for (const fx of fixtures) {
    let lottie;
    let warned = 0;
    try { const ex = spawnSync(BIN, ['export', fx.path, path.join(tmp, fx.name + '.lottie.json')], { encoding: 'utf8' }); if (ex.status !== 0) throw { stderr: ex.stderr }; warned = (ex.stderr.match(/^warning:/gm) || []).length; lottie = JSON.parse(fs.readFileSync(path.join(tmp, fx.name + '.lottie.json'), 'utf8')); }
    catch (e) { rows.push([fx.name, '-', 'EXPORT FAILED: ' + String(e.stderr || e).split('\n')[0], '', '']); if (!fx.hand) bad++; else bad++; continue; }
    const doc = JSON.parse(fs.readFileSync(fx.path, 'utf8'));
    const bg = doc.background || '#ffffff';
    for (const f of [0, Math.floor(fx.frames / 2), fx.frames - 1]) {
      const png = path.join(tmp, `${fx.name}_${f}.png`);
      osa('render', fx.path, String(f), png);
      const b64 = fs.readFileSync(png).toString('base64');
      const r = await page.evaluate(async ({ lottie, f, b64, bg }) => {
        const c = document.getElementById('c'); c.width = lottie.w; c.height = lottie.h;
        const ctx = c.getContext('2d');
        const img = new Image(); img.src = 'data:image/png;base64,' + b64; await img.decode();
        ctx.fillStyle = bg; ctx.fillRect(0, 0, c.width, c.height);
        const anim = window.lottie.loadAnimation({ renderer: 'canvas', loop: false, autoplay: false, animationData: JSON.parse(JSON.stringify(lottie)), rendererSettings: { context: ctx, clearCanvas: false, preserveAspectRatio: 'none' } });
        await new Promise(r => anim.addEventListener('DOMLoaded', r));
        ctx.fillStyle = bg; ctx.fillRect(0, 0, c.width, c.height);
        anim.goToAndStop(f, true);
        const got = ctx.getImageData(0, 0, c.width, c.height).data;
        const ref = document.createElement('canvas'); ref.width = img.width; ref.height = img.height;
        const rc = ref.getContext('2d'); rc.drawImage(img, 0, 0);
        const want = rc.getImageData(0, 0, ref.width, ref.height).data;
        anim.destroy();
        if (ref.width !== c.width || ref.height !== c.height) return { err: `size ${ref.width}x${ref.height} vs ${c.width}x${c.height}` };
        let sum = 0, off = 0; const n = want.length / 4;
        for (let i = 0; i < want.length; i += 4) { let m = 0; for (let k = 0; k < 3; k++) { const d = Math.abs(got[i + k] - want[i + k]); sum += d; if (d > m) m = d; } if (m > 48) off++; }
        return { mean: sum / (n * 3), bad: 100 * off / n };
      }, { lottie, f, b64, bg });
      if (r.err) { rows.push([fx.name, f, 'ERROR ' + r.err, '', 'FAIL']); bad++; continue; }
      const ok = r.mean <= MAX_MEAN && r.bad <= MAX_BAD;
      // a project that uses features Lottie cannot carry (liquid, boolean ops, particles) may differ, but only if the exporter said so
      const expected = !ok && fx.lossy && warned > 0;
      if (!ok && !expected) bad++;
      rows.push([fx.name, f, r.mean.toFixed(2), r.bad.toFixed(2) + ' %', ok ? 'ok' : expected ? `expected: the exporter warned (${warned} warnings)` : 'FAIL']);
    }
  }
  await browser.close();
  const md = ['| project | frame | mean abs error (0-255) | pixels off by >48 | |', '|---|---|---|---|---|', ...rows.map(r => '| ' + r.join(' | ') + ' |')].join('\n');
  console.log(md);
  console.log(`\nlimits: mean <= ${MAX_MEAN}, off-pixels <= ${MAX_BAD} %; ${bad} failing of ${rows.length}`);
  if (outIdx > 0) fs.writeFileSync(process.argv[outIdx + 1], md + '\n');
  process.exit(bad ? 1 : 0);
})();
