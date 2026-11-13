// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
// Headless-browser smoke test for the studio's usability layer: shortcut hints in tooltips, the unsaved-changes dot and the
// leave warning, a readable Save result, autosave, the Export menu, "?" for the shortcut list, the quick-start strip, the
// elapsed time beside the frame number, and the banner shown when the connection to the studio server is lost.
//
//   cargo build -p osatura-cli && node osatura-cli/tests/studio_ux_smoke.js
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
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'osatura-ux-'));
process.on('exit', () => { try { fs.rmSync(tmp, { recursive: true, force: true }); } catch (e) { } });
const PROJ = path.join(tmp, 'mine.json'), UICFG = path.join(tmp, 'cfg', 'ui.json');
execFileSync(BIN, ['demo', PROJ]);
let pass = 0, fail = 0;
const check = (name, ok, extra) => { if (ok) pass++; else fail++; console.log((ok ? 'PASS ' : 'FAIL ') + name + (ok || extra === undefined ? '' : '  -> ' + JSON.stringify(extra))); };
const sleep = ms => new Promise(r => setTimeout(r, ms));
const url = `http://127.0.0.1:${PORT}/`;
let server = null;
const start = () => { server = spawn(BIN, ['studio', PROJ, '--port', String(PORT), '--ui-config', UICFG], { stdio: 'ignore' }); };
const waitUp = async () => { for (let i = 0; i < 80; i++) { try { const r = await fetch(url + 'api/state'); if (r.ok) return true; } catch (e) { } await sleep(100); } return false; };

(async () => {
  start();
  const browser = await chromium.launch({ executablePath: CHROME });
  const errs = [];
  const open = async ctx => { const p = await ctx.newPage(); p.on('pageerror', e => errs.push(String(e))); p.on('console', m => { if (m.type() === 'error' && !/ERR_CONNECTION|Failed to load resource/.test(m.text())) errs.push(m.text()); }); await p.goto(url); await p.waitForFunction(() => (typeof S!=='undefined'&&S) && document.getElementById('json').value.length > 100); await sleep(250); return p; };
  const cls = (p, sel) => p.evaluate(s => document.querySelector(s).className, sel);
  const disk = () => fs.readFileSync(PROJ, 'utf8');
  const editKey = (p, value) => p.evaluate(async v => { await api('/api/key', { target: S.bones[0].name, prop: 'bone_angle', frame: 5, value: v, ease: { type: 'smooth' } }); await refresh(true); }, value);
  try {
    check('the server starts', await waitUp());
    const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 } });
    let p = await open(ctx);

    // tooltips carry the live shortcut
    check('Save shows its shortcut in the tooltip', /Ctrl\+S/.test(await p.getAttribute('#save', 'title')), await p.getAttribute('#save', 'title'));
    check('Undo shows its shortcut in the tooltip', /Ctrl\+Z/.test(await p.getAttribute('#undo', 'title')), await p.getAttribute('#undo', 'title'));
    check('the Commands button names the palette shortcut', /Ctrl\+K/.test(await p.getAttribute('#palbtn', 'title')));

    // first visit: quick-start strip, remembered once dismissed
    check('the quick-start strip shows on the first visit', (await cls(p, '#welcome')).includes('on'));
    check('...and names the palette shortcut', /Ctrl\+K/.test(await p.textContent('#wl-pal')));
    check('nothing is marked as unsaved right after loading', !(await cls(p, 'body')).includes('dirty') && (await p.title()) === 'OSATURA studio');
    check('the frame label shows the elapsed time', / · \d+\.\d\d s$/.test(await p.textContent('#lab')), await p.textContent('#lab'));
    await p.click('#wl-ok');
    check('Got it hides the strip', !(await cls(p, '#welcome')).includes('on'));
    await p.reload(); await p.waitForFunction(() => typeof S!=='undefined'&&!!S); await sleep(250);
    check('the strip stays away after a reload', !(await cls(p, '#welcome')).includes('on'));

    // unsaved changes
    const before = disk();
    await editKey(p, 41); await sleep(200);
    check('an edit marks the project unsaved (dot, tab title)', (await cls(p, 'body')).includes('dirty') && (await p.title()).startsWith('\u25CF '), await p.title());
    await p.click('#undo'); await sleep(500);
    check('undoing back to the saved state clears the mark', !(await cls(p, 'body')).includes('dirty'), await cls(p, 'body'));
    await p.click('#redo'); await sleep(500);
    check('redo marks it again', (await cls(p, 'body')).includes('dirty'));

    // leave warning
    let dialogs = [];
    p.on('dialog', d => { dialogs.push(d.type()); d.dismiss().catch(() => { }); });
    await p.click('#bones'); await p.click('#bones'); // a real user gesture on the page
    await p.close({ runBeforeUnload: true }); await sleep(400);
    check('closing with unsaved changes asks first', dialogs.includes('beforeunload') && !p.isClosed(), dialogs);

    // save
    await p.click('#save'); await p.waitForFunction(() => /^saved \d\d:\d\d$/.test(document.getElementById('ok').textContent), null, { timeout: 5000 }).catch(() => { });
    check('Save writes the file', disk() !== before && JSON.parse(disk()).bones !== undefined);
    check('Save says when it saved', /^saved \d\d:\d\d$/.test(await p.textContent('#ok')), await p.textContent('#ok'));
    check('Save clears the unsaved mark', !(await cls(p, 'body')).includes('dirty') && (await p.title()) === 'OSATURA studio');
    dialogs = [];
    await p.close({ runBeforeUnload: true }); await sleep(400);
    check('a saved project closes without a question', p.isClosed() && dialogs.length === 0, dialogs);

    // Export menu, "?" and autosave on a fresh page
    p = await open(ctx);
    check('the Export menu starts closed', !(await p.evaluate(() => document.getElementById('expmenu').open)));
    await p.click('#expmenu summary');
    check('it lists GIF, HTML player and Lottie', (await p.$$eval('#expmenu .menu-pop a', a => a.map(x => x.getAttribute('href')))).join() === '/export.gif,/export.html,/export.lottie.json');
    check('...and they are visible', await p.isVisible('#expmenu .menu-pop a'));
    await p.keyboard.press('Escape');
    check('Escape closes it', !(await p.evaluate(() => document.getElementById('expmenu').open)));
    await p.click('#expmenu summary'); await p.mouse.click(300, 400);
    check('a click elsewhere closes it', !(await p.evaluate(() => document.getElementById('expmenu').open)));
    await p.focus('#tl'); await p.keyboard.press('?'); await sleep(250);
    check('"?" opens the shortcut list', await p.evaluate(() => UI.modal) === 'shortcuts', await p.evaluate(() => UI.modal));
    await p.keyboard.press('Escape'); await sleep(150);

    check('autosave is off by default', !(await p.isChecked('#autosave')));
    await p.check('#autosave');
    const d0 = disk();
    await editKey(p, 52);
    await sleep(1200);
    check('autosave waits a moment after the edit', disk() === d0);
    await sleep(3200);
    check('autosave then writes the file', disk() !== d0);
    check('...and says so', /^autosaved \d\d:\d\d$/.test(await p.textContent('#ok')), await p.textContent('#ok'));
    await p.reload(); await p.waitForFunction(() => typeof S!=='undefined'&&!!S); await sleep(200);
    check('the autosave choice is remembered', await p.isChecked('#autosave'));
    await p.uncheck('#autosave');

    // lost connection
    server.kill(); await sleep(600);
    await p.evaluate(() => { fetch('/api/state').catch(() => { }); });
    await p.waitForFunction(() => document.getElementById('netbar').classList.contains('on'), null, { timeout: 8000 }).catch(() => { });
    check('a lost server shows the banner', (await cls(p, '#netbar')).includes('on'));
    start(); await waitUp();
    await p.waitForFunction(() => !document.getElementById('netbar').classList.contains('on'), null, { timeout: 12000 }).catch(() => { });
    check('the banner goes away when the server is back', !(await cls(p, '#netbar')).includes('on'));
    check('no page errors', errs.length === 0, errs);
  } catch (e) { fail++; console.log('FAIL exception  -> ' + (e && e.stack || e)); }
  finally { try { await browser.close(); } catch (e) { } try { server && server.kill(); } catch (e) { } }
  console.log(`\n${pass} passed, ${fail} failed`);
  process.exit(fail ? 1 : 0);
})();
