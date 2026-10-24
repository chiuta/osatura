// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `fuzz` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------------------------
// two-parameter grid
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize)]
pub struct GridResult {
    pub x: Param,
    pub y: Param,
    pub res: usize,
    /// Row-major (`y` rows, `x` columns), severity per cell.
    pub severity: Vec<f32>,
    /// Primary failing kind per cell ("" when the cell passes).
    pub kind: Vec<String>,
    pub failure_rate: f32,
}

/// Sweep two parameters over their full ranges (others at rest). Ids as in [`Param::id`].
// public entry point with a stable signature.
#[allow(clippy::too_many_arguments)]
pub fn grid(project: &Project, asset_dir: Option<&Path>, popts: &ParamOptions, th: Thresholds, frame: f32, xid: &str, yid: &str, res: usize) -> Result<GridResult, String> {
    if !(2..=512).contains(&res) {
        return Err("--res must be between 2 and 512".into());
    }
    let all = enumerate_params(project, popts);
    let find = |id: &str| all.iter().find(|p| p.id == id).cloned().ok_or_else(|| format!("unknown parameter '{id}' (examples: {})", all.iter().take(4).map(|p| p.id.clone()).collect::<Vec<_>>().join(", ")));
    let (px, py) = (find(xid)?, find(yid)?);
    if px.id == py.id {
        return Err("--x and --y must be different parameters".into());
    }
    let mut f = Fuzzer::with_params(project, asset_dir, all.clone(), th, frame)?;
    let (ix, iy) = (all.iter().position(|p| p.id == px.id).unwrap(), all.iter().position(|p| p.id == py.id).unwrap());
    let mut sev = Vec::with_capacity(res * res);
    let mut kind = Vec::with_capacity(res * res);
    let mut bad = 0;
    for j in 0..res {
        for i in 0..res {
            let mut v = f.rest_values();
            v[ix] = px.lo + (px.hi - px.lo) * i as f32 / (res - 1) as f32;
            v[iy] = py.lo + (py.hi - py.lo) * j as f32 / (res - 1) as f32;
            let r = f.eval(0, &v);
            bad += r.failed as usize;
            sev.push(r.severity);
            kind.push(if r.failed { r.kinds.first().cloned().unwrap_or_default() } else { String::new() });
        }
    }
    Ok(GridResult { x: px, y: py, res, severity: sev, kind, failure_rate: bad as f32 / (res * res) as f32 })
}

// ---------------------------------------------------------------------------------------------
// SVG output
// ---------------------------------------------------------------------------------------------

pub(super) fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Blue-to-orange ramp (own table): `t` in 0..1; the middle (near white) corresponds to the failure threshold
/// when `t = stress / 2`. Blue and orange stay distinguishable with the common kinds of colour blindness.
pub fn ramp(t: f32) -> [u8; 3] {
    const STOPS: [(f32, [f32; 3]); 6] = [
        (0.0, [44.0, 79.0, 158.0]),
        (0.25, [111.0, 160.0, 214.0]),
        (0.5, [238.0, 234.0, 223.0]),
        (0.7, [244.0, 176.0, 94.0]),
        (0.85, [224.0, 100.0, 28.0]),
        (1.0, [125.0, 26.0, 12.0]),
    ];
    let t = if t.is_finite() { t.clamp(0.0, 1.0) } else { 0.0 };
    for w in STOPS.windows(2) {
        if t <= w[1].0 {
            let u = (t - w[0].0) / (w[1].0 - w[0].0);
            let c = |k: usize| (w[0].1[k] + (w[1].1[k] - w[0].1[k]) * u).round() as u8;
            return [c(0), c(1), c(2)];
        }
    }
    [125, 26, 12]
}

pub(super) fn hex(c: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

/// Ramp position of a normalised stress value: 0..1 maps to the blue half (never reaching the pale middle),
/// 1 jumps to orange so that every failing value is visibly warm, 4 and above is the darkest.
pub fn stress_t(s: f32) -> f32 {
    if !s.is_finite() || s <= 0.0 {
        0.0
    } else if s < 1.0 {
        0.42 * s
    } else {
        0.62 + 0.38 * ((s - 1.0) / 3.0).min(1.0)
    }
}

/// RGB for a normalised stress value (1 = threshold).
pub fn stress_rgb(s: f32) -> [u8; 3] {
    ramp(stress_t(s))
}

/// Colour for a normalised stress value (1 = threshold).
pub fn stress_color(s: f32) -> String {
    hex(stress_rgb(s))
}

pub(super) fn legend(out: &mut String, x: f32, y: f32, w: f32, label: &str) {
    let n = 24;
    for i in 0..n {
        let sv = 4.0 * i as f32 / (n - 1) as f32;
        out.push_str(&format!("<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"10\" fill=\"{}\"/>\n", x + w * i as f32 / n as f32, y, w / n as f32 + 0.5, stress_color(sv)));
    }
    out.push_str(&format!("<text x=\"{x:.1}\" y=\"{:.1}\" font-size=\"10\" fill=\"#333\">0</text>\n", y + 22.0));
    out.push_str(&format!("<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"10\" fill=\"#333\" text-anchor=\"middle\">1 = threshold</text>\n", x + w / 4.0, y + 22.0));
    out.push_str(&format!("<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"10\" fill=\"#333\" text-anchor=\"end\">4+</text>\n", x + w, y + 22.0));
    out.push_str(&format!("<text x=\"{x:.1}\" y=\"{:.1}\" font-size=\"10\" fill=\"#333\">{}</text>\n", y - 4.0, esc(label)));
}

/// SVG with the stress overlay on the rest pose (worst case over all samples), the legend and per-bone failure bars.
pub fn heatmap_svg(rep: &FuzzReport) -> String {
    let (w, h) = (rep.heat.width.max(1.0), rep.heat.height.max(1.0));
    let bars = &rep.per_bone;
    let nb = bars.len().min(24);
    let total_h = h + 60.0 + 18.0 * nb as f32 + 30.0;
    let mut s = String::new();
    s.push_str(&format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w} {total_h}\" width=\"{w}\" height=\"{total_h}\" font-family=\"sans-serif\">\n"));
    s.push_str(&format!("<title>Stress heat-map: {}</title>\n", esc(&rep.project)));
    s.push_str(&format!(
        "<desc>{} samples ({}, seed {}), {} failures ({:.1}%). Colour = worst normalised stress per vertex over all samples; 1 = failure threshold.</desc>\n",
        rep.summary.samples, rep.strategy, rep.seed, rep.summary.failures, 100.0 * rep.summary.failure_rate
    ));
    s.push_str(&format!("<rect width=\"{w}\" height=\"{total_h}\" fill=\"#ffffff\"/>\n<rect x=\"0\" y=\"0\" width=\"{w}\" height=\"{h}\" fill=\"#fafafa\" stroke=\"#bbb\"/>\n"));
    for g in &rep.heat.geos {
        let col = |i: usize| g.max_stress.get(i).copied().unwrap_or(0.0);
        for t in &g.tris {
            if t.iter().any(|&i| i >= g.pts.len()) {
                continue;
            }
            let m = (col(t[0]) + col(t[1]) + col(t[2])) / 3.0;
            let p = |i: usize| format!("{:.2},{:.2}", g.pts[t[i]][0], g.pts[t[i]][1]);
            s.push_str(&format!("<polygon points=\"{} {} {}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"0.3\"/>\n", p(0), p(1), p(2), stress_color(m), stress_color(m)));
        }
        for e in &g.edges {
            if e.iter().any(|&i| i >= g.pts.len()) {
                continue;
            }
            let m = 0.5 * (col(e[0]) + col(e[1]));
            s.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"3\" stroke-linecap=\"round\"/>\n",
                g.pts[e[0]][0], g.pts[e[0]][1], g.pts[e[1]][0], g.pts[e[1]][1], stress_color(m)
            ));
        }
        if g.tris.is_empty() {
            for (i, p) in g.pts.iter().enumerate() {
                s.push_str(&format!("<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"3\" fill=\"{}\" stroke=\"#222\" stroke-width=\"0.5\"/>\n", p[0], p[1], stress_color(col(i))));
            }
        }
    }
    for (name, a, b) in &rep.heat.bones {
        s.push_str(&format!("<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#222\" stroke-width=\"1\" stroke-dasharray=\"4 2\"><title>{}</title></line>\n", a[0], a[1], b[0], b[1], esc(name)));
    }
    legend(&mut s, 10.0, h + 22.0, (w - 20.0).min(320.0), "worst-case stress over all samples");
    s.push_str(&format!("<text x=\"10\" y=\"{:.1}\" font-size=\"12\" font-weight=\"bold\" fill=\"#222\">Extra failure rate at the extremes of each bone's range (outer 40% minus inner 60%)</text>\n", h + 62.0));
    let bw = (w - 170.0).max(40.0);
    for (i, b) in bars.iter().take(nb).enumerate() {
        let y = h + 72.0 + 18.0 * i as f32;
        s.push_str(&format!("<text x=\"10\" y=\"{:.1}\" font-size=\"11\" fill=\"#222\">{}</text>\n", y + 11.0, esc(&b.name)));
        s.push_str(&format!("<rect x=\"150\" y=\"{y:.1}\" width=\"{bw:.1}\" height=\"12\" fill=\"#eeeeee\"/>\n"));
        s.push_str(&format!("<rect x=\"150\" y=\"{y:.1}\" width=\"{:.1}\" height=\"12\" fill=\"{}\"/>\n", bw * b.rate.clamp(0.0, 1.0), hex(ramp(0.62 + 0.38 * b.rate.clamp(0.0, 1.0)))));
        s.push_str(&format!("<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"10\" fill=\"#222\">{:.0}%</text>\n", 154.0 + bw * b.rate.clamp(0.0, 1.0), y + 10.0, 100.0 * b.rate));
    }
    s.push_str("</svg>\n");
    s
}

/// SVG of a two-parameter grid: colour = severity (1 = failure threshold), failing cells get a dark outline.
pub fn grid_svg(g: &GridResult) -> String {
    let cell = (480.0 / g.res as f32).clamp(2.0, 40.0);
    let (ox, oy) = (70.0f32, 40.0f32);
    let side = cell * g.res as f32;
    let (w, h) = (ox + side + 30.0, oy + side + 90.0);
    let mut s = String::new();
    s.push_str(&format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w:.0} {h:.0}\" width=\"{w:.0}\" height=\"{h:.0}\" font-family=\"sans-serif\">\n"));
    s.push_str(&format!("<title>Severity grid {} x {}</title>\n", esc(&g.x.id), esc(&g.y.id)));
    s.push_str(&format!("<desc>{}x{} cells, {:.1}% failing. Colour = severity, 1 = failure threshold.</desc>\n", g.res, g.res, 100.0 * g.failure_rate));
    s.push_str(&format!("<rect width=\"{w:.0}\" height=\"{h:.0}\" fill=\"#ffffff\"/>\n"));
    s.push_str(&format!("<text x=\"{ox}\" y=\"20\" font-size=\"12\" font-weight=\"bold\" fill=\"#222\">{} (x) vs {} (y), {:.1}% of cells fail</text>\n", esc(&g.x.id), esc(&g.y.id), 100.0 * g.failure_rate));
    for j in 0..g.res {
        for i in 0..g.res {
            let v = g.severity[j * g.res + i];
            let bad = v >= 1.0;
            s.push_str(&format!(
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{}\"{}><title>{:.3} {}</title></rect>\n",
                ox + cell * i as f32,
                oy + cell * j as f32,
                cell,
                cell,
                stress_color(v),
                if bad { " stroke=\"#3a0a02\" stroke-width=\"0.6\"" } else { "" },
                v,
                esc(&g.kind[j * g.res + i])
            ));
        }
    }
    let ticks = |p: &Param, horizontal: bool, s: &mut String| {
        for k in 0..=4 {
            let t = k as f32 / 4.0;
            let val = p.lo + (p.hi - p.lo) * t;
            if horizontal {
                s.push_str(&format!("<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"10\" fill=\"#333\" text-anchor=\"middle\">{:.3}</text>\n", ox + side * t, oy + side + 14.0, val));
            } else {
                s.push_str(&format!("<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"10\" fill=\"#333\" text-anchor=\"end\">{:.3}</text>\n", ox - 4.0, oy + side * t + 3.0, val));
            }
        }
    };
    ticks(&g.x, true, &mut s);
    ticks(&g.y, false, &mut s);
    legend(&mut s, ox, oy + side + 44.0, side.min(320.0), "severity");
    s.push_str("</svg>\n");
    s
}

// ---------------------------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------------------------
