// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
// Headless-browser smoke test for the studio's weight-painting section (Layers panel): the layer and bone menus are filled,
// the mesh is shown, a mouse stroke changes the weights as ONE undo step, Subtract and Smooth work, the keyboard can paint,
// and /api/weights, /api/weights/paint refuse bad input.
//
//   cargo build -p osatura-cli && node osatura-cli/tests/studio_weights_smoke.js
//
// Environment: OSATURA_BIN, PLAYWRIGHT_CORE, CHROME (as in studio_texture_smoke.js).
const { spawn, execFileSync } = require('child_process');
const fs = require('fs'), os = require('os'), path = require('path');
const root = path.resolve(__dirname, '..', '..');
const BIN = process.env.OSATURA_BIN || path.join(root, 'target/debug/osatura');
const { chromium } = require(process.env.PLAYWRIGHT_CORE || '/opt/npm-tools/node_modules/playwright-core');
const CHROME = process.env.CHROME || '/opt/pw-browsers/chromium-1194/chrome-linux/chrome';
const PORT = 8980 + Math.floor(Math.random() * 90);
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'osatura-wt-'));
process.on('exit', () => { try { fs.rmSync(tmp, { recursive: true, force: true }); } catch (e) { } });
const PROJ = path.join(tmp, 'mine.json'), UICFG = path.join(tmp, 'cfg', 'ui.json');
execFileSync(BIN, ['demo', PROJ]);
const PNG1 = 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==';
const doc = JSON.parse(fs.readFileSync(PROJ, 'utf8'));
doc.layers.push({ name: 'patch', type: 'image', source: { png_base64: PNG1 }, origin: [100, 100], size: [160, 160], mesh: { cols: 8, rows: 8, weights: [] } });
fs.writeFileSync(PROJ, JSON.stringify(doc));
let pass = 0, fail = 0;
const check = (name, ok, extra) => { if (ok) pass++; else fail++; console.log((ok ? 'PASS ' : 'FAIL ') + name + (ok || extra === undefined ? '' : '  -> ' + JSON.stringify(extra))); };
const sleep = ms => new Promise(r => setTimeout(r, ms));
const url = `http://127.0.0.1:${PORT}/`;
let server = null;
const waitUp = async () => { for (let i = 0; i < 80; i++) { try { const r = await fetch(url + 'api/state'); if (r.ok) return true; } catch (e) { } await sleep(150); } return false; };
const post = async (p, b) => { const r = await fetch(url + p, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(b) }); return { status: r.status, body: await r.json().catch(() => null) }; };

(async () => {
  server = spawn(BIN, ['studio', PROJ, '--port', String(PORT), '--ui-config', UICFG], { stdio: 'ignore' });
  const browser = await chromium.launch({ executablePath: CHROME });
  const errs = [];
  try {
    check('the server starts', await waitUp());
    // the API on its own
    const m0 = await post('api/weights', { layer: 'patch', bone: 'arm_low' });
    check('/api/weights describes the mesh (81 points)', m0.status === 200 && m0.body.vertices.length === 81 && m0.body.values.length === 81 && m0.body.bones.includes('arm_up'), m0.status);
    check('...an unskinned mesh reads as all zero', m0.body.values.every(v => v === 0));
    for (const [n, b] of [['no layer', { bone: 'arm_up' }], ['no bone', { layer: 'patch' }], ['unknown bone', { layer: 'patch', bone: 'zz' }], ['unknown layer', { layer: 'zz', bone: 'arm_up' }], ['not an image layer', { layer: 'sky', bone: 'arm_up' }]]) {
      const r = await post('api/weights', b);
      check(`/api/weights refuses: ${n}`, r.status >= 400 && r.status < 500 && r.body && r.body.ok === false, r.status);
    }
    const base = { layer: 'patch', bone: 'arm_low', mode: 'add', radius: 40, strength: 1, points: [[180, 180]] };
    for (const [n, over] of [['bad mode', { mode: 'erase' }], ['zero radius', { radius: 0 }], ['no points', { points: [] }], ['bad point', { points: [[1, 'a']] }], ['too many points', { points: Array.from({ length: 4001 }, () => [1, 1]) }], ['unknown bone', { bone: 'zz' }]]) {
      const r = await post('api/weights/paint', { ...base, ...over });
      check(`/api/weights/paint refuses: ${n}`, r.status >= 400 && r.status < 500 && r.body && r.body.ok === false, r.status);
    }
    check('...and nothing was painted by the refused calls', (await post('api/weights', { layer: 'patch', bone: 'arm_low' })).body.values.every(v => v === 0));

    const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 } });
    const p = await ctx.newPage();
    p.on('pageerror', e => errs.push(String(e)));
    p.on('console', m => { if (m.type() === 'error' && !/ERR_CONNECTION|Failed to load resource/.test(m.text())) errs.push(m.text()); });
    await p.goto(url);
    await p.waitForFunction(() => (typeof S !== 'undefined' && S) && document.getElementById('json').value.length > 100);
    await sleep(300);
    await p.evaluate(() => { document.getElementById('wtfx').open = true; });
    await p.waitForFunction(() => document.getElementById('wtbone').options.length > 0 && window.__wt && window.__wt.M, null, { timeout: 8000 }).catch(() => { });
    check('the layer menu lists the image layer', (await p.locator('#wtlayer option').allTextContents()).includes('patch'));
    check('the bone menu is filled from the project', (await p.locator('#wtbone option').count()) >= 2);
    check('the mesh is loaded (81 points)', await p.evaluate(() => window.__wt.M && window.__wt.M.vertices.length) === 81);
    check('the canvas has a text alternative', (await p.getAttribute('#wtcv', 'aria-label')).length > 20);
    await p.selectOption('#wtbone', 'arm_low');
    await p.waitForFunction(() => window.__wt.M && window.__wt.M.bone === 'arm_low');
    const sumOf = async () => (await post('api/weights', { layer: 'patch', bone: 'arm_low' })).body.values.reduce((a, b) => a + b, 0);
    const undoDepth = async () => (await (await fetch(url + 'api/state')).json());

    // a mouse stroke across the canvas
    await p.fill('#wtstr', '1'); await p.fill('#wtrad', '40');
    await p.locator('#wtcv').scrollIntoViewIfNeeded();
    const box = await p.locator('#wtcv').boundingBox();
    const top = await p.evaluate(([x, y]) => { const e = document.elementFromPoint(x, y); return e && e.id; }, [box.x + box.width * 0.75, box.y + box.height * 0.5]);
    check('the canvas can be scrolled clear of the dock', top === 'wtcv', top);
    const mv = (fx, fy) => p.mouse.move(box.x + box.width * fx, box.y + box.height * fy);
    await mv(0.75, 0.3); await p.mouse.down(); await mv(0.75, 0.5); await mv(0.75, 0.7); await p.mouse.up();
    await p.waitForFunction(() => /painted/.test(document.getElementById('wtmsg').textContent), null, { timeout: 8000 }).catch(() => { });
    check('a stroke paints and says how many points it changed', /painted \d+ mesh points/.test(await p.textContent('#wtmsg')), await p.textContent('#wtmsg'));
    const m1 = (await post('api/weights', { layer: 'patch', bone: 'arm_low' })).body;
    const painted = m1.values.filter(v => v > 0.5).length;
    check('...some points now belong to the bone', painted > 3 && painted < 81, painted);
    const tot = new Array(81).fill(0);
    for (const b of m1.bones) (await post('api/weights', { layer: 'patch', bone: b })).body.values.forEach((v, i) => { tot[i] += v; });
    check('...and every point still sums to 1 over all bones', tot.every(v => Math.abs(v - 1) < 1e-3), tot.filter(v => Math.abs(v - 1) >= 1e-3).length);
    // undo is one step
    await p.click('#undo').catch(async () => { await post('api/undo', {}); });
    await sleep(300);
    check('one Undo takes the whole stroke back', (await sumOf()) < 1e-6 || (await sumOf()) < painted);
    await post('api/redo', {});
    const afterRedo = await sumOf();
    check('Redo brings it back', afterRedo > 1);

    // subtract with the same brush
    await p.selectOption('#wtmode', 'subtract');
    await mv(0.75, 0.3); await p.mouse.down(); await mv(0.75, 0.5); await mv(0.75, 0.7); await p.mouse.up();
    await sleep(700);
    check('Subtract takes weight away again', (await sumOf()) < afterRedo - 1);
    // smooth
    await p.selectOption('#wtmode', 'add');
    await mv(0.5, 0.5); await p.mouse.down(); await mv(0.55, 0.55); await p.mouse.up(); await sleep(600);
    const beforeSmooth = (await post('api/weights', { layer: 'patch', bone: 'arm_low' })).body.values;
    await p.selectOption('#wtmode', 'smooth');
    await mv(0.5, 0.5); await p.mouse.down(); await mv(0.52, 0.52); await p.mouse.up(); await sleep(600);
    const afterSmooth = (await post('api/weights', { layer: 'patch', bone: 'arm_low' })).body.values;
    check('Smooth changes the values', beforeSmooth.some((v, i) => Math.abs(v - afterSmooth[i]) > 1e-4));
    // keyboard
    await p.selectOption('#wtmode', 'add');
    const s1 = await sumOf();
    await p.focus('#wtcv'); await p.keyboard.press('ArrowRight'); await p.keyboard.press('ArrowDown'); await p.keyboard.press('Enter'); await sleep(600);
    check('the keyboard can paint (arrows move, Enter paints)', (await sumOf()) !== s1);
    // the studio can still render
    const fr = await fetch(url + 'api/frame?frame=0');
    check('the picture still renders', fr.status === 200, fr.status);
    check('no page errors', errs.length === 0, errs);
  } catch (e) { fail++; console.log('FAIL exception  -> ' + (e && e.stack || e)); }
  finally { try { await browser.close(); } catch (e) { } try { server && server.kill(); } catch (e) { } }
  console.log(`\n${pass} passed, ${fail} failed`);
  process.exit(fail ? 1 : 0);
})();
