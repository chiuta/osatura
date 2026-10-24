// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Command-line parsing: positional arguments, flags, `--key value` options, per-command option checks and usage lookup.
#![allow(unused_imports)]
use super::*;

/// Positional arguments, bare flags, and `--key value` options.
pub(crate) struct Args {
    pub(crate) pos: Vec<String>,
    pub(crate) flags: Vec<String>,
    pub(crate) opts: HashMap<String, String>,
    pub(crate) scale: f32,
}

pub(crate) const VALUE_OPTS: &[&str] = &[
    "--map", "--thigh-l", "--thigh-r", "--shin-l", "--shin-r", "--arm-l", "--arm-r", "--hips", "--period", "--stride",
    "--knee", "--bob", "--swing", "--port", "--scale", "--columns", "--threshold", "--tolerance", "--min-area", "--fill",
    "--factor", "--influences", "--codec", "--pairs", "--tol", "--fps", "--frames", "--generations", "--seed",
    "--from", "--to", "--by", "--pivot", "--ease", "--target", "--prop", "--end", "--axis", "--window", "--interval",
    "--amp", "--period", "--freq", "--decay", "--delay", "--closed", "--jitter", "--chain", "--joint-map",
    "--trim-in", "--trim-out", "--fade-in", "--fade-out", "--blend", "--focus", "--strength", "--fit", "--max-radius", "--samples", "--start", "--keep", "--number", "--gain", "--data", "--takes", "--precision", "--at", "--speed", "--loops", "--mode", "--weight", "--include", "--exclude", "--track-weight", "--rename", "--keys", "--track", "--lang", "--words-out", "--head", "--brow", "--nod", "--fall", "--tilt", "--brow-px", "--calibrate-frames", "--calib", "--layout", "--view", "--smooth", "--rest-frame", "--footlock", "--ground", "--min-conf", "--expr", "--remap", "--svg-frames", "--frame", "--x", "--y", "--rect", "--lasso", "--points", "--locked", "--size", "--offset", "--velocity", "--time", "--scroll", "--sel", "--dt", "--dv", "--value", "--collision", "--clip", "--side", "--t", "--v", "--kind", "--channels", "--extrapolate", "--scope", "--ui-config", "--before", "--after", "--step", "--proxy", "--tint-before", "--tint-after", "--opacity", "--falloff", "--layers", "--adapter", "--bench", "--clients", "--ops", "--bind", "--allow-host", "--public-origin", "--source-url", "--collab-token", "--collab-log", "--replays", "--undo-pct",
    "--a", "--b", "--snap", "--top", "--fix", "--reach", "--free", "--hidden", "--epochs", "--lr", "--library", "--project", "--model", "--prior", "--out", "--apply", "--k",
    "--strategy", "--report", "--heatmap", "--db", "--res", "--max-fail-rate", "--props", "--margin", "--only", "--ignore",
    "--max-size", "--max-seconds", "--format", "--tolerance-px", "--levels", "--out-dir", "--screen-px", "--max-px", "--ratio", "--measure", "--min-ssim", "--pareto", "--search", "--colors", "--corner", "-m", "--message", "--collab-secret-file", "--collab-revoke", "--collab-rate", "--secret-file", "--ttl", "--author", "--repo", "--limit", "-d", "--prune", "--max-input-bytes", "--junit", "--html", "--artifacts", "--metric", "--overlay", "--flicker", "--pixel-de", "--aa", "--denoise", "--fit", "--quant", "--zorder", "--into", "--name", "--scene",
    "--min-side", "--max-side", "--max-faces", "--max-dim", "--eval-n", "--mine-rounds", "--overlay", "--reg-win", "--reg-cell", "--det-win", "--det-cell", "--ridge-lambda", "--stages", "--refine-win", "--stage-patch", "--stage-cell", "--stage-lambda",
];

/// Options whose kind differs from the global table for ONE command: `(command, option, takes_value)`.
/// `VALUE_OPTS` lists the options that take a value in most commands; this table is where a command differs
/// (a boolean `--prune` for `ci run` but `--prune N` for `autotrace`, `--png preview.png` only for `autotrace`).
/// Nothing else special-cases an option in the parser, and a test checks every USAGE entry against it.
pub(crate) const CMD_OPTS: &[(&str, &str, bool)] = &[
    ("ci", "--prune", false),
    ("export-oca", "--layers", false),
    ("keyedit", "--free", false),
    ("autotrace", "--png", true),
];

/// Does `opt` take a value in command `cmd`?
pub(crate) fn takes_value(cmd: &str, opt: &str) -> bool {
    if cmd == "ci" {
        return ci_cmd::is_value_opt(opt); // `ci` has its own table (its subcommands share options)
    }
    if let Some((_, _, v)) = CMD_OPTS.iter().find(|(c, o, _)| *c == cmd && *o == opt) {
        return *v;
    }
    VALUE_OPTS.contains(&opt)
}

/// Rewrites `--key=value` to `--key value` for value options of the command; a `--flag=x` for a boolean flag is
/// refused with the right hint. Unknown `--x=y` words are left alone (they may be positional data).
pub(crate) fn expand_eq(raw: &[String]) -> Result<Vec<String>, String> {
    let cmd = raw.first().filter(|c| !c.starts_with('-')).map(String::as_str).unwrap_or("");
    let known = documented_options(cmd);
    let mut out = Vec::with_capacity(raw.len());
    let mut skip_value = false;
    for (i, a) in raw.iter().enumerate() {
        if skip_value {
            skip_value = false;
            out.push(a.clone());
            continue;
        }
        if i > 0 && a.starts_with("--") && a.len() > 2 {
            if let Some((k, v)) = a.split_once('=') {
                if takes_value(cmd, k) {
                    out.push(k.to_string());
                    out.push(v.to_string());
                    continue;
                }
                if known.iter().any(|o| o == k) || GLOBAL_FLAGS.contains(&k) || (cmd == "ci" && ci_cmd::is_flag(k)) {
                    return Err(format!("{k} is a flag and takes no value: write {k} (without =), got '{a}'"));
                }
            } else if takes_value(cmd, a) {
                skip_value = true; // the next word is this option's value, even if it contains '='
            }
        }
        out.push(a.clone());
    }
    Ok(out)
}

pub(crate) fn parse_args(args: &[String]) -> Result<Args, String> {
    let (mut pos, mut flags, mut opts, mut scale) = (Vec::new(), Vec::new(), HashMap::new(), 1.0f32);
    let cmd = args.first().map(String::as_str).unwrap_or("");
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if takes_value(cmd, a) {
            i += 1;
            let v = args.get(i).ok_or_else(|| format!("{a} needs a value"))?.clone();
            if a == "--scale" && !args.iter().any(|x| x == "mocap-retarget") {
                scale = v.parse().ok().filter(|s: &f32| *s > 0.0 && *s <= 16.0).ok_or("--scale needs a number in (0, 16]")?;
            }
            opts.insert(a.to_string(), v);
        } else if a.starts_with("--") {
            flags.push(a.to_string());
        } else {
            pos.push(a.to_string());
        }
        i += 1;
    }
    Ok(Args { pos, flags, opts, scale })
}

/// Flags every command accepts.
/// (`--json`, `--quiet` and `--allow-empty` are per-command options: a command that does not list them in its usage refuses them.)
pub(crate) const GLOBAL_FLAGS: &[&str] = &["--version", "--help", "--follow-symlinks", "--allow-external-assets"];


/// The USAGE text of one command (its line and continuation lines), for `osatura <command> --help`.
pub(crate) fn usage_for(cmd: &str) -> Option<String> {
    let mut out = String::new();
    let mut inside = false;
    for l in USAGE.lines() {
        if l.starts_with("  osatura ") {
            inside = l.split_whitespace().nth(1) == Some(cmd);
        } else if !l.starts_with("      ") {
            inside = false;
        }
        if inside {
            out.push_str(l);
            out.push('\n');
        }
    }
    if out.is_empty() { None } else { Some(format!("USAGE:\n{out}")) }
}

/// One synopsis form of a command: a USAGE line, or one `|`-separated alternative of a line
/// (`ci init ... | run ... | diff ...`, `keyedit ... scale ... | dup ...`).
#[derive(Debug, Clone)]
pub(crate) struct UsageForm {
    pub(crate) cmd: String,
    /// The sub-command word(s) that select this form (`init`, `commit|revert`); empty for a form without one.
    pub(crate) keys: Vec<String>,
    /// Options written outside `[...]`: the form is only meant when these are present (`trail ... --target X`).
    pub(crate) required: Vec<String>,
    /// Every `--option` of the form, including the ones named in its continuation / description text.
    pub(crate) opts: Vec<String>,
    /// Output extensions the form is for (`<out.gif>`, `<out.json|out.lottie>`): `export` picks its form by the extension.
    pub(crate) exts: Vec<String>,
}

/// The extensions named by `out.<ext>` words of a synopsis (`<out.json|out.lottie>` gives `json`, `lottie`).
pub(crate) fn form_exts(text: &str) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    for w in text.split(|c: char| c.is_whitespace() || c == '<' || c == '>' || c == '|' || c == '[' || c == ']') {
        if let Some(e) = w.strip_prefix("out.") {
            if !e.is_empty() && e.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) && !v.iter().any(|x| x == e) {
                v.push(e.to_string());
            }
        }
    }
    v
}

/// `--names` found in `text` (alphanumerics and '-'); with `top_only`, only outside `[...]`.
pub(crate) fn option_names(text: &str, top_only: bool) -> Vec<String> {
    let (mut out, mut depth) = (Vec::<String>::new(), 0i32);
    let b: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            '[' => depth += 1,
            ']' => depth = (depth - 1).max(0),
            '-' if i + 1 < b.len() && b[i + 1] == '-' && (i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == '-')) && (!top_only || depth == 0) => {
                let mut j = i + 2;
                while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == '-') {
                    j += 1;
                }
                let name: String = b[i..j].iter().collect();
                if name.len() > 2 && !out.contains(&name) {
                    out.push(name);
                }
                i = j;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    out
}

/// Splits at ` | ` (a pipe with white space on both sides) outside `[...]`, `<...>` and `(...)`.
pub(crate) fn split_alternatives(s: &str) -> Vec<String> {
    let (mut out, mut cur, mut depth) = (Vec::new(), String::new(), 0i32);
    let b: Vec<char> = s.chars().collect();
    for (i, &c) in b.iter().enumerate() {
        match c {
            '[' | '<' | '(' => depth += 1,
            ']' | '>' | ')' => depth = (depth - 1).max(0),
            _ => {}
        }
        if c == '|' && depth == 0 && i > 0 && b[i - 1].is_whitespace() && b.get(i + 1).is_some_and(|n| n.is_whitespace()) {
            out.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    out.push(cur);
    out.into_iter().map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
}

/// The sub-command word(s) a synopsis starts with: the first bare lower-case word before the first option or `[...]`
/// (`<placeholders>` are skipped), e.g. `init`, `commit|revert`.
pub(crate) fn form_keys(text: &str) -> Vec<String> {
    for t in text.split_whitespace() {
        if t.starts_with('[') || t.starts_with('-') {
            break;
        }
        if t.starts_with('<') {
            continue;
        }
        let ok = t.split('|').all(|w| !w.is_empty() && w.starts_with(|c: char| c.is_ascii_lowercase()) && w.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'));
        if ok {
            return t.split('|').map(String::from).collect();
        }
    }
    Vec::new()
}

/// Every synopsis form of every command in USAGE (parsed once).
pub(crate) fn usage_forms() -> &'static Vec<UsageForm> {
    static FORMS: std::sync::OnceLock<Vec<UsageForm>> = std::sync::OnceLock::new();
    FORMS.get_or_init(|| {
        // (command, body of the first line, description of the first line, continuation lines)
        let mut lines: Vec<(String, String, String, Vec<String>)> = Vec::new();
        for l in USAGE.lines() {
            if let Some(rest) = l.strip_prefix("  osatura ") {
                let rest = rest.trim_start();
                let cmd = rest.split_whitespace().next().unwrap_or("").to_string();
                let tail = rest[cmd.len()..].trim_start();
                let (body, desc) = match tail.find("   ") {
                    Some(i) => (tail[..i].to_string(), tail[i..].trim().to_string()),
                    None => (tail.to_string(), String::new()),
                };
                lines.push((cmd, body, desc, Vec::new()));
            } else if l.starts_with("      ") {
                if let Some(last) = lines.last_mut() {
                    last.3.push(l.trim().to_string());
                }
            }
        }
        let mut forms = Vec::new();
        for (cmd, body, desc, cont) in lines {
            if cmd.starts_with('-') {
                continue;
            }
            let mut segs: Vec<(String, String)> = split_alternatives(&body)
                .into_iter()
                .map(|seg| {
                    let t0 = seg.split_whitespace().next().unwrap_or("");
                    if t0 == cmd || (t0.starts_with(&format!("{cmd}-")) && !t0.contains('<')) {
                        (t0.to_string(), seg[t0.len()..].trim().to_string())
                    } else {
                        (cmd.clone(), seg)
                    }
                })
                .collect();
            if segs.is_empty() {
                segs.push((cmd.clone(), String::new()));
            }
            // the prose after the gap on the first line describes the command; only its synopsis and continuation
            // lines declare options (a `--features gpu` mentioned in prose is not an option of the command)
            let _ = desc;
            // continuation lines belong to the alternatives that name them ("trim/fade options"), else to the last one
            let cont = cont.join(" ");
            let shared: Vec<usize> = (0..segs.len()).filter(|&i| segs[i].1.contains("trim/fade options")).collect();
            let last = segs.len() - 1;
            let mut tails = vec![String::new(); segs.len()];
            for i in if shared.is_empty() { vec![last] } else { shared } {
                tails[i].push(' ');
                tails[i].push_str(&cont);
            }
            for ((c, text), tail) in segs.into_iter().zip(tails) {
                forms.push(UsageForm { cmd: c, keys: form_keys(&text), required: option_names(&text, true), opts: option_names(&format!("{text} {tail}"), false), exts: form_exts(&text) });
            }
        }
        forms
    })
}

/// Every `--option` mentioned in the USAGE block(s) of `cmd` (documented flags and value options), all forms pooled.
pub(crate) fn documented_options(cmd: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for f in usage_forms().iter().filter(|f| f.cmd == cmd) {
        for o in &f.opts {
            if !out.contains(o) {
                out.push(o.clone());
            }
        }
    }
    out
}

/// The synopsis forms that apply to one invocation: the forms whose sub-command word is among the positionals
/// (else the forms without a sub-command word, else all); among several, those whose required options are present.
pub(crate) fn select_forms<'a>(cmd: &str, pos: &[String], present: &[String]) -> Vec<&'a UsageForm> {
    let all: Vec<&UsageForm> = usage_forms().iter().filter(|f| f.cmd == cmd).collect();
    if all.len() <= 1 {
        return all;
    }
    // forms that differ by the output extension (`export`): the form for the extension of the output file
    if all.iter().all(|f| !f.exts.is_empty() && f.keys.is_empty()) {
        let ext = pos.get(2).and_then(|o| Path::new(o).extension()).and_then(|e| e.to_str()).map(str::to_ascii_lowercase).unwrap_or_default();
        let by_ext: Vec<&UsageForm> = all.iter().copied().filter(|f| f.exts.contains(&ext)).collect();
        if !by_ext.is_empty() && by_ext.len() < all.len() {
            return by_ext;
        }
    }
    let keyed: Vec<&UsageForm> = all.iter().copied().filter(|f| f.keys.iter().any(|k| pos.iter().skip(1).any(|p| p == k))).collect();
    let mut cand = if !keyed.is_empty() {
        keyed
    } else {
        let keyless: Vec<&UsageForm> = all.iter().copied().filter(|f| f.keys.is_empty()).collect();
        if keyless.is_empty() { all } else { keyless }
    };
    if cand.len() > 1 {
        let ok: Vec<&UsageForm> = cand.iter().copied().filter(|f| f.required.iter().all(|r| present.contains(r))).collect();
        if let Some(m) = ok.iter().map(|f| f.required.len()).max() {
            cand = ok.into_iter().filter(|f| f.required.len() == m).collect();
        }
    }
    cand
}

/// Lower-case extension of the output file (third positional) of a command whose forms differ by it.
pub(crate) fn pos_ext(pos: &[String]) -> String {
    pos.get(2).and_then(|o| Path::new(o).extension()).and_then(|e| e.to_str()).map(str::to_ascii_lowercase).unwrap_or_default()
}

/// Options every command accepts besides [`GLOBAL_FLAGS`].
pub(crate) const GLOBAL_OPTS: &[&str] = &["--max-input-bytes"];

pub(crate) fn edit_distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i];
        for j in 1..=b.len() {
            cur.push((prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + usize::from(a[i - 1] != b[j - 1])));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// Refuse an option (flag or value option) that the sub-form of the command does not list, with a "did you mean" hint
/// instead of silently ignoring it. Validation is per synopsis form: `ci diff a.png b.png --quiet` is refused although
/// `ci run` documents `--quiet`.
pub(crate) fn check_flags(a: &Args, raw: &[String]) -> Result<(), String> {
    let Some(cmd) = raw.first().filter(|c| !c.starts_with('-')) else { return Ok(()) };
    let pooled = documented_options(cmd);
    if pooled.is_empty() && usage_for(cmd).is_none() {
        return Ok(()); // not a command with a USAGE block (the dispatcher reports it)
    }
    let mut present: Vec<String> = a.flags.clone();
    present.extend(a.opts.keys().cloned());
    let forms = select_forms(cmd, &a.pos, &present);
    let mut known: Vec<&str> = Vec::new();
    for f in &forms {
        known.extend(f.opts.iter().map(String::as_str));
    }
    let label = if forms.len() == 1 && !forms[0].exts.is_empty() && usage_forms().iter().filter(|u| u.cmd == *cmd).count() > 1 && forms[0].keys.is_empty() {
        format!("{cmd} .{}", pos_ext(&a.pos))
    } else {
        let key = forms.first().filter(|_| forms.len() == 1).and_then(|f| f.keys.iter().find(|k| a.pos.iter().skip(1).any(|p| p == *k)));
        match key {
            Some(k) => format!("{cmd} {k}"),
            None => cmd.clone(),
        }
    };
    let mut given: Vec<&String> = a.flags.iter().collect();
    given.extend(a.opts.keys().filter(|k| k.starts_with("--")));
    given.sort();
    for f in given {
        if known.contains(&f.as_str()) || GLOBAL_FLAGS.contains(&f.as_str()) || GLOBAL_OPTS.contains(&f.as_str()) {
            continue;
        }
        let best = known.iter().copied().chain(GLOBAL_FLAGS.iter().copied()).map(|k| (edit_distance(f, k), k)).min();
        let mut hint = match best {
            Some((d, k)) if d <= 3 || k.starts_with(f.as_str()) => format!(" (did you mean {k}?)"),
            _ => String::new(),
        };
        let owners: Vec<String> = usage_forms()
            .iter()
            .filter(|u| u.cmd == *cmd && u.opts.contains(f) && !forms.iter().any(|x| std::ptr::eq(*x, *u)))
            .map(|u| if u.keys.is_empty() && !u.exts.is_empty() { format!("{cmd} .{}", u.exts.join("/.")) } else if u.keys.is_empty() { cmd.clone() } else { format!("{cmd} {}", u.keys.join("|")) })
            .collect();
        if !owners.is_empty() {
            hint = format!("{hint} ({f} applies to: {})", owners.join(", "));
        }
        return Err(format!("unknown option {f} for '{label}'{hint}; see: osatura {cmd} --help"));
    }
    Ok(())
}

