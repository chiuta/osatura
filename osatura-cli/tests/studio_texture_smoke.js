// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
// Headless-browser smoke test for the studio's texture section (Layers panel): the preset list comes from the server, the
// preview is a real PNG, "Apply" adds a `texture` layer effect to every selected layer as one undo step, "Remove texture"
// takes it away, bad input is refused with a message, and the /api/texture endpoint rejects bad queries.
//
//   cargo build -p osatura-cli && node osatura-cli/tests/studio_texture_smoke.js
//
// Environment: OSATURA_BIN (default target/debug/osatura), PLAYWRIGHT_CORE (default /opt/npm-tools/node_modules/playwright-core),
// CHROME (default /opt/pw-browsers/chromium-1194/chrome-linux/chrome). Only Chromium, a mouse and a keyboard are exercised.
const { spawn, execFileSync } = require('child_process');
const fs = require('fs'), os = require('os'), path = require('path');
const root = path.resolve(__dirname, '..', '..');
const BIN = process.env.OSATURA_BIN || path.join(root, 'target/debug/osatura');
const { chromium } = require(process.env.PLAYWRIGHT_CORE || '/opt/npm-tools/node_modules/playwright-core');
const CHROME = process.env.CHROME || '/opt/pw-browsers/chromium-1194/chrome-linux/chrome';
const PORT = 8980 + Math.floor(Math.random() * 90);
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'osatura-tex-'));
process.on('exit', () => { try { fs.rmSync(tmp, { recursive: true, force: true }); } catch (e) { } });
const PROJ = path.join(tmp, 'mine.json'), UICFG = path.join(tmp, 'cfg', 'ui.json');
execFileSync(BIN, ['demo', PROJ]);
let pass = 0, fail = 0;
const check = (name, ok, extra) => { if (ok) pass++; else fail++; console.log((ok ? 'PASS ' : 'FAIL ') + name + (ok || extra === undefined ? '' : '  -> ' + JSON.stringify(extra))); };
const sleep = ms => new Promise(r => setTimeout(r, ms));
const url = `http://127.0.0.1:${PORT}/`;
let server = null;
const waitUp = async () => { for (let i = 0; i < 80; i++) { try { const r = await fetch(url + 'api/state'); if (r.ok) return true; } catch (e) { } await sleep(150); } return false; };

(async () => {
  server = spawn(BIN, ['studio', PROJ, '--port', String(PORT), '--ui-config', UICFG], { stdio: 'ignore' });
  const browser = await chromium.launch({ executablePath: CHROME });
  const errs = [];
  try {
    check('the server starts', await waitUp());
    // the endpoint on its own
    const png = await fetch(url + 'api/texture?preset=rock&seed=3&size=64');
    const buf = Buffer.from(await png.arrayBuffer());
    check('/api/texture returns a PNG', png.status === 200 && png.headers.get('content-type') === 'image/png' && buf.slice(1, 4).toString() === 'PNG', png.status);
    const again = Buffer.from(await (await fetch(url + 'api/texture?preset=rock&seed=3&size=64')).arrayBuffer());
    check('...the same query gives the same bytes', buf.equals(again));
    const other = Buffer.from(await (await fetch(url + 'api/texture?preset=rock&seed=4&size=64')).arrayBuffer());
    check('...another seed gives another picture', !buf.equals(other));
    for (const m of ['albedo', 'normal', 'height']) check(`map=${m} works`, (await fetch(url + 'api/texture?preset=ice&size=32&map=' + m)).status === 200);
    for (const q of ['preset=nope', 'seed=-1', 'seed=1.5', 'size=8', 'size=100000', 'map=zzz']) {
      const r = await fetch(url + 'api/texture?' + q);
      check(`/api/texture?${q} is refused`, r.status >= 400 && r.status < 500, r.status);
    }
    const pl = await (await fetch(url + 'api/texture/presets')).json();
    check('the preset list has 14 names', Array.isArray(pl.presets) && pl.presets.length === 14 && pl.presets.includes('rock'), pl);

    const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 } });
    const p = await ctx.newPage();
    p.on('pageerror', e => errs.push(String(e)));
    p.on('console', m => { if (m.type() === 'error' && !/ERR_CONNECTION|Failed to load resource/.test(m.text())) errs.push(m.text()); });
    await p.goto(url);
    await p.waitForFunction(() => (typeof S !== 'undefined' && S) && document.getElementById('json').value.length > 100);
    await sleep(300);
    const effects = async () => { const j = await (await fetch(url + 'api/project')).json(); const out = []; const walk = ls => { for (const l of ls || []) { if (l.effects && l.effects.length) out.push([l.name, l.effects]); walk(l.layers); } }; walk(j.layers); return out; };
    check('the demo project starts with no effects', (await effects()).length === 0);

    await p.evaluate(() => { document.getElementById('texfx').open = true; });
    await p.waitForFunction(() => document.getElementById('texpre').options.length > 0, null, { timeout: 5000 }).catch(() => { });
    check('the preset menu is filled from the server', (await p.locator('#texpre option').count()) === 14);
    await p.waitForFunction(() => { const i = document.getElementById('texprev'); return i.complete && i.naturalWidth === 192; }, null, { timeout: 15000 }).catch(() => { });
    check('the preview is a 192 px picture', await p.evaluate(() => document.getElementById('texprev').naturalWidth) === 192);
    const src0 = await p.getAttribute('#texprev', 'src');
    await p.selectOption('#texpre', 'lava');
    await p.waitForFunction(s => document.getElementById('texprev').getAttribute('src') !== s, src0, { timeout: 5000 }).catch(() => { });
    check('choosing a preset changes the preview', /preset=lava/.test(await p.getAttribute('#texprev', 'src')));
    check('the image has a text alternative', (await p.getAttribute('#texprev', 'alt')).length > 5);

    // nothing selected
    await p.click('#texapply');
    check('Apply with no selection explains what to do', /select/.test(await p.textContent('#texmsg')), await p.textContent('#texmsg'));
    check('...and changes nothing', (await effects()).length === 0);

    // select two layers and apply
    const names = await p.evaluate(() => [...document.querySelectorAll('#llist .lrow span')].map(s => s.title));
    check('the demo has layers to select', names.length >= 2, names);
    await p.locator('#llist .lrow span').nth(0).click();
    await p.locator('#llist .lrow span').nth(1).click({ modifiers: ['Shift'] });
    await p.fill('#texseed', '7'); await p.fill('#textile', '128');
    await p.click('#texapply');
    await p.waitForFunction(() => /added to/.test(document.getElementById('texmsg').textContent), null, { timeout: 5000 }).catch(() => { });
    const e1 = await effects();
    check('Apply adds one texture effect to each of the 2 selected layers', e1.length === 2 && e1.every(([, fx]) => fx.length === 1 && fx[0].type === 'texture'), e1);
    check('...with the chosen preset, seed and tile', e1.every(([, fx]) => fx[0].preset === 'lava' && fx[0].seed === 7 && fx[0].tile === 128 && fx[0].amount === 1), e1);
    await p.waitForFunction(() => document.body.classList.contains('dirty'), null, { timeout: 5000 }).catch(() => { });
    check('the project shows as modified', (await p.evaluate(() => document.body.className)).includes('dirty'));
    // one undo step
    await p.click('#undo'); await sleep(400);
    check('one Undo takes the texture off both layers', (await effects()).length === 0, await effects());
    await p.click('#redo'); await sleep(400);
    check('Redo puts it back', (await effects()).length === 2);

    // applying again stacks, removing clears every texture
    await p.click('#texapply'); await sleep(500);
    check('applying twice stacks two effects', (await effects()).every(([, fx]) => fx.length === 2));
    await p.click('#texdel'); await sleep(500);
    check('Remove texture clears them all on the selected layers', (await effects()).length === 0, await effects());
    await p.click('#texdel'); await sleep(400);
    check('Remove with nothing to remove says so', /no texture/.test(await p.textContent('#texmsg')), await p.textContent('#texmsg'));

    // bad input
    await p.fill('#texseed', '-3'); await p.click('#texapply');
    check('a negative seed is refused with a message', /seed/.test(await p.textContent('#texmsg')) && (await effects()).length === 0);
    await p.fill('#texseed', '99999999999'); await p.click('#texapply');
    check('a seed above 4294967295 is refused', /seed/.test(await p.textContent('#texmsg')) && (await effects()).length === 0);
    await p.fill('#texseed', '1'); await p.fill('#textile', '5'); await p.click('#texapply'); await sleep(400);
    check('a tile below 16 is refused', /tile/.test(await p.textContent('#texmsg')) && (await effects()).length === 0, await p.textContent('#texmsg'));

    // a layer that already has other effects keeps them
    await p.fill('#textile', '256');
    const first = names[0];
    const ptr = await p.evaluate(n => (window.LROWS.find(l => l.path === n) || {}).ptr, first);
    await p.evaluate(async q => { await api('/api/op', { op: 'set', path: q + '/effects', value: [{ type: 'blur', radius: 2 }] }); }, ptr);
    await p.locator('#llist .lrow span').nth(0).click();
    await p.click('#texapply'); await sleep(500);
    const e2 = await effects();
    check('existing effects are kept and the texture is added after them', e2.length >= 1 && e2[0][1].length === 2 && e2[0][1][0].type === 'blur' && e2[0][1][1].type === 'texture', e2);
    await p.click('#texdel'); await sleep(500);
    const e3 = await effects();
    check('Remove texture leaves the other effects alone', e3.length === 1 && e3[0][1].length === 1 && e3[0][1][0].type === 'blur', e3);
    check('no page errors', errs.length === 0, errs);
  } catch (e) { fail++; console.log('FAIL exception  -> ' + (e && e.stack || e)); }
  finally { try { await browser.close(); } catch (e) { } try { server && server.kill(); } catch (e) { } }
  console.log(`\n${pass} passed, ${fail} failed`);
  process.exit(fail ? 1 : 0);
})();
