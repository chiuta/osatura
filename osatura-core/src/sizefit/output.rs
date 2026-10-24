// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! HTML player and project JSON outputs: number rounding, path simplification, frame dedup (part of `sizefit`; see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// HTML player: number rounding in the SVG text, path simplification, frame dedup

pub(super) fn round_nums(val: &str, d: u8, path_mode: bool) -> String {
    let b = val.as_bytes();
    let mut out = String::with_capacity(val.len());
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        let starts = c.is_ascii_digit()
            || ((c == b'-' || c == b'+' || c == b'.') && b.get(i + 1).is_some_and(|n| n.is_ascii_digit() || (*n == b'.' && c != b'.')));
        let prev_ok = i == 0 || path_mode || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_');
        if starts && prev_ok {
            let mut j = i + 1;
            let mut dot = c == b'.';
            while j < b.len() && (b[j].is_ascii_digit() || (b[j] == b'.' && !dot)) {
                dot |= b[j] == b'.';
                j += 1;
            }
            match val[i..j].parse::<f64>() {
                Ok(x) => out.push_str(&fmt_dec(x, d)),
                Err(_) => out.push_str(&val[i..j]),
            }
            i = j;
        } else {
            let ch = val[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

pub(super) fn round_attr(name: &str, val: &str, d: u8) -> String {
    if name == "id" || name == "class" || name == "xmlns" || name.ends_with("href") || val.contains('#') || val.contains("url(") || val.starts_with("data:") {
        return val.to_string();
    }
    match name {
        "d" => round_nums(val, d, true),
        "transform" if val.starts_with("matrix(") => {
            let inner = &val[7..val.len().saturating_sub(1)];
            let parts: Vec<&str> = inner.split_whitespace().collect();
            if parts.len() != 6 || !val.ends_with(')') {
                return val.to_string();
            }
            let f: Vec<String> = parts
                .iter()
                .enumerate()
                .map(|(k, p)| p.parse::<f64>().map(|x| fmt_dec(x, if k < 4 { d.saturating_add(3) } else { d })).unwrap_or_else(|_| p.to_string()))
                .collect();
            format!("matrix({})", f.join(" "))
        }
        "opacity" | "fill-opacity" | "stroke-opacity" | "stop-opacity" | "offset" => round_nums(val, d.max(2), false),
        _ => round_nums(val, d, false),
    }
}

/// Round every number in the attribute values of an SVG fragment to `d` decimals, print numbers in their shortest
/// form (no trailing zeros), drop default `opacity="1"` attributes and the newlines between elements.
/// Ids, class names, colours, `url(#..)` references and `data:` URIs are left alone.
pub fn svg_round(svg: &str, d: u8) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(p) = rest.find("=\"") {
        let name_start = rest[..p].rfind(|c: char| c.is_whitespace() || c == '<').map_or(0, |i| i + 1);
        out.push_str(&rest[..p + 2]);
        let after = &rest[p + 2..];
        let Some(end) = after.find('"') else {
            rest = after;
            break;
        };
        let name = &rest[name_start..p];
        out.push_str(&round_attr(name, &after[..end], d));
        out.push('"');
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out.replace(">\n", ">").replace(" opacity=\"1\"", "").replace(" fill-opacity=\"1\"", "")
}

/// Apply the rounding of [`svg_round`] to the evaluated frame data (positions, tangents, stroke widths,
/// translations to `d` decimals; linear parts of transforms to `d + 3`; opacities to `max(d, 2)`).
/// This is what the HTML quality model rasterises.
pub(super) fn round_frame(f: &mut Frame, d: u8) {
    let r = |x: f32, k: u8| round_dec(x as f64, k) as f32;
    for nd in f.nodes.iter_mut() {
        nd.world.a = r(nd.world.a, d.saturating_add(3));
        nd.world.b = r(nd.world.b, d.saturating_add(3));
        nd.world.c = r(nd.world.c, d.saturating_add(3));
        nd.world.d = r(nd.world.d, d.saturating_add(3));
        nd.world.tx = r(nd.world.tx, d);
        nd.world.ty = r(nd.world.ty, d);
        nd.opacity = r(nd.opacity, d.max(2));
        for p in nd.paths.iter_mut() {
            for a in p.anchors.iter_mut() {
                a.map_points(&mut |v| Vec2::new(r(v.x, d), r(v.y, d)));
            }
            if let Some(st) = p.stroke.as_mut() {
                st.width = r(st.width, d);
                for x in st.dash.iter_mut() {
                    *x = r(*x, d);
                }
            }
        }
    }
}

pub(super) fn simplify_frame_paths(f: &mut Frame, eps: f64) {
    if eps <= 0.0 {
        return;
    }
    for nd in f.nodes.iter_mut() {
        let sc = (nd.world.x_scale() as f64).max(1e-6);
        for p in nd.paths.iter_mut() {
            simplify_vpath(p, eps / sc);
        }
    }
}

/// The page: frames as inline SVG, deduplicated (`x` maps displayed frame -> unique frame), play / pause / scrubber.
pub fn html_page(title: &str, width: f32, fps: f32, unique: &[String], x: Option<&[usize]>) -> String {
    let data = serde_json::to_string(unique).unwrap_or_else(|_| "[]".into()).replace("</", "<\\/");
    let xs = match x {
        Some(x) => serde_json::to_string(x).unwrap_or_else(|_| "null".into()),
        None => "null".into(),
    };
    let nframes = x.map_or(unique.len(), |x| x.len());
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title>\
<style>body{{margin:0;font:14px system-ui,sans-serif;background:#14161c;color:#e8e8ee;display:flex;flex-direction:column;align-items:center;min-height:100vh}}\
#s{{margin:16px;max-width:96vw;background:#fff;line-height:0}}#s svg{{max-width:96vw;height:auto}}\
#b{{display:flex;gap:12px;align-items:center;width:min(96vw,{}px);margin-bottom:20px}}\
button{{background:#3a4156;color:#fff;border:0;border-radius:6px;padding:6px 14px;cursor:pointer}}input{{flex:1}}</style></head><body>\
<div id=\"s\"></div><div id=\"b\"><button id=\"p\">Pause</button><input id=\"l\" type=\"range\" min=\"0\" max=\"{}\" value=\"0\"><span id=\"n\">0</span></div>\
<script>const F={},X={},R={};let i=0,pl=1,la=performance.now(),ac=0;\
pub(super) const s=document.getElementById('s'),l=document.getElementById('l'),n=document.getElementById('n'),p=document.getElementById('p'),N={};\
function show(k){{i=k;s.innerHTML=F[X?X[k]:k];l.value=k;n.textContent=k+' / '+(N-1)}}\
function tick(t){{if(pl){{ac+=(t-la)/1000*R;if(ac>=1){{const m=Math.floor(ac);ac-=m;show((i+m)%N)}}}}la=t;requestAnimationFrame(tick)}}\
p.onclick=()=>{{pl=!pl;p.textContent=pl?'Pause':'Play'}};l.oninput=()=>{{pl=0;p.textContent='Play';show(+l.value)}};show(0);requestAnimationFrame(tick);\
</script></body></html>\n",
        crate::svg::esc(title),
        width,
        nframes.saturating_sub(1),
        data,
        xs,
        fmt_dec(fps as f64, 4),
        nframes
    )
}

// Project JSON

pub(super) fn round_value(v: &mut Value, d: u8) {
    match v {
        Value::Number(x) => {
            if let Some(f) = x.as_f64() {
                if f.fract() != 0.0 {
                    *v = num(round_dec(f, d));
                }
            }
        }
        Value::Array(a) => a.iter_mut().for_each(|e| round_value(e, d)),
        Value::Object(m) => {
            for (k, e) in m.iter_mut() {
                if matches!(k.as_str(), "fps" | "width" | "height") {
                    continue;
                }
                round_value(e, d);
            }
        }
        _ => {}
    }
}

/// Bytes of the project with keys reduced (`key_tol`) and numbers rounded (`decimals`).
pub fn project_bytes(pre: &Project, s: &Settings) -> Result<Vec<u8>, String> {
    let mut p = pre.clone();
    if s.key_tol > 0.0 {
        reduce_project_keys(&mut p, s.key_tol);
    }
    let mut v = serde_json::to_value(&p).map_err(|e| e.to_string())?;
    round_value(&mut v, s.decimals);
    serde_json::to_vec(&v).map_err(|e| e.to_string())
}
