// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
// Headless-browser + HTTP test for the shared editing session (R-111): two Chromium pages join, edit
// concurrently (one of them from a stale view), see each other's cursor and selection, undo only their own
// edits and end with identical projects. Also HTTP checks of the routes, the event stream, an envelope-sending
// participant, the token and the bind-address refusal, and a run of `osatura collab-sim`.
//
//   cargo build -p osatura-cli && node osatura-cli/tests/studio_collab_smoke.js
//
// Environment: OSATURA_BIN, PLAYWRIGHT_CORE, CHROME as in studio_ui_smoke.js. Chromium only, two pages on one machine.
const { spawn, execFileSync, spawnSync } = require('child_process');
const fs = require('fs'), os = require('os'), path = require('path'), http = require('http');
const root = path.resolve(__dirname, '..', '..');
const BIN = process.env.OSATURA_BIN || path.join(root, 'target/debug/osatura');
const { chromium } = require(process.env.PLAYWRIGHT_CORE || '/opt/npm-tools/node_modules/playwright-core');
const CHROME = process.env.CHROME || '/opt/pw-browsers/chromium-1194/chrome-linux/chrome';
const SHOTS = process.env.SHOTS || '';
const PORT = 8700 + Math.floor(Math.random() * 90);
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'osatura-collab-'));
process.on('exit', () => { try { fs.rmSync(tmp, { recursive: true, force: true }); } catch (e) { } }); // never leave the temp dir behind (also on a crash or exit code)
const PROJ = path.join(tmp, 'demo.json');
execFileSync(BIN, ['demo', PROJ]);
let pass = 0, fail = 0;
const check = (name, ok, extra) => { if (ok) pass++; else fail++; console.log((ok ? 'PASS ' : 'FAIL ') + name + (ok || extra === undefined ? '' : '  -> ' + JSON.stringify(extra))); };
const sleep = ms => new Promise(r => setTimeout(r, ms));
const secrets = {};   // `${port}:${replica}` -> participant secret returned by join (a call as a participant must send it)
const req = (port, method, p, body, headers) => new Promise((res, rej) => {
  const h = { ...(body === undefined ? {} : { 'Content-Type': 'application/json' }), ...(headers || {}) };
  if (body && typeof body === 'object' && body.replica && secrets[`${port}:${body.replica}`]) h['X-Collab-Secret'] = secrets[`${port}:${body.replica}`];
  const rq = http.request({ host: '127.0.0.1', port, path: p, method, headers: h }, r => { const bufs = []; r.on('data', d => bufs.push(d)); r.on('end', () => res({ status: r.statusCode, headers: r.headers, text: Buffer.concat(bufs).toString() })); }); rq.on('error', rej); rq.end(body === undefined ? undefined : JSON.stringify(body)); });
const J = async (port, method, p, body, headers) => { const r = await req(port, method, p, body, headers); let j = null; try { j = JSON.parse(r.text); } catch (e) { } if (j && j.ok && j.replica && j.secret) secrets[`${port}:${j.replica}`] = j.secret; return { status: r.status, headers: r.headers, body: j, text: r.text }; };
const servers = [];
function start(port, proj, extra) {
  const s = spawn(BIN, ['studio', proj, '--port', String(port), '--ui-config', path.join(tmp, 'ui' + port + '.json'), ...(extra || [])], { stdio: 'ignore' });
  servers.push(s);
  return (async () => { for (let i = 0; i < 80; i++) { try { await req(port, 'GET', '/api/state'); return; } catch (e) { await sleep(100); } if (extra && extra.length) { try { await req(port, 'GET', '/'); return; } catch (e) { } } } throw new Error('server did not start'); })();
}
function sse(port, p, headers) {
  const out = { text: '', req: null };
  out.req = http.get({ host: '127.0.0.1', port, path: p, headers }, r => { out.status = r.statusCode; out.type = r.headers['content-type']; r.on('data', d => out.text += d); });
  out.req.on('error', () => { });
  return out;
}

(async () => {
  await start(PORT, PROJ);
  const proj0 = (await J(PORT, 'GET', '/api/project')).body;
  const layerNames = proj0.layers.map(l => l.name);

  // ---------- HTTP: routes, events, envelope participant ----------
  const stream = sse(PORT, '/api/collab/events?since=0');
  await sleep(200);
  check('event stream is text/event-stream and answers at once', stream.status === 200 && /text\/event-stream/.test(stream.type) && stream.text.includes('retry: 2000'), [stream.status, stream.type]);
  const ja = (await J(PORT, 'POST', '/api/collab/join', { name: 'Ana' })).body;
  const jb = (await J(PORT, 'POST', '/api/collab/join', { name: 'Bo' })).body;
  check('join gives distinct replica ids and colours, and the document', ja.ok && jb.ok && ja.replica !== jb.replica && ja.colour !== jb.colour && ja.doc.layers.length === layerNames.length, [ja.replica, jb.replica]);
  const rep = (await J(PORT, 'POST', '/api/collab/join', { name: 'Peer', mode: 'replica' })).body;
  check('a replica-mode join returns a snapshot with identities', rep.ok && rep.snapshot && rep.snapshot.doc && typeof rep.snapshot.lamport === 'number');
  const opA = { replica: ja.replica, base: ja.index, ops: [{ op: 'insert', path: '/layers/0', value: { ...proj0.layers[0], name: 'front' } }] };
  const rA = await J(PORT, 'POST', '/api/collab/ops', opA);
  check('Ana inserts a layer at the front', rA.status === 200 && rA.body.applied === true, rA.body);
  // Bo still sees the old list: his index 1 is the old layer 1
  const rB = await J(PORT, 'POST', '/api/collab/ops', { replica: jb.replica, base: jb.index, ops: [{ op: 'set', path: '/layers/1/visible', value: false }] });
  const p1 = (await J(PORT, 'GET', '/api/project')).body;
  check('Bo\'s edit from the stale view lands on the layer he saw, not on the one now at that index', rB.status === 200 && p1.layers[0].name === 'front' && p1.layers[2].name === layerNames[1] && p1.layers[2].visible === false && p1.layers[1].visible !== false, p1.layers.map(l => [l.name, l.visible]));
  const bad = await J(PORT, 'POST', '/api/collab/ops', { replica: jb.replica, ops: [{ op: 'set', path: '/frames', value: -4 }] });
  check('an edit that would make the project invalid is refused whole', bad.status === 400 && bad.body.ok === false && (await J(PORT, 'GET', '/api/project')).body.frames === proj0.frames);
  check('unknown participant and bad bodies are 400', (await J(PORT, 'POST', '/api/collab/ops', { replica: 999, ops: [] })).status === 400 && (await J(PORT, 'POST', '/api/collab/ops', 'x')).status === 400 && (await J(PORT, 'POST', '/api/collab/join', { mode: 'x' })).status === 400);
  // an envelope-sending participant: edit the root key "fps" (the base document's root object has identity 0.0.0)
  const env = { replica: rep.replica, seq: 1, lamport: rep.snapshot.lamport + 1, deps: rep.snapshot.clock, ops: [{ t: 'set_key', obj: '0.0.0', key: 'fps', value: 31 }] };
  const re = await J(PORT, 'POST', '/api/collab/ops', { replica: rep.replica, envelopes: [env] });
  check('an envelope from a replica participant is merged', re.status === 200 && (await J(PORT, 'GET', '/api/project')).body.fps === 31, re.body);
  const re2 = await J(PORT, 'POST', '/api/collab/ops', { replica: rep.replica, envelopes: [env] });
  check('sending the same envelope again is harmless (idempotent)', re2.status === 200);
  const reBad = await J(PORT, 'POST', '/api/collab/ops', { replica: rep.replica, envelopes: [{ ...env, seq: 5 }] });
  check('an envelope with a missing predecessor is refused with a hint', reBad.status === 400 && reBad.body.errors.some(e => /resend from 2/.test(e)), reBad.body);
  const lg = (await J(PORT, 'GET', '/api/collab/log?since=0')).body;
  check('the log lists every accepted envelope in order', lg.ok && lg.entries.length === lg.index && lg.entries[0].by === ja.replica, [lg.entries.length, lg.index]);
  check('legacy edits through other endpoints are recorded as well (state carries the log index)', (await J(PORT, 'POST', '/api/key', { target: 'hand', prop: 'bone_angle', frame: 3, value: 25 })).body.collab_index >= 0 && (await J(PORT, 'GET', '/api/collab/log?since=0')).body.index === lg.index + 1);
  await sleep(200);
  check('the event stream delivered the operations and users', (stream.text.match(/event: ops/g) || []).length >= 3 && stream.text.includes('event: users'), stream.text.length);
  check('the stream frames carry ids = position + 1', /id: 1\nevent: ops/.test(stream.text));
  for (const id of [ja.replica, jb.replica, rep.replica]) await J(PORT, 'POST', '/api/collab/leave', { replica: id });
  stream.req.destroy();
  check('after everybody left, the log index in state disappears', (await J(PORT, 'GET', '/api/state')).body.collab_index === undefined);

  // ---------- browser: two pages (fresh server and project) ----------
  await new Promise(r => { servers[0].once('exit', r); servers[0].kill(); });
  const PROJ2 = path.join(tmp, 'demo2.json'); execFileSync(BIN, ['demo', PROJ2]);
  await start(PORT, PROJ2);
  const browser = await chromium.launch({ executablePath: CHROME, args: ['--no-sandbox'] });
  const mk = async name => {
    const ctx = await browser.newContext({ viewport: { width: 1400, height: 850 } });
    const page = await ctx.newPage(); const errs = [];
    page.on('pageerror', e => errs.push('PAGEERR ' + e.message));
    page.on('console', m => { if (m.type() === 'error' && !/Failed to load resource/.test(m.text())) errs.push('CONSOLE ' + m.text()); });
    ctx.on('response', r => { if (r.status() >= 400 && !/favicon|api\/frame/.test(r.url())) errs.push('HTTP ' + r.status() + ' ' + r.url()); });
    await page.goto(`http://127.0.0.1:${PORT}/`); await page.waitForFunction(() => typeof S !== 'undefined' && S && document.getElementById('img').naturalWidth > 0 && document.querySelectorAll('#llist .lrow').length > 0);
    return { ctx, page, errs, name };
  };
  const A = await mk('Ana'), B = await mk('Bo');
  const projNow = async () => (await J(PORT, 'GET', '/api/project')).body;
  const base = await projNow(); const base0 = base;
  const joinUi = async u => { await u.page.fill('#cname', u.name); await u.page.click('#cjoin'); await u.page.waitForFunction(() => osaturaCollab.on); };
  await joinUi(A); await joinUi(B);
  for (const u of [A, B]) await u.page.waitForFunction(() => document.querySelectorAll('#cusers .cu').length === 2, null, { timeout: 5000 });
  const chipsA = await A.page.$$eval('#cusers .cu', es => es.map(e => [e.textContent, e.style.borderColor]));
  check('both pages list both users with their own colours', chipsA.length === 2 && chipsA[0][1] !== chipsA[1][1] && chipsA.some(c => /Ana \(you\)/.test(c[0])) && chipsA.some(c => /^Bo/.test(c[0])), chipsA);
  const sameChips = (await B.page.$$eval('#cusers .cu', es => es.map(e => e.textContent.replace(/ \(you\)| @\d+/g, '')).sort().join())) === (await A.page.$$eval('#cusers .cu', es => es.map(e => e.textContent.replace(/ \(you\)| @\d+/g, '')).sort().join()));
  check('both pages show the same set of names', sameChips);

  // selection highlight: Ana selects a layer, Bo sees a dot of Ana's colour on that row
  const target = 'arm';
  await A.page.click(`#llist .lrow[data-path="${target}"] span`);
  await B.page.waitForSelector(`#llist .lrow[data-path="${target}"] .rsel`, { timeout: 5000 });
  const dot = await B.page.$eval(`#llist .lrow[data-path="${target}"] .rsel`, e => [e.style.background, e.title]);
  const anaColour = await A.page.evaluate(() => osaturaCollab.colour);
  check('Bo sees Ana\'s selection as a coloured dot on the layer row', /Ana/.test(dot[1]) && dot[0] !== '', [dot, anaColour]);

  // remote selection box: drawn from bounds the server computed
  await B.page.waitForSelector('#rc .rbox', { timeout: 5000 });
  const boxOf = u => u.page.$eval('#rc .rbox', e => [parseFloat(e.style.left), parseFloat(e.style.top), parseFloat(e.style.width), parseFloat(e.style.height), e.style.getPropertyValue('--c'), e.textContent]);
  const pb = await B.page.evaluate(() => { const p = Object.values(osaturaCollab.pres).find(p => p.name === 'Ana'); return [p.bounds, S.width, S.height]; });
  const bx0 = await boxOf(B);
  const exp = pb[0] && [pb[0][0] / pb[1] * 100, pb[0][1] / pb[2] * 100, (pb[0][2] - pb[0][0]) / pb[1] * 100, (pb[0][3] - pb[0][1]) / pb[2] * 100];
  check('Bo sees a box in Ana\'s colour, labelled with her name, at the bounds the server computed', exp && bx0.every((v, i) => i > 3 || Math.abs(v - exp[i]) < 0.01) && bx0[4] === anaColour && bx0[5] === 'Ana' && bx0[2] > 0 && bx0[3] > 0, [bx0, exp]);
  check('Ana does not get a box for her own selection', (await A.page.$$('#rc .rbox')).length === 0);
  const armIdx = base0.layers.findIndex(l => l.name === target);
  await B.page.evaluate(async i => { await api('/api/op', { op: 'set', path: '/layers/' + i + '/transform/position', value: [120, 0] }); }, armIdx);
  await B.page.waitForFunction(x0 => { const e = document.querySelector('#rc .rbox'); return e && Math.abs(parseFloat(e.style.left) - x0) > 1; }, bx0[0], { timeout: 5000 });
  const bx1 = await boxOf(B);
  check('when Bo moves the layer Ana has selected, her box on Bo\'s page follows (bounds are recomputed on the server)', Math.abs(bx1[0] - bx0[0] - 120 / pb[1] * 100) < 0.05 && Math.abs(bx1[2] - bx0[2]) < 0.01, [bx0, bx1]);
  if (SHOTS) await B.page.screenshot({ path: path.join(SHOTS, 'collab_box.png') });
  await B.page.evaluate(async () => { await api('/api/undo', {}); await refresh(true); });
  await B.page.waitForFunction(x0 => { const e = document.querySelector('#rc .rbox'); return e && Math.abs(parseFloat(e.style.left) - x0) < 0.01; }, bx0[0], { timeout: 5000 });

  // remote cursor
  const box = await A.page.$eval('#ov', e => { const r = e.getBoundingClientRect(); return [r.left, r.top, r.width, r.height]; });
  await A.page.mouse.move(box[0] + box[2] * 0.25, box[1] + box[3] * 0.5);
  await A.page.mouse.move(box[0] + box[2] * 0.3, box[1] + box[3] * 0.5, { steps: 4 });
  await B.page.waitForSelector('#rc .rcur', { timeout: 5000 });
  const curOf = () => B.page.$eval('#rc .rcur', e => [e.textContent, parseFloat(e.style.left), parseFloat(e.style.top), e.style.getPropertyValue('--c')]);
  let cur = await curOf(); for (let i = 0; i < 30 && Math.abs(cur[1] - 30) >= 3; i++) { await sleep(100); cur = await curOf(); }   // presence updates arrive a moment after the mouse moves
  check('Bo sees Ana\'s cursor with her name at the same relative place', cur[0] === 'Ana' && Math.abs(cur[1] - 30) < 3 && Math.abs(cur[2] - 50) < 3 && cur[3] === anaColour, cur);
  if (SHOTS) await B.page.screenshot({ path: path.join(SHOTS, 'collab_bo.png') });

  // concurrent edits
  const insertFront = () => A.page.evaluate(async (l) => { const r = await api('/api/op', { op: 'insert', path: '/layers/0', value: { ...l, name: 'front' } }); return r.ok !== false ? true : r; }, proj0.layers[0]);
  const hideSun = () => B.page.evaluate(async () => { const r = await api('/api/op', { op: 'set', path: '/layers/1/visible', value: false }); return r.ok !== false ? true : r; });
  const staleBase = await B.page.evaluate(() => osaturaCollab.base);
  const [okA, okB] = await Promise.all([insertFront(), hideSun()]);
  check('two simultaneous edits both succeed', okA === true && okB === true, [okA, okB]);
  let p2 = await projNow();
  const hiddenNow = p2.layers.filter(l => l.visible === false).map(l => l.name);
  check('both edits are in the project exactly once (front inserted; one layer hidden, the one Bo saw)', p2.layers.length === base.layers.length + 1 && p2.layers.filter(l => l.name === 'front').length === 1 && hiddenNow.length === 1 && hiddenNow[0] === base.layers[1].name, hiddenNow);
  // now the deliberate stale one: Bo's page still thinks the base is old
  await B.page.evaluate(b => { osaturaCollab.base = b; }, staleBase);
  const showHills = await B.page.evaluate(async () => { const r = await api('/api/op', { op: 'set', path: '/layers/2/blend', value: 'overlay' }); return r.ok !== false; });
  p2 = await projNow();
  const named = n => p2.layers.find(l => l.name === n);
  check('a stale index from the page (base = before the insert) hits the layer Bo saw', showHills && named(base.layers[2].name).blend === 'overlay' && named(base.layers[1].name).blend !== 'overlay' && p2.layers.filter(l => l.blend === 'overlay').length === 1, p2.layers.map(l => [l.name, l.blend]));

  // both pages converge
  const dump = async u => u.page.evaluate(async () => (await (await fetch('/api/project')).json()));
  await A.page.waitForFunction(() => document.querySelectorAll('#llist .lrow').length > 0);
  const waitSame = async () => { for (let i = 0; i < 120; i++) { const ta = await A.page.$eval('#json', e => e.value); const tb = await B.page.$eval('#json', e => e.value); const srv = JSON.stringify(await projNow(), null, 1); if (ta === tb && ta === srv) return true; await sleep(100); } return false; };
  check('both pages show the server\'s project (the remote edits arrived by events)', await waitSame());
  const rowsA = await A.page.$$eval('#llist .lrow span', es => es.map(e => e.title)); const rowsB = await B.page.$$eval('#llist .lrow span', es => es.map(e => e.title));
  check('...and the same layer list', JSON.stringify(rowsA) === JSON.stringify(rowsB) && rowsA[0] === 'front', rowsA.slice(0, 3));

  // per-user undo: Bo undoes twice (blend, then hide); Ana's insert stays
  await B.page.click('#undo'); await sleep(150); await B.page.click('#undo'); await sleep(250);
  p2 = await projNow();
  check('Bo\'s undo reverts only Bo\'s edits (blend, visibility); Ana\'s inserted layer is still there', p2.layers[0].name === 'front' && p2.layers.every(l => l.blend !== 'overlay') && p2.layers.every(l => l.visible !== false), p2.layers.map(l => [l.name, l.visible, l.blend]));
  check('Bo\'s undo reached Ana\'s page too', await waitSame());
  await B.page.click('#undo'); await sleep(250);
  p2 = await projNow();
  check('with nothing of his own left, Bo\'s undo changes nothing (does not undo Ana)', p2.layers[0].name === 'front' && p2.layers.length === base.layers.length + 1);
  await A.page.click('#undo'); await sleep(300);
  p2 = await projNow();
  check('Ana\'s undo removes her inserted layer', p2.layers.length === base.layers.length && p2.layers.every(l => l.name !== 'front'));
  await B.page.click('#redo'); await sleep(250);
  p2 = await projNow();
  check('Bo\'s redo brings back his own hidden layer', p2.layers.filter(l => l.visible === false).length === 1);
  check('pages agree again', await waitSame());

  // leaving
  await A.page.click('#cjoin');
  await B.page.waitForFunction(() => document.querySelectorAll('#cusers .cu').length === 1, null, { timeout: 5000 });
  check('when Ana leaves, Bo\'s list shrinks and her cursor and selection marks disappear', (await B.page.$$('#rc .rcur')).length === 0 && (await B.page.$$('#llist .rsel')).length === 0);
  const errsA = A.errs.filter(e => !/api\/frame/.test(e)), errsB = B.errs.filter(e => !/api\/frame/.test(e));
  check('no page errors', errsA.length === 0 && errsB.length === 0, [...errsA, ...errsB]);

  // ---------- persistence: crash (SIGKILL) and restart with --collab-log ----------
  const RP = PORT + 200, LOG = path.join(tmp, 'session.log');
  const rproj = path.join(tmp, 'persist.json'); fs.copyFileSync(PROJ, rproj);
  const killHard = () => new Promise(r => { const s = servers[servers.length - 1]; s.once('exit', r); s.kill('SIGKILL'); });
  await start(RP, rproj, ['--collab-log', LOG]);
  const pa = (await J(RP, 'POST', '/api/collab/join', { name: 'Ana' })).body, pbb = (await J(RP, 'POST', '/api/collab/join', { name: 'Bo' })).body;
  const edit = async (who, op) => (await J(RP, 'POST', '/api/collab/ops', { replica: who.replica, ops: [op] })).body;
  await edit(pa, { op: 'set', path: '/fps', value: 29 });
  await edit(pbb, { op: 'set', path: '/layers/1/visible', value: false });
  await edit(pa, { op: 'insert', path: '/layers/0', value: { ...proj0.layers[0], name: 'persisted' } });
  await J(RP, 'POST', '/api/collab/undo', { replica: pbb.replica });
  const before = (await J(RP, 'GET', '/api/collab/state?doc=1')).body;
  const projBefore = (await J(RP, 'GET', '/api/project')).body;
  check('with --collab-log the file exists after the first edits and has one line per envelope plus the checkpoint', fs.existsSync(LOG) && fs.readFileSync(LOG, 'utf8').trim().split('\n').length === 1 + before.collab_index, [before.collab_index]);
  await killHard();
  await start(RP, rproj, ['--collab-log', LOG]);
  const after = (await J(RP, 'GET', '/api/collab/state?doc=1')).body;
  check('after a hard kill and restart the server resumes with the same log index, clock and lamport time', after.collab_index === before.collab_index && JSON.stringify(after.collab_clock) === JSON.stringify(before.collab_clock) && after.collab_lamport === before.collab_lamport, [before.collab_index, after.collab_index, before.collab_clock, after.collab_clock]);
  check('...and the same project (the editor took the logged document, not the stale file)', JSON.stringify((await J(RP, 'GET', '/api/project')).body) === JSON.stringify(projBefore) && projBefore.fps === 29 && projBefore.layers[0].name === 'persisted');
  // the old participants are gone; pages join again and converge
  check('the old participant ids are unknown after the restart', (await J(RP, 'POST', '/api/collab/ops', { replica: pa.replica, ops: [{ op: 'set', path: '/fps', value: 1 }] })).status === 400);
  const rctx = await browser.newContext({ viewport: { width: 1400, height: 850 } }); const rp1 = await rctx.newPage(); const rerr = [];
  rp1.on('pageerror', e => rerr.push(e.message));
  await rp1.goto(`http://127.0.0.1:${RP}/`); await rp1.waitForFunction(() => typeof S !== 'undefined' && S && document.querySelectorAll('#llist .lrow').length > 0);
  await rp1.fill('#cname', 'Cy'); await rp1.click('#cjoin'); await rp1.waitForFunction(() => osaturaCollab.on);
  const cyId = await rp1.evaluate(() => osaturaCollab.id);
  check('a rejoining page gets a fresh id above all old ones and the resumed log position', cyId > Math.max(pa.replica, pbb.replica) && (await rp1.evaluate(() => osaturaCollab.base)) === before.collab_index, [cyId, pa.replica, pbb.replica]);
  const pd = (await J(RP, 'POST', '/api/collab/join', { name: 'Di' })).body;
  await J(RP, 'POST', '/api/collab/ops', { replica: pd.replica, ops: [{ op: 'set', path: '/frames', value: 36 }] });
  await rp1.waitForFunction(() => /"frames": 36/.test(document.getElementById('json').value), null, { timeout: 5000 });
  const rtxt = await rp1.$eval('#json', e => e.value), rsrv = JSON.stringify((await J(RP, 'GET', '/api/project')).body, null, 1);
  check('after the restart an edit by one rejoined client reaches the other page and both equal the server project', rtxt === rsrv);
  await rp1.evaluate(async () => { await api('/api/op', { op: 'set', path: '/fps', value: 33 }); });
  await J(RP, 'POST', '/api/collab/leave', { replica: pd.replica });
  await rp1.click('#cjoin');
  const logLines = fs.readFileSync(LOG, 'utf8').trim().split('\n').length;
  await killHard();
  await start(RP, rproj, ['--collab-log', LOG]);
  check('a second restart (log extended after the first) resumes with everything', (await J(RP, 'GET', '/api/project')).body.fps === 33 && (await J(RP, 'GET', '/api/project')).body.frames === 36 && (await J(RP, 'GET', '/api/collab/state')).body.collab_index === logLines - 1, [logLines]);
  check('the resumed page loads without errors', rerr.length === 0, rerr);
  await killHard();
  // a damaged log, and a log of another project, stop the server with a clear error and are not modified
  const good = fs.readFileSync(LOG, 'utf8'), lines = good.split('\n');
  const broken = good.replace('"fps"', '"fpz"'); fs.writeFileSync(LOG, broken);
  const bad1 = spawnSync(BIN, ['studio', rproj, '--port', String(RP + 1), '--collab-log', LOG, '--ui-config', path.join(tmp, 'ui-bad.json')], { encoding: 'utf8', timeout: 8000 });
  check('a damaged log refuses to start, naming the line, and is left untouched', bad1.status !== 0 && /damaged at line \d+/.test(bad1.stderr + bad1.stdout) && fs.readFileSync(LOG, 'utf8') === broken, bad1.stderr);
  fs.writeFileSync(LOG, good);
  const other = path.join(tmp, 'other.json'); const oj = JSON.parse(fs.readFileSync(rproj, 'utf8')); oj.width = oj.width + 7; fs.writeFileSync(other, JSON.stringify(oj));
  const bad2 = spawnSync(BIN, ['studio', other, '--port', String(RP + 1), '--collab-log', LOG, '--ui-config', path.join(tmp, 'ui-bad.json')], { encoding: 'utf8', timeout: 8000 });
  check('a log of a different project is refused', bad2.status !== 0 && /different project/.test(bad2.stderr + bad2.stdout) && fs.readFileSync(LOG, 'utf8') === good, bad2.stderr);
  fs.writeFileSync(LOG, good.slice(0, good.length - 12));
  await start(RP, rproj, ['--collab-log', LOG]);
  check('a cut-off last line (crash while appending) is dropped and the session resumes one envelope earlier', (await J(RP, 'GET', '/api/collab/state')).body.collab_index === lines.length - 3, [lines.length]);
  await rctx.close();
  await browser.close();

  // ---------- token and bind address ----------
  const TP = PORT + 100;
  const tproj = path.join(tmp, 'tok.json'); fs.copyFileSync(PROJ, tproj);
  const TOK = 'sesame-sesame-12345';
  await start(TP, tproj, ['--collab-token', TOK]);
  const r1 = await J(TP, 'GET', '/api/state');
  check('with a token, requests without it are 401', r1.status === 401);
  const r1b = await J(TP, 'GET', '/');
  check('the page without a token is a 401 sign-in form that contains no project data', r1b.status === 401 && /api\/login/.test(r1b.text) && !/"layers"/.test(r1b.text));
  const r2 = await J(TP, 'GET', '/api/state', undefined, { 'X-Collab-Token': TOK.slice(0, -1) });
  check('a wrong (prefix) token is 401', r2.status === 401);
  const r3 = await J(TP, 'GET', `/?token=${TOK}`);
  check('the token in the URL is NOT accepted', r3.status === 401 && !r3.headers['set-cookie'], r3.status);
  const r3b = await J(TP, 'POST', '/api/login', { token: TOK });
  check('POST /api/login sets an HttpOnly SameSite=Strict cookie', r3b.status === 200 && new RegExp('osatura_token=' + TOK + '; Path=/; HttpOnly; SameSite=Strict').test(String(r3b.headers['set-cookie'])), r3b.headers['set-cookie']);
  const r3c = await J(TP, 'GET', '/', undefined, { Cookie: 'osatura_token=' + TOK });
  check('the page opens with the cookie', r3c.status === 200 && /<title>/.test(r3c.text));
  const r4 = await J(TP, 'POST', '/api/collab/join', { name: 'x' }, { Cookie: 'osatura_token=' + TOK });
  check('the cookie authorises API calls', r4.status === 200 && r4.body.ok);
  const r4b = await J(TP, 'GET', '/api/state', undefined, { 'X-Collab-Token': TOK });
  check('the X-Collab-Token header authorises API calls', r4b.status === 200);
  const s2 = sse(TP, '/api/collab/events', { Cookie: 'osatura_token=' + TOK });
  const s3 = sse(TP, '/api/collab/events');
  await sleep(300);
  check('the event stream needs the token too', s2.status === 200 && s3.status === 401, [s2.status, s3.status]);
  s2.req.destroy(); s3.req.destroy();
  // five wrong tokens in a row start a back-off
  for (let k = 0; k < 6; k++) await J(TP, 'GET', '/api/state', undefined, { 'X-Collab-Token': 'guess-' + k });
  const r5b = await J(TP, 'GET', '/api/state', undefined, { 'X-Collab-Token': 'guess-again' });
  check('the guesser is told to wait (429)', r5b.status === 429, r5b.status);
  const r5 = await J(TP, 'GET', '/api/state', undefined, { 'X-Collab-Token': TOK });
  check('the right token is still accepted meanwhile (a guesser cannot lock the owner out)', r5.status === 200, r5.status);
  const refuse = spawnSync(BIN, ['studio', PROJ, '--port', String(TP + 1), '--bind', '0.0.0.0'], { encoding: 'utf8', timeout: 5000 });
  check('a non-local bind without a token is refused', refuse.status !== 0 && /refusing to listen/.test(refuse.stderr + refuse.stdout), refuse.stderr);
  const short = spawnSync(BIN, ['studio', PROJ, '--port', String(TP + 2), '--collab-token', 'abc'], { encoding: 'utf8', timeout: 5000 });
  check('a token shorter than 16 characters is refused', short.status !== 0 && /at least 16/.test(short.stderr + short.stdout));

  // ---------- Rust-side simulation ----------
  const sim = spawnSync(BIN, ['collab-sim', PROJ, '--clients', '6', '--ops', '600', '--seed', '2026', '--replays', '12', '--json'], { encoding: 'utf8' });
  let sj = null; try { sj = JSON.parse(sim.stdout); } catch (e) { }
  check('collab-sim: 6 replicas, 600 random concurrent edits converge, and 12 other delivery orders give the same project', sim.status === 0 && sj && sj.converged_ok && sj.ok === true && sj.merged_project_compiles === true && sj.converged && sj.state_identical && sj.replays_identical === 12 && sj.pending_left === 0, sim.stdout + sim.stderr);
  console.log('simulation: ' + JSON.stringify(sj));

  for (const s of servers) s.kill();
  console.log(`\n${pass} passed, ${fail} failed`);
  process.exit(fail ? 1 : 0);
})().catch(e => { console.error('FATAL', e); for (const s of servers) s.kill(); process.exit(2); });
