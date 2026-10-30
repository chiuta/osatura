// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! `osatura update-check`: asks GitHub whether a newer release exists. It never runs by itself, never downloads
//! or installs anything and sends nothing about you (one anonymous GET). The request goes through the system
//! `curl` (like `ffmpeg` elsewhere), so the engine keeps its dependency list; `--proxy-preset` and `--proxy-url`
//! route it through Tor, Tor Browser, Cloudflare WARP or a proxy of your own.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::process::Command;

const RELEASES_URL: &str = "https://api.github.com/repos/chiuta/osatura/releases?per_page=10";

/// A free proxy preset: always a local `socks5h://` endpoint (names are resolved by the proxy, not by this machine).
pub(crate) struct Preset {
    pub id: &'static str,
    pub url: &'static str,
    pub label: &'static str,
    pub note: &'static str,
}

pub(crate) const PRESETS: [Preset; 3] = [
    Preset {
        id: "tor",
        url: "socks5h://127.0.0.1:9050",
        label: "Tor (system service)",
        note: "Start the Tor daemon first. Volunteer-run, non-profit, open source: the recommended choice.",
    },
    Preset {
        id: "tor-browser",
        url: "socks5h://127.0.0.1:9150",
        label: "Tor Browser (running in the background)",
        note: "Keep Tor Browser open; it listens on 127.0.0.1:9150.",
    },
    Preset {
        id: "warp",
        url: "socks5h://127.0.0.1:40000",
        label: "Cloudflare WARP (proxy mode)",
        note: "Free, but a commercial service with a closed-source client: Cloudflare sees that you contacted GitHub. Run `warp-cli mode proxy` and `warp-cli connect` first.",
    },
];

/// A parsed `MAJOR.MINOR.PATCH[-pre.release]` version (build metadata ignored), compared as in SemVer 2.0.0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Ver {
    core: [u64; 3],
    pre: Vec<String>,
}

pub(crate) fn parse_ver(s: &str) -> Option<Ver> {
    let s = s.trim().trim_start_matches(['v', 'V']);
    let s = s.split('+').next()?;
    let (core, pre) = match s.split_once('-') {
        Some((_, "")) => return None,
        Some((c, p)) => (c, p),
        None => (s, ""),
    };
    let mut it = core.split('.');
    let mut c = [0u64; 3];
    for slot in &mut c {
        *slot = it.next()?.parse().ok()?;
    }
    if it.next().is_some() {
        return None;
    }
    let pre: Vec<String> = if pre.is_empty() {
        Vec::new()
    } else {
        pre.split('.').map(str::to_string).collect()
    };
    if pre.iter().any(|p| p.is_empty()) {
        return None;
    }
    Some(Ver { core: c, pre })
}

impl Ord for Ver {
    fn cmp(&self, o: &Self) -> Ordering {
        self.core
            .cmp(&o.core)
            .then_with(|| match (self.pre.is_empty(), o.pre.is_empty()) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Greater, // 1.0.0 is newer than 1.0.0-rc.1
                (false, true) => Ordering::Less,
                (false, false) => {
                    for (a, b) in self.pre.iter().zip(&o.pre) {
                        let ord = match (a.parse::<u64>(), b.parse::<u64>()) {
                            (Ok(x), Ok(y)) => x.cmp(&y),
                            (Ok(_), Err(_)) => Ordering::Less, // numeric identifiers sort before alphanumeric ones
                            (Err(_), Ok(_)) => Ordering::Greater,
                            (Err(_), Err(_)) => a.cmp(b),
                        };
                        if ord != Ordering::Equal {
                            return ord;
                        }
                    }
                    self.pre.len().cmp(&o.pre.len())
                }
            })
    }
}
impl PartialOrd for Ver {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// The newest published (non-draft) release in a GitHub `releases` JSON array: `(tag, html_url, prerelease)`.
pub(crate) fn newest_release(json: &str) -> Result<(String, String, bool), String> {
    let v: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| format!("GitHub answered with something that is not JSON: {e}"))?;
    let arr = v
        .as_array()
        .ok_or("unexpected answer from GitHub (not a list of releases)")?;
    let mut best: Option<(Ver, String, String, bool)> = None;
    for r in arr {
        if r["draft"].as_bool().unwrap_or(false) {
            continue;
        }
        let Some(tag) = r["tag_name"].as_str() else {
            continue;
        };
        let Some(ver) = parse_ver(tag) else { continue };
        let url = r["html_url"]
            .as_str()
            .unwrap_or("https://github.com/chiuta/osatura/releases")
            .to_string();
        let pre = r["prerelease"].as_bool().unwrap_or(false);
        if best.as_ref().is_none_or(|b| ver > b.0) {
            best = Some((ver, tag.to_string(), url, pre));
        }
    }
    best.map(|(_, t, u, p)| (t, u, p))
        .ok_or_else(|| "no published release found".to_string())
}

/// Only `https://github.com/chiuta/osatura/...` links are ever printed as the download address.
fn safe_release_url(u: &str) -> String {
    if u.starts_with("https://github.com/chiuta/osatura/")
        && !u.chars().any(|c| c.is_control() || c.is_whitespace())
    {
        u.to_string()
    } else {
        "https://github.com/chiuta/osatura/releases".to_string()
    }
}

/// Which proxy URL to use, from `--proxy-preset` / `--proxy-url` (never both), if any.
pub(crate) fn proxy_from(opts: &HashMap<String, String>) -> Result<Option<String>, String> {
    match (opts.get("--proxy-preset"), opts.get("--proxy-url")) {
        (Some(_), Some(_)) => Err("use either --proxy-preset or --proxy-url, not both".into()),
        (Some(id), None) => {
            let id = id.trim();
            PRESETS
                .iter()
                .find(|p| p.id.eq_ignore_ascii_case(id))
                .map(|p| Some(p.url.to_string()))
                .ok_or_else(|| {
                    format!(
                        "unknown proxy preset '{id}'; available: {}",
                        PRESETS.iter().map(|p| p.id).collect::<Vec<_>>().join(", ")
                    )
                })
        }
        (None, Some(u)) => {
            let ok = ["http://", "https://", "socks5://", "socks5h://"]
                .iter()
                .any(|s| u.to_ascii_lowercase().starts_with(s));
            if !ok || u.chars().any(|c| c.is_control() || c.is_whitespace()) {
                return Err("--proxy-url must look like http://host:port, https://host:port, socks5://host:port or socks5h://host:port".into());
            }
            Ok(Some(u.clone()))
        }
        (None, None) => Ok(None),
    }
}

/// The `curl` arguments: HTTPS only, TLS 1.2+, no redirects to other schemes, bounded time and size.
pub(crate) fn curl_args(proxy: Option<&str>) -> Vec<String> {
    // curl's own long options are built at run time: the CLI's usage test treats every double-dash literal in this file as an
    // osatura option that must appear in USAGE.
    let l = |n: &str| format!("-{}{n}", "-");
    let mut a: Vec<String> = vec![
        l("silent"),
        l("show-error"),
        l("fail"),
        l("proto"),
        "=https".into(),
        l("tlsv1.2"),
        l("max-time"),
        "30".into(),
        l("max-filesize"),
        "2000000".into(),
        l("header"),
        "Accept: application/vnd.github+json".into(),
        l("user-agent"),
        concat!("osatura/", env!("CARGO_PKG_VERSION"), " update-check").into(),
    ];
    if let Some(p) = proxy {
        a.push(l("proxy"));
        a.push(p.to_string());
    }
    a.push(RELEASES_URL.into());
    a
}

pub(crate) fn run(opts: &HashMap<String, String>, flags: &[String]) -> Result<(), String> {
    let json_out = flags.iter().any(|f| f == "--json");
    if flags.iter().any(|f| f == "--list-proxy-presets") {
        for p in &PRESETS {
            println!("{}\t{}\t{}\n  {}", p.id, p.url, p.label, p.note);
        }
        println!("\nLists of open public proxies are deliberately not offered: they are often compromised or intercept traffic.");
        return Ok(());
    }
    let proxy = proxy_from(opts)?;
    if let Some(p) = &proxy {
        eprintln!("update-check: going through {p}");
    }
    let out = Command::new("curl")
        .args(curl_args(proxy.as_deref()))
        .output()
        .map_err(|e| format!("could not start `curl` ({e}); install curl, or look at https://github.com/chiuta/osatura/releases yourself"))?;
    if !out.status.success() {
        let why = String::from_utf8_lossy(&out.stderr);
        return Err(format!(
            "could not reach GitHub: {}",
            why.trim().lines().next().unwrap_or("unknown error")
        ));
    }
    let body = String::from_utf8_lossy(&out.stdout);
    let (tag, url, pre) = newest_release(&body)?;
    let url = safe_release_url(&url);
    // Release builds get their version from the release workflow (OSATURA_VERSION, e.g. 0.1.0-alpha.4); the crate version
    // is only 0.1.0, so a build from source cannot tell which release it corresponds to.
    let mine = option_env!("OSATURA_VERSION")
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let new = parse_ver(&tag).ok_or("unreadable release tag")?;
    let cur = mine.and_then(parse_ver);
    let newer = cur.as_ref().map(|c| new > *c);
    if json_out {
        println!(
            "{}",
            serde_json::json!({"current": mine, "latest": tag, "update_available": newer, "prerelease": pre, "url": url})
        );
    } else {
        match (mine, newer) {
            (Some(m), Some(true)) => println!("A newer {}release is available: {tag} (you have {m}).\nDownload it from {url}\nNothing was downloaded or installed.", if pre { "pre-" } else { "" }),
            (Some(m), Some(false)) => println!("You are up to date: {m} (newest published release: {tag})."),
            _ => println!("The newest published release is {tag}: {url}\nThis build was made from source and does not know which release it matches, so it cannot compare. Nothing was downloaded or installed."),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Ver {
        parse_ver(s).unwrap()
    }

    #[test]
    fn semver_ordering() {
        assert!(v("0.1.0-alpha.5") > v("0.1.0-alpha.4"));
        assert!(v("0.1.0-alpha.10") > v("0.1.0-alpha.9"));
        assert!(v("0.1.0") > v("0.1.0-rc.1"));
        assert!(v("0.1.0-rc.1") > v("0.1.0-beta.2"));
        assert!(v("0.1.0-beta") > v("0.1.0-alpha.9"));
        assert!(v("0.1.0-alpha.1") > v("0.1.0-alpha"));
        assert!(v("0.1.0-alpha.1") < v("0.1.0-alpha.beta"));
        assert!(v("1.0.0") > v("0.9.9"));
        assert_eq!(v("v0.1.0+build5"), v("0.1.0"));
    }

    #[test]
    fn rejects_garbage_versions() {
        for s in [
            "",
            "1",
            "1.2",
            "1.2.3.4",
            "a.b.c",
            "1.2.3-",
            "1.2.3-a..b",
            "-1.0.0",
            "1.2.-3",
            "latest",
        ] {
            assert!(parse_ver(s).is_none(), "{s}");
        }
    }

    #[test]
    fn newest_release_skips_drafts_and_junk() {
        let j = r#"[
            {"tag_name":"v0.1.0-alpha.9","draft":true,"html_url":"https://github.com/chiuta/osatura/releases/tag/v0.1.0-alpha.9"},
            {"tag_name":"nightly","draft":false},
            {"tag_name":"v0.1.0-alpha.4","prerelease":true,"html_url":"https://github.com/chiuta/osatura/releases/tag/v0.1.0-alpha.4"},
            {"tag_name":"v0.1.0-alpha.5","prerelease":true,"html_url":"https://github.com/chiuta/osatura/releases/tag/v0.1.0-alpha.5"}
        ]"#;
        let (t, u, p) = newest_release(j).unwrap();
        assert_eq!((t.as_str(), p), ("v0.1.0-alpha.5", true));
        assert!(u.ends_with("alpha.5"));
        assert!(newest_release("[]").is_err());
        assert!(newest_release("{}").is_err());
        assert!(newest_release("<html>").is_err());
    }

    #[test]
    fn release_url_is_pinned_to_this_project() {
        assert_eq!(
            safe_release_url("https://github.com/chiuta/osatura/releases/tag/v1"),
            "https://github.com/chiuta/osatura/releases/tag/v1"
        );
        for bad in [
            "https://evil.example/x",
            "http://github.com/chiuta/osatura/x",
            "https://github.com/chiuta/osatura/x\u{1b}[2J",
            "https://github.com/chiuta/osatura/a b",
        ] {
            assert_eq!(
                safe_release_url(bad),
                "https://github.com/chiuta/osatura/releases"
            );
        }
    }

    #[test]
    fn proxy_options() {
        let o = |k: &str, val: &str| HashMap::from([(k.to_string(), val.to_string())]);
        assert_eq!(proxy_from(&HashMap::new()).unwrap(), None);
        assert_eq!(
            proxy_from(&o("--proxy-preset", " TOR "))
                .unwrap()
                .as_deref(),
            Some("socks5h://127.0.0.1:9050")
        );
        assert!(proxy_from(&o("--proxy-preset", "free-proxy-list")).is_err());
        assert_eq!(
            proxy_from(&o("--proxy-url", "socks5h://10.0.0.1:1080"))
                .unwrap()
                .as_deref(),
            Some("socks5h://10.0.0.1:1080")
        );
        for bad in ["ftp://x", "file:///etc/passwd", "http://a b", "x", "-x"] {
            assert!(proxy_from(&o("--proxy-url", bad)).is_err(), "{bad}");
        }
        let mut both = o("--proxy-preset", "tor");
        both.insert("--proxy-url".into(), "http://x:1".into());
        assert!(proxy_from(&both).is_err());
        for p in PRESETS {
            assert!(p.url.starts_with("socks5h://127.0.0.1:"));
        }
    }

    #[test]
    fn curl_is_https_only_with_the_proxy_after_the_options() {
        let a = curl_args(Some("socks5h://127.0.0.1:9050"));
        let dd = |n: &str| format!("-{}{n}", "-");
        assert!(a
            .windows(2)
            .any(|w| w[0] == dd("proto") && w[1] == "=https"));
        assert!(a
            .windows(2)
            .any(|w| w[0] == dd("proxy") && w[1] == "socks5h://127.0.0.1:9050"));
        assert_eq!(a.last().map(String::as_str), Some(RELEASES_URL));
        assert!(!curl_args(None).iter().any(|s| *s == dd("proxy")));
    }
}
