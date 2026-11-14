// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! OSATURA Studio as a native window.
//!
//! `osatura-studio [project.json]` starts the `osatura` engine next to it as `studio` on a free local port and shows
//! it in a window (WebView2 on Windows, WKWebView on macOS); closing the window stops the server. Without a project it
//! opens `Documents/OSATURA/first-rig.json`, created from the built-in demo when missing.
//! Exports made in the window open a native "Save as" dialog and report where the file went.
//! `osatura-studio --selftest` starts the server, fetches the page, stops it and exits 0 or 1 (no window; used by CI).
//! Linux has no window here (it would need system web-view packages): use `osatura studio` and a browser.
#![cfg_attr(windows, windows_subsystem = "windows")]

mod launch;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--selftest") {
        let ok = match launch::start(None) {
            Ok(s) => launch::page_ok(&s),
            Err(_) => false,
        };
        std::process::exit(if ok { 0 } else { 1 });
    }
    let project = args
        .first()
        .filter(|a| !a.starts_with("--"))
        .map(std::path::PathBuf::from);
    window::run(project);
}

#[cfg(any(windows, target_os = "macos"))]
mod window {
    use super::launch;
    use std::cell::RefCell;
    use std::path::PathBuf;
    use std::rc::Rc;
    use tao::dpi::LogicalSize;
    use tao::event::{Event, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoop};
    use tao::window::WindowBuilder;
    use wry::WebViewBuilder;

    fn escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }

    pub fn run(project: Option<std::path::PathBuf>) {
        let started = launch::start(project);
        let event_loop = EventLoop::new();
        let title = match &started {
            Ok(s) => format!(
                "OSATURA Studio - {}",
                s.project
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default()
            ),
            Err(_) => "OSATURA Studio".to_string(),
        };
        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(LogicalSize::new(1400.0, 900.0))
            .build(&event_loop)
            .expect("cannot create the window");
        // Exports (GIF, HTML player, Lottie) are downloads of the page. Without a handler the web view would save them
        // silently into the Downloads folder; instead ask where to put them (starting in the project's folder) and tell
        // the person where the file went.
        let export_dir = started
            .as_ref()
            .ok()
            .and_then(|s| s.project.parent().map(std::path::Path::to_path_buf));
        let chosen: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));
        let chosen_start = chosen.clone();
        let chosen_done = chosen.clone();
        let builder = WebViewBuilder::new()
            .with_download_started_handler(move |_url, dest: &mut PathBuf| {
                let name = dest
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "export".to_string());
                let mut dialog = rfd::FileDialog::new()
                    .set_title("Save export")
                    .set_file_name(name);
                if let Some(dir) = &export_dir {
                    dialog = dialog.set_directory(dir);
                }
                match dialog.save_file() {
                    Some(path) => {
                        *dest = path.clone();
                        *chosen_start.borrow_mut() = Some(path);
                        true
                    }
                    None => false, // cancelled: no download
                }
            })
            .with_download_completed_handler(move |_url, path, success| {
                let shown = chosen_done.borrow_mut().take().or(path);
                let text = match (success, shown) {
                    (true, Some(p)) => format!("Saved to {}", p.display()),
                    (true, None) => "Saved.".to_string(),
                    (false, _) => "The export could not be saved.".to_string(),
                };
                let level = if success {
                    rfd::MessageLevel::Info
                } else {
                    rfd::MessageLevel::Error
                };
                let _ = rfd::MessageDialog::new()
                    .set_title("OSATURA Studio")
                    .set_level(level)
                    .set_description(text)
                    .show();
            });
        let builder = match &started {
            Ok(s) => builder.with_url(&s.url),
            Err(e) => builder.with_html(format!(
                "<html><body style=\"font-family:sans-serif;margin:2em\"><h2>OSATURA Studio could not start</h2><p>{}</p></body></html>",
                escape(e)
            )),
        };
        let _webview = builder.build(&window).expect("cannot create the web view");
        let mut server = started.ok();
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            if let Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } = event
            {
                server.take(); // Drop stops the server
                *control_flow = ControlFlow::Exit;
            }
        });
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
mod window {
    pub fn run(_project: Option<std::path::PathBuf>) {
        eprintln!("osatura-studio has no window on this platform; run `osatura studio <project.json>` and open the printed address in a browser.");
        std::process::exit(2);
    }
}
