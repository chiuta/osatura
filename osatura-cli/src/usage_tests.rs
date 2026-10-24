// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tests of the argument parser, usage text and shared helpers of `main.rs`.
#![allow(unused_imports)]

use super::*;

/// C12g: `ok` may not be true when the merged project does not compile.
#[test]
fn collab_sim_ok_requires_a_compiling_merge() {
    assert!(collab_ok(true, true));
    assert!(!collab_ok(true, false));
    assert!(!collab_ok(false, true));
}

/// H3 (wave 4): the same word is a flag in one command and a value option in another.
#[test]
fn option_table_is_per_command() {
    assert!(!takes_value("ci", "--prune") && !takes_value("export-oca", "--layers") && !takes_value("keyedit", "--free"));
    assert!(takes_value("autotrace", "--png") && takes_value("render", "--scale") && takes_value("lod", "--tolerance-px"));
    let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    let a = parse_args(&s(&["ci", "run", "--prune"])).unwrap();
    assert!(a.flags.iter().any(|f| f == "--prune") && a.opts.is_empty());
    assert_eq!(expand_eq(&s(&["lod", "d.json", "--levels=3"])).unwrap(), s(&["lod", "d.json", "--levels", "3"]));
    assert!(expand_eq(&s(&["ci", "run", "--prune=1"])).is_err());
    assert_eq!(expand_eq(&s(&["render", "--scale", "a=b"])).unwrap(), s(&["render", "--scale", "a=b"]));
}

#[test]
fn unknown_flags_are_refused_with_a_suggestion() {
    let go = |v: &[&str]| {
        let raw: Vec<String> = v.iter().map(|s| s.to_string()).collect();
        check_flags(&parse_args(&raw).unwrap(), &raw)
    };
    let e = go(&["render", "p.json", "0", "o.png", "--bone"]).unwrap_err();
    assert!(e.contains("--bone") && e.contains("did you mean --bones"), "{e}");
    let e = go(&["export-fit", "p.json", "o.json", "--audi"]).unwrap_err();
    assert!(e.contains("--audit"), "{e}");
    assert!(go(&["render", "p.json", "0", "o.png", "--bones"]).is_ok());
    // options are per command: `--json` belongs to the commands whose usage lists it
    let e = go(&["render", "p.json", "0", "o.png", "--json"]).unwrap_err();
    assert!(e.contains("unknown option --json for 'render'"), "{e}");
    assert!(go(&["timeline", "p.json", "--json"]).is_ok());
    // commands whose usage line lists no option at all refuse stray flags too
    for c in [vec!["check", "p.json"], vec!["stats", "p.json"], vec!["lint", "p.json"], vec!["coach", "p.json"], vec!["contracts", "p.json", "c.json"], vec!["machine", "list", "p.json"]] {
        let mut v = c.clone();
        v.push("--foo");
        let e = go(&v).unwrap_err();
        assert!(e.contains("unknown option --foo"), "{c:?}: {e}");
        assert!(go(&c).is_ok(), "{c:?}");
    }
}

/// Splits a CLI source file into `(commands, text)` chunks: in `main.rs` one chunk per `["command", ...] =>` arm of
/// the dispatcher (commands = every `["name"` of the arm's pattern) and `None` for code outside the arms (shared
/// helpers); other modules are one chunk for the commands they implement.
fn option_read_sites() -> Vec<(String, Option<Vec<String>>, String)> {
    let strip = |body: &str| -> String {
        let body = body.split("mod usage_tests").next().unwrap().to_string();
        // the option tables (VALUE_OPTS, CMD_OPTS) list names for the parser, they are not reads
        let body = match (body.find("const VALUE_OPTS"), body.find("const GLOBAL_FLAGS")) {
            (Some(a), Some(b)) if a < b => format!("{}{}", &body[..a], &body[b..]),
            _ => body,
        };
        match (body.find("const USAGE"), body.find("\";\n")) {
            (Some(a), Some(_)) => {
                let end = body[a..].find("\";\n").map(|e| a + e).unwrap_or(a);
                format!("{}{}", &body[..a], &body[end..])
            }
            _ => body,
        }
    };
    let cmds = |names: &[&str]| Some(names.iter().map(|s| s.to_string()).collect::<Vec<_>>());
    let face: Vec<&str> = vec!["face-synth", "face-train", "face-eval", "face-detect", "face-track", "face-calibrate", "face-bake"];
    let mut out = vec![
        ("lod_cmd.rs".to_string(), cmds(&["lod", "lod-pick"]), strip(include_str!("lod_cmd.rs"))),
        ("fit_cmd.rs".to_string(), cmds(&["export-fit", "size-report"]), strip(include_str!("fit_cmd.rs"))),
        ("fuzz_cmd.rs".to_string(), cmds(&["fuzz", "fuzz-grid"]), strip(include_str!("fuzz_cmd.rs"))),
        ("vcs_cmd.rs".to_string(), cmds(&["vcs"]), strip(include_str!("vcs_cmd.rs"))),
        ("face_cmd.rs".to_string(), cmds(&face), strip(include_str!("face_cmd.rs"))),
    ];
    // main.rs: chunk per dispatcher arm
    let main = strip(include_str!("main.rs"));
    let mut cur: (Option<Vec<String>>, String) = (None, String::new());
    let mut in_header = false;
    for l in main.lines() {
        let is_arm = l.starts_with("        [\"");
        let top = !l.is_empty() && !l.starts_with(' ');
        if is_arm || top {
            if !cur.1.is_empty() {
                out.push(("main.rs".to_string(), cur.0.take(), std::mem::take(&mut cur.1)));
            }
            cur.0 = None;
            in_header = is_arm;
            if is_arm {
                cur.0 = Some(Vec::new());
            }
        }
        if in_header {
            let mut rest = l;
            while let Some(i) = rest.find("[\"") {
                let t = &rest[i + 2..];
                let e = t.find('"').unwrap_or(0);
                if let Some(v) = cur.0.as_mut() {
                    v.push(t[..e].to_string());
                }
                rest = &t[e..];
            }
            if l.contains("=>") {
                in_header = false;
            }
        }
        cur.1.push_str(l);
        cur.1.push('\n');
    }
    out.push(("main.rs".to_string(), cur.0, cur.1));
    out
}

/// Every `"--flag"` literal the CLI sources read is listed in the USAGE of THE COMMAND whose code reads it
/// (a dispatcher arm of `main.rs`, or the command's own module), not merely somewhere in USAGE. Code shared by
/// several commands (outside the arms) is checked against the USAGE as a whole.
#[test]
fn every_flag_the_code_reads_is_in_the_strict_tables() {
    let all: Vec<String> = usage_forms().iter().flat_map(|f| f.opts.clone()).collect();
    let mut missing: Vec<String> = Vec::new();
    for (file, cmds, body) in option_read_sites() {
        let allowed: Option<Vec<String>> = cmds.filter(|c| !c.is_empty()).map(|c| c.iter().flat_map(|c| documented_options(c)).collect());
        let mut rest = body.as_str();
        while let Some(i) = rest.find("\"--") {
            let tail = &rest[i + 1..];
            let name = &tail[..tail.find('"').unwrap_or(0)];
            rest = &tail[name.len().max(1)..];
            if name.len() <= 2 || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') || GLOBAL_FLAGS.contains(&name) || GLOBAL_OPTS.contains(&name) {
                continue;
            }
            let ok = match &allowed {
                Some(a) => a.iter().any(|o| o == name),
                None => all.iter().any(|o| o == name),
            };
            let label = match &allowed { Some(_) => file.clone(), None => format!("{file} (shared code)") };
            let entry = format!("{label}: {name}");
            if !ok && !missing.contains(&entry) {
                missing.push(entry);
            }
        }
    }
    assert!(missing.is_empty(), "flags read by a command but not in that command's USAGE: {missing:?}");
}

fn args(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

fn refused(v: &[&str]) -> Option<String> {
    let raw = args(v);
    let expanded = expand_eq(&raw).unwrap();
    check_flags(&parse_args(&expanded).unwrap(), &expanded).err()
}

/// Flag validation is per synopsis form: an option listed for one sub-command is refused on its siblings.
#[test]
fn options_are_validated_per_sub_command() {
    // ci: --quiet only for `run`, --out/--no-goldens only for `init`, --metric only for `diff`
    assert!(refused(&["ci", "run", "--quiet"]).is_none());
    assert!(refused(&["ci", "diff", "a.png", "b.png", "--quiet"]).unwrap().contains("unknown option --quiet for 'ci diff'"));
    assert!(refused(&["ci", "init", "s.json", "--quiet"]).unwrap().contains("--quiet"));
    assert!(refused(&["ci", "init", "s.json", "--no-goldens"]).is_none());
    assert!(refused(&["ci", "run", "--no-goldens"]).is_some());
    assert!(refused(&["ci", "diff", "a.png", "b.png", "--metric", "mse", "--tol", "3"]).is_none());
    // trail: --json only on the --target form
    assert!(refused(&["trail", "s.json", "hand", "t.svg", "--json"]).unwrap().contains("--json"));
    assert!(refused(&["trail", "s.json", "t.svg", "--target", "bone:hand", "--json"]).is_none());
    assert!(refused(&["trail", "s.json", "hand", "t.svg", "--from", "1", "--to", "5"]).is_none());
    // ui: --json for commands|shortcuts only
    assert!(refused(&["ui", "layout", "presets", "--json"]).unwrap().contains("--json"));
    assert!(refused(&["ui", "search", "x", "--json"]).is_some());
    assert!(refused(&["ui", "commands", "--json"]).is_none());
    assert!(refused(&["ui", "shortcuts", "--json", "--ui-config", "f.json"]).is_none());
    assert!(refused(&["ui", "layout", "show", "rig", "--ui-config", "f.json"]).is_some());
    // golden: --scale only for write, --tol only for check (value options are validated too)
    assert!(refused(&["golden", "write", "p.json", "g.json", "--tol", "0.1"]).unwrap().contains("--tol"));
    assert!(refused(&["golden", "check", "p.json", "g.json", "--scale", "0.25"]).unwrap().contains("--scale"));
    assert!(refused(&["golden", "write", "p.json", "g.json", "--scale", "0.25"]).is_none());
    assert!(refused(&["golden", "check", "p.json", "g.json", "--tol", "0.1"]).is_none());
    // vcs: --json only where the sub-command prints JSON; --force only for checkout
    assert!(refused(&["vcs", "status", "--json"]).is_none());
    assert!(refused(&["vcs", "commit", "-m", "x", "--json"]).is_some());
    assert!(refused(&["vcs", "gc", "--json"]).is_some());
    assert!(refused(&["vcs", "log", "--force"]).is_some());
    assert!(refused(&["vcs", "checkout", "main", "--force"]).is_none());
    assert!(refused(&["vcs", "merge", "dev", "--no-ff", "--repo", "r"]).is_none());
    assert!(refused(&["vcs", "revert", "abc", "--ours"]).is_some());
    // keyedit: each edit lists its own options
    assert!(refused(&["keyedit", "p.json", "o.json", "move", "--sel", "a:b", "--dt", "2", "--free"]).is_none());
    assert!(refused(&["keyedit", "p.json", "o.json", "delete", "--sel", "a:b", "--dt", "2"]).unwrap().contains("--dt"));
    assert!(refused(&["keyedit", "p.json", "o.json", "smooth", "--sel", "a:b", "--kind", "monotone"]).is_none());
    assert!(refused(&["keyedit", "p.json", "o.json", "ease", "--sel", "a:b", "--ease", "linear", "--kind", "x"]).is_some());
    // pose-suggest / pose-db / clip / sound / bind / container / scroll sub-forms
    assert!(refused(&["pose-suggest", "next", "p.json", "-", "--frame", "1", "--json"]).is_some());
    assert!(refused(&["pose-suggest", "between", "p.json", "-", "--a", "1", "--b", "2", "--json"]).is_none());
    assert!(refused(&["pose-db", "info", "d.json", "--step", "2"]).is_some());
    assert!(refused(&["clip", "list", "p.json", "--at", "2"]).is_some());
    assert!(refused(&["sound", "remove", "p.json", "o.json", "n", "--gain", "2"]).is_some());
    assert!(refused(&["sound", "edit", "p.json", "o.json", "n", "--gain", "2", "--fade-in", "1"]).is_none());
    assert!(refused(&["container", "list", "p.json", "--size", "10x10"]).is_some());
    assert!(refused(&["container", "solve", "p.json", "--size", "10x10"]).is_none());
    assert!(refused(&["scroll", "list", "p.json", "--offset", "1,1"]).is_some());
    assert!(refused(&["bind-eval", "p.json", "v.json", "--data", "o.json"]).is_none());
    assert!(refused(&["bind", "list", "p.json", "--expr", "x"]).is_some());
    // onion: the two forms
    assert!(refused(&["onion", "p.json", "0", "o.png", "--keyed"]).is_some());
    assert!(refused(&["onion", "p.json", "o.svg", "--frame", "1", "--keyed"]).is_none());
    // keys / idle / comp / snapshot: one option set per sub-command
    assert!(refused(&["keys", "shift", "p.json", "o.json", "--by", "2", "--target", "a", "--prop", "bone_angle", "--from", "1", "--to", "5"]).is_none());
    assert!(refused(&["keys", "shift", "p.json", "o.json", "--by", "2", "--factor", "2"]).unwrap().contains("unknown option --factor for 'keys shift'"));
    assert!(refused(&["keys", "scale", "p.json", "o.json", "--factor", "2", "--pivot", "0"]).is_none());
    assert!(refused(&["keys", "scale", "p.json", "o.json", "--factor", "2", "--ease", "linear"]).unwrap().contains("applies to: keys ease"));
    assert!(refused(&["keys", "ease", "p.json", "o.json", "--ease", "smooth", "--by", "1"]).unwrap().contains("--by"));
    assert!(refused(&["keys", "reverse", "p.json", "o.json", "--from", "1", "--pivot", "3"]).unwrap().contains("--pivot"));
    assert!(refused(&["idle", "blink", "p.json", "o.json", "--target", "a", "--prop", "layer_switch", "--interval", "2", "--seed", "3"]).is_none());
    assert!(refused(&["idle", "blink", "p.json", "o.json", "--target", "a", "--prop", "p", "--amp", "2"]).unwrap().contains("unknown option --amp for 'idle blink'"));
    assert!(refused(&["idle", "breath", "p.json", "o.json", "--target", "a", "--prop", "p", "--seed", "2"]).unwrap().contains("--seed"));
    assert!(refused(&["idle", "breath", "p.json", "o.json", "--target", "a", "--prop", "p", "--period", "40"]).is_none());
    assert!(refused(&["idle", "sway", "p.json", "o.json", "--target", "a", "--prop", "p", "--period", "40"]).unwrap().contains("applies to: idle breath"));
    assert!(refused(&["idle", "sway", "p.json", "o.json", "--target", "a", "--prop", "p", "--freq", "1", "--seed", "4"]).is_none());
    assert!(refused(&["comp", "list", "p.json", "--name", "x"]).is_some());
    assert!(refused(&["comp", "capture", "p.json", "o.json", "n"]).is_none());
    assert!(refused(&["snapshot", "save", "p.json", "d", "--keep", "3"]).is_none());
    assert!(refused(&["snapshot", "list", "p.json", "d", "--keep", "3"]).unwrap().contains("applies to: snapshot save"));
    assert!(refused(&["snapshot", "restore", "p.json", "d", "o.json", "--number", "2"]).is_none());
    assert!(refused(&["snapshot", "save", "p.json", "d", "--number", "2"]).unwrap().contains("applies to: snapshot restore"));
    assert!(refused(&["snapshot", "recover", "p.json", "d", "o.json", "--number", "2"]).is_some());
    // export: the output extension picks the form, and an option that the format does not use is refused
    assert!(refused(&["export", "p.json", "o.gif", "--scale", "0.5", "--report", "r.json"]).is_none());
    assert!(refused(&["export", "p.json", "O.GIF", "--scale", "0.5"]).is_none());
    let e = refused(&["export", "p.json", "o.json", "--scale", "0.5"]).unwrap();
    assert!(e.contains("unknown option --scale for 'export .json'") && e.contains("applies to: export .gif"), "{e}");
    assert!(refused(&["export", "p.json", "o.lottie", "--report", "r.json"]).is_none());
    assert!(refused(&["export", "p.json", "o.html", "--report", "r.json"]).unwrap().contains("--report"));
    assert!(refused(&["export", "p.json", "o.htm", "--scale", "2"]).is_some());
    assert!(refused(&["export", "p.json", "o.mp4", "--scale", "2"]).is_none());
    assert!(refused(&["export", "p.json", "o.mp4", "--codec", "av1"]).unwrap().contains("applies to: export .webm"));
    assert!(refused(&["export", "p.json", "o.webm", "--codec", "av1", "--scale", "2"]).is_none());
    assert!(refused(&["export", "p.json", "o.apng", "--columns", "2"]).unwrap().contains("--columns"));
    assert!(refused(&["export", "p.json", "o.png", "--columns", "2", "--scale", "0.5"]).is_none());
    assert!(refused(&["export", "p.json", "o.glb", "--scale", "2"]).is_some());
    assert!(refused(&["export", "p.json", "o.glb", "--report", "r.json"]).is_none());
    // an unknown or missing extension: every export option is let through, the command reports the format error
    assert!(refused(&["export", "p.json", "o.xyz", "--scale", "2"]).is_none());
    // the error names the sub-command that does accept the option
    let e = refused(&["ci", "diff", "a.png", "b.png", "--quiet"]).unwrap();
    assert!(e.contains("applies to: ci run"), "{e}");
}

/// The first form of every USAGE line is parsed into the options the docs and the validation use; a few anchors.
#[test]
fn usage_forms_split_alternatives() {
    let f: Vec<&UsageForm> = usage_forms().iter().filter(|f| f.cmd == "ci").collect();
    assert_eq!(f.len(), 4, "{f:?}");
    assert_eq!(f.iter().map(|x| x.keys[0].as_str()).collect::<Vec<_>>(), ["init", "run", "bless", "diff"]);
    assert!(select_forms("golden", &args(&["golden", "write"]), &[]).iter().all(|f| f.keys == ["write"]));
    assert_eq!(usage_forms().iter().filter(|f| f.cmd == "keyedit").count(), 16);
    assert_eq!(usage_forms().iter().filter(|f| f.cmd == "keys").count(), 4);
    assert_eq!(usage_forms().iter().filter(|f| f.cmd == "idle").count(), 3);
    assert_eq!(usage_forms().iter().filter(|f| f.cmd == "snapshot").count(), 4);
    assert_eq!(usage_forms().iter().filter(|f| f.cmd == "comp").count(), 2);
    let ex: Vec<&UsageForm> = usage_forms().iter().filter(|f| f.cmd == "export").collect();
    assert_eq!(ex.len(), 8);
    // every extension the dispatcher accepts has exactly one export form
    for e in ["gif", "html", "htm", "json", "lottie", "mp4", "apng", "webm", "png", "glb"] {
        assert_eq!(ex.iter().filter(|f| f.exts.iter().any(|x| x == e)).count(), 1, "{e}");
    }
    assert!(usage_forms().iter().any(|f| f.cmd == "bind-eval"));
}

/// CLI_REFERENCE.md: every option of a command's USAGE appears in that command's synopsis lines of the reference
/// (and no synopsis line there names an option the USAGE of the command does not have).
#[test]
fn cli_reference_synopses_match_usage() {
    let doc = include_str!("../../docs/CLI_REFERENCE.md");
    // synopsis lines: bold-label lines (`**Synopsis:** `...``) and table cells with a backticked `osatura <cmd> ...`, and
    // bare `osatura <cmd> ...` lines inside code fences with their indented continuation lines (never `$ ` examples)
    let mut syn: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
    let mut fence = false;
    let mut last: Option<(String, usize)> = None; // the fence piece an indented continuation line extends
    for l in doc.lines() {
        if l.trim_start().starts_with("```") {
            fence = !fence;
            last = None;
            continue;
        }
        let t = l.trim();
        let mut pieces: Vec<&str> = Vec::new();
        if fence {
            if t.starts_with("osatura ") {
                pieces.push(t);
            } else if l.starts_with(char::is_whitespace) && !t.starts_with('$') {
                if let Some((c, i)) = &last {
                    let v = syn.get_mut(c).unwrap();
                    v[*i].push(' ');
                    v[*i].push_str(t);
                }
                continue;
            } else {
                last = None;
            }
        } else if t.starts_with("**") || t.starts_with('|') {
            let mut rest = t;
            while let Some(i) = rest.find("`osatura ") {
                let s = &rest[i + 1..];
                let e = s.find('`').unwrap_or(s.len());
                pieces.push(&s[..e]);
                rest = &s[e..];
            }
        }
        for p in pieces {
            if let Some(cmd) = p.split_whitespace().nth(1) {
                let v = syn.entry(cmd.to_string()).or_default();
                v.push(p.to_string());
                last = if fence { Some((cmd.to_string(), v.len() - 1)) } else { None };
            }
        }
    }
    let mut problems: Vec<String> = Vec::new();
    let mut cmds: Vec<String> = usage_forms().iter().map(|f| f.cmd.clone()).collect();
    cmds.sort();
    cmds.dedup();
    for cmd in &cmds {
        let usage_opts = documented_options(cmd);
        let Some(lines) = syn.get(cmd) else {
            // commands with no synopsis line of their own (sub-forms of another command's section)
            if !usage_opts.is_empty() {
                problems.push(format!("{cmd}: no synopsis line in CLI_REFERENCE.md"));
            }
            continue;
        };
        let text = lines.join(" ").replace("\\|", "|");
        let doc_opts = option_names(&text, false);
        for o in &usage_opts {
            if !doc_opts.contains(o) {
                problems.push(format!("{cmd}: {o} is in USAGE but not in the CLI_REFERENCE.md synopsis"));
            }
        }
        for o in &doc_opts {
            if !usage_opts.contains(o) && !GLOBAL_FLAGS.contains(&o.as_str()) && !GLOBAL_OPTS.contains(&o.as_str()) {
                problems.push(format!("{cmd}: {o} is in the CLI_REFERENCE.md synopsis but not in USAGE"));
            }
        }
    }
    assert!(problems.is_empty(), "{} problem(s):\n{}", problems.len(), problems.join("\n"));
}

/// Every USAGE line that has no alternatives appears word for word (white space aside) in CLI_REFERENCE.md.
#[test]
fn cli_reference_repeats_usage_lines_verbatim() {
    let norm = |t: &str| t.split_whitespace().collect::<Vec<_>>().join(" ").replace("\\|", "|");
    let doc = norm(include_str!("../../docs/CLI_REFERENCE.md"));
    let mut missing = Vec::new();
    for l in USAGE.lines().filter(|l| l.starts_with("  osatura ")) {
        let body = l.trim().split("   ").next().unwrap();
        if body.starts_with("osatura --") || split_alternatives(body).len() > 1 {
            continue;
        }
        if !doc.contains(&norm(body)) {
            missing.push(body.to_string());
        }
    }
    assert!(missing.is_empty(), "USAGE lines missing from CLI_REFERENCE.md:\n{}", missing.join("\n"));
}

/// `golden write --scale` above 1 is refused (it used to be clamped silently); `golden check --scale` is refused.
#[test]
fn golden_scale_rules() {
    assert!(refused(&["golden", "check", "p.json", "g.json", "--scale", "0.5"]).is_some());
    assert!(usage_for("golden").unwrap().contains("(0, 1]"));
}

/// C2: every option in USAGE must parse the way the documentation says: `[--x]` is a boolean
/// flag, `[--x VALUE]` takes a value.
#[test]
fn every_documented_option_parses_as_documented() {
    // join continuation lines to their command
    let mut cmds: Vec<String> = Vec::new();
    for l in USAGE.lines() {
        if l.starts_with("  osatura ") {
            // drop the padding after the command name and the prose after the gap (padded names such as `ci       init`
            // used to cut the line short, so those commands were never checked)
            let t = l.trim();
            let mut it = t.split_whitespace();
            let (a0, a1) = (it.next().unwrap(), it.next().unwrap());
            let rest = t[t.find(a1).unwrap() + a1.len()..].trim_start();
            cmds.push(format!("{a0} {a1} {}", rest.split("   ").next().unwrap()));
        } else if l.starts_with("      ") && !cmds.is_empty() {
            let last = cmds.len() - 1;
            cmds[last].push(' ');
            cmds[last].push_str(l.trim().split("   ").next().unwrap());
        }
    }
    assert!(cmds.len() > 50);
    let mut checked = 0;
    let mut bad: Vec<String> = Vec::new();
    for c in &cmds {
        let toks: Vec<&str> = c.split_whitespace().collect();
        let cmd = toks[1];
        if cmd.starts_with('-') {
            continue;
        }
        for (i, t) in toks.iter().enumerate() {
            let name = t.trim_start_matches('[').trim_end_matches([']', ',']);
            let name = name.split('|').next().unwrap();
            if !name.starts_with("--") || name.len() < 3 || name.contains('<') || name.contains('=') {
                continue;
            }
            // value if the next token exists and is a placeholder (not a closing/another option)
            let closes = t.ends_with(']');
            let next = toks.get(i + 1).copied().unwrap_or("");
            let takes_value = !closes && !next.is_empty() && !next.starts_with('[') && !next.starts_with("--") && !next.starts_with('(') && !next.starts_with('<') || (!closes && next.starts_with('<'));
            let args: Vec<String> = [cmd, name, "2", "after"].iter().map(|s| s.to_string()).collect();
            let Ok(a) = parse_args(&args) else { panic!("{cmd} {name}: parse failed") };
            if takes_value {
                if a.opts.get(name).map(String::as_str) != Some("2") {
                    bad.push(format!("{cmd} {name}: documented with a value but parsed as a flag"));
                }
            } else {
                if !(a.flags.iter().any(|f| f == name) && a.pos.iter().any(|p| p == "2")) {
                    bad.push(format!("{cmd} {name}: documented as a boolean flag but swallows the next argument"));
                }
            }
            checked += 1;
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
    assert!(checked > 100, "only {checked} options checked");
}
