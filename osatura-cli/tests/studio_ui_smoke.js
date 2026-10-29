// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
// Headless-browser smoke test for the studio UI infrastructure (dock panels, palette, shortcut editor) and a
// regression check that canvas selection and the timeline still work.
//
//   cargo build -p osatura-cli && node osatura-cli/tests/studio_ui_smoke.js
//
// Environment: OSATURA_BIN (default target/debug/osatura), PLAYWRIGHT_CORE (default /opt/npm-tools/node_modules/playwright-core),
// CHROME (default /opt/pw-browsers/chromium-1194/chrome-linux/chrome), SHOTS (directory for screenshots, optional).
// Only Chromium, a mouse and a keyboard are exercised: no touch input, no other browsers, no screen reader.
const { spawn, execFileSync } = require('child_process');
const fs = require('fs'), os = require('os'), path = require('path'), http = require('http');
const root = path.resolve(__dirname, '..', '..');
const BIN = process.env.OSATURA_BIN || path.join(root, 'target/debug/osatura');
const { chromium } = require(process.env.PLAYWRIGHT_CORE || '/opt/npm-tools/node_modules/playwright-core');
const CHROME = process.env.CHROME || '/opt/pw-browsers/chromium-1194/chrome-linux/chrome';
const SHOTS = process.env.SHOTS || '';
const PORT = 8790 + Math.floor(Math.random() * 90);
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'osatura-ui-'));
process.on('exit', () => { try { fs.rmSync(tmp, { recursive: true, force: true }); } catch (e) { } }); // never leave the temp dir behind (also on a crash or exit code)
const UICFG = path.join(tmp, 'cfg', 'ui.json'), PROJ = path.join(tmp, 'demo.json');
execFileSync(BIN, ['demo', PROJ]);
let pass = 0, fail = 0;
const check = (name, ok, extra) => { if (ok) pass++; else fail++; console.log((ok ? 'PASS ' : 'FAIL ') + name + (ok || extra === undefined ? '' : '  -> ' + JSON.stringify(extra))); };
const sleep = ms => new Promise(r => setTimeout(r, ms));
const getJson = p => new Promise((res, rej) => http.get({ host: '127.0.0.1', port: PORT, path: p }, r => { let b = ''; r.on('data', d => b += d); r.on('end', () => { try { res(JSON.parse(b)); } catch (e) { rej(e); } }); }).on('error', rej));
const postRaw = (p, body) => new Promise((res, rej) => { const rq = http.request({ host: '127.0.0.1', port: PORT, path: p, method: 'POST', headers: { 'Content-Type': 'application/json' } }, r => { let b = ''; r.on('data', d => b += d); r.on('end', () => res({ status: r.statusCode, body: JSON.parse(b) })); }); rq.on('error', rej); rq.end(JSON.stringify(body)); });

let server;
function startServer() {
  server = spawn(BIN, ['studio', PROJ, '--port', String(PORT), '--ui-config', UICFG], { stdio: 'ignore' });
  return (async () => { for (let i = 0; i < 60; i++) { try { await getJson('/api/state'); return; } catch (e) { await sleep(100); } } throw new Error('server did not start'); })();
}
const stopServer = () => new Promise(r => { server.once('exit', r); server.kill(); });

(async () => {
  await startServer();
  const browser = await chromium.launch({ executablePath: CHROME, args: ['--no-sandbox'] });
  const ctx = await browser.newContext({ viewport: { width: 1500, height: 900 } });
  let page = await ctx.newPage();
  const errs = [];
  const watch = p => { p.on('pageerror', e => errs.push('PAGEERR ' + e.message)); p.on('console', m => { if (m.type() === 'error') errs.push('CONSOLE ' + m.text()); }); };
  watch(page);
  await ctx.addInitScript(() => { window.addEventListener('unhandledrejection', e => console.error('UNHANDLED ' + (e.reason && e.reason.stack || e.reason))); });
  ctx.on('response', r => { if (r.status() >= 400) errs.push('HTTP ' + r.status() + ' ' + r.url()); });
  const shot = async n => { if (SHOTS) await page.screenshot({ path: path.join(SHOTS, n + '.png') }); };
  const ev = (f, a) => page.evaluate(f, a);
  const ready = async () => { try { await page.waitForFunction(() => typeof UI !== 'undefined' && UI.ready && typeof TK !== 'undefined' && TK.m && S && $('img').complete && $('img').naturalWidth > 0, null, { timeout: 15000 }); } catch (e) { console.log('timeline api:', JSON.stringify(await getJson('/api/timeline')).slice(0, 300)); console.log('page not ready; page errors so far:', errs, await page.evaluate(() => JSON.stringify({ ui: typeof UI !== 'undefined' && UI.ready, tk: typeof TK !== 'undefined' && !!TK.m, s: typeof S !== 'undefined' && !!S, img: $('img').complete && $('img').naturalWidth, json: $('json').value.length, res: performance.getEntriesByType('resource').filter(r => r.responseEnd === 0 || r.duration > 2000).map(r => r.name + ' ' + Math.round(r.duration)) }))); console.log('retry tlLoad:', await page.evaluate(async () => { await tlLoad(); return !!TK.m; })); throw e; } await sleep(400); };
  const rectOf = async sel => { const b = await page.locator(sel).first().boundingBox(); return b; };

  await page.goto(`http://127.0.0.1:${PORT}/`); await ready(); await shot('01-default');

  // ---------- registry and wiring ----------
  const nocmd = await ev(() => UI.cmds.filter(c => !handlerFor(c.id)).map(c => c.id));
  check('every registry command has a page handler', nocmd.length === 0, nocmd);
  check('registry delivered by the server (not the fallback)', await ev(() => UI.server && UI.cmds.length > 60));
  check('no shortcut conflicts among the defaults', await ev(() => UI.conflicts.length === 0));
  check('layout valid and every panel placed once', await ev(() => jsValidate(UI.layout).length === 0));
  const vis = await ev(() => ['stage', 'timeline', 'layers'].map(id => { const r = PEL[id].getBoundingClientRect(); return [id, Math.round(r.width), Math.round(r.height)]; }));
  check('default layout shows canvas, timeline and the active side tab with a real size', vis.every(v => v[1] > 50 && v[2] > 20), vis);
  check('timeline canvas got its pixel size', await ev(() => TK.w > 300 && TK.h > 60), await ev(() => [TK.w, TK.h]));
  check('canvas image loaded', await ev(() => $('img').complete && $('img').naturalWidth > 0));

  // ---------- canvas selection still works (R-100) ----------
  const hitPt = await ev(async () => { const ov = $('ov'); for (let gy = 0.1; gy < 0.95; gy += 0.08) for (let gx = 0.1; gx < 0.95; gx += 0.08) { const r = await api('/api/pick', { frame: 0, x: gx * S.width, y: gy * S.height, options: pickOpts() }); if (r.hit) { const b = ov.getBoundingClientRect(); return { x: b.left + gx * b.width, y: b.top + gy * b.height, t: r.hit.target }; } } return null; });
  check('found something to pick on the canvas', !!hitPt, hitPt);
  if (hitPt) {
    await page.mouse.click(hitPt.x, hitPt.y); await sleep(500);
    check('click on the canvas selects it', await ev(() => csel.length > 0), await ev(() => csel));
    await page.mouse.move(hitPt.x, hitPt.y); await page.mouse.down(); await page.mouse.move(hitPt.x + 20, hitPt.y + 8, { steps: 6 }); await page.mouse.up(); await sleep(700);
    check('dragging the selection edits the project (undo available)', await ev(() => S.can_undo === true));
    await ev(() => $('ov').focus());
    const before = await getJson('/api/project');
    const picBefore = await ev(() => $('img').getAttribute('src'));
    await page.keyboard.press('ArrowRight'); await sleep(500);
    check('an edit at the same frame changes the picture address, so the picture is fetched again', (await ev(() => $('img').getAttribute('src'))) !== picBefore, picBefore);
    const after = await getJson('/api/project');
    check('arrow key nudges through the registry dispatcher', JSON.stringify(before) !== JSON.stringify(after));
    await page.keyboard.press('Control+z'); await sleep(500);
    const undone = await getJson('/api/project');
    check('Ctrl+Z undoes the nudge', JSON.stringify(undone) === JSON.stringify(before));
    await page.keyboard.press('Control+Shift+z'); await sleep(400);
    check('Ctrl+Shift+Z redoes it', JSON.stringify(await getJson('/api/project')) === JSON.stringify(after));
    await page.keyboard.press('Control+y'); await sleep(200); // nothing to redo: harmless
    await page.keyboard.press('Control+z'); await sleep(400);
    await page.keyboard.press('Escape'); await sleep(400);
    check('Escape clears the canvas selection', await ev(() => csel.length === 0));
    await page.keyboard.press('Control+a'); await sleep(600);
    check('Ctrl+A selects all layers (canvas context)', await ev(() => csel.length > 1), await ev(() => csel.length));
    await page.keyboard.press('Escape'); await sleep(300);
  }

  // ---------- persistent layer locks (R-100) ----------
  if (hitPt && hitPt.t.kind === 'layer') {
    const lp = hitPt.t.path;
    const lrows = await getJson('/api/layers');
    check('/api/layers reports locked and locked_eff (false for a project from before the flag existed)', lrows.layers.every(l => l.locked === false && l.locked_eff === false) && !JSON.stringify(await getJson('/api/project')).includes('"locked"'));
    await page.locator('.tab[data-panel="layers"]').click(); await sleep(200);
    await ev(p => { const r = document.querySelector(`#llist .lrow[data-path="${p}"]`); r.querySelectorAll('input')[1].click(); }, lp); await sleep(700);
    const pj = await getJson('/api/project');
    const flagged = pj.layers.find(l => l.name === lp.split('/')[0]);
    check('ticking the lock box saves a locked flag in the project (an edit with undo)', (lp.includes('/') || flagged.locked === true) && await ev(() => S.can_undo === true), flagged && flagged.locked);
    check('/api/layers now lists it as locked', (await getJson('/api/layers')).layers.find(l => l.path === lp).locked_eff === true);
    const ptCanvas = await ev(async lpath => { for (let gy = 0.1; gy < 0.95; gy += 0.04) for (let gx = 0.1; gx < 0.95; gx += 0.04) { const o = { tolerance: 3, bone_radius: 7, point_radius: 7, bones: false, ignore_layer_locks: true }; const r = await api('/api/pick', { frame: 0, x: gx * S.width, y: gy * S.height, options: o }); if (r.hit && r.hit.target.path === lpath) { const r2 = await api('/api/pick', { frame: 0, x: gx * S.width, y: gy * S.height, options: { ...o, ignore_layer_locks: false } }); return { x: gx * S.width, y: gy * S.height, still: r2.hit ? r2.hit.target.path : null }; } } return null; }, lp);
    check('the server pick skips the locked layer by itself (no lock list from the page), unless asked to ignore locks', ptCanvas && ptCanvas.still !== lp, ptCanvas);
    const bad = await ev(async p => ({ nudge: await api('/api/nudge', { frame: 0, selection: [{ kind: 'layer', path: p }], delta: [3, 0], locked: [] }), del: await api('/api/delete', { frame: 0, selection: [{ kind: 'layer', path: p }] }) }), lp);
    check('nudge and delete of the locked layer are refused by the server even with an empty page lock list', bad.nudge.ok === false && /locked/.test(JSON.stringify(bad.nudge)) && bad.del.ok === false && /locked/.test(JSON.stringify(bad.del)), bad);
    // reload: the lock is still there and the row is ticked
    await page.reload(); await ready(); await page.locator('.tab[data-panel="layers"]').click(); await sleep(400);
    check('after a reload the lock box is ticked and the layer cannot be selected from the list', await ev(p => { const r = document.querySelector(`#llist .lrow[data-path="${p}"]`); r.querySelector('span').click(); return r.querySelectorAll('input')[1].checked === true && !selLayers().includes(p); }, lp));
    // the Lock button works on a selection and unlock goes through the same flag
    await ev(p => { const r = document.querySelector(`#llist .lrow[data-path="${p}"]`); r.querySelectorAll('input')[1].click(); }, lp); await sleep(600);
    check('unticking removes the lock; the layer can be selected and moved again', (await getJson('/api/layers')).layers.find(l => l.path === lp).locked_eff === false);
    await ev(p => selApply([{ kind: 'layer', path: p }], 'replace'), lp); await sleep(400);
    await page.click('#lockbtn'); await sleep(700);
    check('the Lock button locks the selected layer (saved in the project) and clears the selection', (await getJson('/api/layers')).layers.find(l => l.path === lp).locked === true && await ev(() => csel.length === 0));
    await page.keyboard.press('Control+z'); await sleep(500);
    check('Ctrl+Z undoes the lock like any edit', (await getJson('/api/layers')).layers.find(l => l.path === lp).locked_eff === false);
    await page.keyboard.press('Escape'); await sleep(200);
  }

  // ---------- timeline still works (R-101) ----------
  const tr = await ev(() => { const i = TK.m.tracks.findIndex(t => t.editable && t.keys.length > 1); return i; });
  check('demo has an editable multi-key track', tr >= 0, tr);
  if (tr >= 0) {
    const pos = await ev(i => { const vr = TK.vr; const ri = vr.findIndex(r => r.t === 't' && r.i === i); const b = $('dk-cv').getBoundingClientRect(); const t = TK.m.tracks[i]; return { x: b.left + f2x(t.keys[0].frame), y: b.top + RULER + ri * RH - TK.y0 + RH / 2, f: t.keys[0].frame, ri }; }, tr);
    check('track row visible in the dope sheet', pos.ri >= 0, pos);
    await page.mouse.click(pos.x, pos.y); await sleep(500);
    check('click selects a key', await ev(() => TK.sel.length === 1), await ev(() => TK.sel));
    const f0 = await ev(i => TK.m.tracks[i].keys.map(k => k.frame), tr);
    await page.keyboard.press('Alt+ArrowRight'); await sleep(700);
    const f1 = await ev(i => TK.m.tracks[i].keys.map(k => k.frame), tr);
    check('Alt+Right moves the selected key one frame (timeline context)', f1[0] === f0[0] + 1, [f0, f1]);
    await page.keyboard.press('Control+z'); await sleep(600);
    const f2 = await ev(i => TK.m.tracks[i].keys.map(k => k.frame), tr);
    check('Ctrl+Z inside the timeline undoes it (global command, timeline focus)', f2[0] === f0[0], [f0, f2]);
    // drag the key by the mouse
    await page.mouse.move(pos.x, pos.y); await page.mouse.down(); await page.mouse.move(pos.x + TK_PPF(await ev(() => TK.ppf), 2) / 2, pos.y, { steps: 3 }); await page.mouse.move(pos.x + TK_PPF(await ev(() => TK.ppf), 2), pos.y, { steps: 3 }); await page.mouse.up(); await sleep(700);
    check('mouse drag-moves a key', (await ev(i => TK.m.tracks[i].keys[0].frame, tr)) === f0[0] + 2, await ev(i => TK.m.tracks[i].keys.map(k => k.frame), tr));
    await page.keyboard.press('Control+z'); await sleep(500);
    // timeline context bindings
    await page.keyboard.press('Control+a'); await sleep(500);
    check('Ctrl+A in the timeline selects keys, not layers', await ev(() => TK.sel.length > 1), await ev(() => TK.sel.length));
    await page.keyboard.press('Escape'); await sleep(200);
    check('Escape in the timeline deselects keys', await ev(() => TK.sel.length === 0));
    const fr0 = await ev(() => frame);
    await page.keyboard.press('ArrowRight'); await sleep(100);
    check('ArrowRight in the timeline steps the playhead (not a nudge)', (await ev(() => frame)) === fr0 + 1);
    await page.keyboard.press('Shift+ArrowRight'); await sleep(100);
    check('Shift+ArrowRight steps ten frames', (await ev(() => frame)) === Math.min(fr0 + 11, (await ev(() => S.frames)) - 1));
    // add a key at the playhead through the command
    const nkeys = await ev(i => TK.m.tracks[i].keys.length, tr);
    await ev(i => { TK.focus = i; }, tr); await ev(() => { frame = 7; show(); });
    await page.keyboard.press('i'); await sleep(700);
    check('"I" adds a key at the playhead on the focused channel', (await ev(i => TK.m.tracks[i].keys.length, tr)) >= nkeys, await ev(i => TK.m.tracks[i].keys.map(k => k.frame), tr));
    await page.keyboard.press('Control+z'); await sleep(400);
    await page.keyboard.press('Alt+v'); await sleep(300);
    check('Alt+V switches the dock to the curve graph', await ev(() => TK.view === 'graph'));
    await page.keyboard.press('Alt+v'); await sleep(200);
    check('...and back to the dope sheet', await ev(() => TK.view === 'sheet'));
    await page.mouse.click(pos.x + 200, pos.y - 40, { button: 'left' }); // ruler/blank area click path must not throw
  }
  // playback keys from anywhere
  await ev(() => { document.activeElement && document.activeElement.blur(); frame = 0; show(); });
  await page.keyboard.press('Space'); await sleep(250);
  check('Space starts playback (global)', await ev(() => playing === true));
  await page.keyboard.press('Space'); await sleep(100);
  check('Space stops it', await ev(() => playing === false));
  await page.keyboard.press('End'); await sleep(100);
  check('End goes to the last frame', await ev(() => frame === S.frames - 1));
  await page.keyboard.press('Home'); await sleep(100);
  check('Home goes to frame 0', await ev(() => frame === 0));
  await page.keyboard.press('Alt+k'); await sleep(100);
  check('Alt+K toggles auto-key', await ev(() => $('akey').checked === true));
  await page.keyboard.press('Alt+k'); await sleep(100);
  await page.keyboard.press('Alt+b'); await sleep(100);
  check('Alt+B toggles bones', await ev(() => $('bones').checked === true));
  await page.keyboard.press('Alt+b'); await sleep(100);
  // typing in a text field must not fire plain-key commands
  await page.locator('.tab[data-panel="json"]').click(); await ev(() => { $('json').focus(); });
  await page.keyboard.type(' ..,, '); await sleep(100);
  check('typing in a text area does not step frames', await ev(() => frame === 0));
  await ev(() => { $('json').blur(); });
  await page.locator('.tab[data-panel="layers"]').click();

  // ---------- command palette ----------
  await page.keyboard.press('Control+k'); await sleep(300);
  check('Ctrl+K opens the palette', await page.locator('#pal').isVisible());
  check('palette input is focused', await ev(() => document.activeElement && document.activeElement.id === 'pal-q'));
  const emptyRows = await page.locator('#pal-list .pi').count();
  check('empty query lists commands', emptyRows > 30, emptyRows);
  await page.keyboard.type('next fr'); await sleep(500);
  const top = await ev(() => PLT.hits.map(h => h.id).slice(0, 3));
  check('typing filters and ranks "Next frame" first', top[0] === 'playback.step_next', top);
  check('hit rows show shortcut hints', (await page.locator('#pal-list .pi.on kbd').count()) >= 1);
  check('matched letters are highlighted', (await page.locator('#pal-list .pi.on .t b').count()) >= 3);
  await shot('02-palette');
  await page.keyboard.press('Enter'); await sleep(500);
  check('Enter runs the command and closes the palette', !(await page.locator('#pal').isVisible()) && (await ev(() => frame)) === 1);
  const rec1 = (await getJson('/api/ui')).recents;
  check('the run was stored as a recent on the server', rec1[0] === 'playback.step_next', rec1);
  await page.keyboard.press('Control+k'); await sleep(300);
  check('empty palette lists the recent first', (await ev(() => PLT.hits[0].id)) === 'playback.step_next');
  await page.keyboard.type('export'); await sleep(400);
  await page.keyboard.press('ArrowDown'); await sleep(50);
  check('arrow keys move the highlight', (await ev(() => PLT.i)) === 1);
  await page.keyboard.press('ArrowUp'); await page.keyboard.press('Escape'); await sleep(200);
  check('Escape closes the palette', !(await page.locator('#pal').isVisible()));
  await page.keyboard.press('Control+k'); await sleep(200); await page.keyboard.type('zzzzqq'); await sleep(300);
  check('no match shows a message', (await page.locator('#pal-list').innerText()).includes('No matching'));
  await page.keyboard.press('Control+k'); await sleep(200);
  check('Ctrl+K toggles the palette closed', !(await page.locator('#pal').isVisible()));
  await page.keyboard.press('Control+k'); await sleep(200); await page.keyboard.type('toggle bone display'); await sleep(400);
  await page.locator('#pal-list .pi').first().click(); await sleep(300);
  check('clicking a palette row runs it (toggle bones)', await ev(() => $('bones').checked === true));
  await ev(() => { $('bones').click(); });
  // palette command that changes the layout
  await page.keyboard.press('Control+Shift+p'); await sleep(200); await page.keyboard.type('layout compact'); await sleep(400); await page.keyboard.press('Enter'); await sleep(500);
  check('palette runs a layout preset', await ev(() => UI.layout.root.type === 'split' && UI.layout.root.dir === 'v' && UI.layout.root.b.panels.length === 7));
  await page.keyboard.press('Control+k'); await sleep(200); await page.keyboard.type('reset layout'); await sleep(400); await page.keyboard.press('Enter'); await sleep(500);
  check('palette resets the layout', await ev(() => JSON.stringify(UI.layout) === JSON.stringify(UI.defLayout)));

  // ---------- shortcut editor ----------
  await page.keyboard.press('Control+,'); await sleep(300);
  check('Ctrl+, opens the shortcut editor', await page.locator('#sc').isVisible());
  check('editor lists commands by category', (await page.locator('#sc-list .scc').count()) >= 8 && (await page.locator('#sc-list .scr').count()) > 50);
  await page.fill('#sc-q', 'auto-key'); await sleep(200);
  const row = page.locator('#sc-list .scr[data-cmd="anim.toggle_autokey"]');
  check('filter finds the auto-key command', (await row.count()) === 1);
  await row.locator('kbd').first().click(); await sleep(150);
  check('clicking a chord starts recording', await ev(() => !!SC.rec));
  await page.keyboard.press('Alt+Shift+k'); await sleep(500);
  const b1 = await ev(() => UI.byId['anim.toggle_autokey'].bindings);
  check('recorded chord replaces the binding', JSON.stringify(b1) === JSON.stringify(['global:Alt+Shift+K']), b1);
  const srv1 = await getJson('/api/ui');
  check('...and the server stored it in the user config file', srv1.shortcuts['anim.toggle_autokey'] && fs.existsSync(UICFG) && JSON.parse(fs.readFileSync(UICFG, 'utf8')).shortcuts['anim.toggle_autokey'][0] === 'global:Alt+Shift+K');
  check('config file lives outside the project', !path.resolve(UICFG).startsWith(path.resolve(PROJ) + path.sep) && path.dirname(UICFG) !== path.dirname(PROJ));
  // conflict
  await row.locator('button.add').click(); await sleep(100);
  await page.keyboard.press('Control+z'); await sleep(300);
  check('a conflicting chord shows a warning naming the other command', (await page.locator('#sc-list .warn').innerText()).includes('Undo'));
  const b2 = await ev(() => UI.byId['anim.toggle_autokey'].bindings);
  check('nothing changed until confirmed', b2.length === 1);
  await shot('03-shortcuts');
  await page.locator('#sc-list .warn button', { hasText: 'Cancel' }).click(); await sleep(200);
  await row.locator('button.add').click(); await page.keyboard.press('Control+y'); await sleep(300);
  await page.locator('#sc-list .warn button', { hasText: 'Take it over' }).click(); await sleep(600);
  const eff = await ev(() => ({ a: UI.byId['anim.toggle_autokey'].bindings, r: UI.byId['edit.redo'].bindings, c: UI.conflicts.length }));
  check('"Take it over" moves the chord and leaves no conflict', eff.a.includes('global:Ctrl+Y') && !eff.r.includes('global:Ctrl+Y') && eff.r.length === 1 && eff.c === 0, eff);
  // reserved chord
  await row.locator('button.add').click(); await page.keyboard.press('Control+w'); await sleep(200);
  check('browser-reserved chords are refused', (await page.locator('#sc-list .bad').innerText()).includes('reserved'));
  // server-side validation returns 400
  const bad1 = await postRaw('/api/ui', { shortcuts: { 'edit.undo': ['Ctrl+Y'] } });
  check('server rejects a conflicting shortcut map with 400', bad1.status === 400 && bad1.body.ok === false && bad1.body.errors.length > 0, bad1);
  const bad2 = await postRaw('/api/ui', { layout: { version: 1, root: { type: 'tabs', panels: ['stage'] } } });
  check('server rejects an incomplete layout with 400', bad2.status === 400 && bad2.body.errors.some(e => e.includes('missing')), bad2);
  const bad3 = await postRaw('/api/ui', { shortcuts: { nope: ['K'] } });
  check('server rejects an unknown command with 400', bad3.status === 400);
  const bad4 = await postRaw('/api/ui', 'not an object');
  check('server rejects a non-object body with 400', bad4.status === 400);
  await page.keyboard.press('Escape'); await sleep(100); await page.keyboard.press('Escape'); await sleep(200);
  check('Escape closes the editor', !(await page.locator('#sc').isVisible()));
  // the new chord works, the old one does not
  await ev(() => { document.activeElement && document.activeElement.blur(); $('akey').checked = false; });
  await page.keyboard.press('Alt+k'); await sleep(100);
  check('old chord no longer fires', await ev(() => $('akey').checked === false));
  await page.keyboard.press('Control+y'); await sleep(200);
  check('new chord Ctrl+Y fires the rebound command', await ev(() => $('akey').checked === true));
  await page.keyboard.press('Control+y'); await sleep(100);
  // persists across reload
  await page.reload(); await ready();
  check('rebinding survives a reload (stored on the server)', await ev(() => UI.byId['anim.toggle_autokey'].bindings.includes('global:Ctrl+Y') && UI.byId['anim.toggle_autokey'].user));
  await ev(() => { document.activeElement && document.activeElement.blur(); $('akey').checked = false; });
  await page.keyboard.press('Control+y'); await sleep(100);
  check('...and the dispatcher uses it after the reload', await ev(() => $('akey').checked === true));
  await page.keyboard.press('Control+,'); await sleep(300);
  await page.fill('#sc-q', 'auto-key');
  await page.locator('#sc-list .scr[data-cmd="anim.toggle_autokey"] button.reset').click(); await sleep(500);
  check('per-command Reset restores the defaults', await ev(() => UI.byId['anim.toggle_autokey'].bindings.join() === 'global:Alt+K' && !UI.byId['anim.toggle_autokey'].user));
  await page.locator('#sc-list .scr[data-cmd="anim.toggle_autokey"] kbd').first().click(); await page.keyboard.press('Control+Alt+9'); await sleep(400);
  await page.locator('#sc-reset').click(); await sleep(500);
  check('Reset all clears every override (also the file)', await ev(() => Object.keys(UI.shortcuts).length === 0) && !JSON.parse(fs.readFileSync(UICFG, 'utf8')).shortcuts);
  await page.locator('#sc-close').click(); await sleep(200);

  // ---------- dockable panels ----------
  await ev(() => { document.activeElement && document.activeElement.blur(); });
  const tabs = await ev(() => [...document.querySelectorAll('.tab')].map(t => t.dataset.panel));
  check('tab bars show every docked panel', tabs.length === 8, tabs);
  // resize the splitter between canvas and side group
  const sp = await page.locator('.spl.h').first().boundingBox();
  const ratio0 = await ev(() => UI.layout.root.a.ratio);
  await page.mouse.move(sp.x + 2, sp.y + sp.height / 2); await page.mouse.down(); await page.mouse.move(sp.x - 120, sp.y + sp.height / 2, { steps: 8 }); await page.mouse.up(); await sleep(700);
  const ratio1 = await ev(() => UI.layout.root.a.ratio);
  check('dragging a vertical splitter changes the ratio', ratio1 < ratio0 - 0.03, [ratio0, ratio1]);
  const stageW = (await rectOf('#pn-stage')).width;
  check('...and the canvas panel really shrank', stageW < 1100, stageW);
  const sv = (await getJson('/api/ui')).layout;
  check('...and the ratio was stored on the server', Math.abs(sv.root.a.ratio - ratio1) < 1e-4, sv && sv.root.a.ratio);
  // horizontal splitter (canvas / timeline)
  const sp2 = await page.locator('.spl.v').first().boundingBox();
  await page.mouse.move(sp2.x + sp2.width / 2, sp2.y + 2); await page.mouse.down(); await page.mouse.move(sp2.x + sp2.width / 2, sp2.y - 150, { steps: 8 }); await page.mouse.up(); await sleep(600);
  check('dragging the horizontal splitter grows the timeline', await ev(() => UI.layout.root.ratio < 0.5), await ev(() => UI.layout.root.ratio));
  check('timeline canvas redrew at the new size', await ev(() => Math.abs(TK.h - $('dk-body').clientHeight) < 3), await ev(() => [TK.h, $('dk-body').clientHeight]));
  // splitter clamps to the minimum sizes
  const sp3 = await page.locator('.spl.v').first().boundingBox();
  await page.mouse.move(sp3.x + 10, sp3.y + 2); await page.mouse.down(); await page.mouse.move(sp3.x + 10, 10, { steps: 6 }); await page.mouse.up(); await sleep(400);
  check('splitter cannot squash the canvas below its minimum', (await rectOf('#pn-stage')).height >= 150, (await rectOf('#pn-stage')).height);
  // dock: drag the Bones tab onto the left edge of the canvas
  const bonesTab = await page.locator('.tab[data-panel="bones"]').boundingBox();
  const stageR = await rectOf('#pn-stage');
  await page.mouse.move(bonesTab.x + 10, bonesTab.y + 10); await page.mouse.down();
  await page.mouse.move(bonesTab.x - 30, bonesTab.y + 60, { steps: 5 });
  await page.mouse.move(stageR.x + 25, stageR.y + stageR.height / 2, { steps: 8 });
  const zone = await ev(() => dz.dataset.zone + ':' + dz.style.display);
  await shot('04-dragging');
  await page.mouse.up(); await sleep(700);
  check('drop zone showed "left" while hovering the canvas edge', zone === 'left:block', zone);
  const lay1 = await ev(() => { const g = findGroup(UI.layout, 'bones'), s = findGroup(UI.layout, 'stage'); return { alone: g.panels.length, stageAlone: s.panels.length, valid: jsValidate(UI.layout), sameGroup: g === s }; });
  check('Bones now sits in its own group next to the canvas', lay1.alone === 1 && !lay1.sameGroup && lay1.valid.length === 0, lay1);
  const bR = await rectOf('#pn-bones'), sR = await rectOf('#pn-stage');
  check('...to the left of the canvas on screen', bR.x < sR.x && bR.width > 100, [bR, sR]);
  await shot('05-docked');
  // drag it into the timeline's tab group (center zone, onto its tab bar)
  const bt = await page.locator('.tab[data-panel="bones"]').boundingBox();
  const tlTab = await page.locator('.tab[data-panel="timeline"]').boundingBox();
  await page.mouse.move(bt.x + 10, bt.y + 10); await page.mouse.down(); await page.mouse.move(bt.x + 40, bt.y + 40, { steps: 3 });
  await page.mouse.move(tlTab.x + tlTab.width + 20, tlTab.y + 10, { steps: 8 }); await page.mouse.up(); await sleep(700);
  check('dropping on a tab bar merges into that tab group', await ev(() => findGroup(UI.layout, 'bones') === findGroup(UI.layout, 'timeline') && jsValidate(UI.layout).length === 0));
  check('...and the emptied group disappeared (tree stays valid)', await ev(() => groupsOf(UI.layout).every(g => g.panels.length > 0)));
  // click on a tab activates it
  await page.locator('.tab[data-panel="bones"]').click(); await sleep(300);
  check('clicking a tab shows that panel', (await rectOf('#pn-bones')).height > 30 && !(await ev(() => PEL.timeline.getBoundingClientRect().height > 0)));
  await page.locator('.tab[data-panel="timeline"]').click(); await sleep(400);
  check('...and switching back redraws the timeline', await ev(() => TK.w > 300));
  // collapse / fold
  const sideTabs = page.locator('.grp', { has: page.locator('.tab[data-panel="layers"]') });
  await sideTabs.locator('.tb[title*="fold"]').click(); await sleep(500);
  check('fold button collapses the group to its tab bar', await ev(() => findGroup(UI.layout, 'layers').collapsed === true) && (await sideTabs.boundingBox()).width < 60);
  await sideTabs.locator('.tab').first().click(); await sleep(400);
  check('clicking a folded tab unfolds it', await ev(() => findGroup(UI.layout, 'layers').collapsed === false));
  // hide / show with the panel toggle command and the menu
  await page.keyboard.press('Alt+6'); await sleep(500);
  check('Alt+6 hides the JSON panel (or brings it to front)', await ev(() => { const st = panelState(UI.layout, 'json'); return st === 'hidden' || st === 'docked'; }));
  await page.keyboard.press('Alt+6'); await sleep(500);
  await page.keyboard.press('Alt+6'); await sleep(500);
  const jsonStates = await ev(() => panelState(UI.layout, 'json'));
  await page.locator('#lmenubtn').click(); await sleep(200);
  check('Layout menu lists panels as checkboxes and presets', (await page.locator('#lmenu input[type=checkbox]').count()) === 7 && (await page.locator('#lmenu button[data-preset]').count()) === 4);
  await shot('06-menu');
  const jsonBox = page.locator('#lmenu input[data-panel="json"]');
  const wasChecked = await jsonBox.isChecked();
  await jsonBox.click(); await sleep(400);
  check('menu checkbox toggles panel visibility', (await ev(() => panelState(UI.layout, 'json') === 'hidden')) === wasChecked, [jsonStates, wasChecked]);
  await jsonBox.click(); await sleep(400);
  check('...and shows it again', await ev(() => panelState(UI.layout, 'json') === 'docked'));
  await page.keyboard.press('Escape'); await sleep(200);
  check('Escape closes the menu', !(await page.locator('#lmenu').isVisible()));
  // float and dock back
  const lg = page.locator('.grp', { has: page.locator('.tab[data-panel="json"]') });
  await lg.locator('.tab[data-panel="json"]').click(); await sleep(300);
  await lg.locator('.tb[title*="float"]').click(); await sleep(500);
  check('float button turns the panel into a window', (await page.locator('.fw').count()) === 1 && await ev(() => panelState(UI.layout, 'json') === 'floating'));
  const fwb = await page.locator('.fw').boundingBox();
  await page.mouse.move(fwb.x + 40, fwb.y + 12); await page.mouse.down(); await page.mouse.move(fwb.x + 100, fwb.y + 70, { steps: 5 }); await page.mouse.up(); await sleep(600);
  const fwb2 = await page.locator('.fw').boundingBox();
  check('floating window can be dragged by its header', Math.abs(fwb2.x - fwb.x - 60) < 6 && Math.abs(fwb2.y - fwb.y - 58) < 6, [fwb, fwb2]);
  check('...and the textarea inside still works', await ev(() => $('json').getBoundingClientRect().width > 100));
  await shot('07-floating');
  const floatSaved = (await getJson('/api/ui')).layout.floating;
  check('floating geometry is stored', floatSaved && floatSaved.length === 1 && floatSaved[0].panel === 'json', floatSaved);
  await page.locator('.fw .tb[title*="dock"]').click(); await sleep(500);
  check('dock button puts it back', (await page.locator('.fw').count()) === 0 && await ev(() => panelState(UI.layout, 'json') === 'docked'));
  // presets
  for (const [name, expect] of [['rig', l => (l.hidden || []).includes('live') && (l.hidden || []).includes('face')], ['live', l => (l.hidden || []).includes('layers')], ['compact', l => !(l.hidden || []).length && l.root.dir === 'v']]) {
    await page.locator('#lmenubtn').click(); await sleep(150);
    await page.locator(`#lmenu button[data-preset="${name}"]`).click(); await sleep(700);
    const l = (await getJson('/api/ui')).layout;
    check(`preset "${name}" applied, rendered and stored`, expect(l) && (await ev(() => jsValidate(UI.layout).length === 0 && PEL.stage.getBoundingClientRect().width > 100)), l && l.root);
  }
  await shot('08-compact');
  // everything still works after all that docking: the timeline canvas has a size and a key click still selects
  check('timeline still sized after re-docking', await ev(() => { document.querySelector('.tab[data-panel="timeline"]').click(); return true; }));
  await sleep(500);
  check('...timeline canvas size', await ev(() => TK.w > 250 && TK.h > 40), await ev(() => [TK.w, TK.h]));
  // reload keeps the layout
  const keep = await ev(() => JSON.stringify(UI.layout));
  await sleep(500);
  await page.reload(); await ready();
  check('layout survives a reload (server copy)', (await ev(() => JSON.stringify(UI.layout))) === keep);
  // localStorage mirror
  check('layout mirrored to localStorage', (await ev(() => localStorage.getItem('osatura.ui.layout'))) === keep);
  // reset via menu
  await page.locator('#lmenubtn').click(); await sleep(200); await page.locator('#lmenu button[data-act="reset"]').click(); await sleep(700);
  check('Reset layout returns to the default', await ev(() => JSON.stringify(UI.layout) === JSON.stringify(UI.defLayout)));
  const afterReset = (await getJson('/api/ui')).layout;
  check('...and stores it', JSON.stringify(await ev(l => norm(l), afterReset)) === JSON.stringify(await ev(() => UI.defLayout)));
  // a corrupt stored configuration is ignored (second server instance on another port: killing and restarting the
  // same port leaves stale keep-alive sockets in the browser, which is a test artefact and not what is tested here)
  const BAD = path.join(tmp, 'bad', 'ui.json'); fs.mkdirSync(path.dirname(BAD), { recursive: true }); fs.writeFileSync(BAD, '{ broken');
  const server2 = spawn(BIN, ['studio', PROJ, '--port', String(PORT + 1), '--ui-config', BAD], { stdio: 'ignore' });
  for (let i = 0; i < 60; i++) { try { await new Promise((res, rej) => http.get({ host: '127.0.0.1', port: PORT + 1, path: '/api/state' }, r => { r.resume(); r.on('end', res); }).on('error', rej)); break; } catch (e) { await sleep(100); } }
  const ctx2 = await browser.newContext({ viewport: { width: 1500, height: 900 } }); const p3 = await ctx2.newPage(); watch(p3);
  await p3.goto(`http://127.0.0.1:${PORT + 1}/`);
  await p3.waitForFunction(() => typeof UI !== 'undefined' && UI.ready && TK.m && S, null, { timeout: 15000 }); await sleep(300);
  check('a corrupt config file is ignored, the studio still starts with a valid layout', await p3.evaluate(() => jsValidate(UI.layout).length === 0 && UI.server));
  check('...and the page says why', await p3.evaluate(() => !!UI.startWarning && UI.startWarning.includes('ignored')), await p3.evaluate(() => UI.startWarning));
  await p3.evaluate(() => { document.querySelector('.tab[data-panel="bones"]').click(); });
  await p3.locator('#lmenubtn').click(); await sleep(200); await p3.locator('#lmenu button[data-preset="rig"]').click(); await sleep(700);
  check('the first successful save replaces the corrupt file with a valid one', JSON.parse(fs.readFileSync(BAD, 'utf8')).layout.root.type === 'split');
  await ctx2.close(); server2.kill();

  // ---------- fallback when the settings API is unreachable ----------
  await ev(() => { document.querySelector('.tab[data-panel="timeline"]'); });
  const p2 = await ctx.newPage(); watch(p2);
  await p2.route('**/api/ui**', r => r.abort());
  await p2.goto(`http://127.0.0.1:${PORT}/`);
  await p2.waitForFunction(() => typeof UI !== 'undefined' && UI.ready && S, null, { timeout: 15000 }); await sleep(500);
  check('without /api/ui the page falls back to built-in key bindings and layout', await p2.evaluate(() => UI.server === false && jsValidate(UI.layout).length === 0 && UI.cmds.length > 20));
  await p2.evaluate(() => { frame = 0; show(); });
  await p2.keyboard.press('Space'); await sleep(200);
  check('fallback: Space still plays', await p2.evaluate(() => playing === true));
  await p2.keyboard.press('Space');
  await p2.keyboard.press('Control+k'); await sleep(250);
  await p2.keyboard.type('undo'); await sleep(300);
  check('fallback: palette filters locally', (await p2.evaluate(() => PLT.hits.length)) >= 1 && (await p2.evaluate(() => PLT.hits[0].id)) === 'edit.undo');
  await p2.keyboard.press('Escape');
  await p2.close();

  // ---------- no script errors ----------
  const real = errs.filter(e => !/net::ERR_FAILED|Failed to load resource|ERR_ABORTED|HTTP 400 http:\/\/127\.0\.0\.1:\d+\/api\/(nudge|delete)$/.test(e));
  // Simple view: hides the advanced header controls, is remembered across a reload, and can be undone
  await page.click('#simplebtn'); await sleep(300);
  check('simple view hides overlays, suggestions, collaboration and the editing options', await ev(() => ['ovbtn', 'sgbtn', 'cb'].every(i => getComputedStyle($(i)).display === 'none') && getComputedStyle(document.querySelector('header .tool')).display === 'none'));
  check('simple view keeps undo, save, layout and the exports', await ev(() => ['undo', 'save', 'lmenubtn'].every(i => getComputedStyle($(i)).display !== 'none') && getComputedStyle(document.querySelector('a[href="/export.gif"]')).display !== 'none'));
  await page.reload(); await ready();
  check('simple view is remembered after a reload', await ev(() => document.body.classList.contains('simple') && $('simplebtn').textContent === 'Full view'));
  await page.click('#simplebtn'); await sleep(300);
  check('full view brings everything back', await ev(() => !document.body.classList.contains('simple') && getComputedStyle($('ovbtn')).display !== 'none'));
  check('no page errors or console errors', real.length === 0, real.slice(0, 5));

  await browser.close(); await stopServer();
  console.log(`\n${pass} passed, ${fail} failed`);
  fs.rmSync(tmp, { recursive: true, force: true });
  process.exit(fail ? 1 : 0);
})().catch(e => { console.error('SMOKE TEST CRASHED', e); try { server.kill(); } catch (x) { } process.exit(2); });

function TK_PPF(ppf, frames) { return ppf * frames; }
