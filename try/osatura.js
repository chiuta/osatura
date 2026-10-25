// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
// OSATURA WebAssembly wrapper. Dependency-free ES module; browsers and node >= 18.
// Talks to the plain extern "C" ABI of osatura-wasm (see src/lib.rs); no wasm-bindgen glue.

const enc = new TextEncoder();
const dec = new TextDecoder();

export class Osatura {
  /** @param {WebAssembly.Exports} x  @param {number} handle */
  constructor(x, handle) {
    this.x = x;
    this.h = handle;
    this.frames = 0;
    this.fps = 0;
    this.width = 0;
    this.height = 0;
    this._raf = null;
    this._caf = null;
  }

  /** Instantiate from bytes, a Response promise, or a URL string/URL. Returns the raw exports. */
  static async instantiate(source, imports = {}) {
    let inst;
    if (source instanceof ArrayBuffer || ArrayBuffer.isView(source)) {
      inst = (await WebAssembly.instantiate(source, imports)).instance;
    } else if (typeof Response !== 'undefined' && source instanceof Response) {
      if (!source.ok) throw new Error(`cannot load the wasm module: ${source.url || 'response'} answered HTTP ${source.status} ${source.statusText}`.trim());
      inst = (await WebAssembly.instantiateStreaming(source, imports)).instance;
    } else {
      let r;
      try {
        r = await fetch(source);
      } catch (e) {
        throw new Error(`cannot load the wasm module from ${source}: ${e && e.message ? e.message : e}`);
      }
      if (!r.ok) {
        throw new Error(`cannot load the wasm module from ${source}: HTTP ${r.status} ${r.statusText}. Build it first (osatura-wasm/build-wasm.sh) and serve osatura-wasm/ so that osatura_wasm.wasm is reachable at that URL.`.trim());
      }
      const buf = await r.arrayBuffer();
      inst = (await WebAssembly.instantiate(buf, imports)).instance;
    }
    return inst.exports;
  }

  /** Create from a wasm source (or already-instantiated exports) and a project (JSON string or object). */
  static async create(source, project) {
    const x = source && typeof source.arm_open === 'function' ? source : await Osatura.instantiate(source);
    const a = new Osatura(x, 0);
    a.load(project);
    return a;
  }

  /**
   * Feed the module's clock (wasm32-unknown-unknown has no system clock: std time calls would panic). Modules built
   * before `arm_set_clock` existed simply lack it. Called before every call that can take time.
   */
  _tick() {
    const f = this.x.arm_set_clock;
    if (typeof f === 'function') f(typeof performance !== 'undefined' ? performance.now() : 0, Date.now());
  }

  _err() {
    const x = this.x;
    const n = x.arm_error_len();
    if (!n) return 'unknown error';
    return dec.decode(new Uint8Array(x.memory.buffer, x.arm_error_ptr(), n).slice());
  }

  /** Load (or replace) the project. Throws Error with the engine's message on failure. */
  load(project) {
    const x = this.x;
    this._tick();
    const text = typeof project === 'string' ? project : JSON.stringify(project);
    const bytes = enc.encode(text);
    const ptr = x.arm_alloc(bytes.length);
    if (!ptr) throw new Error(this._err());
    let h;
    try {
      new Uint8Array(x.memory.buffer, ptr, bytes.length).set(bytes);
      h = x.arm_open(ptr, bytes.length);
    } finally {
      x.arm_free(ptr, bytes.length);
    }
    if (!h) throw new Error(this._err());
    if (this.h) x.arm_close(this.h);
    this.h = h;
    this.frames = x.arm_frame_count(h);
    this.fps = x.arm_fps(h);
    this.width = x.arm_width(h);
    this.height = x.arm_height(h);
    return this;
  }

  _out(n) {
    if (n < 0) throw new Error(this._err());
    const x = this.x;
    // Copy out: the view is invalidated by the next call or by memory growth.
    return new Uint8Array(x.memory.buffer, x.arm_out_ptr(this.h), n).slice();
  }

  /** @returns {string} SVG document for frame n (fractional frames allowed). */
  svg(n, showBones = false) {
    this._tick();
    return dec.decode(this._out(this.x.arm_render_svg(this.h, n, showBones ? 1 : 0)));
  }

  /** @returns {{width:number,height:number,data:Uint8ClampedArray}} straight-alpha RGBA. */
  rgba(n, scale = 1) {
    this._tick();
    const len = this.x.arm_render_rgba(this.h, n, scale);
    const data = this._out(len);
    return { width: this.x.arm_out_width(this.h), height: this.x.arm_out_height(this.h), data: new Uint8ClampedArray(data.buffer) };
  }

  /** Browser only: frame as ImageData. kind 'svg' returns the SVG string, 'image' returns ImageData. */
  frame(n, { kind = 'image', scale = 1, bones = false } = {}) {
    if (kind === 'svg') return { svg: this.svg(n, bones) };
    const { width, height, data } = this.rgba(n, scale);
    if (typeof ImageData === 'undefined') return { width, height, data };
    return { image: new ImageData(data, width, height) };
  }

  // ---- interactive playback: state machines -------------------------------------------------

  /** Call `fn(ptr, len)` with `str` copied into module memory (freed afterwards). */
  _withStr(str, fn) {
    const x = this.x;
    const bytes = enc.encode(str);
    if (!bytes.length) throw new Error('empty string');
    const ptr = x.arm_alloc(bytes.length);
    if (!ptr) throw new Error(this._err());
    try {
      new Uint8Array(x.memory.buffer, ptr, bytes.length).set(bytes);
      return fn(ptr, bytes.length);
    } finally {
      x.arm_free(ptr, bytes.length);
    }
  }

  _ok(rc) {
    if (rc < 0) throw new Error(this._err());
    return rc;
  }

  /** @returns {string[]} names of the project's state machines. */
  machines() {
    const x = this.x;
    const out = [];
    for (let i = 0, n = x.arm_machine_count(this.h); i < n; i++) out.push(dec.decode(this._out(x.arm_machine_name(this.h, i))));
    return out;
  }

  /** Start (or restart) a state machine; inputs return to their defaults, time to 0. */
  startPlayer(name) {
    this._ok(this._withStr(name, (p, n) => this.x.arm_player_start(this.h, p, n)));
    return this;
  }

  stopPlayer() {
    this.x.arm_player_stop(this.h);
  }

  setBool(name, value) {
    this._ok(this._withStr(name, (p, n) => this.x.arm_player_set_bool(this.h, p, n, value ? 1 : 0)));
    return this;
  }

  setNumber(name, value) {
    this._ok(this._withStr(name, (p, n) => this.x.arm_player_set_number(this.h, p, n, value)));
    return this;
  }

  /** Fire a trigger input. */
  fire(name) {
    this._ok(this._withStr(name, (p, n) => this.x.arm_player_fire(this.h, p, n)));
    return this;
  }

  /** Publish an external value for data bindings, e.g. setValue('player.health', 0.5). */
  setValue(path, value) {
    this._ok(this._withStr(path, (p, n) => this.x.arm_player_set_value(this.h, p, n, value)));
    return this;
  }

  /** Advance the machine by dt seconds (0..10). */
  advance(dt) {
    this._ok(this.x.arm_player_advance(this.h, dt));
    return this;
  }

  /** @returns {string} current state name of a machine layer. */
  state(layer) {
    return dec.decode(this._out(this._ok(this._withStr(layer, (p, n) => this.x.arm_player_state(this.h, p, n)))));
  }

  /** @returns {{time:number,layer:string,kind:'entered'|'exited',state:string}[]} events since the last call. */
  events() {
    return JSON.parse(dec.decode(this._out(this.x.arm_player_events(this.h))));
  }

  /** Lay the project out for another canvas size in stage units (containers re-solve). */
  setCanvas(width, height) {
    this._ok(this.x.arm_player_set_canvas(this.h, width, height));
    return this;
  }

  /** @returns {{canvas:[number,number],scrolls:string[],lod:number}} layout state of the started player. */
  layout() {
    return JSON.parse(dec.decode(this._out(this._ok(this.x.arm_player_layout(this.h)))));
  }

  /** Pointer drag of a scroll region: the raw offset in pixels (content moved up / left). */
  scrollDrag(id, x, y) {
    this._ok(this._withStr(id, (p, n) => this.x.arm_player_scroll_drag(this.h, p, n, x, y)));
    return this;
  }

  /** Let go with a velocity in pixels per second; advance() then flings and settles. */
  scrollRelease(id, vx = 0, vy = 0) {
    this._ok(this._withStr(id, (p, n) => this.x.arm_player_scroll_release(this.h, p, n, vx, vy)));
    return this;
  }

  /** @returns {{offset:[number,number],max:[number,number],settled:boolean}} state of a scroll region. */
  scrollInfo(id) {
    return JSON.parse(dec.decode(this._out(this._ok(this._withStr(id, (p, n) => this.x.arm_player_scroll_info(this.h, p, n))))));
  }

  /** Screen size in pixels and the largest error accepted there: picks the level of detail. */
  setScreen(widthPx, heightPx, maxErrorPx = 0.5) {
    this._ok(this.x.arm_player_set_screen(this.h, widthPx, heightPx, maxErrorPx));
    return this;
  }

  /** SVG of the player's current pose. */
  playerSvg(showBones = false) {
    return dec.decode(this._out(this.x.arm_player_render_svg(this.h, showBones ? 1 : 0)));
  }

  /** @returns {{width:number,height:number,data:Uint8ClampedArray}} RGBA of the player's current pose. */
  playerRgba(scale = 1) {
    const len = this.x.arm_player_render_rgba(this.h, scale);
    const data = this._out(len);
    return { width: this.x.arm_out_width(this.h), height: this.x.arm_out_height(this.h), data: new Uint8ClampedArray(data.buffer) };
  }

  /**
   * Interactive loop on a canvas: advances the started machine by real time each animation frame and
   * draws it. `onTick(dt)` runs before each advance (set inputs there). Returns a stop function.
   */
  playMachine(canvas, { scale = 1, onTick, onError } = {}) {
    this.stop();
    const ctx = canvas.getContext('2d');
    const raf = (typeof requestAnimationFrame === 'function') ? requestAnimationFrame : (f) => setTimeout(() => f(Date.now()), 16);
    const caf = (typeof cancelAnimationFrame === 'function') ? cancelAnimationFrame : clearTimeout;
    let last = null;
    const step = (ts) => {
      try {
        const dt = last === null ? 0 : Math.min((ts - last) / 1000, 0.25);
        last = ts;
        if (onTick) onTick(dt);
        this.advance(dt);
        const { width, height, data } = this.playerRgba(scale);
        if (canvas.width !== width) canvas.width = width;
        if (canvas.height !== height) canvas.height = height;
        ctx.putImageData(new ImageData(data, width, height), 0, 0);
      } catch (e) { this._raf = null; if (onError) onError(e); else throw e; return; }
      this._raf = raf(step);
    };
    this._raf = raf(step);
    this._caf = caf;
    return () => this.stop();
  }

  /** Play on a canvas using requestAnimationFrame; loops. Returns a stop function. */
  play(canvas, { scale = 1, loop = true, onFrame, onError } = {}) {
    this.stop();
    const ctx = canvas.getContext('2d');
    const raf = (typeof requestAnimationFrame === 'function') ? requestAnimationFrame : (f) => setTimeout(() => f(Date.now()), 16);
    const caf = (typeof cancelAnimationFrame === 'function') ? cancelAnimationFrame : clearTimeout;
    let t0 = null;
    const step = (ts) => {
      try {
        if (t0 === null) t0 = ts;
        let f = ((ts - t0) / 1000) * this.fps;
        if (f >= this.frames) {
          if (!loop) { this._raf = null; return; }
          t0 = ts; f = 0;
        }
        const { width, height, data } = this.rgba(f, scale);
        if (canvas.width !== width) canvas.width = width;
        if (canvas.height !== height) canvas.height = height;
        ctx.putImageData(new ImageData(data, width, height), 0, 0);
        if (onFrame) onFrame(f);
      } catch (e) { this._raf = null; if (onError) onError(e); else throw e; return; }
      this._raf = raf(step);
    };
    this._raf = raf(step);
    this._caf = caf;
    return () => this.stop();
  }

  stop() {
    if (this._raf !== null && this._caf) this._caf(this._raf);
    this._raf = null;
  }

  close() {
    this.stop();
    if (this.h) this.x.arm_close(this.h);
    this.h = 0;
  }
}

export default Osatura;
