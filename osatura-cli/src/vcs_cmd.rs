// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! `osatura vcs ...`: command line for the animation version control (see `osatura_core::vcs`).

use osatura_core::semdiff::{Change, Choice, Conflict};
use osatura_core::vcs::{Commit, MergeOutcome, Repo};
use std::collections::HashMap;
use std::path::Path;

pub const HELP: &str = "usage: osatura vcs init <project.json> | commit -m MSG | log [REV] [--limit N] | status | diff [A [B]] | show [REV] | branch [NAME [AT] | -d NAME] | checkout REV [--force] | merge REV [--no-ff] [--abort] | resolve PATH --ours|--theirs|--value JSON | tag [NAME [REV]] | revert REV | cherry-pick REV | fsck | gc   (options: --repo DIR, --author NAME, --json)";

fn short(h: &str) -> &str {
    &h[..h.len().min(10)]
}

fn print_changes(changes: &[Change], json: bool) -> Result<(), String> {
    if json {
        println!("{}", serde_json::to_string_pretty(changes).map_err(|e| e.to_string())?);
    } else if changes.is_empty() {
        println!("no semantic changes");
    } else {
        for c in changes {
            println!("  {}", c.description);
        }
    }
    Ok(())
}

fn print_conflicts(cs: &[Conflict]) {
    for c in cs {
        println!("  CONFLICT {}\n      {}", c.path, c.description);
    }
}

fn print_commit(h: &str, c: &Commit) {
    println!("commit {h}\nAuthor: {}\nTime:   {}\n\n    {}\n", c.author, c.time, c.message.replace('\n', "\n    "));
}

/// Warn (stderr) if the project file as it stands is not a valid OSATURA project.
fn warn_invalid(r: &Repo) {
    if let Ok(p) = r.project_path() {
        let ok = std::fs::read_to_string(&p).map_err(|e| e.to_string()).and_then(|t| osatura_core::Project::from_json(&t).map_err(|e| e.to_string())).and_then(|pr| osatura_core::Engine::new(pr).map(|_| ()).map_err(|e| e.join("; ")));
        if let Err(e) = ok {
            eprintln!("warning: the project file does not validate as an OSATURA project: {e}");
        }
    }
}

fn report_merge(r: &Repo, out: MergeOutcome, what: &str) -> Result<(), String> {
    match out {
        MergeOutcome::UpToDate => println!("already up to date"),
        MergeOutcome::FastForward(h) => println!("fast-forward to {}", short(&h)),
        MergeOutcome::Merged(h) => {
            println!("{what} committed as {}", short(&h));
            warn_invalid(r);
        }
        MergeOutcome::Conflicted(cs) => {
            println!("{what} stopped with {} conflict(s); the project file holds our side for each:", cs.len());
            print_conflicts(&cs);
            println!("resolve each with: osatura vcs resolve <path> --ours|--theirs|--value <json>, then: osatura vcs commit   (or: osatura vcs merge --abort)");
            return Err(format!("{what} has unresolved conflicts"));
        }
    }
    Ok(())
}

pub fn run(pos: &[&str], opts: &HashMap<String, String>, flags: &[String]) -> Result<(), String> {
    let has = |f: &str| flags.iter().any(|x| x == f);
    let json = has("--json");
    let author = opts.get("--author").map(String::as_str);
    let msg = opts.get("-m").or_else(|| opts.get("--message")).map(String::as_str);
    let sub = *pos.first().ok_or(HELP)?;
    if sub == "init" {
        let f = pos.get(1).ok_or("vcs init needs the project file")?;
        let r = Repo::init(Path::new(f), author)?;
        println!("initialised {}/.osatura-vcs for {f} (no commits yet; run: osatura vcs commit -m \"...\")", r.root.display());
        return Ok(());
    }
    let r = Repo::open(Path::new(opts.get("--repo").map(String::as_str).unwrap_or(".")))?;
    let arg = |i: usize| pos.get(i).copied();
    match sub {
        "commit" => {
            let h = r.commit(msg, author)?;
            println!("[{}] {}", short(&h), r.read_commit(&h)?.message.lines().next().unwrap_or(""));
            warn_invalid(&r);
        }
        "log" => {
            let limit = opts.get("--limit").map(|l| l.parse::<usize>().map_err(|_| format!("--limit needs a number, got '{l}'"))).transpose()?.unwrap_or(50);
            let log = r.log(arg(1).unwrap_or("HEAD"), limit)?;
            if json {
                let v: Vec<_> = log.iter().map(|(h, c)| serde_json::json!({"hash": h, "commit": c})).collect();
                println!("{}", serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?);
            } else {
                for (h, c) in &log {
                    let merge = if c.parents.len() > 1 { " (merge)" } else { "" };
                    println!("{}  {}  {}{}", short(h), c.author, c.message.lines().next().unwrap_or(""), merge);
                }
            }
        }
        "status" => {
            let s = r.status()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&s).map_err(|e| e.to_string())?);
            } else {
                match &s.branch {
                    Some(b) => println!("on branch {b}"),
                    None => println!("detached HEAD"),
                }
                match &s.head {
                    Some(h) => println!("HEAD {}", short(h)),
                    None => println!("no commits yet"),
                }
                if let Some(k) = &s.merge {
                    println!("{k} in progress, {} unresolved conflict(s)", s.conflicts.len());
                    print_conflicts(&s.conflicts);
                }
                if s.changes.is_empty() {
                    println!("working file is clean");
                } else {
                    println!("uncommitted changes:");
                    print_changes(&s.changes, false)?;
                }
            }
        }
        "diff" => print_changes(&r.diff(Some(arg(1).unwrap_or("HEAD")), arg(2))?, json)?,
        "show" => {
            let (h, c, ch) = r.show(arg(1).unwrap_or("HEAD"))?;
            if json {
                println!("{}", serde_json::to_string_pretty(&serde_json::json!({"hash": h, "commit": c, "changes": ch})).map_err(|e| e.to_string())?);
            } else {
                print_commit(&h, &c);
                print_changes(&ch, false)?;
            }
        }
        "branch" => match (arg(1), opts.get("-d")) {
            (_, Some(name)) => {
                r.branch_delete(name)?;
                println!("deleted branch {name}");
            }
            (None, None) => {
                for (n, h, cur) in r.branches()? {
                    println!("{} {n}  {}", if cur { "*" } else { " " }, short(&h));
                }
            }
            (Some(n), None) => {
                let h = r.branch_create(n, arg(2))?;
                println!("branch {n} at {}", short(&h));
            }
        },
        "tag" => match arg(1) {
            None => {
                for (n, h) in r.tags()? {
                    println!("{n}  {}", short(&h));
                }
            }
            Some(n) => {
                let h = r.tag_create(n, arg(2))?;
                println!("tag {n} at {}", short(&h));
            }
        },
        "checkout" => {
            let t = arg(1).ok_or("vcs checkout needs a branch, tag or commit")?;
            let h = r.checkout(t, has("--force"))?;
            println!("checked out {t} ({})", short(&h));
        }
        "merge" => {
            if has("--abort") {
                r.abort()?;
                println!("merge aborted; project restored to HEAD");
            } else {
                let rev = arg(1).ok_or("vcs merge needs a branch, tag or commit")?;
                let out = r.merge(rev, has("--no-ff"), msg, author)?;
                report_merge(&r, out, "merge")?;
            }
        }
        "cherry-pick" => report_merge(&r, r.cherry_pick(arg(1).ok_or("vcs cherry-pick needs a commit")?, author)?, "cherry-pick")?,
        "revert" => report_merge(&r, r.revert(arg(1).ok_or("vcs revert needs a commit")?, author)?, "revert")?,
        "resolve" => {
            let path = arg(1).ok_or("vcs resolve needs the conflict path (see vcs status)")?;
            let n = [has("--ours"), has("--theirs"), opts.contains_key("--value")].iter().filter(|x| **x).count();
            if n != 1 {
                return Err("vcs resolve needs exactly one of --ours, --theirs, --value <json>".into());
            }
            let choice = if has("--ours") {
                Choice::Ours
            } else if has("--theirs") {
                Choice::Theirs
            } else {
                let t = &opts["--value"];
                Choice::Value(serde_json::from_str(t).map_err(|e| format!("--value is not valid JSON ({e}): {t}"))?)
            };
            let left = r.resolve(path, &choice)?;
            println!("resolved {path}; {left} conflict(s) left{}", if left == 0 { " - run: osatura vcs commit" } else { "" });
        }
        "fsck" => {
            let bad = r.fsck();
            if bad.is_empty() {
                println!("ok: {} objects, history and refs are consistent", r.object_count()?);
            } else {
                for b in &bad {
                    println!("  PROBLEM {b}");
                }
                return Err(format!("fsck found {} problem(s)", bad.len()));
            }
        }
        "gc" => {
            let g = r.gc()?;
            println!("gc: kept {} objects, removed {} unreachable", g.kept, g.removed);
        }
        _ => return Err(HELP.to_string()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn go(repo: &str, args: &[&str], opts: &[(&str, &str)], flags: &[&str]) -> Result<(), String> {
        let mut o: HashMap<String, String> = opts.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        o.insert("--repo".into(), repo.to_string());
        run(args, &o, &flags.iter().map(|f| f.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn cli_session_with_conflict_and_bad_arguments() {
        let scratch = osatura_core::fsutil::ScratchDir::new("armvcs_cli");
        let d = scratch.to_path_buf();
        let f = d.join("p.json");
        std::fs::write(&f, serde_json::to_string(&osatura_core::demo::project()).unwrap()).unwrap();
        let (fs, rs) = (f.to_str().unwrap(), d.to_str().unwrap());
        run(&["init", fs], &HashMap::new(), &[]).unwrap();
        assert!(run(&["init", fs], &HashMap::new(), &[]).is_err());
        assert!(go(rs, &["commit"], &[], &[]).is_err()); // no message
        go(rs, &["commit"], &[("-m", "first")], &[]).unwrap();
        go(rs, &["branch", "t"], &[], &[]).unwrap();
        let edit = |opacity: f64| {
            let mut v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap();
            v["layers"][0]["opacity"] = serde_json::json!(opacity);
            std::fs::write(&f, v.to_string()).unwrap();
        };
        edit(0.3);
        go(rs, &["commit"], &[("-m", "main")], &[]).unwrap();
        go(rs, &["checkout", "t"], &[], &[]).unwrap();
        edit(0.7);
        go(rs, &["diff"], &[], &["--json"]).unwrap();
        go(rs, &["commit"], &[("-m", "t")], &[]).unwrap();
        go(rs, &["checkout", "main"], &[], &[]).unwrap();
        assert!(go(rs, &["merge", "t"], &[], &[]).unwrap_err().contains("conflicts"));
        assert!(go(rs, &["resolve", "/layers/sky/opacity"], &[], &[]).is_err()); // no choice
        assert!(go(rs, &["resolve", "/layers/sky/opacity"], &[("--value", "{oops")], &[]).is_err());
        assert!(go(rs, &["resolve", "/layers/sky/opacity", "--ours"], &[], &["--ours", "--theirs"]).is_err());
        go(rs, &["resolve", "/layers/sky/opacity"], &[], &["--theirs"]).unwrap();
        go(rs, &["commit"], &[], &[]).unwrap();
        go(rs, &["log"], &[("--limit", "2")], &[]).unwrap();
        assert!(go(rs, &["log"], &[("--limit", "x")], &[]).is_err());
        go(rs, &["status"], &[], &[]).unwrap();
        go(rs, &["show"], &[], &[]).unwrap();
        go(rs, &["tag", "v1"], &[], &[]).unwrap();
        go(rs, &["fsck"], &[], &[]).unwrap();
        go(rs, &["gc"], &[], &[]).unwrap();
        assert!(go(rs, &["bogus"], &[], &[]).is_err());
        assert!(go(rs, &[], &[], &[]).is_err());
        assert!(go(rs, &["checkout"], &[], &[]).is_err());
        assert!(go(rs, &["merge", "nope"], &[], &[]).is_err());
        assert!(go(d.join("none").to_str().unwrap(), &["status"], &[], &[]).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }
}
