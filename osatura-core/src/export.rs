// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Exporters: animated GIF, self-contained HTML player, Lottie (Bodymovin) JSON.
//! (MP4 goes through `ffmpeg` in the CLI.)

use crate::engine::Engine;
#[cfg(test)]
use crate::math::Vec2;
use crate::raster::{render_rgba, RasterOptions};
use crate::svg::{render_frame, RenderOptions};

/// Most `frames x anchors` values a vector exporter (Lottie, HTML player, glTF) writes: 40 million, which is a few
/// hundred MB of JSON. Above it the export is refused instead of running for minutes and filling the disk.
pub const MAX_EXPORT_VALUES: u64 = 40_000_000;

/// Refuses an export whose size would explode (`frames x anchors`, also bones and layers per frame).
pub fn check_budget(p: &crate::project::Project) -> Result<(), String> {
    fn anchors(layers: &[crate::project::Layer]) -> u64 {
        layers
            .iter()
            .map(|l| {
                let own = match &l.kind {
                    crate::project::LayerKind::Vector { paths } => paths.iter().map(|p| p.anchors.len() as u64).sum(),
                    _ => 1,
                };
                own + anchors(l.children())
            })
            .sum()
    }
    let per_frame = anchors(&p.layers) + p.bones.len() as u64 + 1;
    let total = per_frame.saturating_mul(p.frames as u64);
    if total > MAX_EXPORT_VALUES {
        return Err(format!(
            "export would write about {} million values ({} frames x {} anchors, bones and layers); the limit is {} million (shorten the animation or simplify the paths)",
            total / 1_000_000, p.frames, per_frame, MAX_EXPORT_VALUES / 1_000_000
        ));
    }
    Ok(())
}

/// Which source frames a GIF holds and how long each is shown (in 1/100 s).
#[derive(Clone, Debug, PartialEq)]
pub struct GifPlan {
    /// `(source frame index, delay in centiseconds)`.
    pub picks: Vec<(usize, u16)>,
    /// Set when the rate cannot be represented and frames were dropped.
    pub warning: Option<String>,
}

/// GIF delays are whole centiseconds and viewers treat anything under 2 cs as 10 cs, so 50 fps is
/// the fastest rate a GIF can really hold. Up to 50 fps the delays are error-diffused (cumulative
/// rounding: 29.97 fps alternates 3 and 4 cs) so the total duration stays within 1 cs of
/// `frames / fps`. Above 50 fps (60, 100) the sequence is decimated to 50 fps: output frame `j`
/// shows the source frame nearest to time `j / 50 s`, which keeps the duration and drops frames.
pub fn gif_plan(frames: usize, fps: f64) -> Result<GifPlan, String> {
    if !fps.is_finite() || fps <= 0.0 {
        return Err(format!("GIF needs a positive finite frame rate, got {fps}"));
    }
    if frames == 0 {
        return Ok(GifPlan { picks: vec![], warning: None });
    }
    if fps <= 50.0 + 1e-9 {
        let cs = |i: usize| (i as f64 * 100.0 / fps).round();
        let picks = (0..frames).map(|i| (i, ((cs(i + 1) - cs(i)) as u16).max(2))).collect();
        return Ok(GifPlan { picks, warning: None });
    }
    let n_out = ((frames as f64 * 50.0 / fps).round() as usize).max(1);
    let picks = (0..n_out).map(|j| (((j as f64 * fps / 50.0).round() as usize).min(frames - 1), 2u16)).collect();
    let warning = format!(
        "GIF cannot hold {fps} fps (the shortest delay viewers honour is 2 cs = 50 fps): decimated {frames} frames to {n_out} at 50 fps, duration kept; use APNG, WebM or MP4 for the full rate"
    );
    Ok(GifPlan { picks, warning: Some(warning) })
}

/// Encode the whole timeline as a looping GIF at the project's frame rate (see [`gif_plan`] for
/// the delay rounding and the decimation above 50 fps).
pub fn gif_bytes(engine: &Engine, scale: f32) -> Result<Vec<u8>, String> {
    gif_bytes_warn(engine, scale).map(|(b, _)| b)
}

/// Same, also returning the warnings (frame-rate decimation).
pub fn gif_bytes_warn(engine: &Engine, scale: f32) -> Result<(Vec<u8>, Vec<String>), String> {
    gif_bytes_warn_with(engine, scale, None)
}

/// [`gif_bytes_warn`] with optional motion blur (`crate::motionblur`): only the frames the plan keeps are rendered, each as the
/// average of the shutter's samples. `None` is exactly [`gif_bytes_warn`].
pub fn gif_bytes_warn_with(engine: &Engine, scale: f32, shutter: Option<&crate::motionblur::Shutter>) -> Result<(Vec<u8>, Vec<String>), String> {
    let p = &engine.project;
    crate::raster::check_sequence(p, scale)?;
    let plan = gif_plan(p.frames as usize, p.fps as f64)?;
    if let Some(sh) = shutter {
        crate::motionblur::check_sequence_cost(plan.picks.len() as u32, sh)?;
    }
    let opts = RasterOptions { scale, show_bones: false, transparent: false };
    let mut frames = Vec::new();
    let mut delays = Vec::new();
    let (mut w, mut h) = (0u32, 0u32);
    for &(f, d) in &plan.picks {
        let (fw, fh, rgba) = match shutter {
            Some(sh) => crate::motionblur::render_engine(engine, f as f32, &opts, sh)?,
            None => render_rgba(engine, f as f32, &opts),
        };
        (w, h) = (fw, fh);
        frames.push(rgba);
        delays.push(d);
    }
    Ok((gif_encode(w, h, frames, &delays)?, plan.warning.into_iter().collect()))
}

/// Looping GIF from ready RGBA frames (all `w`x`h`) at `fps`, with error-diffused delays; above
/// 50 fps frames are dropped to 50 fps ([`gif_plan`]).
pub fn gif_from_frames(w: u32, h: u32, frames: Vec<Vec<u8>>, fps: f32) -> Result<Vec<u8>, String> {
    let plan = gif_plan(frames.len(), fps as f64)?;
    let mut src: Vec<Option<Vec<u8>>> = frames.into_iter().map(Some).collect();
    let mut out_frames = Vec::new();
    let mut delays = Vec::new();
    for &(i, d) in &plan.picks {
        // a source frame can be picked twice only when decimating the other way, which never happens
        out_frames.push(src[i].take().ok_or("internal: GIF frame picked twice")?);
        delays.push(d);
    }
    gif_encode(w, h, out_frames, &delays)
}

fn gif_encode(w: u32, h: u32, frames: Vec<Vec<u8>>, delays: &[u16]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    if w > u16::MAX as u32 || h > u16::MAX as u32 {
        return Err("image too large for GIF".into());
    }
    {
        let mut e = gif::Encoder::new(&mut out, w as u16, h as u16, &[]).map_err(|e| e.to_string())?;
        e.set_repeat(gif::Repeat::Infinite).map_err(|e| e.to_string())?;
        for (mut rgba, &d) in frames.into_iter().zip(delays) {
            let mut fr = gif::Frame::from_rgba_speed(w as u16, h as u16, &mut rgba, 10);
            fr.delay = d;
            e.write_frame(&fr).map_err(|e| e.to_string())?;
        }
    }
    Ok(out)
}

/// A single self-contained HTML file: every frame as inline SVG plus play/pause and a scrubber.
pub fn html_player(engine: &Engine) -> String {
    html_player_with_audio(engine, None)
}

/// Soft limit (bytes of audio before base64) above which the CLI warns; the page still works.
pub const HTML_AUDIO_WARN: usize = 3 << 20;
/// Hard limit: the CLI leaves the audio out above this, so the page stays loadable.
pub const HTML_AUDIO_MAX: usize = 16 << 20;

/// Same page with one pre-mixed audio clip embedded as a `data:` URI in an `<audio>` element
/// (`mime`, bytes). The clip must already contain start offsets, gain, trims and fades - the
/// page only keeps it in step with the frame clock (play, pause, scrub, loop). No external requests.
pub fn html_player_with_audio(engine: &Engine, audio: Option<(&str, &[u8])>) -> String {
    let p = &engine.project;
    let frames: Vec<String> =
        (0..p.frames).map(|f| render_frame(engine, f as f32, &RenderOptions { show_bones: false })).collect();
    // `</` must not appear inside a <script>; JSON escaping of '/' avoids it.
    let data = serde_json::to_string(&frames).unwrap_or_else(|_| "[]".into()).replace("</", "<\\/");
    let title = crate::svg::esc(&p.name);
    let mime_ok = |m: &str| !m.is_empty() && m.bytes().all(|c| c.is_ascii_alphanumeric() || b"/+.-".contains(&c));
    let audio_tag = match audio {
        Some((mime, bytes)) if mime_ok(mime) => format!("<audio id=\"au\" preload=\"auto\" src=\"data:{mime};base64,{}\"></audio>\n", crate::base64::encode(bytes)),
        _ => String::new(),
    };
    let audio = if audio_tag.is_empty() { None } else { audio };
    // Audio follows the frame clock: resync when it drifts, restart on loop wrap, pause with the
    // animation. Browsers block autoplay until a click, so the first gesture starts it.
    let audio_js = if audio.is_some() {
        r##"const au=document.getElementById('au'),mu=document.getElementById('mu');let snd=true;
function sync(){if(!snd||!playing){au.pause();return;}const t=i/FPS;if(Math.abs(au.currentTime-t)>0.25)au.currentTime=t;if(au.paused){const r=au.play();if(r&&r.catch)r.catch(()=>{});}}
const _show=show;show=function(n){const wrap=n<i&&playing;_show(n);if(wrap)au.currentTime=0;sync();};
mu.onclick=()=>{snd=!snd;mu.textContent=snd?'Sound on':'Sound off';sync();};
document.addEventListener('pointerdown',sync);
const _pp=pp.onclick;pp.onclick=()=>{_pp();sync();};
"##
    } else {
        ""
    };
    format!(
        r##"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title}</title>
<style>
body{{margin:0;font:14px system-ui,sans-serif;background:#14161c;color:#e8e8ee;display:flex;flex-direction:column;align-items:center;min-height:100vh}}
#stage{{margin:16px;max-width:96vw;background:#fff;line-height:0;box-shadow:0 8px 30px #0008}}
#stage svg{{max-width:96vw;height:auto}}
#bar{{display:flex;gap:12px;align-items:center;width:min(96vw,{w}px);margin-bottom:20px}}
button{{background:#3a4156;color:#fff;border:0;border-radius:6px;padding:6px 14px;cursor:pointer}}
input[type=range]{{flex:1}}
</style></head><body>
<div id="stage"></div>
{audio_tag}<div id="bar"><button id="pp">Pause</button><input id="sl" type="range" min="0" max="{max}" value="0"><span id="lab">0</span>{mute_btn}</div>
<script>
const F={data},FPS={fps};let i=0,playing=true,last=performance.now(),acc=0;
const st=document.getElementById('stage'),sl=document.getElementById('sl'),lab=document.getElementById('lab'),pp=document.getElementById('pp');
function show(n){{i=n;st.innerHTML=F[n];sl.value=n;lab.textContent=n+' / '+(F.length-1);}}
function tick(t){{if(playing){{acc+=(t-last)/1000*FPS;if(acc>=1){{const s=Math.floor(acc);acc-=s;show((i+s)%F.length);}}}}last=t;requestAnimationFrame(tick);}}
pp.onclick=()=>{{playing=!playing;pp.textContent=playing?'Pause':'Play';}};
sl.oninput=()=>{{playing=false;pp.textContent='Play';show(+sl.value);}};
{audio_js}show(0);requestAnimationFrame(tick);
</script></body></html>
"##,
        audio_tag = audio_tag,
        mute_btn = if audio.is_some() { "<button id=\"mu\">Sound on</button>" } else { "" },
        audio_js = audio_js,
        w = p.width,
        max = p.frames.saturating_sub(1),
        data = data,
        fps = p.fps
    )
}

pub use crate::lottie_export::{lottie_json, lottie_json_mapped, lottie_export, lottie_report_json, LottieExport, LottieLayerMap, LottieWarning};

#[cfg(test)]
mod tests {
    #[test]
    fn export_budget_refuses_exploding_sizes() {
        let mut p = crate::demo::project();
        assert!(check_budget(&p).is_ok());
        p.frames = 20_000;
        for _ in 0..40 {
            let more = p.layers.clone();
            for (i, mut l) in more.into_iter().enumerate() {
                l.name = format!("{}_{}_{}", l.name, i, p.layers.len());
                p.layers.push(l);
            }
            if p.layers.len() > 400 {
                break;
            }
        }
        let err = check_budget(&p).unwrap_err();
        assert!(err.contains("the limit is 40 million"), "{err}");
    }

    use super::*;

    /// C6: error-diffused delays keep the total within 1 cs of frames / fps; above 50 fps frames are
    /// decimated (with a warning) and the duration is kept.
    #[test]
    fn gif_delays_are_error_diffused_and_total_duration_matches() {
        for (fps, frames) in [(24.0f64, 48usize), (24.0, 241), (30000.0 / 1001.0, 300), (30.0, 100), (25.0, 77), (50.0, 33), (12.5, 31), (60.0, 120), (100.0, 250), (59.94, 600)] {
            let plan = gif_plan(frames, fps).unwrap();
            let total: u32 = plan.picks.iter().map(|&(_, d)| d as u32).sum();
            let want = frames as f64 * 100.0 / fps;
            assert!((total as f64 - want).abs() <= 1.0 + 1e-9, "{fps} fps x {frames}: {total} cs vs {want}");
            assert!(plan.picks.iter().all(|&(_, d)| d >= 2));
            assert_eq!(plan.warning.is_some(), fps > 50.0, "{fps}");
            // picks are strictly increasing source frames
            assert!(plan.picks.windows(2).all(|w| w[1].0 > w[0].0 || fps <= 50.0 && w[1].0 == w[0].0 + 1));
        }
        // 29.97 alternates 3 and 4 cs instead of a constant 3
        let p = gif_plan(10, 30000.0 / 1001.0).unwrap();
        let ds: Vec<u16> = p.picks.iter().map(|x| x.1).collect();
        assert!(ds.contains(&3) && ds.contains(&4), "{ds:?}");
        // 24 fps: 4,4,4,... plus a 5? 100/24 = 4.1667 -> mostly 4 with a 5 every 6th
        let p = gif_plan(24, 24.0).unwrap();
        assert_eq!(p.picks.iter().map(|x| x.1 as u32).sum::<u32>(), 100);
        assert!(gif_plan(5, f64::NAN).is_err() && gif_plan(5, 0.0).is_err() && gif_plan(5, -3.0).is_err() && gif_plan(5, f64::INFINITY).is_err());
        // the delays really land in the file
        let mut e = small();
        e.project.frames = 6;
        let bytes = gif_bytes(&e, 1.0).unwrap();
        let mut dec = gif::DecodeOptions::new().read_info(std::io::Cursor::new(bytes)).unwrap();
        let mut total = 0u32;
        while let Some(fr) = dec.read_next_frame().unwrap() {
            total += fr.delay as u32;
        }
        let want = 6.0 * 100.0 / e.project.fps as f64;
        assert!((total as f64 - want).abs() <= 1.0, "{total} vs {want}");
    }
    use crate::path::VPath;
    use crate::project::{Layer, Project};

    fn small() -> Engine {
        let mut p = Project::new("exp", 64.0, 48.0, 12.0, 6);
        p.background = Some(crate::style::Rgba::hex("#ffffff"));
        let mut l = Layer::vector("sq", vec![VPath::rect("r", 10.0, 10.0, 20.0, 20.0).with_fill("#ff0000").with_stroke("#000000", 2.0)]);
        l.transform.position = Vec2::new(0.0, 0.0);
        p.layers = vec![l];
        use crate::anim::{Ease, Track};
        use crate::project::{Channel, Prop};
        p.channels = vec![Channel::keys("sq", Prop::LayerX, Track::with_keys(&[(0.0, 0.0, Ease::Linear), (5.0, 20.0, Ease::Linear)]))];
        Engine::new(p).unwrap()
    }

    #[test]
    fn gif_has_every_frame_and_the_right_size() {
        let bytes = gif_bytes(&small(), 1.0).unwrap();
        assert_eq!(&bytes[..6], b"GIF89a");
        let mut opts = gif::DecodeOptions::new();
        opts.set_color_output(gif::ColorOutput::RGBA);
        let mut d = opts.read_info(&bytes[..]).unwrap();
        assert_eq!((d.width(), d.height()), (64, 48));
        let mut n = 0;
        let mut first_red = 0;
        while let Some(fr) = d.read_next_frame().unwrap() {
            if n == 0 {
                let i = (20 * 64 + 20) * 4;
                first_red = fr.buffer[i] as u32;
            }
            n += 1;
        }
        assert_eq!(n, 6);
        assert!(first_red > 200, "red square is drawn");
    }

    #[test]
    fn html_player_is_self_contained() {
        let h = html_player(&small());
        assert!(h.starts_with("<!doctype html>") && h.contains("<svg") && h.contains("requestAnimationFrame"));
        assert!(!h.contains("src=\"http") && !h.contains("href=\"http"));
        assert_eq!(h.matches("</script>").count(), 1, "frame data cannot terminate the script");
    }

    #[test]
    fn html_player_embeds_audio_without_external_requests() {
        let wav = [b'R', b'I', b'F', b'F', 1, 2, 3, 4, 250, 251, 252, 253];
        let h = html_player_with_audio(&small(), Some(("audio/wav", &wav)));
        let b64 = crate::base64::encode(&wav);
        assert!(h.contains(&format!("<audio id=\"au\" preload=\"auto\" src=\"data:audio/wav;base64,{b64}\">")));
        assert!(h.contains("au.currentTime") && h.contains("id=\"mu\""));
        assert!(!h.contains("src=\"http") && !h.contains("href=\"http") && !h.contains("fetch("));
        assert_eq!(h.matches("</script>").count(), 1);
        // no audio: byte-identical to the plain player and no audio element
        assert_eq!(html_player_with_audio(&small(), None), html_player(&small()));
        assert!(!html_player(&small()).contains("<audio"));
        // a hostile mime string is refused (page stays silent rather than injecting markup)
        let h = html_player_with_audio(&small(), Some(("x\"><script>", &wav)));
        assert!(!h.contains("<audio") && !h.contains("x\"><script>"));
    }

    #[test]
    fn lottie_structure_and_baked_motion() {
        let j = lottie_json(&small());
        assert_eq!(j["fr"], 12.0);
        assert_eq!(j["op"], 6);
        let layers = j["layers"].as_array().unwrap();
        assert_eq!(layers.len(), 2, "one shape layer + background solid");
        let kfs = layers[0]["shapes"][0]["it"][0]["ks"]["k"].as_array().unwrap();
        assert_eq!(kfs.len(), 6);
        let x = |f: usize| kfs[f]["s"][0]["v"][0][0].as_f64().unwrap();
        assert!((x(0) - 10.0).abs() < 1e-3 && (x(5) - 30.0).abs() < 1e-3, "the layer's x animation is baked into the vertices");
        let items = layers[0]["shapes"][0]["it"].as_array().unwrap();
        assert!(items.iter().any(|i| i["ty"] == "fl") && items.iter().any(|i| i["ty"] == "st"));
        serde_json::to_string(&j).unwrap();
    }

    #[test]
    fn lottie_blend_gradient_and_matte() {
                use crate::project::{Layer, LayerKind, MaskRef};
        use crate::style::{Blend, Paint};
        let mut p = crate::Project::new("m", 100.0, 100.0, 12.0, 3);
        let mut grad = Layer::vector("grad", vec![VPath::rect("r", 0.0, 0.0, 50.0, 50.0)]);
        if let LayerKind::Vector { paths } = &mut grad.kind {
            paths[0].fill = Some(Paint::linear(Vec2::new(0.0, 0.0), Vec2::new(50.0, 0.0), &[(0.0, "#ff0000"), (1.0, "#0000ff")]));
        }
        grad.blend = Blend::Multiply;
        grad.mask = Some(MaskRef { layer: "hole".into(), invert: true });
        let mut hole = Layer::vector("hole", vec![VPath::rect("r", 10.0, 10.0, 20.0, 20.0).with_fill("#ffffff")]);
        hole.visible = false;
        p.layers = vec![hole, grad];
        let j = lottie_json(&Engine::new(p).unwrap());
        let layers = j["layers"].as_array().unwrap();
        let target = layers.iter().find(|l| l["nm"] == "grad").unwrap();
        assert_eq!(target["bm"], 1, "multiply");
        assert_eq!(target["tt"], 2, "inverted alpha matte");
        let ti = layers.iter().position(|l| l["nm"] == "grad").unwrap();
        assert_eq!(layers[ti - 1]["td"], 1, "matte source sits directly above its target");
        assert_eq!(layers[ti - 1]["ks"]["o"]["k"], 100.0, "an invisible mask source still acts as a matte");
        let items = target["shapes"][0]["it"].as_array().unwrap();
        let gf = items.iter().find(|i| i["ty"] == "gf").expect("gradient fill");
        assert_eq!(gf["t"], 1);
        assert_eq!(gf["g"]["p"], 2);
        assert_eq!(gf["g"]["k"]["k"].as_array().unwrap().len(), 8);
    }
}
