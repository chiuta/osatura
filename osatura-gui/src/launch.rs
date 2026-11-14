// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Starting the studio server as a child process: shared by the window and by `--selftest` (which runs on every OS).

use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// A running studio server.
pub struct Studio {
    pub child: Child,
    pub url: String,
    pub project: PathBuf,
}

impl Drop for Studio {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The `osatura` executable next to this one.
fn engine_path() -> Result<PathBuf, String> {
    let me = std::env::current_exe().map_err(|e| format!("cannot find myself: {e}"))?;
    let name = if cfg!(windows) { "osatura.exe" } else { "osatura" };
    let p = me.parent().ok_or("no parent folder")?.join(name);
    if p.exists() {
        Ok(p)
    } else {
        Err(format!("the engine {} was not found next to the studio application ({}); keep both files in the same folder", name, p.display()))
    }
}

fn hide_console(c: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    #[cfg(not(windows))]
    {
        let _ = c;
    }
}

/// The folder for the first project: `OSATURA_DOCUMENTS` when set and not empty (a portable installation, e.g. the
/// PortableApps.com package, points it into its own data folder), else `<home>/Documents/OSATURA`.
fn projects_dir(env: impl Fn(&str) -> Option<std::ffi::OsString>) -> Result<PathBuf, String> {
    if let Some(d) = env("OSATURA_DOCUMENTS").filter(|v| !v.is_empty()) {
        return Ok(PathBuf::from(d));
    }
    let home = env("USERPROFILE").or_else(|| env("HOME")).ok_or("no home folder")?;
    Ok(PathBuf::from(home).join("Documents").join("OSATURA"))
}

/// `<Documents>/OSATURA/first-rig.json`, created from the built-in demo when missing (an existing file is never touched).
pub fn default_project(engine: &Path) -> Result<PathBuf, String> {
    let dir = projects_dir(|k| std::env::var_os(k))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let p = dir.join("first-rig.json");
    if !p.exists() {
        let mut c = Command::new(engine);
        c.arg("demo-person").arg(&p).stdout(Stdio::null()).stderr(Stdio::null());
        hide_console(&mut c);
        let st = c.status().map_err(|e| format!("cannot run the engine: {e}"))?;
        if !st.success() {
            return Err("the engine could not write the first project".into());
        }
    }
    Ok(p)
}

fn free_port() -> Result<u16, String> {
    let l = TcpListener::bind("127.0.0.1:0").map_err(|e| format!("no free port: {e}"))?;
    l.local_addr().map(|a| a.port()).map_err(|e| e.to_string())
}

/// Start `osatura studio <project> --port <free>` and wait (at most 20 s) until it accepts connections.
pub fn start(project: Option<PathBuf>) -> Result<Studio, String> {
    let engine = engine_path()?;
    let project = match project {
        Some(p) => p,
        None => default_project(&engine)?,
    };
    let port = free_port()?;
    let mut c = Command::new(&engine);
    c.arg("studio").arg(&project).arg("--port").arg(port.to_string()).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    hide_console(&mut c);
    let child = c.spawn().map_err(|e| format!("cannot start the engine: {e}"))?;
    let mut s = Studio { child, url: format!("http://127.0.0.1:{port}"), project };
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    let t0 = Instant::now();
    loop {
        if let Some(st) = s.child.try_wait().map_err(|e| e.to_string())? {
            return Err(format!("the studio server stopped at start-up ({st}); is the project file valid? {}", s.project.display()));
        }
        if TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok() {
            return Ok(s);
        }
        if t0.elapsed() > Duration::from_secs(20) {
            return Err("the studio server did not answer within 20 seconds".into());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// One HTTP request to the server's page; `true` when it answers 200.
pub fn page_ok(s: &Studio) -> bool {
    use std::io::{Read, Write};
    let port: u16 = s.url.rsplit(':').next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let Ok(mut t) = TcpStream::connect(("127.0.0.1", port)) else { return false };
    let _ = t.set_read_timeout(Some(Duration::from_secs(5)));
    let req = format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    if t.write_all(req.as_bytes()).is_err() {
        return false;
    }
    let mut buf = Vec::new();
    let _ = t.read_to_end(&mut buf);
    buf.starts_with(b"HTTP/1.1 200") || buf.starts_with(b"HTTP/1.0 200")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<OsString> + 'a {
        move |k| pairs.iter().find(|(n, _)| *n == k).map(|(_, v)| OsString::from(v))
    }

    #[test]
    fn the_projects_folder_follows_osatura_documents_then_the_home_folder() {
        assert_eq!(projects_dir(env(&[("OSATURA_DOCUMENTS", "/p/data"), ("HOME", "/h")])).unwrap(), PathBuf::from("/p/data"));
        assert_eq!(projects_dir(env(&[("OSATURA_DOCUMENTS", ""), ("HOME", "/h")])).unwrap(), PathBuf::from("/h").join("Documents").join("OSATURA"));
        assert_eq!(projects_dir(env(&[("USERPROFILE", "/u"), ("HOME", "/h")])).unwrap(), PathBuf::from("/u").join("Documents").join("OSATURA"));
        assert!(projects_dir(env(&[])).is_err());
    }
}
