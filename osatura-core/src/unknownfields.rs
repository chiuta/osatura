// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Unknown-key detection for project files (R4).
//!
//! `serde` drops keys it does not know, so a typo such as `"anchors_"` or `"fil"` is silently ignored and the
//! file "works" while doing nothing. Rejecting unknown keys everywhere would break files written by newer
//! versions, so instead the input JSON is compared with the re-serialised [`Project`]: a key that is in the
//! input but not in the output was not understood. Keys whose value is a default-looking one (`null`, `false`,
//! `0`, `""`, `[]`, `{}`) are not reported, because the writer omits defaults and they would be false alarms.
//! Fields whose default is not one of those (`camera.focus = 1.0`, `camera.blur = 12.0`, ...) are recognised by a
//! probe: the suspicious value is changed and the file is read again; if the key then appears in the output (or the
//! changed value is rejected) the model knows it, otherwise it is ignored by the model and reported.

use crate::project::Project;
use serde_json::Value;

/// Paths such as `layers[2].paths[0].fil` of keys in `text` that the project model ignores.
/// `Err` only if the text is not a project at all.
pub fn unknown_fields(text: &str) -> Result<Vec<String>, String> {
    let input: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let p = Project::from_json(text).map_err(|e| e.to_string())?;
    let output = serde_json::to_value(&p).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    let probe = |v: &Value| -> Option<Value> {
        serde_json::to_value(serde_json::from_value::<Project>(v.clone()).ok()?).ok()
    };
    walk(
        &input,
        &output,
        String::new(),
        &mut out,
        &input,
        &mut Vec::new(),
        &probe,
    );
    Ok(out)
}

/// Parse a user-authored configuration file (mapping table, machine, events, goals, ...) and refuse keys the type does
/// not understand: the same re-serialise-and-compare test as [`unknown_fields`], for any serde type. Typos fail loudly
/// instead of producing a config that silently does nothing.
pub fn parse_strict<T: serde::Serialize + serde::de::DeserializeOwned>(
    text: &str,
) -> Result<T, String> {
    let input: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let t: T = serde_json::from_value(input.clone()).map_err(|e| e.to_string())?;
    let output = serde_json::to_value(&t).map_err(|e| e.to_string())?;
    let mut found = Vec::new();
    let probe = |v: &Value| -> Option<Value> {
        serde_json::to_value(serde_json::from_value::<T>(v.clone()).ok()?).ok()
    };
    walk(
        &input,
        &output,
        String::new(),
        &mut found,
        &input,
        &mut Vec::new(),
        &probe,
    );
    if found.is_empty() {
        Ok(t)
    } else {
        Err(format!(
            "unknown field(s): {} (typo? nothing was applied)",
            found.join(", ")
        ))
    }
}

fn default_like(v: &Value) -> bool {
    match v {
        Value::Null => true,
        Value::Bool(b) => !*b,
        Value::Number(n) => n.as_f64() == Some(0.0),
        Value::String(s) => s.is_empty(),
        Value::Array(a) => a.is_empty(),
        // an object whose members are all default-like (`{"keys": []}` for an empty track, which the writer omits)
        Value::Object(m) => m.values().all(default_like),
    }
}

/// One step of the way from the document root to a value.
#[derive(Clone)]
enum Seg {
    Key(String),
    Idx(usize),
}

fn at_mut<'a>(root: &'a mut Value, way: &[Seg]) -> Option<&'a mut Value> {
    let mut cur = root;
    for s in way {
        cur = match s {
            Seg::Key(k) => cur.get_mut(k)?,
            Seg::Idx(i) => cur.get_mut(*i)?,
        };
    }
    Some(cur)
}

fn at<'a>(root: &'a Value, way: &[Seg]) -> Option<&'a Value> {
    let mut cur = root;
    for s in way {
        cur = match s {
            Seg::Key(k) => cur.get(k)?,
            Seg::Idx(i) => cur.get(*i)?,
        };
    }
    Some(cur)
}

/// Is the key at `way` (missing from the first output) one the model reads? Change its value and read again.
fn known_by_probe(root: &Value, way: &[Seg], probe: &dyn Fn(&Value) -> Option<Value>) -> bool {
    let mut changed = root.clone();
    let Some(slot) = at_mut(&mut changed, way) else {
        return false;
    };
    *slot = match &*slot {
        Value::Number(n) => serde_json::json!(n.as_f64().unwrap_or(0.0) + 1.0),
        Value::Bool(b) => Value::Bool(!*b),
        Value::String(t) => Value::String(format!("{t}~")),
        _ => return false,
    };
    match probe(&changed) {
        None => true, // the model rejects the changed value: it reads this key
        Some(out) => at(&out, way).is_some(),
    }
}

fn walk(
    input: &Value,
    output: &Value,
    path: String,
    found: &mut Vec<String>,
    root: &Value,
    way: &mut Vec<Seg>,
    probe: &dyn Fn(&Value) -> Option<Value>,
) {
    match (input, output) {
        (Value::Object(i), Value::Object(o)) => {
            for (k, v) in i {
                let here = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                way.push(Seg::Key(k.clone()));
                match o.get(k) {
                    Some(ov) => walk(v, ov, here, found, root, way, probe),
                    None if default_like(v) => {}
                    None => {
                        if !known_by_probe(root, way, probe) {
                            found.push(here);
                        }
                    }
                }
                way.pop();
            }
        }
        (Value::Array(i), Value::Array(o)) if i.len() == o.len() => {
            for (n, (iv, ov)) in i.iter().zip(o).enumerate() {
                way.push(Seg::Idx(n));
                walk(iv, ov, format!("{path}[{n}]"), found, root, way, probe);
                way.pop();
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo_json() -> String {
        crate::demo::project().to_json_pretty()
    }

    #[test]
    fn a_clean_project_has_no_unknown_fields() {
        assert_eq!(unknown_fields(&demo_json()).unwrap(), Vec::<String>::new());
    }

    #[test]
    fn typos_are_reported_with_their_path() {
        let mut v: Value = serde_json::from_str(&demo_json()).unwrap();
        v["layers"][0]["opacty"] = serde_json::json!(0.5);
        v["layers"][0]["paths"][0]["fil"] = serde_json::json!({"color": "#fff"});
        v["bogus_top"] = serde_json::json!([1, 2]);
        let found = unknown_fields(&v.to_string()).unwrap();
        assert!(found.contains(&"layers[0].opacty".to_string()), "{found:?}");
        assert!(
            found.contains(&"layers[0].paths[0].fil".to_string()),
            "{found:?}"
        );
        assert!(found.contains(&"bogus_top".to_string()), "{found:?}");
    }

    #[test]
    fn explicit_defaults_are_not_false_alarms() {
        let mut v: Value = serde_json::from_str(&demo_json()).unwrap();
        v["layers"][0]["reference"] = serde_json::json!(false);
        v["layers"][0]["locked"] = serde_json::json!(false);
        assert_eq!(
            unknown_fields(&v.to_string()).unwrap(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn parse_strict_rejects_typos_in_any_config_type() {
        let ok = r#"{"params":[{"source":"midi_cc","id":"1:7","target":"arm","prop":"bone_angle"}],"triggers":[]}"#;
        let r = parse_strict::<crate::liveinput::Mapping>(ok);
        assert!(r.is_ok(), "{r:?}");
        let typo =
            r#"{"parms":[{"source":"midi_cc","id":"1:7","target":"arm","prop":"bone_angle"}]}"#;
        let e = parse_strict::<crate::liveinput::Mapping>(typo).unwrap_err();
        assert!(e.contains("parms"), "{e}");
        let typo2 = r#"{"params":[{"source":"midi_cc","id":"1:7","target":"arm","prop":"bone_angle","in_mx":3}]}"#;
        assert!(parse_strict::<crate::liveinput::Mapping>(typo2)
            .unwrap_err()
            .contains("in_mx"));
    }

    #[test]
    fn not_a_project_is_an_error() {
        assert!(unknown_fields("[1,2").is_err());
        assert!(unknown_fields("{\"x\":1}").is_err());
    }

    /// Fields with a non-zero default (`camera.focus` 1.0, `camera.blur` 12.0) are not "unknown" when written out.
    #[test]
    fn fields_with_non_zero_defaults_are_not_false_alarms() {
        let mut v: Value = serde_json::from_str(
            &crate::project::Project::new("d", 10.0, 10.0, 24.0, 4).to_json_pretty(),
        )
        .unwrap();
        v["camera"]["focus"] = serde_json::json!(1.0);
        v["camera"]["blur"] = serde_json::json!(12.0);
        assert_eq!(
            unknown_fields(&v.to_string()).unwrap(),
            Vec::<String>::new(),
            "old behaviour: camera.focus / camera.blur reported"
        );
        // a real typo next to them is still found, and so is a typo with a plausible non-zero value
        v["camera"]["blurr"] = serde_json::json!(12.0);
        v["camera"]["focuss"] = serde_json::json!(true);
        let found = unknown_fields(&v.to_string()).unwrap();
        assert_eq!(
            found,
            vec!["camera.blurr".to_string(), "camera.focuss".to_string()]
        );
    }

    /// Every field of every Project / Layer / Camera default, written out explicitly, produces no warning.
    #[test]
    fn a_file_with_every_default_spelled_out_has_no_warnings() {
        let mut p = crate::project::Project::new("d", 64.0, 48.0, 24.0, 4);
        p.layers.push(crate::project::Layer::vector(
            "a",
            vec![crate::path::VPath::rect("r", 1.0, 1.0, 5.0, 5.0).with_fill("#112233")],
        ));
        // fully expanded: serialise, then add back every field the writer omitted because it was the default
        let text = p.to_json_pretty();
        let mut v: Value = serde_json::from_str(&text).unwrap();
        v["camera"] = serde_json::json!({"zoom": 1.0, "rotation": 0.0, "focus": 1.0, "blur": 12.0});
        assert_eq!(
            unknown_fields(&v.to_string()).unwrap(),
            Vec::<String>::new()
        );
        // the round trip of the writer's own output is clean as well
        assert_eq!(unknown_fields(&text).unwrap(), Vec::<String>::new());
        // a hand-written minimal file
        let hand = r#"{"format":1,"name":"h","width":10,"height":10,"fps":24,"frames":2,"camera":{"focus":1.0,"blur":12.0,"zoom":1.0},"layers":[]}"#;
        assert_eq!(unknown_fields(hand).unwrap(), Vec::<String>::new());
    }

    /// A valid empty track (`clips[].weight_keys: {"keys": []}`): the writer omits it, so it must not be reported.
    #[test]
    fn an_explicit_empty_track_is_not_unknown() {
        let mut p = crate::demo::project();
        let take = crate::motionlib::Take {
            name: "t".into(),
            length: 10.0,
            tracks: vec![],
        };
        crate::clipstack::add_clip(
            &mut p,
            crate::clipstack::Clip {
                name: "c".into(),
                take,
                start: 0.0,
                speed: 1.0,
                loops: 1,
                mode: Default::default(),
                weight: 1.0,
                weight_keys: Default::default(),
                include: vec![],
                exclude: vec![],
                track_weights: vec![],
                fade_in: 0.0,
                fade_out: 0.0,
                rename: vec![],
            },
        )
        .unwrap();
        let mut v: Value = serde_json::from_str(&p.to_json_pretty()).unwrap();
        assert!(
            v["clips"][0].get("weight_keys").is_none(),
            "the writer omits an empty track"
        );
        v["clips"][0]["weight_keys"] = serde_json::json!({"keys": []});
        assert_eq!(
            unknown_fields(&v.to_string()).unwrap(),
            Vec::<String>::new()
        );
        // a typo inside a non-empty track is still found
        v["clips"][0]["weight_keys"] = serde_json::json!({"keys": [], "kes": [1]});
        let found = unknown_fields(&v.to_string()).unwrap();
        assert_eq!(
            found,
            vec!["clips[0].weight_keys".to_string()],
            "a non-default object that the model drops is still reported"
        );
    }

    /// Every optional structure present but empty (what hand-written files and other tools produce) gives no warning.
    /// The list of custom `skip_serializing_if` predicates in the sources is pinned: a new kind of omitted-when-empty
    /// field needs a line here, which is the moment to check that spelling it out explicitly does not warn.
    #[test]
    fn every_optional_structure_present_but_empty_has_no_warning() {
        let mut p = crate::demo::project();
        let take = crate::motionlib::Take {
            name: "t".into(),
            length: 10.0,
            tracks: vec![],
        };
        crate::clipstack::add_clip(
            &mut p,
            crate::clipstack::Clip {
                name: "c".into(),
                take,
                start: 0.0,
                speed: 1.0,
                loops: 1,
                mode: Default::default(),
                weight: 1.0,
                weight_keys: Default::default(),
                include: vec![],
                exclude: vec![],
                track_weights: vec![],
                fade_in: 0.0,
                fade_out: 0.0,
                rename: vec![],
            },
        )
        .unwrap();
        let mut v: Value = serde_json::from_str(&p.to_json_pretty()).unwrap();
        // top level: every optional list, spelled out empty
        for k in [
            "markers",
            "comps",
            "events",
            "bone_vis",
            "sounds",
            "skins",
            "channel_meta",
            "shots",
            "clip_baked",
            "mesh_locks",
            "glues",
            "machines",
            "bindings",
            "containers",
            "scrolls",
            "lods",
        ] {
            if !v.as_object().unwrap().contains_key(k)
                || v[k].as_array().is_some_and(|a| a.is_empty())
            {
                v[k] = serde_json::json!([]);
            }
        }
        // clips: every optional member, spelled out empty
        for k in ["include", "exclude", "track_weights", "rename"] {
            v["clips"][0][k] = serde_json::json!([]);
        }
        v["clips"][0]["weight_keys"] = serde_json::json!({"keys": []});
        // layers and paths: optional members present but empty
        for l in v["layers"].as_array_mut().unwrap() {
            l["deformers"] = serde_json::json!([]);
            l["mask"] = Value::Null;
            l["bone"] = Value::Null;
            l["wobble"] = Value::Null;
            l["effects"] = serde_json::json!([]);
            l["normal_map"] = Value::Null;
            l["reference"] = serde_json::json!(false);
            l["locked"] = serde_json::json!(false);
        }
        v["color_space"] = serde_json::json!("srgb");
        v["camera"]["position"] = Value::Null;
        v["camera"]["focus"] = serde_json::json!(1.0);
        v["camera"]["blur"] = serde_json::json!(12.0);
        // channels: an empty keyed track
        v["channels"].as_array_mut().unwrap().push(serde_json::json!({"target": "arm_up", "prop": "bone_angle", "source": "keys", "track": {"keys": []}}));
        let found = unknown_fields(&v.to_string()).unwrap();
        assert!(
            found.is_empty(),
            "false positives for explicitly empty structures: {found:?}"
        );
        // the explicit-empty file is the same project as the original
        assert_eq!(Project::from_json(&v.to_string()).unwrap().clips, p.clips);

        // the predicates the writer uses to omit fields; each one has been considered above
        let known = [
            "Option::is_none",
            "Vec::is_empty",
            "Value::is_null",
            "serde_json::Value::is_null",
            "is_default_blur",
            "is_default_focus",
            "is_false",
            "is_zero",
            "track_empty",
            "ColorSpace::is_default",
        ];
        let mut unknown = Vec::new();
        for dir in [concat!(env!("CARGO_MANIFEST_DIR"), "/src")] {
            for e in std::fs::read_dir(dir).unwrap().flatten() {
                let Ok(text) = std::fs::read_to_string(e.path()) else {
                    continue;
                };
                let mut rest = text.as_str();
                const NEEDLE: &str = "skip_serializing_if = \"";
                while let Some(i) = rest.find(NEEDLE) {
                    let tail = &rest[i + NEEDLE.len()..];
                    let name = &tail[..tail.find('"').unwrap_or(0)];
                    if !name.is_empty()
                        && !known.contains(&name)
                        && !unknown.contains(&name.to_string())
                    {
                        unknown.push(name.to_string());
                    }
                    rest = tail;
                }
            }
        }
        assert!(unknown.is_empty(), "new skip_serializing_if predicate(s) {unknown:?}: add the matching explicit-empty value to this test (and to unknownfields::default_like if it is not a default-looking JSON value)");
    }
}
