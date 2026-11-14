// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Studio UI infrastructure, pure and testable: command registry, shortcut model, panel layout model,
//! fuzzy command search and the persisted user configuration.
//!
//! Own design (clean-room): command ids, titles, default shortcuts, preset layouts and the scoring rules below
//! were chosen for this project; nothing is taken from any other tool's layout, names or key map.
//! The browser page (`studio.html`) gets the registry, the effective bindings and the presets from this module
//! over `/api/ui`, so the Rust side is the single source of truth.

use osatura_core::overlays::OverlaySettings;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

// ------------------------------------------------------------------------------------------------
// Contexts and chords
// ------------------------------------------------------------------------------------------------

/// Where a shortcut is active. `Global` is active everywhere and therefore overlaps with every context.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Context {
    Global,
    Canvas,
    Timeline,
}

impl Context {
    pub const ALL: [Context; 3] = [Context::Global, Context::Canvas, Context::Timeline];
    pub fn name(self) -> &'static str {
        match self {
            Context::Global => "global",
            Context::Canvas => "canvas",
            Context::Timeline => "timeline",
        }
    }
    pub fn parse(s: &str) -> Result<Context, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "global" => Ok(Context::Global),
            "canvas" => Ok(Context::Canvas),
            "timeline" => Ok(Context::Timeline),
            o => Err(format!("unknown context '{o}' (global, canvas, timeline)")),
        }
    }
    /// Two bindings in overlapping contexts can both fire for one key press.
    pub fn overlaps(self, other: Context) -> bool {
        self == other || self == Context::Global || other == Context::Global
    }
}

/// A key combination. `ctrl` means Control or the platform command key (the page folds both together).
#[derive(Clone, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct Chord {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub key: String,
}

const PUNCT: &str = "`-=[]\\;',./";

fn canon_key(tok: &str) -> Option<String> {
    let mut it = tok.chars();
    let (c, rest) = (it.next()?, it.next());
    if rest.is_none() {
        if c.is_ascii_alphabetic() {
            return Some(c.to_ascii_uppercase().to_string());
        }
        if c.is_ascii_digit() || PUNCT.contains(c) {
            return Some(c.to_string());
        }
        return None;
    }
    let l = tok.to_ascii_lowercase();
    if let Some(n) = l.strip_prefix('f') {
        if let Ok(v) = n.parse::<u32>() {
            if (1..=24).contains(&v) && v.to_string() == n {
                return Some(format!("F{v}"));
            }
        }
    }
    Some(
        match l.as_str() {
            "space" | "spacebar" => "Space",
            "enter" | "return" => "Enter",
            "escape" | "esc" => "Escape",
            "tab" => "Tab",
            "backspace" => "Backspace",
            "delete" | "del" => "Delete",
            "insert" | "ins" => "Insert",
            "home" => "Home",
            "end" => "End",
            "pageup" | "pgup" => "PageUp",
            "pagedown" | "pgdn" => "PageDown",
            "arrowleft" | "left" => "ArrowLeft",
            "arrowright" | "right" => "ArrowRight",
            "arrowup" | "up" => "ArrowUp",
            "arrowdown" | "down" => "ArrowDown",
            "comma" => ",",
            "period" | "dot" => ".",
            "slash" => "/",
            "minus" => "-",
            "equal" | "equals" => "=",
            _ => return None,
        }
        .to_string(),
    )
}

impl Chord {
    /// Parse `Ctrl+Shift+K`, `cmd+k`, `Alt+Left`, `Space`, `F5` ... into the canonical form.
    pub fn parse(s: &str) -> Result<Chord, String> {
        let s = s.trim();
        if s.is_empty() {
            return Err("empty shortcut".into());
        }
        let toks: Vec<&str> = s.split('+').map(str::trim).collect();
        if toks.iter().any(|t| t.is_empty()) {
            return Err(format!("bad shortcut '{s}' (empty part; the '+' key itself cannot be bound, use '='/'Plus' is not supported)"));
        }
        let (mods, key) = toks.split_at(toks.len() - 1);
        let mut c = Chord { ctrl: false, alt: false, shift: false, key: String::new() };
        for m in mods {
            let slot = match m.to_ascii_lowercase().as_str() {
                "ctrl" | "control" | "cmd" | "command" | "meta" | "mod" | "super" => &mut c.ctrl,
                "alt" | "option" | "opt" => &mut c.alt,
                "shift" => &mut c.shift,
                _ => return Err(format!("'{m}' in '{s}' is not a modifier (Ctrl, Alt, Shift)")),
            };
            if *slot {
                return Err(format!("modifier '{m}' repeated in '{s}'"));
            }
            *slot = true;
        }
        c.key = canon_key(key[0]).ok_or_else(|| format!("unknown key '{}' in '{s}'", key[0]))?;
        Ok(c)
    }
}

impl std::fmt::Display for Chord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.ctrl {
            f.write_str("Ctrl+")?;
        }
        if self.alt {
            f.write_str("Alt+")?;
        }
        if self.shift {
            f.write_str("Shift+")?;
        }
        f.write_str(&self.key)
    }
}

/// Chords the browser or window system keeps for itself; binding them would either not work or break the page.
pub const RESERVED: &[&str] = &["Ctrl+W", "Ctrl+T", "Ctrl+N", "Ctrl+Q", "Ctrl+R", "Ctrl+Tab", "Ctrl+Shift+Tab", "Ctrl+Shift+W", "Ctrl+Shift+N", "F5", "F11", "F12", "Alt+F4"];

/// `ctx:Chord` or just `Chord` (then `default_ctx` applies).
pub fn parse_binding(s: &str, default_ctx: Context) -> Result<(Context, Chord), String> {
    match s.split_once(':') {
        Some((c, k)) => Ok((Context::parse(c)?, Chord::parse(k)?)),
        None => Ok((default_ctx, Chord::parse(s)?)),
    }
}

pub fn binding_string(ctx: Context, c: &Chord) -> String {
    format!("{}:{}", ctx.name(), c)
}

// ------------------------------------------------------------------------------------------------
// Command registry
// ------------------------------------------------------------------------------------------------

pub struct Command {
    pub id: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    /// default bindings: (context, chord)
    pub defaults: &'static [(Context, &'static str)],
}

impl Command {
    /// The context a newly recorded shortcut for this command gets.
    pub fn primary_context(&self) -> Context {
        self.defaults.first().map(|d| d.0).unwrap_or(Context::Global)
    }
}

use Context::{Canvas as C, Global as G, Timeline as T};

macro_rules! cmd {
    ($id:expr, $cat:expr, $title:expr, $desc:expr, [$($d:expr),*]) => {
        Command { id: $id, title: $title, category: $cat, description: $desc, defaults: &[$($d),*] }
    };
}

pub const COMMANDS: &[Command] = &[
    cmd!("edit.undo", "Edit", "Undo", "Undo the last edit", [(G, "Ctrl+Z")]),
    cmd!("edit.redo", "Edit", "Redo", "Redo the last undone edit", [(G, "Ctrl+Shift+Z"), (G, "Ctrl+Y")]),
    cmd!("file.save", "Edit", "Save project", "Write the project to its file", [(G, "Ctrl+S")]),
    cmd!("select.all", "Selection", "Select all layers", "Select every unlocked layer", [(C, "Ctrl+A")]),
    cmd!("select.clear", "Selection", "Clear selection", "Cancel a drag, or clear the canvas selection", [(C, "Escape")]),
    cmd!("select.delete", "Selection", "Delete selection", "Remove the selected layers", [(C, "Delete"), (C, "Backspace")]),
    cmd!("select.lock_toggle", "Selection", "Lock or unlock selection", "Locked layers are skipped by clicks and boxes", [(C, "Alt+L")]),
    cmd!("canvas.nudge_left", "Canvas", "Nudge left", "Move the selection 1 unit left", [(C, "ArrowLeft")]),
    cmd!("canvas.nudge_right", "Canvas", "Nudge right", "Move the selection 1 unit right", [(C, "ArrowRight")]),
    cmd!("canvas.nudge_up", "Canvas", "Nudge up", "Move the selection 1 unit up", [(C, "ArrowUp")]),
    cmd!("canvas.nudge_down", "Canvas", "Nudge down", "Move the selection 1 unit down", [(C, "ArrowDown")]),
    cmd!("canvas.nudge_left_10", "Canvas", "Nudge left by 10", "Move the selection 10 units left", [(C, "Shift+ArrowLeft")]),
    cmd!("canvas.nudge_right_10", "Canvas", "Nudge right by 10", "Move the selection 10 units right", [(C, "Shift+ArrowRight")]),
    cmd!("canvas.nudge_up_10", "Canvas", "Nudge up by 10", "Move the selection 10 units up", [(C, "Shift+ArrowUp")]),
    cmd!("canvas.nudge_down_10", "Canvas", "Nudge down by 10", "Move the selection 10 units down", [(C, "Shift+ArrowDown")]),
    cmd!("canvas.toggle_bones", "Canvas", "Toggle bone display", "Show or hide bones on the picture", [(G, "Alt+B")]),
    cmd!("canvas.toggle_points", "Canvas", "Toggle point editing", "Show and drag control points of the selected layers", [(G, "Alt+P")]),
    cmd!("canvas.toggle_snap", "Canvas", "Toggle object snapping", "Snap drags to other layers and the canvas edges", [(G, "Alt+S")]),
    cmd!("canvas.toggle_whole_groups", "Canvas", "Toggle whole-group picking", "A click selects the top-level group", [(G, "Alt+G")]),
    cmd!("canvas.grid_cycle", "Canvas", "Cycle snap grid", "Grid spacing: off, 8, 16, 32", []),
    cmd!("anim.toggle_autokey", "Animation", "Toggle auto-key", "A drag keys the edited properties at the current frame", [(G, "Alt+K")]),
    cmd!("anim.add_key", "Animation", "Add key at playhead", "Key the focused timeline channel, or the selected bone, at the current frame", [(C, "I"), (T, "I")]),
    cmd!("playback.toggle", "Playback", "Play / pause", "Start or stop playback", [(G, "Space")]),
    cmd!("playback.first", "Playback", "Go to first frame", "Jump to the start of the range", [(G, "Home")]),
    cmd!("playback.last", "Playback", "Go to last frame", "Jump to the last frame", [(G, "End")]),
    cmd!("playback.step_next", "Playback", "Next frame", "Step one frame forward", [(G, "."), (T, "ArrowRight")]),
    cmd!("playback.step_prev", "Playback", "Previous frame", "Step one frame back", [(G, ","), (T, "ArrowLeft")]),
    cmd!("playback.step_next_10", "Playback", "Forward 10 frames", "Step ten frames forward", [(G, "Shift+."), (T, "Shift+ArrowRight")]),
    cmd!("playback.step_prev_10", "Playback", "Back 10 frames", "Step ten frames back", [(G, "Shift+,"), (T, "Shift+ArrowLeft")]),
    cmd!("playback.toggle_loop", "Playback", "Toggle loop", "Wrap around at the end instead of stopping", [(G, "Alt+O")]),
    cmd!("playback.toggle_range", "Playback", "Toggle play range", "Play only between in and out", [(G, "Alt+R")]),
    cmd!("playback.set_in", "Playback", "Set range start", "Range start = playhead", [(G, "[")]),
    cmd!("playback.set_out", "Playback", "Set range end", "Range end = playhead", [(G, "]")]),
    cmd!("timeline.select_all", "Timeline", "Select all keys", "Select every unlocked key", [(T, "Ctrl+A")]),
    cmd!("timeline.deselect", "Timeline", "Deselect keys", "Clear the key selection", [(T, "Escape")]),
    cmd!("timeline.delete_keys", "Timeline", "Delete selected keys", "Remove the selected keys", [(T, "Delete"), (T, "Backspace")]),
    cmd!("timeline.copy", "Timeline", "Copy keys", "Copy the selected keys", [(T, "Ctrl+C")]),
    cmd!("timeline.paste", "Timeline", "Paste keys", "Paste at the playhead", [(T, "Ctrl+V")]),
    cmd!("timeline.duplicate", "Timeline", "Duplicate keys", "Duplicate the selected keys after themselves", [(T, "Ctrl+D")]),
    cmd!("timeline.key_earlier", "Timeline", "Move keys one frame earlier", "Shift the selected keys by -1 frame", [(T, "Alt+ArrowLeft")]),
    cmd!("timeline.key_later", "Timeline", "Move keys one frame later", "Shift the selected keys by +1 frame", [(T, "Alt+ArrowRight")]),
    cmd!("timeline.key_up", "Timeline", "Raise keys", "Raise the selected keys by 1% of the visible range", [(T, "Alt+ArrowUp")]),
    cmd!("timeline.key_down", "Timeline", "Lower keys", "Lower the selected keys by 1% of the visible range", [(T, "Alt+ArrowDown")]),
    cmd!("timeline.zoom_in", "Timeline", "Zoom in", "Zoom the timeline in", [(T, "="), (G, "Alt+=")]),
    cmd!("timeline.zoom_out", "Timeline", "Zoom out", "Zoom the timeline out", [(T, "-"), (G, "Alt+-")]),
    cmd!("timeline.zoom_fit", "Timeline", "Zoom to fit", "Fit the whole clip", [(T, "F")]),
    cmd!("timeline.toggle_view", "Timeline", "Switch dope sheet / curve graph", "Switch the dock view", [(G, "Alt+V")]),
    cmd!("timeline.toggle_snap", "Timeline", "Toggle key snapping", "Round moved keys to whole frames", [(G, "Alt+Shift+S")]),
    cmd!("timeline.toggle_shy", "Timeline", "Show or hide shy channels", "Channels marked shy are normally hidden", []),
    cmd!("timeline.apply_ease", "Timeline", "Apply chosen interpolation", "Set the interpolation of the selected keys", []),
    cmd!("timeline.smooth", "Timeline", "Auto-smooth keys", "Bake overshoot-free tangents into the selected keys", []),
    cmd!("timeline.ripple_insert", "Timeline", "Insert frames at playhead", "Ripple insert", []),
    cmd!("timeline.ripple_delete", "Timeline", "Remove frames at playhead", "Ripple delete", []),
    cmd!("panel.toggle.timeline", "Panels", "Toggle Timeline panel", "Show, hide or expand the timeline panel", [(G, "Alt+T")]),
    cmd!("panel.toggle.layers", "Panels", "Toggle Layers panel", "Show, hide or expand the layers panel", [(G, "Alt+1")]),
    cmd!("panel.toggle.selection", "Panels", "Toggle Selection panel", "Show, hide or expand the selection panel", [(G, "Alt+2")]),
    cmd!("panel.toggle.bones", "Panels", "Toggle Bones panel", "Show, hide or expand the bones panel", [(G, "Alt+3")]),
    cmd!("panel.toggle.live", "Panels", "Toggle Live input panel", "Show, hide or expand the live input panel", [(G, "Alt+4")]),
    cmd!("panel.toggle.face", "Panels", "Toggle Webcam face panel", "Show, hide or expand the face panel", [(G, "Alt+5")]),
    cmd!("panel.toggle.json", "Panels", "Toggle Project JSON panel", "Show, hide or expand the JSON panel", [(G, "Alt+6")]),
    cmd!("layout.reset", "Panels", "Reset layout", "Back to the default arrangement", []),
    cmd!("layout.preset.animate", "Panels", "Layout: animate", "Big timeline, layers and bones at the side", []),
    cmd!("layout.preset.rig", "Panels", "Layout: rig", "Tall side panels, timeline folded", []),
    cmd!("layout.preset.compact", "Panels", "Layout: compact", "Canvas and timeline, everything else in one tab group", []),
    cmd!("layout.preset.live", "Panels", "Layout: live", "Live input and face panels beside the canvas", []),
    cmd!("demo.panel", "Demo", "Guided demo", "Open the guided step-by-step demo that builds the walking person with the real commands", []),
    cmd!("suggest.panel", "Suggest", "Pose suggestions", "Open the pose suggestion panel (in-between, completion, next pose; learned from this project)", [(G, "Alt+8")]),
    cmd!("overlay.panel", "Overlays", "Overlay settings", "Open the onion skin, motion trail and scrub proxy settings", [(G, "Alt+7")]),
    cmd!("overlay.onion_toggle", "Overlays", "Toggle onion skin", "Show ghosts of neighbouring frames", [(G, "Alt+N")]),
    cmd!("overlay.trails_toggle", "Overlays", "Toggle motion trails", "Show the path of the selected layers, bones and points", [(G, "Alt+M")]),
    cmd!("overlay.onion_before_more", "Overlays", "Onion skin: one more past ghost", "Increase the number of ghosts before the playhead", []),
    cmd!("overlay.onion_before_less", "Overlays", "Onion skin: one less past ghost", "Decrease the number of ghosts before the playhead", []),
    cmd!("overlay.onion_after_more", "Overlays", "Onion skin: one more future ghost", "Increase the number of ghosts after the playhead", []),
    cmd!("overlay.onion_after_less", "Overlays", "Onion skin: one less future ghost", "Decrease the number of ghosts after the playhead", []),
    cmd!("overlay.onion_keyed_toggle", "Overlays", "Onion skin: key frames or fixed steps", "Ghost the neighbouring key frames instead of frames at a fixed step", []),
    cmd!("overlay.onion_selection_toggle", "Overlays", "Onion skin: selection only", "Ghost only the selected layers", []),
    cmd!("overlay.onion_outline_toggle", "Overlays", "Onion skin: outlines or silhouettes", "Draw ghosts as hairline outlines", []),
    cmd!("overlay.trails_smooth_toggle", "Overlays", "Motion trails: smooth curve", "Draw the trail as a spline through the frame positions", []),
    cmd!("overlay.proxy_toggle", "Overlays", "Toggle scrub proxy", "Use a fast low-quality picture while the playhead is dragged", [(G, "Alt+Q")]),
    cmd!("overlay.proxy_level_cycle", "Overlays", "Cycle scrub proxy quality", "Proxy resolution 1/2, 1/4, 1/8, 1/16", []),
    cmd!("export.gif", "Export", "Export GIF", "Download the animation as a GIF", []),
    cmd!("export.html", "Export", "Export HTML player", "Download a self-contained player page", []),
    cmd!("export.lottie", "Export", "Export Lottie JSON", "Download the animation as Lottie JSON", []),
    cmd!("ui.palette", "Interface", "Command palette", "Search and run any command", [(G, "Ctrl+K"), (G, "Ctrl+Shift+P")]),
    cmd!("ui.shortcuts", "Interface", "Keyboard shortcuts", "View and change the shortcuts", [(G, "Ctrl+,")]),
];

pub fn command(id: &str) -> Option<&'static Command> {
    COMMANDS.iter().find(|c| c.id == id)
}

// ------------------------------------------------------------------------------------------------
// Shortcut model
// ------------------------------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Binding {
    pub command: String,
    pub context: Context,
    pub chord: Chord,
    pub user: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Conflict {
    pub chord: String,
    pub a: (String, Context),
    pub b: (String, Context),
}

fn default_strings(c: &Command) -> Vec<String> {
    c.defaults.iter().filter_map(|(x, k)| Chord::parse(k).ok().map(|ch| binding_string(*x, &ch))).collect()
}

/// Parse and normalise a user override list for one command (errors name the command).
fn parse_list(c: &Command, list: &[String]) -> Result<Vec<(Context, Chord)>, Vec<String>> {
    let mut out: Vec<(Context, Chord)> = Vec::new();
    let mut errs = Vec::new();
    for s in list {
        match parse_binding(s, c.primary_context()) {
            Ok(b) => {
                if RESERVED.iter().any(|r| Chord::parse(r).ok().as_ref() == Some(&b.1)) {
                    errs.push(format!("{}: '{}' is reserved by the browser", c.id, b.1));
                } else if out.iter().any(|o| o.0.overlaps(b.0) && o.1 == b.1) {
                    errs.push(format!("{}: '{}' is listed twice", c.id, b.1));
                } else {
                    out.push(b);
                }
            }
            Err(e) => errs.push(format!("{}: {e}", c.id)),
        }
    }
    if errs.is_empty() {
        Ok(out)
    } else {
        Err(errs)
    }
}

/// Effective bindings in registry order: defaults, replaced wholesale by an override entry for that command.
pub fn effective_bindings(overrides: &BTreeMap<String, Vec<String>>) -> Result<Vec<Binding>, Vec<String>> {
    let mut errs = Vec::new();
    for k in overrides.keys() {
        if command(k).is_none() {
            errs.push(format!("unknown command '{k}'"));
        }
    }
    let mut out = Vec::new();
    for c in COMMANDS {
        match overrides.get(c.id) {
            Some(list) => match parse_list(c, list) {
                Ok(v) => out.extend(v.into_iter().map(|(x, ch)| Binding { command: c.id.into(), context: x, chord: ch, user: true })),
                Err(e) => errs.extend(e),
            },
            None => {
                for (x, k) in c.defaults {
                    if let Ok(ch) = Chord::parse(k) {
                        out.push(Binding { command: c.id.into(), context: *x, chord: ch, user: false });
                    }
                }
            }
        }
    }
    if errs.is_empty() {
        Ok(out)
    } else {
        Err(errs)
    }
}

/// Pairs of different commands that would both fire for one key press.
pub fn conflicts(b: &[Binding]) -> Vec<Conflict> {
    let mut out = Vec::new();
    for i in 0..b.len() {
        for j in i + 1..b.len() {
            if b[i].command != b[j].command && b[i].chord == b[j].chord && b[i].context.overlaps(b[j].context) {
                out.push(Conflict { chord: b[i].chord.to_string(), a: (b[i].command.clone(), b[i].context), b: (b[j].command.clone(), b[j].context) });
            }
        }
    }
    out
}

/// What a key press does in `ctx`: a binding of that context wins over a global one; ties go to registry order.
///
/// Test oracle only: the studio page does not call into Rust for dispatch, it re-implements this rule in
/// `studio.html` (`UI.bind` is a map `ctx:Chord -> command`, first registry entry wins; `onKey` looks up
/// `ctx:Chord`, then `global:Chord`). `page_dispatch_matches_the_rust_resolver` runs that JavaScript under node and
/// compares it with this function for every default chord and context, so the two cannot drift apart unnoticed.
#[cfg(test)]
pub fn resolve<'a>(b: &'a [Binding], ctx: Context, chord: &Chord) -> Option<&'a Binding> {
    b.iter().find(|x| x.context == ctx && &x.chord == chord).or_else(|| b.iter().find(|x| x.context == Context::Global && &x.chord == chord))
}

/// Validate an override map and return it normalised (`ctx:Chord`, entries equal to the defaults dropped).
/// Conflicts between commands are errors, so the stored configuration is never ambiguous.
pub fn normalise_shortcuts(overrides: &BTreeMap<String, Vec<String>>) -> Result<BTreeMap<String, Vec<String>>, Vec<String>> {
    let eff = effective_bindings(overrides)?;
    let cf = conflicts(&eff);
    if !cf.is_empty() {
        return Err(cf.iter().map(|c| format!("{} conflicts: '{}' ({}) and '{}' ({}) share {}", c.chord, c.a.0, c.a.1.name(), c.b.0, c.b.1.name(), c.chord)).collect());
    }
    let mut out = BTreeMap::new();
    for (id, list) in overrides {
        let c = command(id).ok_or_else(|| vec![format!("unknown command '{id}'")])?;
        let norm: Vec<String> = parse_list(c, list)?.iter().map(|(x, k)| binding_string(*x, k)).collect();
        let a: BTreeSet<&String> = norm.iter().collect();
        let defs = default_strings(c);
        let d: BTreeSet<&String> = defs.iter().collect();
        if a != d {
            out.insert(id.clone(), norm);
        }
    }
    Ok(out)
}

// ------------------------------------------------------------------------------------------------
// Fuzzy command search
// ------------------------------------------------------------------------------------------------

const MAX_Q: usize = 32;
const MAX_T: usize = 120;

fn is_boundary(t: &[char], j: usize) -> bool {
    if j == 0 {
        return true;
    }
    let (p, c) = (t[j - 1], t[j]);
    !p.is_alphanumeric() || (p.is_lowercase() && c.is_uppercase())
}

/// Best subsequence alignment of `query` in `text` (case-insensitive, whitespace in the query ignored).
/// Rewards a match at the start of the text, at word starts and runs of adjacent characters; penalises gaps.
/// Integer arithmetic only, so the ranking is identical on every machine. Returns score and matched char indices.
pub fn fuzzy_score(query: &str, text: &str) -> Option<(i32, Vec<usize>)> {
    let q: Vec<char> = query.chars().filter(|c| !c.is_whitespace()).flat_map(|c| c.to_lowercase()).take(MAX_Q).collect();
    if q.is_empty() {
        return Some((0, vec![]));
    }
    let orig: Vec<char> = text.chars().take(MAX_T).collect();
    let low: Vec<char> = orig.iter().map(|c| c.to_lowercase().next().unwrap_or(*c)).collect();
    let (m, n) = (q.len(), low.len());
    if m > n {
        return None;
    }
    const NEG: i32 = i32::MIN / 2;
    // best[i][j]: best score with q[i] matched at text position j; from[i][j]: previous match position
    let mut best = vec![vec![NEG; n]; m];
    let mut from = vec![vec![usize::MAX; n]; m];
    let base = |j: usize| -> i32 { 10 + if j == 0 { 12 } else if is_boundary(&orig, j) { 8 } else { 0 } };
    for j in 0..n {
        if low[j] == q[0] {
            best[0][j] = base(j) - if is_boundary(&orig, j) { 0 } else { (j as i32).min(6) };
        }
    }
    for i in 1..m {
        for j in i..n {
            if low[j] != q[i] {
                continue;
            }
            for p in i - 1..j {
                if best[i - 1][p] == NEG {
                    continue;
                }
                let gap = (j - p - 1) as i32;
                let trans = if gap == 0 { 6 } else { -gap.min(4) };
                let s = best[i - 1][p] + trans + base(j);
                if s > best[i][j] {
                    best[i][j] = s;
                    from[i][j] = p;
                }
            }
        }
    }
    let (mut bj, mut bs) = (usize::MAX, NEG);
    for (j, &sc) in best[m - 1].iter().enumerate().take(n) {
        if sc > bs {
            bs = sc;
            bj = j;
        }
    }
    if bj == usize::MAX {
        return None;
    }
    let mut pos = vec![0usize; m];
    let mut j = bj;
    for i in (0..m).rev() {
        pos[i] = j;
        if i > 0 {
            j = from[i][j];
        }
    }
    let joined: String = low.iter().collect();
    let qs: String = q.iter().collect();
    if joined == qs {
        bs += 60;
    } else if joined.starts_with(&qs) {
        bs += 30;
    } else if pos.windows(2).all(|w| w[1] == w[0] + 1) {
        bs += 8;
    }
    Some((bs, pos))
}

#[derive(Clone, PartialEq, Debug)]
pub struct Hit {
    pub id: &'static str,
    pub score: i32,
    /// matched character indices in the title (empty when the match came from the category or id)
    pub positions: Vec<usize>,
}

/// Rank the commands for `query`. An empty query lists recents first (most recent first), then everything in
/// registry order. Ties are broken by recency, then title, then id.
pub fn search(query: &str, recents: &[String], limit: usize) -> Vec<Hit> {
    let rank = |id: &str| recents.iter().position(|r| r == id);
    let mut hits: Vec<Hit> = Vec::new();
    if query.chars().all(char::is_whitespace) {
        for r in recents {
            if let Some(c) = command(r) {
                if !hits.iter().any(|h| h.id == c.id) {
                    hits.push(Hit { id: c.id, score: 0, positions: vec![] });
                }
            }
        }
        for c in COMMANDS {
            if !hits.iter().any(|h| h.id == c.id) {
                hits.push(Hit { id: c.id, score: 0, positions: vec![] });
            }
        }
        hits.truncate(limit);
        return hits;
    }
    for c in COMMANDS {
        let mut best: Option<(i32, Vec<usize>)> = fuzzy_score(query, c.title);
        let cat = format!("{} {}", c.category, c.title);
        if let Some((s, _)) = fuzzy_score(query, &cat) {
            if best.as_ref().is_none_or(|b| s - 8 > b.0) {
                best = Some((s - 8, vec![]));
            }
        }
        if let Some((s, _)) = fuzzy_score(query, c.id) {
            if best.as_ref().is_none_or(|b| s - 12 > b.0) {
                best = Some((s - 12, vec![]));
            }
        }
        if let Some((s, p)) = best {
            let bonus = rank(c.id).map_or(0, |r| (12 - 2 * r as i32).max(0));
            hits.push(Hit { id: c.id, score: s + bonus, positions: p });
        }
    }
    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| rank(a.id).unwrap_or(usize::MAX).cmp(&rank(b.id).unwrap_or(usize::MAX)))
            .then_with(|| command(a.id).map(|c| c.title).cmp(&command(b.id).map(|c| c.title)))
            .then_with(|| a.id.cmp(b.id))
    });
    hits.truncate(limit);
    hits
}

// ------------------------------------------------------------------------------------------------
// Layout model
// ------------------------------------------------------------------------------------------------

pub struct PanelDef {
    pub id: &'static str,
    pub title: &'static str,
    pub min_w: f32,
    pub min_h: f32,
    pub closable: bool,
}

pub const PANELS: &[PanelDef] = &[
    PanelDef { id: "stage", title: "Canvas", min_w: 240.0, min_h: 160.0, closable: false },
    PanelDef { id: "selection", title: "Selection", min_w: 200.0, min_h: 80.0, closable: true },
    PanelDef { id: "layers", title: "Layers", min_w: 160.0, min_h: 80.0, closable: true },
    PanelDef { id: "bones", title: "Bones", min_w: 180.0, min_h: 80.0, closable: true },
    PanelDef { id: "live", title: "Live input", min_w: 200.0, min_h: 100.0, closable: true },
    PanelDef { id: "face", title: "Webcam face", min_w: 200.0, min_h: 100.0, closable: true },
    PanelDef { id: "json", title: "Project JSON", min_w: 200.0, min_h: 100.0, closable: true },
    PanelDef { id: "timeline", title: "Timeline", min_w: 320.0, min_h: 160.0, closable: true },
];

pub fn panel(id: &str) -> Option<&'static PanelDef> {
    PANELS.iter().find(|p| p.id == id)
}

/// Height of a tab bar and width of a splitter in CSS pixels (used for minimum sizes).
pub const TAB_H: f32 = 28.0;
pub const SPLIT_W: f32 = 5.0;
pub const MIN_RATIO: f32 = 0.05;
const MAX_DEPTH: usize = 12;
const MAX_FLOAT: f32 = 10_000.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Dir {
    /// children side by side; `ratio` is the width share of `a`
    #[serde(rename = "h")]
    H,
    /// children stacked; `ratio` is the height share of `a`
    #[serde(rename = "v")]
    V,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum Node {
    Split { dir: Dir, ratio: f32, a: Box<Node>, b: Box<Node> },
    Tabs {
        panels: Vec<String>,
        #[serde(default)]
        active: usize,
        #[serde(default)]
        collapsed: bool,
    },
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Floating {
    pub panel: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub version: u32,
    pub root: Node,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub floating: Vec<Floating>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hidden: Vec<String>,
}

fn tabs(p: &[&str]) -> Node {
    Node::Tabs { panels: p.iter().map(|s| s.to_string()).collect(), active: 0, collapsed: false }
}
fn folded(p: &[&str]) -> Node {
    Node::Tabs { panels: p.iter().map(|s| s.to_string()).collect(), active: 0, collapsed: true }
}
fn split(dir: Dir, ratio: f32, a: Node, b: Node) -> Node {
    Node::Split { dir, ratio, a: Box::new(a), b: Box::new(b) }
}
fn layout(root: Node, hidden: &[&str]) -> Layout {
    Layout { version: 1, root, floating: vec![], hidden: hidden.iter().map(|s| s.to_string()).collect() }
}

pub const PRESET_NAMES: &[&str] = &["animate", "rig", "compact", "live"];

/// The built-in arrangements. The first one is the default.
pub fn preset(name: &str) -> Option<Layout> {
    use Dir::{H, V};
    Some(match name {
        "animate" => layout(
            split(V, 0.62, split(H, 0.74, tabs(&["stage"]), tabs(&["layers", "selection", "bones", "live", "face", "json"])), tabs(&["timeline"])),
            &[],
        ),
        "rig" => layout(
            split(H, 0.66, split(V, 0.84, tabs(&["stage"]), folded(&["timeline"])), split(V, 0.5, tabs(&["layers", "bones"]), tabs(&["selection", "json"]))),
            &["live", "face"],
        ),
        "compact" => layout(split(V, 0.62, tabs(&["stage"]), tabs(&["timeline", "layers", "selection", "bones", "live", "face", "json"])), &[]),
        "live" => layout(
            split(H, 0.58, split(V, 0.7, tabs(&["stage"]), tabs(&["timeline"])), tabs(&["live", "face", "bones", "selection"])),
            &["layers", "json"],
        ),
        _ => return None,
    })
}

pub fn default_layout() -> Layout {
    preset(PRESET_NAMES[0]).expect("builtin preset")
}

impl Node {
    fn collect<'a>(&'a self, out: &mut Vec<&'a str>) {
        match self {
            Node::Tabs { panels, .. } => out.extend(panels.iter().map(String::as_str)),
            Node::Split { a, b, .. } => {
                a.collect(out);
                b.collect(out);
            }
        }
    }

    fn check(&self, depth: usize, errs: &mut Vec<String>) {
        if depth > MAX_DEPTH {
            errs.push(format!("layout nested deeper than {MAX_DEPTH}"));
            return;
        }
        match self {
            Node::Split { ratio, a, b, .. } => {
                if !ratio.is_finite() || *ratio < MIN_RATIO || *ratio > 1.0 - MIN_RATIO {
                    errs.push(format!("split ratio {ratio} is outside {MIN_RATIO}..{}", 1.0 - MIN_RATIO));
                }
                a.check(depth + 1, errs);
                b.check(depth + 1, errs);
            }
            Node::Tabs { panels, active, collapsed } => {
                if panels.is_empty() {
                    errs.push("empty tab group".into());
                }
                if !panels.is_empty() && *active >= panels.len() {
                    errs.push(format!("active tab {active} out of range for a group of {}", panels.len()));
                }
                if *collapsed && panels.iter().any(|p| p == "stage") {
                    errs.push("the group holding the canvas cannot be collapsed".into());
                }
            }
        }
    }

    /// Smallest size the node can be laid out in, from the panels' minimums plus tab bars and splitters.
    pub fn min_size(&self) -> (f32, f32) {
        match self {
            Node::Tabs { panels, collapsed, .. } => {
                let (mut w, mut h) = (TAB_H, 0.0f32);
                for p in panels.iter().filter_map(|p| panel(p)) {
                    w = w.max(p.min_w);
                    h = h.max(p.min_h);
                }
                if *collapsed {
                    (TAB_H, TAB_H)
                } else {
                    (w, h + TAB_H)
                }
            }
            Node::Split { dir, a, b, .. } => {
                let (aw, ah) = a.min_size();
                let (bw, bh) = b.min_size();
                match dir {
                    Dir::H => (aw + bw + SPLIT_W, ah.max(bh)),
                    Dir::V => (aw.max(bw), ah + bh + SPLIT_W),
                }
            }
        }
    }
}

impl Layout {
    /// All problems at once. A valid layout places every panel exactly once (tree, floating or hidden)
    /// and keeps the canvas in the tree.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errs = Vec::new();
        if self.version != 1 {
            errs.push(format!("unsupported layout version {}", self.version));
        }
        self.root.check(0, &mut errs);
        let mut seen: Vec<&str> = Vec::new();
        let mut in_tree: Vec<&str> = Vec::new();
        self.root.collect(&mut in_tree);
        for p in &in_tree {
            if panel(p).is_none() {
                errs.push(format!("unknown panel '{p}'"));
            }
        }
        seen.extend(&in_tree);
        for f in &self.floating {
            match panel(&f.panel) {
                None => errs.push(format!("unknown floating panel '{}'", f.panel)),
                Some(d) => {
                    if !d.closable {
                        errs.push(format!("panel '{}' cannot float", d.id));
                    }
                    if ![f.x, f.y, f.w, f.h].iter().all(|v| v.is_finite()) || f.x.abs() > MAX_FLOAT || f.y.abs() > MAX_FLOAT || f.w > MAX_FLOAT || f.h > MAX_FLOAT {
                        errs.push(format!("floating '{}' has a bad position or size", f.panel));
                    } else if f.w < d.min_w || f.h < d.min_h {
                        errs.push(format!("floating '{}' is smaller than its minimum {}x{}", f.panel, d.min_w, d.min_h));
                    }
                }
            }
            seen.push(&f.panel);
        }
        for h in &self.hidden {
            match panel(h) {
                None => errs.push(format!("unknown hidden panel '{h}'")),
                Some(d) if !d.closable => errs.push(format!("panel '{h}' cannot be hidden")),
                _ => {}
            }
            seen.push(h);
        }
        let mut count: BTreeMap<&str, usize> = BTreeMap::new();
        for s in &seen {
            *count.entry(s).or_default() += 1;
        }
        for (p, n) in &count {
            if *n > 1 {
                errs.push(format!("panel '{p}' appears {n} times"));
            }
        }
        for d in PANELS {
            if !count.contains_key(d.id) {
                errs.push(format!("panel '{}' is missing (put it in a group or in hidden)", d.id));
            }
        }
        if !in_tree.contains(&"stage") {
            errs.push("the canvas must stay docked in the layout".into());
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }

    /// Check that the layout can be drawn at all in a window of `w` x `h` pixels.
    pub fn fits(&self, w: f32, h: f32) -> Result<(), String> {
        let (mw, mh) = self.root.min_size();
        if mw > w || mh > h {
            Err(format!("layout needs at least {mw:.0}x{mh:.0}, window is {w:.0}x{h:.0}"))
        } else {
            Ok(())
        }
    }

    /// Bring a stored layout into a valid shape (panels added by a newer version, removed ones, bad numbers).
    /// Falls back to the default layout when the canvas itself is lost. Never fails.
    pub fn repaired(&self) -> Layout {
        fn walk(n: &Node, seen: &mut BTreeSet<String>, depth: usize) -> Option<Node> {
            if depth > MAX_DEPTH {
                return None;
            }
            match n {
                Node::Tabs { panels, active, collapsed } => {
                    let mut keep: Vec<String> = Vec::new();
                    for p in panels {
                        if panel(p).is_some() && seen.insert(p.clone()) {
                            keep.push(p.clone());
                        }
                    }
                    if keep.is_empty() {
                        return None;
                    }
                    let act = (*active).min(keep.len() - 1);
                    let col = *collapsed && !keep.iter().any(|p| p == "stage");
                    Some(Node::Tabs { panels: keep, active: act, collapsed: col })
                }
                Node::Split { dir, ratio, a, b } => {
                    let (na, nb) = (walk(a, seen, depth + 1), walk(b, seen, depth + 1));
                    match (na, nb) {
                        (Some(x), Some(y)) => {
                            let r = if ratio.is_finite() { ratio.clamp(MIN_RATIO, 1.0 - MIN_RATIO) } else { 0.5 };
                            Some(split(*dir, r, x, y))
                        }
                        (Some(x), None) | (None, Some(x)) => Some(x),
                        (None, None) => None,
                    }
                }
            }
        }
        let mut seen = BTreeSet::new();
        let root = match walk(&self.root, &mut seen, 0) {
            Some(r) if seen.contains("stage") => r,
            _ => return default_layout(),
        };
        let mut floating = Vec::new();
        for f in &self.floating {
            if let Some(d) = panel(&f.panel) {
                if d.closable && !seen.contains(&f.panel) && [f.x, f.y, f.w, f.h].iter().all(|v| v.is_finite()) {
                    seen.insert(f.panel.clone());
                    floating.push(Floating { panel: f.panel.clone(), x: f.x.clamp(-MAX_FLOAT, MAX_FLOAT), y: f.y.clamp(-MAX_FLOAT, MAX_FLOAT), w: f.w.clamp(d.min_w, MAX_FLOAT), h: f.h.clamp(d.min_h, MAX_FLOAT) });
                }
            }
        }
        let mut hidden = Vec::new();
        for h in self.hidden.iter().map(String::as_str).chain(PANELS.iter().map(|p| p.id)) {
            if panel(h).is_some_and(|d| d.closable) && seen.insert(h.to_string()) {
                hidden.push(h.to_string());
            }
        }
        Layout { version: 1, root, floating, hidden }
    }

    /// One line per group for the CLI.
    pub fn summary(&self) -> String {
        fn s(n: &Node) -> String {
            match n {
                Node::Tabs { panels, active, collapsed } => format!("[{}{}{}]", panels.iter().enumerate().map(|(i, p)| if i == *active { format!("*{p}") } else { p.clone() }).collect::<Vec<_>>().join(" "), if *collapsed { " (folded)" } else { "" }, ""),
                Node::Split { dir, ratio, a, b } => format!("{}{:.2}({} | {})", if *dir == Dir::H { "H" } else { "V" }, ratio, s(a), s(b)),
            }
        }
        let mut t = s(&self.root);
        if !self.floating.is_empty() {
            t += &format!("  floating: {}", self.floating.iter().map(|f| f.panel.as_str()).collect::<Vec<_>>().join(","));
        }
        if !self.hidden.is_empty() {
            t += &format!("  hidden: {}", self.hidden.join(","));
        }
        t
    }
}

// ------------------------------------------------------------------------------------------------
// Persisted user configuration
// ------------------------------------------------------------------------------------------------

pub const MAX_RECENTS: usize = 12;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiConfig {
    #[serde(default = "one")]
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,
    /// command id -> bindings (`ctx:Chord`), replacing that command's defaults; empty list = unbound
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub shortcuts: BTreeMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recents: Vec<String>,
    /// onion skin, motion trail and scrub proxy settings (`osatura_core::overlays`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overlays: Option<OverlaySettings>,
}

fn one() -> u32 {
    1
}

impl Default for UiConfig {
    fn default() -> Self {
        UiConfig { version: 1, layout: None, shortcuts: BTreeMap::new(), recents: Vec::new(), overlays: None }
    }
}

impl UiConfig {
    pub fn validate(&self) -> Result<UiConfig, Vec<String>> {
        let mut errs = Vec::new();
        if self.version != 1 {
            errs.push(format!("unsupported config version {}", self.version));
        }
        if let Some(l) = &self.layout {
            if let Err(e) = l.validate() {
                errs.extend(e.into_iter().map(|x| format!("layout: {x}")));
            }
        }
        let shortcuts = match normalise_shortcuts(&self.shortcuts) {
            Ok(s) => s,
            Err(e) => {
                errs.extend(e.into_iter().map(|x| format!("shortcuts: {x}")));
                BTreeMap::new()
            }
        };
        if self.recents.len() > MAX_RECENTS {
            errs.push(format!("recents: at most {MAX_RECENTS} entries"));
        }
        let mut seen = BTreeSet::new();
        for r in &self.recents {
            if command(r).is_none() {
                errs.push(format!("recents: unknown command '{r}'"));
            } else if !seen.insert(r) {
                errs.push(format!("recents: '{r}' listed twice"));
            }
        }
        if let Some(o) = &self.overlays {
            if let Err(e) = o.validate() {
                errs.push(format!("overlays: {e}"));
            }
        }
        if errs.is_empty() {
            Ok(UiConfig { version: 1, layout: self.layout.clone(), shortcuts, recents: self.recents.clone(), overlays: self.overlays.clone() })
        } else {
            Err(errs)
        }
    }
}

/// `$OSATURA_UI_CONFIG`, else `$XDG_CONFIG_HOME/osatura/ui.json`, else on Windows `%APPDATA%\OSATURA\ui.json`
/// (then `%USERPROFILE%\.config\osatura\ui.json`), else `~/.config/osatura/ui.json`. Never a path inside the project.
pub fn default_config_path() -> Option<PathBuf> {
    config_path_from(|k| std::env::var_os(k).filter(|v| !v.is_empty()), cfg!(windows))
}

/// The lookup behind [`default_config_path`], with the environment and the platform as parameters (testable).
fn config_path_from(env: impl Fn(&str) -> Option<std::ffi::OsString>, windows: bool) -> Option<PathBuf> {
    if let Some(p) = env("OSATURA_UI_CONFIG") {
        return Some(PathBuf::from(p));
    }
    if let Some(x) = env("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(x).join("osatura").join("ui.json"));
    }
    if windows {
        if let Some(a) = env("APPDATA") {
            return Some(PathBuf::from(a).join("OSATURA").join("ui.json"));
        }
        return env("USERPROFILE").map(|h| PathBuf::from(h).join(".config").join("osatura").join("ui.json"));
    }
    env("HOME").map(|h| PathBuf::from(h).join(".config").join("osatura").join("ui.json"))
}

#[cfg(test)]
pub fn load_config(path: &Path) -> Result<UiConfig, String> {
    let t = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let c: UiConfig = serde_json::from_str(&t).map_err(|e| format!("{}: {e}", path.display()))?;
    c.validate().map_err(|e| format!("{}: {}", path.display(), e.join("; ")))
}

pub fn save_config(path: &Path, cfg: &UiConfig) -> Result<(), String> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    }
    let text = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    osatura_core::fsutil::write_atomic(path, text.as_bytes())
}

/// Removes, one at a time, the keys that `UiConfig` (and the types below it) do not know, as reported by serde's
/// `unknown field` error. Returns the config that remains and the paths of the removed keys.
fn parse_ignoring_unknown(mut v: Value) -> Result<(UiConfig, Vec<String>), String> {
    fn take(v: &mut Value, key: &str, expected: &[String], path: &str, exact: bool) -> Option<String> {
        match v {
            Value::Object(o) => {
                if o.contains_key(key) && (!exact || o.keys().all(|k| k == key || k == "type" || expected.iter().any(|e| e == k))) {
                    o.remove(key);
                    return Some(if path.is_empty() { key.to_string() } else { format!("{path}.{key}") });
                }
                for (k, c) in o.iter_mut() {
                    let p = if path.is_empty() { k.clone() } else { format!("{path}.{k}") };
                    if let Some(r) = take(c, key, expected, &p, exact) {
                        return Some(r);
                    }
                }
                None
            }
            Value::Array(a) => {
                for (i, c) in a.iter_mut().enumerate() {
                    if let Some(r) = take(c, key, expected, &format!("{path}[{i}]"), exact) {
                        return Some(r);
                    }
                }
                None
            }
            _ => None,
        }
    }
    let mut removed = Vec::new();
    for _ in 0..200 {
        match serde_json::from_value::<UiConfig>(v.clone()) {
            Ok(c) => return Ok((c, removed)),
            Err(e) => {
                let m = e.to_string();
                let parts: Vec<&str> = m.split('`').collect();
                if !m.starts_with("unknown field `") || parts.len() < 2 {
                    return Err(m);
                }
                let key = parts[1].to_string();
                let expected: Vec<String> = parts.iter().skip(2).step_by(2).map(|s| s.to_string()).collect();
                match take(&mut v, &key, &expected, "", true).or_else(|| take(&mut v, &key, &expected, "", false)) {
                    Some(p) => removed.push(p),
                    None => return Err(m),
                }
            }
        }
    }
    Err("too many unknown keys".into())
}

/// Like `load_config`, but keys this version does not know (a newer version's settings, a typo) are ignored and
/// listed instead of discarding the whole file. Values that are invalid are still an error.
pub fn load_config_tolerant(path: &Path) -> Result<(UiConfig, Vec<String>), String> {
    let t = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let v: Value = serde_json::from_str(&t).map_err(|e| format!("{}: {e}", path.display()))?;
    let (c, removed) = parse_ignoring_unknown(v).map_err(|e| format!("{}: {e}", path.display()))?;
    let c = c.validate().map_err(|e| format!("{}: {}", path.display(), e.join("; ")))?;
    Ok((c, removed))
}

/// Copy `path` to `<path>.bak` (`.bak.1`, `.bak.2`, ... when that exists): the first copy of a file that could not be
/// read completely is never overwritten. Returns the backup's path.
pub fn backup_config(path: &Path) -> Result<PathBuf, String> {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "ui.json".into());
    for i in 0..1000 {
        let b = path.with_file_name(if i == 0 { format!("{name}.bak") } else { format!("{name}.bak.{i}") });
        if !b.exists() {
            std::fs::copy(path, &b).map_err(|e| format!("cannot back up {} to {}: {e}", path.display(), b.display()))?;
            return Ok(b);
        }
    }
    Err(format!("too many backups of {}", path.display()))
}

/// The server-side holder of the user configuration.
pub struct UiStore {
    pub path: Option<PathBuf>,
    pub cfg: UiConfig,
    /// why the stored file was ignored or could not be written
    pub warning: Option<String>,
    pub persisted: bool,
    /// the stored file was not read completely (unknown keys dropped, or unreadable): the first save backs it up
    needs_backup: bool,
}

impl UiStore {
    pub fn open(path: Option<PathBuf>) -> UiStore {
        let (mut cfg, mut warning, mut needs_backup, mut failed) = (UiConfig::default(), None, false, false);
        if let Some(p) = &path {
            if p.exists() {
                match load_config_tolerant(p) {
                    Ok((c, ignored)) => {
                        cfg = c;
                        if !ignored.is_empty() {
                            // the settings stay in force; the next save rewrites the file without these keys, so it is backed up first
                            needs_backup = true;
                            warning = Some(format!("{}: ignored unknown key(s) {} (a newer version or a typo); the other settings are kept and the file is backed up before it is rewritten", p.display(), ignored.join(", ")));
                        }
                    }
                    Err(e) => {
                        needs_backup = true;
                        failed = true;
                        warning = Some(format!("ignored stored configuration: {e}"));
                    }
                }
            }
        }
        UiStore { persisted: path.is_some() && !failed, path, cfg, warning, needs_backup }
    }

    /// Apply a partial update. Keys: `layout` (object, or null to reset), `shortcuts` (whole override map, or null
    /// to reset), `recents` (list), `reset` ("layout" | "shortcuts" | "all"). Nothing changes when anything is invalid.
    pub fn update(&mut self, patch: &Value) -> Result<(), Vec<String>> {
        let obj = patch.as_object().ok_or_else(|| vec!["body must be a JSON object".to_string()])?;
        let mut errs = Vec::new();
        for k in obj.keys() {
            if !["layout", "shortcuts", "recents", "reset", "overlays"].contains(&k.as_str()) {
                errs.push(format!("unknown field '{k}'"));
            }
        }
        let mut next = self.cfg.clone();
        match obj.get("reset").map(|v| v.as_str()) {
            None => {}
            Some(Some("layout")) => next.layout = None,
            Some(Some("shortcuts")) => next.shortcuts.clear(),
            Some(Some("overlays")) => next.overlays = None,
            Some(Some("all")) => next = UiConfig::default(),
            _ => errs.push("reset must be \"layout\", \"shortcuts\", \"overlays\" or \"all\"".into()),
        }
        if let Some(v) = obj.get("layout") {
            if v.is_null() {
                next.layout = None;
            } else {
                match serde_json::from_value::<Layout>(v.clone()) {
                    Ok(l) => next.layout = Some(l),
                    Err(e) => errs.push(format!("layout: {e}")),
                }
            }
        }
        if let Some(v) = obj.get("shortcuts") {
            if v.is_null() {
                next.shortcuts.clear();
            } else {
                match serde_json::from_value::<BTreeMap<String, Vec<String>>>(v.clone()) {
                    Ok(m) => next.shortcuts = m,
                    Err(e) => errs.push(format!("shortcuts: {e}")),
                }
            }
        }
        if let Some(v) = obj.get("overlays") {
            if v.is_null() {
                next.overlays = None;
            } else {
                match next.overlays.clone().unwrap_or_default().merged(v) {
                    Ok(o) => next.overlays = Some(o),
                    Err(e) => errs.push(format!("overlays: {e}")),
                }
            }
        }
        if let Some(v) = obj.get("recents") {
            match serde_json::from_value::<Vec<String>>(v.clone()) {
                Ok(r) => next.recents = r,
                Err(e) => errs.push(format!("recents: {e}")),
            }
        }
        if !errs.is_empty() {
            return Err(errs);
        }
        let checked = next.validate()?;
        self.cfg = checked;
        if self.needs_backup {
            if let Some(p) = self.path.clone().filter(|p| p.exists()) {
                match backup_config(&p) {
                    Ok(b) => {
                        eprintln!("ui config: {} could not be read completely; a copy was kept as {}", p.display(), b.display());
                        self.needs_backup = false;
                    }
                    Err(e) => {
                        // never overwrite what could not be read without a copy: keep the change in memory only
                        self.persisted = false;
                        self.warning = Some(format!("{e}; settings kept for this session only"));
                        return Ok(());
                    }
                }
            } else {
                self.needs_backup = false;
            }
        }
        match &self.path {
            Some(p) => match save_config(p, &self.cfg) {
                Ok(()) => {
                    self.persisted = true;
                    self.warning = None;
                }
                Err(e) => {
                    self.persisted = false;
                    self.warning = Some(e);
                }
            },
            None => {
                self.persisted = false;
                self.warning = Some("no configuration path (set HOME or pass --ui-config); kept for this session only".into());
            }
        }
        Ok(())
    }

    /// The stored overlay settings, or the defaults.
    pub fn overlays(&self) -> OverlaySettings {
        self.cfg.overlays.clone().unwrap_or_default()
    }

    /// Search results with titles and effective shortcuts, for the palette and the CLI.
    pub fn search_view(&self, query: &str, limit: usize) -> Value {
        let eff = effective_bindings(&self.cfg.shortcuts).unwrap_or_default();
        let hits: Vec<Value> = search(query, &self.cfg.recents, limit)
            .into_iter()
            .filter_map(|h| {
                let c = command(h.id)?;
                let b: Vec<String> = eff.iter().filter(|x| x.command == c.id).map(|x| binding_string(x.context, &x.chord)).collect();
                Some(json!({"id": c.id, "title": c.title, "category": c.category, "description": c.description, "score": h.score, "positions": h.positions, "bindings": b}))
            })
            .collect();
        json!({"ok": true, "query": query, "hits": hits})
    }

    /// Everything the page needs: registry, effective bindings, conflicts, panels, presets and the stored layout.
    pub fn view(&self) -> Value {
        let eff = effective_bindings(&self.cfg.shortcuts).unwrap_or_else(|_| effective_bindings(&BTreeMap::new()).unwrap_or_default());
        let commands: Vec<Value> = COMMANDS
            .iter()
            .map(|c| {
                let b: Vec<String> = eff.iter().filter(|x| x.command == c.id).map(|x| binding_string(x.context, &x.chord)).collect();
                json!({"id": c.id, "title": c.title, "category": c.category, "description": c.description,
                       "primary": c.primary_context().name(), "defaults": default_strings(c), "bindings": b, "user": self.cfg.shortcuts.contains_key(c.id)})
            })
            .collect();
        let cf: Vec<Value> = conflicts(&eff).iter().map(|c| json!({"chord": c.chord, "a": c.a.0, "b": c.b.0})).collect();
        let panels: Vec<Value> = PANELS.iter().map(|p| json!({"id": p.id, "title": p.title, "min_w": p.min_w, "min_h": p.min_h, "closable": p.closable})).collect();
        let presets: BTreeMap<&str, Value> = PRESET_NAMES.iter().filter_map(|n| preset(n).map(|l| (*n, serde_json::to_value(l).unwrap_or(Value::Null)))).collect();
        let layout = self.cfg.layout.as_ref().map(|l| l.repaired());
        json!({
            "ok": true, "path": self.path.as_ref().map(|p| p.display().to_string()), "persisted": self.persisted, "warning": self.warning,
            "commands": commands, "conflicts": cf, "reserved": RESERVED, "panels": panels, "presets": presets, "preset_names": PRESET_NAMES,
            "layout": layout, "default_layout": default_layout(), "shortcuts": self.cfg.shortcuts, "recents": self.cfg.recents,
            "overlays": self.overlays(),
            "tab_h": TAB_H, "split_w": SPLIT_W, "min_ratio": MIN_RATIO,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch(s: &str) -> Chord {
        Chord::parse(s).unwrap()
    }

    /// The page's JavaScript key lookup (extracted verbatim from studio.html) agrees with `resolve` for every default
    /// chord in every context. Skipped (with a note) when `node` is not installed.
    #[test]
    fn page_dispatch_matches_the_rust_resolver() {
        let html = include_str!("studio.html");
        let build = "UI.bind=new Map(); for(const c of UI.cmds)for(const b of c.bindings){ if(!UI.bind.has(b))UI.bind.set(b,c.id); }";
        let look1 = "let hit=UI.bind.get(ctx+':'+chord), scope=ctx;";
        let look2 = "if(!hit){ hit=UI.bind.get('global:'+chord); scope='global'; }";
        for part in [build, look1, look2] {
            assert!(html.contains(part), "studio.html changed: update this parity test together with the dispatcher ({part})");
        }
        if std::process::Command::new("node").arg("--version").output().is_err() {
            eprintln!("note: node not installed, page/Rust dispatch parity not checked");
            return;
        }
        let eff = effective_bindings(&BTreeMap::new()).unwrap();
        let view = UiStore::open(None).view();
        let mut chords: Vec<String> = eff.iter().map(|b| b.chord.to_string()).collect();
        chords.sort();
        chords.dedup();
        let scratch = osatura_core::fsutil::ScratchDir::new("osatura-ui-parity");
        let script = scratch.join("parity.js");
        let src = format!(
            "const UI={{cmds:{}}}; {build}\nconst out={{}}; for (const ctx of ['canvas','timeline']) for (const chord of {}) {{ out[ctx+'|'+chord] = UI.bind.get(ctx+':'+chord) || UI.bind.get('global:'+chord) || null; }}\nconsole.log(JSON.stringify(out));",
            view["commands"],
            serde_json::to_string(&chords).unwrap()
        );
        std::fs::write(&script, src).unwrap();
        let o = std::process::Command::new("node").arg(&script).output().unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        let got: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
        for ctx in [Context::Canvas, Context::Timeline] {
            for c in &chords {
                let want = resolve(&eff, ctx, &Chord::parse(c).unwrap()).map(|b| b.command.clone());
                let have = got[format!("{}|{c}", ctx.name())].as_str().map(String::from);
                assert_eq!(have, want, "{} in {}", c, ctx.name());
            }
        }
    }

    #[test]
    fn chord_parsing_and_normalisation() {
        assert_eq!(ch("ctrl+shift+k").to_string(), "Ctrl+Shift+K");
        assert_eq!(ch("Shift+Ctrl+K").to_string(), "Ctrl+Shift+K");
        assert_eq!(ch("cmd+k"), ch("Control+K"));
        assert_eq!(ch("Option+left").to_string(), "Alt+ArrowLeft");
        assert_eq!(ch("space").to_string(), "Space");
        assert_eq!(ch("esc").to_string(), "Escape");
        assert_eq!(ch("f5").to_string(), "F5");
        assert_eq!(ch("Alt+Period").to_string(), "Alt+.");
        assert_eq!(ch("ctrl+,").to_string(), "Ctrl+,");
        assert_eq!(ch("1").to_string(), "1");
        // round trip of every canonical string
        for s in ["Ctrl+Alt+Shift+F12", "Alt+PageDown", "Shift+=", "[", "Ctrl+\\"] {
            assert_eq!(ch(s).to_string(), s);
        }
    }

    #[test]
    fn chord_errors() {
        for bad in ["", "  ", "Ctrl+", "+K", "Ctrl+Ctrl+K", "Ctrl+Shift", "Foo+K", "Ctrl+K+L", "F0", "F25", "F05", "Ctrl+Wibble", "é", "Ctrl++"] {
            assert!(Chord::parse(bad).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn binding_with_context() {
        let (c, k) = parse_binding("timeline:alt+left", Context::Global).unwrap();
        assert_eq!((c, k.to_string()), (Context::Timeline, "Alt+ArrowLeft".to_string()));
        assert_eq!(parse_binding("K", Context::Canvas).unwrap().0, Context::Canvas);
        assert!(parse_binding("nowhere:K", Context::Global).is_err());
        assert_eq!(binding_string(Context::Canvas, &ch("i")), "canvas:I");
    }

    #[test]
    fn registry_is_sane() {
        let mut ids = BTreeSet::new();
        for c in COMMANDS {
            assert!(ids.insert(c.id), "duplicate id {}", c.id);
            assert!(!c.title.is_empty() && !c.category.is_empty() && !c.description.is_empty());
            for (_, k) in c.defaults {
                let p = Chord::parse(k).unwrap_or_else(|e| panic!("{}: {e}", c.id));
                assert_eq!(&p.to_string(), k, "{} default must be written canonically", c.id);
                assert!(!RESERVED.iter().any(|r| ch(r) == p), "{} default {k} is reserved", c.id);
            }
        }
        // the actions the studio already had
        for id in ["edit.undo", "edit.redo", "playback.toggle", "playback.step_next", "select.all", "select.delete", "canvas.nudge_left", "panel.toggle.timeline", "canvas.toggle_snap", "timeline.toggle_snap", "anim.add_key", "anim.toggle_autokey", "timeline.toggle_view", "export.gif", "timeline.zoom_in", "ui.palette", "ui.shortcuts", "layout.reset"] {
            assert!(command(id).is_some(), "missing {id}");
        }
        for p in PANELS.iter().filter(|p| p.closable) {
            assert!(command(&format!("panel.toggle.{}", p.id)).is_some(), "no toggle for {}", p.id);
        }
        for n in PRESET_NAMES {
            assert!(command(&format!("layout.preset.{n}")).is_some());
        }
    }

    #[test]
    fn default_bindings_do_not_conflict() {
        let eff = effective_bindings(&BTreeMap::new()).unwrap();
        assert!(eff.iter().all(|b| !b.user));
        assert_eq!(conflicts(&eff), vec![], "defaults must be unambiguous");
        // every command's defaults come through
        assert_eq!(eff.iter().filter(|b| b.command == "edit.redo").count(), 2);
    }

    #[test]
    fn conflict_detection_and_scoping() {
        // same chord, canvas vs timeline: fine
        let mut o = BTreeMap::new();
        o.insert("canvas.nudge_left".to_string(), vec!["canvas:Ctrl+J".to_string()]);
        o.insert("timeline.copy".to_string(), vec!["timeline:Ctrl+J".to_string()]);
        assert!(normalise_shortcuts(&o).is_ok());
        // global overlaps everything
        let mut o = BTreeMap::new();
        o.insert("canvas.nudge_left".to_string(), vec!["canvas:Ctrl+J".to_string()]);
        o.insert("playback.first".to_string(), vec!["global:Ctrl+J".to_string()]);
        let e = normalise_shortcuts(&o).unwrap_err();
        assert!(e.iter().any(|m| m.contains("conflicts") && m.contains("canvas.nudge_left") && m.contains("playback.first")), "{e:?}");
        // taking a default chord of another command conflicts
        let mut o = BTreeMap::new();
        o.insert("playback.first".to_string(), vec!["Ctrl+Z".to_string()]);
        assert!(normalise_shortcuts(&o).is_err());
        // ...unless the other one is moved away in the same map
        o.insert("edit.undo".to_string(), vec!["Ctrl+Alt+Z".to_string()]);
        let n = normalise_shortcuts(&o).unwrap();
        assert_eq!(n["playback.first"], vec!["global:Ctrl+Z"]);
        assert_eq!(n["edit.undo"], vec!["global:Ctrl+Alt+Z"]);
    }

    #[test]
    fn override_validation_errors() {
        let one = |id: &str, v: &[&str]| {
            let mut o = BTreeMap::new();
            o.insert(id.to_string(), v.iter().map(|s| s.to_string()).collect());
            normalise_shortcuts(&o)
        };
        assert!(one("nope.nothing", &["K"]).unwrap_err()[0].contains("unknown command"));
        assert!(one("edit.undo", &["Ctrl+"]).is_err());
        assert!(one("edit.undo", &["Ctrl+W"]).unwrap_err()[0].contains("reserved"));
        assert!(one("edit.undo", &["Ctrl+Q"]).is_err());
        assert!(one("edit.undo", &["Ctrl+9", "ctrl+9"]).unwrap_err()[0].contains("twice"));
        assert!(one("edit.undo", &["moon:Ctrl+9"]).is_err());
        // unbinding is allowed
        assert_eq!(one("edit.undo", &[]).unwrap()["edit.undo"], Vec::<String>::new());
    }

    #[test]
    fn overrides_normalise_and_reset_to_defaults() {
        let mut o = BTreeMap::new();
        o.insert("anim.add_key".to_string(), vec!["shift+i".to_string()]);
        let n = normalise_shortcuts(&o).unwrap();
        assert_eq!(n["anim.add_key"], vec!["canvas:Shift+I"], "unprefixed chords take the command's primary context");
        // writing the defaults back drops the entry (that is "reset")
        let mut o = BTreeMap::new();
        o.insert("anim.add_key".to_string(), vec!["timeline:i".to_string(), "canvas:I".to_string()]);
        assert!(normalise_shortcuts(&o).unwrap().is_empty());
        // effective view follows the override
        let mut o = BTreeMap::new();
        o.insert("edit.undo".to_string(), vec!["Ctrl+9".to_string()]);
        let eff = effective_bindings(&o).unwrap();
        assert!(resolve(&eff, Context::Canvas, &ch("Ctrl+9")).is_some_and(|b| b.command == "edit.undo" && b.user));
        assert!(resolve(&eff, Context::Canvas, &ch("Ctrl+Z")).is_none());
    }

    #[test]
    fn resolve_prefers_the_active_context() {
        let eff = effective_bindings(&BTreeMap::new()).unwrap();
        let sel_all = |c| resolve(&eff, c, &ch("Ctrl+A")).map(|b| b.command.as_str());
        assert_eq!(sel_all(Context::Canvas), Some("select.all"));
        assert_eq!(sel_all(Context::Timeline), Some("timeline.select_all"));
        assert_eq!(sel_all(Context::Global), None);
        // global command found from any context
        assert_eq!(resolve(&eff, Context::Timeline, &ch("Ctrl+Z")).map(|b| b.command.as_str()), Some("edit.undo"));
        // a timeline binding is not active on the canvas
        assert_eq!(resolve(&eff, Context::Canvas, &ch("Ctrl+C")), None);
        assert_eq!(resolve(&eff, Context::Timeline, &ch("ArrowLeft")).map(|b| b.command.as_str()), Some("playback.step_prev"));
        assert_eq!(resolve(&eff, Context::Canvas, &ch("ArrowLeft")).map(|b| b.command.as_str()), Some("canvas.nudge_left"));
    }

    #[test]
    fn presets_are_valid_and_fit() {
        for n in PRESET_NAMES {
            let l = preset(n).unwrap();
            l.validate().unwrap_or_else(|e| panic!("{n}: {e:?}"));
            l.fits(1100.0, 640.0).unwrap_or_else(|e| panic!("{n}: {e}"));
            assert_eq!(l.repaired(), l, "{n} should already be in repaired form");
            assert!(!l.summary().is_empty());
        }
        assert!(preset("nonsense").is_none());
        assert_eq!(default_layout(), preset("animate").unwrap());
        // presets differ
        assert_ne!(preset("rig"), preset("compact"));
        assert!(preset("rig").unwrap().hidden.contains(&"live".to_string()));
    }

    #[test]
    fn layout_serialisation_round_trip() {
        for n in PRESET_NAMES {
            let l = preset(n).unwrap();
            let j = serde_json::to_string(&l).unwrap();
            let back: Layout = serde_json::from_str(&j).unwrap();
            assert_eq!(back, l);
        }
        let mut l = preset("animate").unwrap();
        l.root = split(Dir::H, 0.3125, tabs(&["stage", "json"]), tabs(&["timeline", "layers", "selection", "bones", "live"]));
        l.hidden = vec![];
        l.floating = vec![Floating { panel: "face".into(), x: 40.5, y: 60.0, w: 300.0, h: 220.25 }];
        l.validate().unwrap();
        let back: Layout = serde_json::from_value(serde_json::to_value(&l).unwrap()).unwrap();
        assert_eq!(back, l);
        // wire shape the page relies on
        let v = serde_json::to_value(preset("compact").unwrap()).unwrap();
        assert_eq!(v["root"]["type"], "split");
        assert_eq!(v["root"]["dir"], "v");
        assert_eq!(v["root"]["a"]["type"], "tabs");
        assert!(serde_json::from_str::<Layout>(r#"{"version":1,"root":{"type":"tabs","panels":["stage"],"bogus":1}}"#).is_err());
    }

    #[test]
    fn layout_validation_errors() {
        let base = || preset("animate").unwrap();
        let errs = |l: Layout| l.validate().unwrap_err().join(" | ");
        let mut l = base();
        l.version = 2;
        assert!(errs(l).contains("version"));
        let mut l = base();
        l.hidden.push("timeline".into());
        assert!(errs(l).contains("'timeline' appears 2 times"));
        let mut l = base();
        l.hidden.push("warp".into());
        assert!(errs(l).contains("unknown hidden panel 'warp'"));
        let l = layout(tabs(&["stage"]), &[]);
        assert!(errs(l).contains("'timeline' is missing"));
        let l = layout(split(Dir::H, 0.5, tabs(&["layers", "selection", "bones", "live", "face", "json", "timeline"]), tabs(&[])), &["stage"]);
        let e = errs(l);
        assert!(e.contains("empty tab group") && e.contains("cannot be hidden") && e.contains("canvas must stay docked"), "{e}");
        let mut l = base();
        if let Node::Split { ratio, .. } = &mut l.root {
            *ratio = 1.5;
        }
        assert!(errs(l).contains("ratio"));
        let mut l = base();
        if let Node::Split { ratio, .. } = &mut l.root {
            *ratio = f32::NAN;
        }
        assert!(errs(l).contains("ratio"));
        let l = layout(Node::Tabs { panels: vec!["stage".into()], active: 3, collapsed: true }, &[]);
        let e = errs(l);
        assert!(e.contains("out of range") && e.contains("cannot be collapsed"), "{e}");
        let mut l = base();
        l.hidden = vec![];
        l.root = split(Dir::V, 0.5, tabs(&["stage", "layers", "selection", "bones", "live", "face", "json"]), tabs(&["stage"]));
        assert!(errs(l).contains("'stage' appears 2 times"));
        // floating
        let mut l = preset("rig").unwrap();
        l.hidden.retain(|h| h != "live");
        l.floating = vec![Floating { panel: "live".into(), x: 0.0, y: 0.0, w: 10.0, h: 10.0 }];
        assert!(errs(l.clone()).contains("smaller than its minimum"));
        l.floating[0].w = f32::INFINITY;
        assert!(errs(l.clone()).contains("bad position"));
        l.floating[0] = Floating { panel: "live".into(), x: 5.0, y: 5.0, w: 300.0, h: 200.0 };
        l.validate().unwrap();
        l.floating.push(Floating { panel: "stage".into(), x: 0.0, y: 0.0, w: 300.0, h: 300.0 });
        assert!(errs(l).contains("cannot float"));
        // too deep
        let mut n = tabs(&["stage"]);
        for _ in 0..20 {
            n = split(Dir::H, 0.5, n, tabs(&[]));
        }
        assert!(layout(n, &[]).validate().unwrap_err().iter().any(|e| e.contains("deeper")));
    }

    #[test]
    fn min_sizes_and_fit() {
        let c = preset("compact").unwrap();
        let (w, h) = c.root.min_size();
        assert_eq!(w, 320.0);
        assert_eq!(h, (160.0 + TAB_H) + SPLIT_W + (160.0 + TAB_H));
        assert!(c.fits(w, h).is_ok());
        assert!(c.fits(w - 1.0, h).is_err());
        assert!(c.fits(w, h - 1.0).unwrap_err().contains("needs at least"));
        // folding a group makes it cost only its tab bar
        let (_, fh) = folded(&["timeline"]).min_size();
        assert_eq!(fh, TAB_H);
    }

    #[test]
    fn repair_heals_old_and_broken_layouts() {
        // an older file without the newest panels
        let l = Layout { version: 1, root: split(Dir::H, 0.7, tabs(&["stage"]), tabs(&["layers", "bones"])), floating: vec![], hidden: vec![] };
        assert!(l.validate().is_err());
        let r = l.repaired();
        r.validate().unwrap();
        for p in ["selection", "live", "face", "json", "timeline"] {
            assert!(r.hidden.contains(&p.to_string()), "{p} should be hidden");
        }
        // unknown / duplicate panels, empty groups, silly ratio
        let l = Layout { version: 1, root: split(Dir::V, 9.0, tabs(&["stage", "stage", "ghost"]), tabs(&["ghost2"])), floating: vec![Floating { panel: "stage".into(), x: 0.0, y: 0.0, w: 100.0, h: 100.0 }, Floating { panel: "json".into(), x: 1.0, y: 2.0, w: 5.0, h: 5.0 }], hidden: vec!["json".into(), "x".into()] };
        let r = l.repaired();
        r.validate().unwrap();
        assert_eq!(r.root, tabs(&["stage"]));
        assert_eq!(r.floating.len(), 1);
        assert!(r.floating[0].w >= 200.0, "clamped to the panel minimum");
        // canvas lost: back to the default
        let l = Layout { version: 1, root: tabs(&["layers"]), floating: vec![], hidden: vec![] };
        assert_eq!(l.repaired(), default_layout());
        // repaired is idempotent
        for n in PRESET_NAMES {
            let p = preset(n).unwrap();
            assert_eq!(p.repaired().repaired(), p.repaired());
        }
    }

    #[test]
    fn fuzzy_basics() {
        let s = |q: &str, t: &str| fuzzy_score(q, t).map(|x| x.0);
        assert!(s("udo", "Undo").is_some());
        assert!(s("zzz", "Undo").is_none());
        assert!(s("undoo", "Undo").is_none(), "query longer than text");
        assert!(s("", "Undo").is_some());
        assert_eq!(s("UNDO", "undo"), s("undo", "UNDO"), "case-insensitive");
        // prefix beats mid-word, word starts beat scattered letters
        assert!(s("pl", "Play / pause") > s("pl", "Apply"));
        assert!(s("tv", "Toggle view") > s("tv", "Settings overview mode"));
        assert!(s("play", "Play / pause") > s("play", "Display"));
        // exact beats prefix beats the rest
        assert!(s("undo", "Undo") > s("undo", "Undo all"));
        assert!(s("undo", "Undo all") > s("undo", "Cancel undo"));
        // spaces in the query are ignored
        assert_eq!(s("nx fr", "Next frame"), s("nxfr", "Next frame"));
        // matched positions
        assert_eq!(fuzzy_score("npf", "Next previous frame").unwrap().1, vec![0, 5, 14]);
        assert_eq!(fuzzy_score("pau", "Play / pause").unwrap().1, vec![7, 8, 9]);
    }

    #[test]
    fn search_ranking_is_deterministic_and_sensible() {
        let a = search("tog", &[], 50);
        let b = search("tog", &[], 50);
        assert_eq!(a, b);
        assert!(a.len() > 5);
        assert!(a.windows(2).all(|w| w[0].score >= w[1].score));
        // best hit for an exact title is that command
        assert_eq!(search("undo", &[], 5)[0].id, "edit.undo");
        assert_eq!(search("redo", &[], 5)[0].id, "edit.redo");
        assert_eq!(search("play pause", &[], 3)[0].id, "playback.toggle");
        assert_eq!(search("export gif", &[], 3)[0].id, "export.gif");
        // abbreviations on word starts
        assert_eq!(search("nf", &[], 3)[0].id, "playback.step_next");
        assert_eq!(search("zoom fit", &[], 1)[0].id, "timeline.zoom_fit");
        // category words find commands
        assert!(search("export", &[], 10).iter().filter(|h| h.id.starts_with("export.")).count() == 3);
        // id search works
        assert!(search("canvas.nudge", &[], 3).iter().all(|h| h.id.starts_with("canvas.nudge")));
        assert!(search("qqqqzx", &[], 10).is_empty());
        assert_eq!(search("tog", &[], 3).len(), 3);
        // title positions are returned for title matches
        let h = &search("undo", &[], 1)[0];
        assert_eq!(h.positions, vec![0, 1, 2, 3]);
    }

    #[test]
    fn recents_boost_and_order() {
        let rec = vec!["export.html".to_string(), "file.save".to_string()];
        let e = search("", &rec, 5);
        assert_eq!(e[0].id, "export.html");
        assert_eq!(e[1].id, "file.save");
        // unknown ids in recents are ignored
        let e = search("  ", &["ghost".to_string()], 3);
        assert_eq!(e[0].id, COMMANDS[0].id);
        // a recent command wins a tie it would otherwise lose
        let plain: Vec<&str> = search("export", &[], 3).iter().map(|h| h.id).collect();
        let boosted: Vec<&str> = search("export", &["export.lottie".to_string()], 3).iter().map(|h| h.id).collect();
        assert_eq!(boosted[0], "export.lottie");
        assert_ne!(plain[0], "export.lottie", "the order really changed because of the recent");
        // limit respected
        assert_eq!(search("", &[], 4).len(), 4);
    }

    #[test]
    fn config_validation_and_round_trip() {
        let mut c = UiConfig { layout: Some(preset("rig").unwrap()), ..UiConfig::default() };
        c.shortcuts.insert("edit.undo".into(), vec!["ctrl+9".into()]);
        c.recents = vec!["edit.undo".into(), "ui.palette".into()];
        let v = c.validate().unwrap();
        assert_eq!(v.shortcuts["edit.undo"], vec!["global:Ctrl+9"]);
        let j = serde_json::to_string(&v).unwrap();
        assert_eq!(serde_json::from_str::<UiConfig>(&j).unwrap().validate().unwrap(), v);
        // errors
        let mut bad = v.clone();
        bad.recents = vec!["ghost".into(), "edit.undo".into(), "edit.undo".into()];
        let e = bad.validate().unwrap_err().join("|");
        assert!(e.contains("unknown command 'ghost'") && e.contains("listed twice"));
        let mut bad = v.clone();
        bad.recents = (0..13).map(|_| "edit.undo".to_string()).collect();
        assert!(bad.validate().is_err());
        let mut bad = v.clone();
        bad.layout.as_mut().unwrap().hidden.push("stage".into());
        assert!(bad.validate().unwrap_err()[0].starts_with("layout:"));
        assert!(serde_json::from_str::<UiConfig>(r#"{"surprise":1}"#).is_err());
    }

    fn tmpfile(name: &str) -> (osatura_core::fsutil::ScratchDir, PathBuf) {
        let d = osatura_core::fsutil::ScratchDir::new(&format!("osatura-ui-test-{name}"));
        let f = d.join("sub").join("ui.json");
        (d, f)
    }

    #[test]
    fn store_persists_and_rejects_bad_updates() {
        let (_scratch, p) = tmpfile("store");
        let mut s = UiStore::open(Some(p.clone()));
        assert!(s.cfg == UiConfig::default() && s.warning.is_none());
        s.update(&json!({"layout": preset("compact").unwrap(), "shortcuts": {"edit.undo": ["Ctrl+9"]}, "recents": ["ui.palette"]})).unwrap();
        assert!(s.persisted && p.exists());
        // a fresh store reads it back
        let s2 = UiStore::open(Some(p.clone()));
        assert_eq!(s2.cfg, s.cfg);
        let v = s2.view();
        assert_eq!(v["layout"]["root"]["type"], "split");
        let undo = v["commands"].as_array().unwrap().iter().find(|c| c["id"] == "edit.undo").unwrap();
        assert_eq!(undo["bindings"], json!(["global:Ctrl+9"]));
        assert_eq!(undo["user"], true);
        // invalid updates change nothing and are not written
        let before = std::fs::read_to_string(&p).unwrap();
        for bad in [json!({"shortcuts": {"nope": ["K"]}}), json!({"layout": {"version": 1}}), json!({"recents": ["ghost"]}), json!({"extra": 1}), json!({"reset": "everything"}), json!([1]), json!({"shortcuts": {"file.save": ["Ctrl+Z"]}})] {
            assert!(s.update(&bad).is_err(), "{bad}");
        }
        assert_eq!(std::fs::read_to_string(&p).unwrap(), before);
        assert_eq!(s.cfg.shortcuts["edit.undo"], vec!["global:Ctrl+9"]);
        // partial update keeps the rest
        s.update(&json!({"recents": []})).unwrap();
        assert!(s.cfg.layout.is_some() && !s.cfg.shortcuts.is_empty());
        // resets
        s.update(&json!({"reset": "shortcuts"})).unwrap();
        assert!(s.cfg.shortcuts.is_empty() && s.cfg.layout.is_some());
        s.update(&json!({"layout": null})).unwrap();
        assert!(s.cfg.layout.is_none());
        s.update(&json!({"reset": "all"})).unwrap();
        assert_eq!(s.cfg, UiConfig::default());
        let _ = std::fs::remove_dir_all(p.parent().unwrap().parent().unwrap());
    }

    #[test]
    fn overlay_settings_persist_merge_and_reset() {
        let (_scratch, p) = tmpfile("ovl");
        let mut s = UiStore::open(Some(p.clone()));
        assert_eq!(s.overlays(), OverlaySettings::default());
        assert_eq!(s.view()["overlays"]["onion"]["before"], 2);
        s.update(&json!({"overlays": {"onion": {"enabled": true, "before": 5}}})).unwrap();
        s.update(&json!({"overlays": {"trails": {"enabled": true}}})).unwrap();
        let o = s.overlays();
        assert!(o.onion.enabled && o.onion.before == 5 && o.trails.enabled, "sections merge field by field");
        // a fresh store reads it back from disk
        let s2 = UiStore::open(Some(p.clone()));
        assert_eq!(s2.overlays(), o);
        assert_eq!(s2.view()["overlays"]["onion"]["before"], 5);
        // invalid updates change nothing
        let before = std::fs::read_to_string(&p).unwrap();
        for bad in [json!({"overlays": {"onion": {"before": 40}}}), json!({"overlays": {"proxy": {"level": 7}}}), json!({"overlays": {"bogus": {}}}), json!({"overlays": 3}), json!({"overlays": {"onion": {"tint_before": "red-ish"}}})] {
            assert!(s.update(&bad).is_err(), "{bad}");
        }
        assert_eq!(std::fs::read_to_string(&p).unwrap(), before);
        // a hand-edited file with bad overlay values is ignored with a warning, not trusted
        std::fs::write(&p, r#"{"version":1,"overlays":{"onion":{"before":99}}}"#).unwrap();
        let bad = UiStore::open(Some(p.clone()));
        assert!(bad.warning.is_some() && bad.cfg.overlays.is_none());
        s.update(&json!({"reset": "overlays"})).unwrap();
        assert!(s.cfg.overlays.is_none());
        s.update(&json!({"overlays": {"proxy": {"enabled": false}}})).unwrap();
        s.update(&json!({"overlays": null})).unwrap();
        assert!(s.cfg.overlays.is_none());
        let _ = std::fs::remove_dir_all(p.parent().unwrap().parent().unwrap());
    }

    #[test]
    fn overlay_commands_are_registered_with_free_shortcuts() {
        for id in ["overlay.panel", "overlay.onion_toggle", "overlay.trails_toggle", "overlay.proxy_toggle", "overlay.proxy_level_cycle", "overlay.onion_before_more"] {
            assert!(command(id).is_some(), "{id}");
        }
        let eff = effective_bindings(&BTreeMap::new()).unwrap();
        assert!(conflicts(&eff).is_empty(), "no default conflicts");
        let b = |id: &str| eff.iter().filter(|x| x.command == id).map(|x| binding_string(x.context, &x.chord)).collect::<Vec<_>>();
        assert_eq!(b("overlay.onion_toggle"), vec!["global:Alt+N"]);
        assert_eq!(b("overlay.trails_toggle"), vec!["global:Alt+M"]);
        assert_eq!(b("overlay.proxy_toggle"), vec!["global:Alt+Q"]);
        let hits = search("onion", &[], 20);
        assert!(hits.iter().any(|h| h.id == "overlay.onion_toggle"));
    }

    #[test]
    fn store_survives_a_corrupt_file_and_missing_path() {
        let (_scratch, p) = tmpfile("corrupt");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "{ not json").unwrap();
        let mut s = UiStore::open(Some(p.clone()));
        assert!(s.warning.as_deref().unwrap().contains("ignored stored configuration"));
        assert_eq!(s.cfg, UiConfig::default());
        s.update(&json!({"recents": ["ui.palette"]})).unwrap();
        assert!(s.persisted && s.warning.is_none(), "a successful save replaces the broken file");
        assert_eq!(load_config(&p).unwrap().recents, vec!["ui.palette"]);
        // a file that parses but is invalid is ignored too
        std::fs::write(&p, r#"{"recents":["ghost"]}"#).unwrap();
        assert!(UiStore::open(Some(p.clone())).warning.is_some());
        // no path: memory only
        let mut m = UiStore::open(None);
        m.update(&json!({"recents": ["ui.palette"]})).unwrap();
        assert!(!m.persisted && m.warning.is_some() && m.cfg.recents.len() == 1);
        let _ = std::fs::remove_dir_all(p.parent().unwrap().parent().unwrap());
    }

    /// M2: one unknown key used to discard the whole stored configuration and the next save overwrote it.
    #[test]
    fn unknown_keys_are_ignored_with_a_warning_and_the_file_is_backed_up_before_a_save() {
        let (_scratch, p) = tmpfile("unknown");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        let original = r#"{"version":1,"recents":["ui.palette"],"from_the_future":{"x":1},"shortcuts":{"ui.palette":["global:Ctrl+Alt+Shift+K"]}}"#;
        std::fs::write(&p, original).unwrap();
        let mut s = UiStore::open(Some(p.clone()));
        assert_eq!(s.cfg.recents, vec!["ui.palette"], "known settings are kept: {:?}", s.warning);
        assert_eq!(s.cfg.shortcuts.len(), 1);
        let w = s.warning.clone().unwrap();
        assert!(w.contains("from_the_future") && !w.contains("ignored stored configuration"), "{w}");
        assert!(s.persisted);
        // nothing is overwritten before a save, and the save keeps a copy first
        assert_eq!(std::fs::read_to_string(&p).unwrap(), original);
        s.update(&json!({"recents": ["ui.palette", "canvas.toggle_points"]})).unwrap();
        let bak = p.with_file_name("ui.json.bak");
        assert_eq!(std::fs::read_to_string(&bak).unwrap(), original, "the original is preserved");
        assert!(load_config(&p).is_ok());
        // a second broken file does not replace the first backup
        std::fs::write(&p, "{ broken").unwrap();
        let mut s2 = UiStore::open(Some(p.clone()));
        assert!(s2.warning.is_some() && !s2.persisted);
        s2.update(&json!({"recents": ["ui.palette"]})).unwrap();
        assert_eq!(std::fs::read_to_string(&bak).unwrap(), original);
        assert_eq!(std::fs::read_to_string(p.with_file_name("ui.json.bak.1")).unwrap(), "{ broken");
        // unknown keys nested in other settings are found too, API bodies stay strict
        let nested = parse_ignoring_unknown(json!({"overlays": {"onion": {"enabled": true, "sparkle": 1}}, "recents": []}));
        let (_, removed) = nested.expect("nested unknown key");
        assert_eq!(removed, vec!["overlays.onion.sparkle"]);
        let mut strict = UiStore::open(None);
        let e = strict.update(&json!({"recents": [], "nonsense": 1})).unwrap_err();
        assert!(e.iter().any(|m| m.contains("unknown field 'nonsense'")), "{e:?}");
    }

    #[test]
    fn config_path_lookup_per_platform() {
        use std::ffi::OsString;
        let env = |pairs: &'static [(&'static str, &'static str)]| move |k: &str| pairs.iter().find(|(n, _)| *n == k).map(|(_, v)| OsString::from(*v));
        // explicit override wins everywhere
        let p = config_path_from(env(&[("OSATURA_UI_CONFIG", "/x/u.json"), ("APPDATA", "/a"), ("HOME", "/h")]), true).unwrap();
        assert_eq!(p, PathBuf::from("/x/u.json"));
        // XDG before platform defaults
        let p = config_path_from(env(&[("XDG_CONFIG_HOME", "/xdg"), ("APPDATA", "/a")]), true).unwrap();
        assert_eq!(p, PathBuf::from("/xdg").join("osatura").join("ui.json"));
        // Windows: %APPDATA%\OSATURA\ui.json, then %USERPROFILE%
        let p = config_path_from(env(&[("APPDATA", "/a"), ("USERPROFILE", "/u")]), true).unwrap();
        assert_eq!(p, PathBuf::from("/a").join("OSATURA").join("ui.json"));
        let p = config_path_from(env(&[("USERPROFILE", "/u")]), true).unwrap();
        assert_eq!(p, PathBuf::from("/u").join(".config").join("osatura").join("ui.json"));
        assert!(config_path_from(env(&[("HOME", "/h")]), true).is_none());
        // Unix and macOS: ~/.config/osatura/ui.json (APPDATA is ignored)
        let p = config_path_from(env(&[("HOME", "/h"), ("APPDATA", "/a")]), false).unwrap();
        assert_eq!(p, PathBuf::from("/h").join(".config").join("osatura").join("ui.json"));
        assert!(config_path_from(env(&[]), false).is_none());
    }

    #[test]
    fn default_path_is_outside_the_project() {
        if let Some(p) = default_config_path() {
            assert!(p.ends_with("ui.json") || std::env::var_os("OSATURA_UI_CONFIG").is_some());
        }
    }
}
