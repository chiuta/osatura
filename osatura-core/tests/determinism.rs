// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Cross-platform determinism check: three frames of the built-in demo project are rendered and compared with PNGs
//! committed in `tests/golden/`, which were written on Linux x86-64.
//!
//! * Always: two renders in one process are byte-identical, and every frame is within a small tolerance of the golden
//!   (at most 8 per channel on at most 0.5 % of the pixels; anti-aliased edges may differ between OS, CPU and libm,
//!   see docs/LIMITATIONS.md L-26).
//! * With `OSATURA_DETERMINISM_STRICT=1`: the render must equal the golden byte for byte. CI runs this on Linux, Windows
//!   and macOS (the non-Linux jobs are informational) and writes the measured differences to the job summary.
//! * With `OSATURA_GOLDEN_UPDATE=1`: the goldens are rewritten (do this deliberately, on Linux x86-64, and say why in the commit).
//!
//! Run `cargo test -p osatura-core --test determinism -- --nocapture` to see the numbers.
use osatura_core::demo;
use osatura_core::engine::Engine;
use osatura_core::open_formats::encode_rgba_png;
use osatura_core::raster::{render_rgba, RasterOptions};
use std::path::PathBuf;

const FRAMES: [f32; 3] = [0.0, 8.0, 16.0];
const SCALE: f32 = 0.25;

fn golden_path(frame: f32) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden").join(format!("demo_f{}_s025.png", frame as u32))
}

fn decode(png_bytes: &[u8]) -> (u32, u32, Vec<u8>) {
    let mut rd = png::Decoder::new(std::io::Cursor::new(png_bytes)).read_info().expect("golden PNG header");
    let mut buf = vec![0; rd.output_buffer_size().expect("golden PNG size")];
    let info = rd.next_frame(&mut buf).expect("golden PNG data");
    buf.truncate(info.buffer_size());
    (info.width, info.height, buf)
}

#[test]
fn demo_frames_match_the_goldens_across_platforms() {
    let mut p = demo::project();
    p.frames = 24;
    let engine = Engine::new(p).expect("demo project compiles");
    let opts = RasterOptions { scale: SCALE, ..Default::default() };
    let strict = std::env::var("OSATURA_DETERMINISM_STRICT").is_ok_and(|v| v == "1");
    let update = std::env::var("OSATURA_GOLDEN_UPDATE").is_ok_and(|v| v == "1");
    let mut report = Vec::new();
    let mut failures = Vec::new();
    for f in FRAMES {
        let (w, h, a) = render_rgba(&engine, f, &opts);
        let (_, _, b) = render_rgba(&engine, f, &opts);
        assert_eq!(a, b, "frame {f}: two renders in one process differ");
        let path = golden_path(f);
        if update {
            std::fs::write(&path, encode_rgba_png(w, h, &a).unwrap()).unwrap();
            continue;
        }
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("missing golden {}: {e} (run with OSATURA_GOLDEN_UPDATE=1 on Linux x86-64)", path.display()));
        let (gw, gh, g) = decode(&bytes);
        assert_eq!((gw, gh), (w, h), "frame {f}: golden size differs");
        assert_eq!(g.len(), a.len());
        let mut max = 0u8;
        let mut differing = 0usize;
        for (x, y) in a.chunks(4).zip(g.chunks(4)) {
            let d = x.iter().zip(y).map(|(p, q)| p.abs_diff(*q)).max().unwrap_or(0);
            if d > 0 {
                differing += 1;
                max = max.max(d);
            }
        }
        let pixels = a.len() / 4;
        let pct = 100.0 * differing as f64 / pixels as f64;
        report.push(format!("frame {f}: {differing} of {pixels} pixels differ ({pct:.3} %), largest channel difference {max}"));
        if strict && differing > 0 {
            failures.push(format!("frame {f} is not byte-identical to the golden ({differing} pixels, max {max})"));
        } else if max > 8 && pct > 0.5 {
            failures.push(format!("frame {f} is outside the tolerance ({differing} pixels, max {max})"));
        }
    }
    for l in &report {
        println!("{l}");
    }
    if let Ok(path) = std::env::var("OSATURA_DETERMINISM_REPORT") {
        let _ = std::fs::write(path, report.join("\n") + "\n");
    }
    assert!(failures.is_empty(), "{}", failures.join("; "));
}
