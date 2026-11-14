// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Agent server: a Model-Context-Protocol-style JSON-RPC 2.0 endpoint (one JSON message per
//! line on stdio) that lets an AI agent or script inspect, edit, check and render one project.
//!
//! Security model: the server edits only the single project it was started with; `save` writes only to that
//! file's own path (atomically, never through a symbolic link); it executes no commands. It can cause other
//! files to be read in exactly one way: an edit may add a file reference to the project (an image
//! `source.file`, a normal map, a sound), and the engine opens such files when it validates or renders. Those
//! reads all go through `assets::resolve`: a relative path without `..` that stays inside the project's
//! directory after symbolic links are resolved, a regular file, under a size cap. Absolute paths and
//! paths outside the directory are refused unless the process was started with `--allow-external-assets`.
//! All edits go through [`Editor`], so they are validated, undoable and atomic.

use crate::anim::Ease;
use crate::base64;
use crate::edit::{Editor, Op};
use crate::inspect;
use crate::lint;
use crate::project::Prop;
use crate::raster::{render_png, RasterOptions};
use crate::safety;
use serde_json::{json, Value};
use std::path::PathBuf;

pub struct Agent {
    editor: Editor,
    save_path: Option<PathBuf>,
}

fn text(s: impl Into<String>) -> Value {
    json!({"content": [{"type": "text", "text": s.into()}], "isError": false})
}

fn error_result(s: impl Into<String>) -> Value {
    json!({"content": [{"type": "text", "text": s.into()}], "isError": true})
}

fn tool(name: &str, description: &str, props: Value, required: &[&str]) -> Value {
    json!({"name": name, "description": description, "inputSchema": {"type": "object", "properties": props, "required": required}})
}

impl Agent {
    pub fn new(editor: Editor, save_path: Option<PathBuf>) -> Agent {
        Agent { editor, save_path }
    }

    pub fn tools() -> Vec<Value> {
        vec![
            tool("get_project", "Return the project JSON, or the value at a JSON pointer (e.g. /bones/0).", json!({"pointer": {"type": "string"}}), &[]),
            tool("stats", "Counts of bones, layers, channels and keys.", json!({}), &[]),
            tool("lint", "Static checks of the rig and animation. Returns findings with codes L0xx.", json!({}), &[]),
            tool("coach", "Timing and animation-principles suggestions (C0xx).", json!({}), &[]),
            tool("safety", "Flash / red-flash / camera-motion report (approximation of WCAG 2.3.1).", json!({"scale": {"type": "number"}}), &[]),
            tool(
                "apply_ops",
                "Apply JSON-pointer edit operations atomically as one undo step. Ops: {op:set|insert|remove|batch, path, value}.",
                json!({"ops": {"type": "array", "items": {"type": "object"}}}),
                &["ops"],
            ),
            tool(
                "set_key",
                "Set a keyframe, creating the channel if needed.",
                json!({"target": {"type": "string"}, "prop": {"type": "string"}, "frame": {"type": "number"}, "value": {"type": "number"}, "ease": {"type": "string"}}),
                &["target", "prop", "frame", "value"],
            ),
            tool("undo", "Undo the last edit.", json!({}), &[]),
            tool("redo", "Redo the last undone edit.", json!({}), &[]),
            tool("render_frame", "Render one frame to a PNG image.", json!({"frame": {"type": "number"}, "scale": {"type": "number"}, "bones": {"type": "boolean"}}), &["frame"]),
            tool("save", "Write the project back to the file it was opened from.", json!({}), &[]),
        ]
    }

    /// Handle one JSON-RPC message. Returns `None` for notifications.
    pub fn handle(&mut self, req: &Value) -> Option<Value> {
        let id = req.get("id").cloned();
        let method = req.get("method").and_then(Value::as_str).unwrap_or("");
        let params = req.get("params").cloned().unwrap_or(Value::Null);
        let reply = |result: Result<Value, (i64, String)>| -> Option<Value> {
            let id = id.clone()?; // notifications carry no id and get no reply
            Some(match result {
                Ok(r) => json!({"jsonrpc": "2.0", "id": id, "result": r}),
                Err((code, message)) => json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}}),
            })
        };
        match method {
            "initialize" => {
                let v = params.get("protocolVersion").and_then(Value::as_str).unwrap_or("2024-11-05").to_string();
                reply(Ok(json!({
                    "protocolVersion": v,
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": "osatura", "version": env!("CARGO_PKG_VERSION")}
                })))
            }
            "ping" => reply(Ok(json!({}))),
            "tools/list" => reply(Ok(json!({"tools": Agent::tools()}))),
            "tools/call" => {
                let Some(name) = params.get("name").and_then(Value::as_str) else {
                    return reply(Err((-32602, "tools/call needs a tool name".into())));
                };
                let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
                reply(Ok(self.call(name, &args)))
            }
            m if m.starts_with("notifications/") => None,
            "" => reply(Err((-32600, "invalid request".into()))),
            other => reply(Err((-32601, format!("unknown method '{other}'")))),
        }
    }

    fn call(&mut self, name: &str, a: &Value) -> Value {
        let num = |k: &str, d: f64| a.get(k).and_then(Value::as_f64).unwrap_or(d);
        match name {
            "get_project" => match a.get("pointer").and_then(Value::as_str) {
                None | Some("") => text(self.editor.doc().to_string()),
                Some(p) => match self.editor.doc().pointer(p) {
                    Some(v) => text(v.to_string()),
                    None => error_result(format!("nothing at '{p}'")),
                },
            },
            "stats" => text(serde_json::to_string_pretty(&inspect::stats(self.editor.project())).unwrap_or_default()),
            "lint" => text(serde_json::to_string_pretty(&lint::lint(self.editor.project(), None)).unwrap_or_default()),
            "coach" => text(serde_json::to_string_pretty(&lint::coach(self.editor.project())).unwrap_or_default()),
            "safety" => {
                let r = safety::analyze_engine(self.editor.engine(), (num("scale", 0.5) as f32).clamp(0.05, 1.0));
                let m = safety::motion_report(self.editor.engine());
                text(json!({
                    "passes": r.passes(),
                    "flashes": {"worst_transitions_per_s": r.general.worst_transitions, "fails": r.general.fails, "warns": r.general.warns},
                    "red_flashes": {"worst_transitions_per_s": r.red.worst_transitions, "fails": r.red.fails, "warns": r.red.warns},
                    "camera": {"pan_widths_per_s": m.max_pan_per_sec, "rotation_deg_per_s": m.max_rotation_deg_per_sec, "zoom_ratio_per_s": m.max_zoom_ratio_per_sec, "warnings": m.warnings},
                    "note": "engineering aid based on the public WCAG 2.3.1 definition; not a certification"
                }).to_string())
            }
            "apply_ops" => {
                let Some(ops) = a.get("ops").and_then(Value::as_array) else { return error_result("'ops' must be an array") };
                let parsed: Result<Vec<Op>, _> = ops.iter().map(|o| serde_json::from_value::<Op>(o.clone())).collect();
                match parsed {
                    Err(e) => error_result(format!("bad op: {e}")),
                    Ok(list) => match self.editor.apply(Op::Batch { ops: list }) {
                        Ok(()) => text("applied; the project still validates"),
                        Err(errs) => error_result(format!("rejected, nothing changed:\n- {}", errs.join("\n- "))),
                    },
                }
            }
            "set_key" => {
                let (Some(target), Some(prop)) = (a.get("target").and_then(Value::as_str), a.get("prop").and_then(Value::as_str)) else {
                    return error_result("target and prop are required");
                };
                let Ok(prop) = serde_json::from_value::<Prop>(json!(prop)) else { return error_result(format!("unknown property '{prop}'")) };
                let ease = match a.get("ease").and_then(Value::as_str) {
                    Some(e) => match serde_json::from_value::<Ease>(json!({"type": e})) {
                        Ok(e) => e,
                        Err(_) => return error_result(format!("unknown ease '{e}'")),
                    },
                    None => Ease::Linear,
                };
                match self.editor.set_key(target, prop, num("frame", 0.0) as f32, num("value", 0.0) as f32, ease) {
                    Ok(()) => text("key set"),
                    Err(errs) => error_result(format!("rejected, nothing changed:\n- {}", errs.join("\n- "))),
                }
            }
            "undo" => text(if self.editor.undo() { "undone" } else { "nothing to undo" }),
            "redo" => text(if self.editor.redo() { "redone" } else { "nothing to redo" }),
            "render_frame" => {
                let scale = (num("scale", 0.5) as f32).clamp(0.05, 2.0);
                let frame = num("frame", 0.0) as f32;
                if !(0.0..self.editor.project().frames as f32).contains(&frame) {
                    return error_result(format!("frame must be within 0..{}", self.editor.project().frames));
                }
                let bones = a.get("bones").and_then(Value::as_bool).unwrap_or(false);
                match render_png(self.editor.engine(), frame, &RasterOptions { scale, show_bones: bones, transparent: false }) {
                    Ok(png) => json!({"content": [{"type": "image", "data": base64::encode(&png), "mimeType": "image/png"}], "isError": false}),
                    Err(e) => error_result(e),
                }
            }
            "save" => match &self.save_path {
                None => error_result("this session has no file to save to"),
                Some(p) => match crate::fsutil::write_atomic(p, self.editor.project().to_json_pretty().as_bytes()) {
                    Ok(()) => text(format!("saved {}", p.display())),
                    Err(e) => error_result(format!("cannot save: {e}")),
                },
            },
            other => error_result(format!("unknown tool '{other}'")),
        }
    }

    /// Serve newline-delimited JSON-RPC until the input closes.
    pub fn serve<R: std::io::BufRead, W: std::io::Write>(&mut self, input: R, mut output: W) -> std::io::Result<()> {
        for line in input.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let resp = match serde_json::from_str::<Value>(&line) {
                Ok(req) if req.is_object() => self.handle(&req),
                Ok(_) | Err(_) => Some(json!({"jsonrpc": "2.0", "id": Value::Null, "error": {"code": -32700, "message": "parse error"}})),
            };
            if let Some(r) = resp {
                writeln!(output, "{r}")?;
                output.flush()?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo;

    fn agent() -> Agent {
        Agent::new(Editor::new(demo::project()).unwrap(), None)
    }
    fn call(a: &mut Agent, tool: &str, args: Value) -> Value {
        let r = a.handle(&json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": tool, "arguments": args}})).unwrap();
        r["result"].clone()
    }
    fn body(r: &Value) -> String {
        r["content"][0]["text"].as_str().unwrap_or_default().to_string()
    }

    #[test]
    fn handshake_and_tool_list() {
        let mut a = agent();
        let r = a.handle(&json!({"jsonrpc": "2.0", "id": 7, "method": "initialize", "params": {"protocolVersion": "2025-03-26"}})).unwrap();
        assert_eq!(r["id"], 7);
        assert_eq!(r["result"]["protocolVersion"], "2025-03-26");
        assert_eq!(r["result"]["serverInfo"]["name"], "osatura");
        assert!(a.handle(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"})).is_none(), "notifications get no reply");
        let l = a.handle(&json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"})).unwrap();
        let names: Vec<&str> = l["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
        for n in ["get_project", "apply_ops", "render_frame", "lint", "safety", "save"] {
            assert!(names.contains(&n), "{n}");
        }
        let bad = a.handle(&json!({"jsonrpc": "2.0", "id": 3, "method": "nope"})).unwrap();
        assert_eq!(bad["error"]["code"], -32601);
    }

    #[test]
    fn edit_validate_undo_cycle() {
        let mut a = agent();
        let r = call(&mut a, "apply_ops", json!({"ops": [{"op": "set", "path": "/fps", "value": 30.0}]}));
        assert_eq!(r["isError"], false, "{r}");
        assert!(body(&call(&mut a, "get_project", json!({"pointer": "/fps"}))).contains("30"));
        // invalid edit is rejected atomically
        let r = call(&mut a, "apply_ops", json!({"ops": [{"op": "set", "path": "/fps", "value": 31.0}, {"op": "set", "path": "/frames", "value": 0}]}));
        assert_eq!(r["isError"], true);
        assert!(body(&call(&mut a, "get_project", json!({"pointer": "/fps"}))).contains("30"), "first op of a failed batch rolled back");
        assert_eq!(body(&call(&mut a, "undo", json!({}))), "undone");
        assert!(body(&call(&mut a, "get_project", json!({"pointer": "/fps"}))).contains("24"));
        assert_eq!(body(&call(&mut a, "redo", json!({}))), "redone");
        assert_eq!(call(&mut a, "apply_ops", json!({"ops": [{"op": "explode"}]}))["isError"], true);
        assert_eq!(call(&mut a, "apply_ops", json!({"ops": "x"}))["isError"], true);
    }

    #[test]
    fn locked_channel_rejects_set_key() {
        let mut a = Agent::new(Editor::new(demo::project()).unwrap(), None);
        let before = call(&mut a, "set_key", json!({"target": "arm_up", "prop": "bone_angle", "frame": 5, "value": 1}));
        assert_eq!(before["isError"], false);
        let mut p = demo::project();
        crate::chanmeta::set_meta(&mut p, "arm_up", Some(crate::project::Prop::BoneAngle), &crate::chanmeta::Change { lock: Some(true), ..Default::default() }).unwrap();
        let mut b = Agent::new(Editor::new(p).unwrap(), None);
        let r = call(&mut b, "set_key", json!({"target": "arm_up", "prop": "bone_angle", "frame": 5, "value": 2}));
        assert_eq!(r["isError"], true);
        assert!(r.to_string().contains("locked"));
    }

    #[test]
    fn set_key_stats_lint_coach_safety() {
        let mut a = agent();
        let r = call(&mut a, "set_key", json!({"target": "arm_up", "prop": "bone_angle", "frame": 5, "value": 12.5, "ease": "ease_in_out"}));
        assert_eq!(r["isError"], false, "{r}");
        assert_eq!(call(&mut a, "set_key", json!({"target": "arm_up", "prop": "wiggle", "frame": 1, "value": 1}))["isError"], true);
        assert_eq!(call(&mut a, "set_key", json!({"target": "ghost", "prop": "bone_angle", "frame": 1, "value": 1}))["isError"], true, "unknown bone rejected by validation");
        assert!(body(&call(&mut a, "stats", json!({}))).contains("\"bones\""));
        assert!(body(&call(&mut a, "lint", json!({}))).starts_with('['));
        assert!(body(&call(&mut a, "coach", json!({}))).starts_with('['));
        let s: Value = serde_json::from_str(&body(&call(&mut a, "safety", json!({"scale": 0.2})))).unwrap();
        assert_eq!(s["passes"], true);
    }

    #[test]
    fn render_returns_a_png_and_checks_range() {
        let mut a = agent();
        let r = call(&mut a, "render_frame", json!({"frame": 3, "scale": 0.2}));
        assert_eq!(r["content"][0]["type"], "image");
        let png = base64::decode(r["content"][0]["data"].as_str().unwrap()).unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(call(&mut a, "render_frame", json!({"frame": 9999}))["isError"], true);
    }

    #[test]
    fn save_only_to_the_opened_path() {
        let mut a = agent();
        assert_eq!(call(&mut a, "save", json!({"path": "/etc/passwd"}))["isError"], true, "no path configured ⇒ refuses");
        let scratch = crate::fsutil::ScratchDir::new("osatura-agent");
        let dir = scratch.to_path_buf();
        let file = dir.join("p.json");
        let mut a = Agent::new(Editor::new(demo::project()).unwrap(), Some(file.clone()));
        let r = call(&mut a, "save", json!({"path": "/etc/passwd"}));
        assert_eq!(r["isError"], false);
        assert!(file.exists(), "argument 'path' is ignored; the opened file is written");
    }

    #[test]
    fn serve_loop_handles_garbage_and_blank_lines() {
        let mut a = agent();
        let input = b"\n{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\nnot json\n[1,2]\n{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n";
        let mut out = Vec::new();
        a.serve(&input[..], &mut out).unwrap();
        let lines: Vec<Value> = String::from_utf8(out).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect();
        assert_eq!(lines.len(), 3, "ping reply + two parse errors, nothing for the notification");
        assert_eq!(lines[0]["result"], json!({}));
        assert_eq!(lines[1]["error"]["code"], -32700);
    }
}
