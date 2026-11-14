// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Robustness tests: deliberately corrupted projects must be rejected or rendered, never
//! panic, hang, or allocate without bound.

#[cfg(test)]
mod tests {
    use crate::{demo, Engine, Project};
    use serde_json::Value;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        fn below(&mut self, n: usize) -> usize {
            (self.next() % n.max(1) as u64) as usize
        }
    }

    fn leaves(v: &Value, path: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
        match v {
            Value::Object(m) => {
                for (k, x) in m {
                    path.push(k.clone());
                    leaves(x, path, out);
                    path.pop();
                }
            }
            Value::Array(a) => {
                for (i, x) in a.iter().enumerate() {
                    path.push(i.to_string());
                    leaves(x, path, out);
                    path.pop();
                }
            }
            _ => out.push(path.clone()),
        }
    }

    fn get_mut<'a>(v: &'a mut Value, path: &[String]) -> Option<&'a mut Value> {
        let mut cur = v;
        for p in path {
            cur = match cur {
                Value::Object(m) => m.get_mut(p)?,
                Value::Array(a) => a.get_mut(p.parse::<usize>().ok()?)?,
                _ => return None,
            };
        }
        Some(cur)
    }

    // 1e30 and -1e30 are refused at load (outside +-1e7); 9e6 and 4194303 are the largest values that must still work
    const NASTY: [f64; 9] = [0.0, -1.0, 1e-30, 1e30, -1e30, 65536.0, 4194303.0, 0.5, 9.0e6];

    #[test]
    fn mutated_projects_never_panic() {
        let base = serde_json::to_value(demo::project()).unwrap();
        let mut all = Vec::new();
        leaves(&base, &mut Vec::new(), &mut all);
        let mut rng = Rng(0xC0FFEE);
        let mut accepted = 0;
        for i in 0..400 {
            let mut v = base.clone();
            for _ in 0..1 + rng.below(3) {
                let path = &all[rng.below(all.len())];
                if let Some(slot) = get_mut(&mut v, path) {
                    *slot = match rng.below(5) {
                        0..=2 => serde_json::json!(NASTY[rng.below(NASTY.len())]),
                        3 => Value::Null,
                        _ => Value::String("x".repeat(1 + rng.below(40))),
                    };
                }
            }
            let text = v.to_string();
            let r = catch_unwind(AssertUnwindSafe(|| {
                let Ok(p) = Project::from_json(&text) else { return false };
                let Ok(e) = Engine::new(p.clone()) else { return false };
                let frames = e.project.frames.min(200);
                for f in [0, frames / 2, frames.saturating_sub(1)] {
                    let _ = e.evaluate(f as f32);
                }
                let _ = crate::svg::render_frame(&e, 0.0, &Default::default());
                true
            }));
            match r {
                Ok(true) => accepted += 1,
                Ok(false) => {}
                Err(_) => panic!("mutation {i} caused a panic:\n{}", &text[..text.len().min(400)]),
            }
        }
        assert!(accepted > 50, "mutations that stay valid should still render ({accepted})");
    }

    #[test]
    fn truncated_json_is_an_error_not_a_panic() {
        let text = demo::project().to_json_pretty();
        for cut in [0, 1, 10, text.len() / 3, text.len() / 2, text.len() - 2] {
            assert!(Project::from_json(&text[..cut]).is_err());
        }
    }

    #[test]
    fn absurd_sizes_are_refused_by_validation() {
        for (field, value) in [("frames", 4_000_000_000u64), ("width", 1_000_000_000), ("height", 0)] {
            let mut v = serde_json::to_value(demo::project()).unwrap();
            v[field] = serde_json::json!(value);
            let p: Project = serde_json::from_value(v).unwrap();
            assert!(Engine::new(p).is_err(), "{field}={value} must be rejected");
        }
    }

    #[test]
    fn newer_format_versions_are_refused() {
        let mut v = serde_json::to_value(demo::project()).unwrap();
        v["format"] = serde_json::json!(99);
        let err = Project::from_json(&v.to_string()).unwrap_err().to_string();
        assert!(err.contains("newer"), "{err}");
        v.as_object_mut().unwrap().remove("format");
        assert!(Project::from_json(&v.to_string()).is_ok(), "files without a version load as v1");
    }
}
