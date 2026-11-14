// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! AGPL-3.0 section 13 source offer: `/source` is served by the studio (also when a token is required),
//! `--source-url` is shown, a bad URL is refused, and `--version` prints the notice.
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const BIN: &str = env!("CARGO_BIN_EXE_osatura");

struct Kill(Child);
impl Drop for Kill {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn get(port: u16, path: &str) -> String {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    write!(
        s,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut out = String::new();
    let _ = s.read_to_string(&mut out);
    out
}

fn start(project: &str, port: u16, extra: &[&str]) -> Kill {
    let mut args = vec![
        "studio".to_string(),
        project.to_string(),
        "--port".into(),
        port.to_string(),
        "--ui-config".into(),
        format!("{project}.ui.json"),
    ];
    args.extend(extra.iter().map(|s| s.to_string()));
    let child = Command::new(BIN)
        .args(&args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    for _ in 0..100 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Kill(child)
}

fn tmp_project(tag: &str) -> (osatura_core::fsutil::ScratchDir, String) {
    let dir = osatura_core::fsutil::ScratchDir::new(&format!("osatura-src-{tag}"));
    let p = dir.join("p.json").to_string_lossy().to_string();
    assert!(Command::new(BIN)
        .args(["demo", &p])
        .status()
        .unwrap()
        .success());
    (dir, p)
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[test]
fn source_route_default_and_configured_url() {
    let (_dir, p) = tmp_project("a");
    let port = free_port();
    let _k = start(&p, port, &[]);
    let r = get(port, "/source");
    assert!(r.starts_with("HTTP/1.1 200"), "{r}");
    assert!(
        r.contains("GNU Affero General Public License")
            && r.contains("section 13")
            && r.contains("--source-url"),
        "{r}"
    );
    assert!(
        get(port, "/").contains("href=\"/source\""),
        "footer link in studio page"
    );
    drop(_k);

    let port = free_port();
    let _k = start(
        &p,
        port,
        &["--source-url", "https://example.org/osatura-src?a=1&b=2"],
    );
    let r = get(port, "/source");
    assert!(
        r.contains("https://example.org/osatura-src?a=1&amp;b=2"),
        "{r}"
    );
}

#[test]
fn source_route_is_open_even_with_a_token_and_bad_urls_are_refused() {
    let (_dir, p) = tmp_project("b");
    let port = free_port();
    let _k = start(&p, port, &["--collab-token", "0123456789abcdef-token"]);
    assert!(get(port, "/source").starts_with("HTTP/1.1 200"));
    assert!(
        get(port, "/api/state").starts_with("HTTP/1.1 401"),
        "API still needs the token"
    );
    let out = Command::new(BIN)
        .args(["studio", &p, "--source-url", "javascript:alert(1)"])
        .output()
        .unwrap();
    assert!(!out.status.success() && String::from_utf8_lossy(&out.stderr).contains("--source-url"));
}

#[test]
fn version_prints_licence_source_offer_and_support() {
    let out = Command::new(BIN).arg("--version").output().unwrap();
    let t = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success()
            && t.contains("AGPL-3.0-or-later")
            && t.contains("buymeacoffee.com/echo.of.the.strings"),
        "{t}"
    );
}
