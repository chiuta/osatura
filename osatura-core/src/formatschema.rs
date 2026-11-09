// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Publishes a JSON Schema (dialect 2020-12) for project files from a small own description of the format.
//!
//! Basis (public): *JSON Schema: A Media Type for Describing JSON Documents* (core, 2020-12) and *JSON Schema Validation*
//! (2020-12), https://json-schema.org/specification. Keywords used: `$schema`, `$id`, `$defs`, `$ref`, `type`, `enum`,
//! `const`, `minimum`, `maximum`, `required`, `default`, `description`, `properties`, `items`, `additionalProperties`.
//! Spec: docs/clean-room/specs/08-schema-json-export-format-proiect.md.
//!
//! The mechanism is [`FormatDesc`] -> [`generate`]; the output is deterministic (object keys are sorted because the
//! JSON object type of this crate is ordered by key; `required` keeps declaration order). [`project_format`] describes the
//! project file: the root, the camera, bones and IK chains in full, the other list members as free-form objects (the
//! loader reports unknown fields instead of refusing them, so the schema does not forbid extra properties unless
//! `strict` is set).
//! [`validate`] checks an instance against the subset of keywords this module emits (used by the tests and by `schema --check`).

use serde_json::{json, Map, Value};

/// URI of the dialect the generated schema declares.
pub const DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

/// Type of one field.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldType {
    Int,
    Real,
    Text,
    Bool,
    /// Accepted names, exactly as the loader accepts them.
    Enum(Vec<String>),
    /// Exactly this value (format version).
    Const(Value),
    /// A list whose members have the given type.
    List(Box<FieldType>),
    /// A fixed-length list of numbers (`[x, y]`, `[min, max]`).
    Tuple(usize),
    /// Reference to a named type of the description.
    Ref(String),
    /// Any JSON object (members not described).
    FreeObject,
    /// Any JSON value.
    Any,
}

/// One field of a type.
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: FieldType,
    /// A field with a default is optional and carries `default`; a field without one is required.
    pub default: Option<Value>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub doc: Option<String>,
}

impl Field {
    pub fn new(name: &str, ty: FieldType) -> Field {
        Field { name: name.into(), ty, default: None, min: None, max: None, doc: None }
    }
    pub fn default(mut self, v: Value) -> Field {
        self.default = Some(v);
        self
    }
    pub fn range(mut self, min: Option<f64>, max: Option<f64>) -> Field {
        self.min = min;
        self.max = max;
        self
    }
    pub fn doc(mut self, d: &str) -> Field {
        self.doc = Some(d.into());
        self
    }
    /// Optional without a default value (the writer omits it).
    pub fn optional(mut self) -> Field {
        self.default = Some(Value::Null);
        self
    }
}

/// A named object type.
#[derive(Clone, Debug, PartialEq)]
pub struct TypeDesc {
    pub name: String,
    pub doc: Option<String>,
    pub fields: Vec<Field>,
}

impl TypeDesc {
    pub fn new(name: &str, fields: Vec<Field>) -> TypeDesc {
        TypeDesc { name: name.into(), doc: None, fields }
    }
}

/// The whole description: a root type plus reusable types.
#[derive(Clone, Debug, PartialEq)]
pub struct FormatDesc {
    pub id: String,
    pub title: String,
    pub root: TypeDesc,
    pub types: Vec<TypeDesc>,
    /// Forbid properties that are not described.
    pub strict: bool,
}

const MAX_DEPTH: usize = 64;

fn num(v: f64) -> Value {
    if v.is_finite() && v.fract() == 0.0 && v.abs() < 9.0e15 {
        json!(v as i64)
    } else {
        json!(v)
    }
}

fn check_id(id: &str) -> Result<(), String> {
    let (scheme, rest) = id.split_once(':').ok_or_else(|| format!("$id '{id}' is not an absolute URI"))?;
    let ok_scheme = !scheme.is_empty() && scheme.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) && scheme.chars().all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c));
    if !ok_scheme || rest.is_empty() {
        return Err(format!("$id '{id}' is not an absolute URI"));
    }
    if let Some((_, frag)) = id.split_once('#') {
        if !frag.is_empty() {
            return Err(format!("$id '{id}' has a non-empty fragment"));
        }
    }
    Ok(())
}

fn type_schema(ty: &FieldType, depth: usize) -> Result<Value, String> {
    if depth > MAX_DEPTH {
        return Err("field type nested too deeply".into());
    }
    Ok(match ty {
        FieldType::Int => json!({"type": "integer"}),
        FieldType::Real => json!({"type": "number"}),
        FieldType::Text => json!({"type": "string"}),
        FieldType::Bool => json!({"type": "boolean"}),
        FieldType::Enum(names) => {
            if names.is_empty() {
                return Err("an enumeration needs at least one name".into());
            }
            let mut seen = std::collections::BTreeSet::new();
            for n in names {
                if !seen.insert(n.clone()) {
                    return Err(format!("enumeration name '{n}' appears twice"));
                }
            }
            json!({"type": "string", "enum": names})
        }
        FieldType::Const(v) => json!({"const": v}),
        FieldType::List(inner) => json!({"type": "array", "items": type_schema(inner, depth + 1)?}),
        FieldType::Tuple(n) => {
            json!({"type": "array", "items": {"type": "number"}, "minItems": n, "maxItems": n})
        }
        FieldType::Ref(name) => json!({"$ref": format!("#/$defs/{name}")}),
        FieldType::FreeObject => json!({"type": "object"}),
        FieldType::Any => json!({}),
    })
}

/// True when `value` is acceptable for `ty` within `[min, max]` (used to check declared defaults).
fn default_fits(ty: &FieldType, v: &Value, min: Option<f64>, max: Option<f64>) -> bool {
    if v.is_null() {
        return true; // "optional, omitted by the writer"
    }
    let in_range = |x: f64| min.is_none_or(|m| x >= m) && max.is_none_or(|m| x <= m);
    match ty {
        FieldType::Int => v.as_i64().is_some_and(|x| in_range(x as f64)) || v.as_u64().is_some_and(|x| in_range(x as f64)),
        FieldType::Real => v.as_f64().is_some_and(in_range),
        FieldType::Text => v.is_string(),
        FieldType::Bool => v.is_boolean(),
        FieldType::Enum(names) => v.as_str().is_some_and(|s| names.iter().any(|n| n == s)),
        FieldType::Const(c) => c == v,
        FieldType::List(_) => v.is_array(),
        FieldType::Tuple(n) => v.as_array().is_some_and(|a| a.len() == *n && a.iter().all(|x| x.is_number())),
        FieldType::Ref(_) | FieldType::FreeObject => v.is_object(),
        FieldType::Any => true,
    }
}

fn object_schema(t: &TypeDesc, strict: bool, known: &std::collections::BTreeSet<String>) -> Result<Value, String> {
    let mut props = Map::new();
    let mut required = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for f in &t.fields {
        if !seen.insert(f.name.clone()) {
            return Err(format!("type '{}': field '{}' declared twice", t.name, f.name));
        }
        if let (Some(a), Some(b)) = (f.min, f.max) {
            if !a.is_finite() || !b.is_finite() {
                return Err(format!("type '{}', field '{}': range is not finite", t.name, f.name));
            }
            if a > b {
                return Err(format!("type '{}', field '{}': minimum {a} is above maximum {b}", t.name, f.name));
            }
        }
        for r in refs_of(&f.ty) {
            if !known.contains(&r) {
                return Err(format!("type '{}', field '{}': unknown type '{r}'", t.name, f.name));
            }
        }
        let mut s = match type_schema(&f.ty, 0) {
            Ok(Value::Object(m)) => m,
            Ok(_) => Map::new(),
            Err(e) => return Err(format!("type '{}', field '{}': {e}", t.name, f.name)),
        };
        if matches!(f.ty, FieldType::Int | FieldType::Real) {
            if let Some(m) = f.min {
                s.insert("minimum".into(), num(m));
            }
            if let Some(m) = f.max {
                s.insert("maximum".into(), num(m));
            }
        }
        match &f.default {
            Some(d) => {
                if !default_fits(&f.ty, d, f.min, f.max) {
                    return Err(format!("type '{}', field '{}': default {d} does not satisfy the field's own schema", t.name, f.name));
                }
                if d.is_null() {
                    // the writer may emit `null` for an absent optional value
                    if let Some(Value::String(t)) = s.get("type").cloned() {
                        s.insert("type".into(), json!([t, "null"]));
                    }
                } else {
                    s.insert("default".into(), d.clone());
                }
            }
            None => required.push(Value::String(f.name.clone())),
        }
        if let Some(d) = &f.doc {
            s.insert("description".into(), json!(d));
        }
        props.insert(f.name.clone(), Value::Object(s));
    }
    let mut o = Map::new();
    o.insert("type".into(), json!("object"));
    o.insert("properties".into(), Value::Object(props));
    if !required.is_empty() {
        o.insert("required".into(), Value::Array(required));
    }
    if strict {
        o.insert("additionalProperties".into(), json!(false));
    }
    if let Some(d) = &t.doc {
        o.insert("description".into(), json!(d));
    }
    Ok(Value::Object(o))
}

fn refs_of(ty: &FieldType) -> Vec<String> {
    match ty {
        FieldType::Ref(n) => vec![n.clone()],
        FieldType::List(i) => refs_of(i),
        _ => vec![],
    }
}

/// Refs that must be present as a direct required member (no list, no optional): a cycle made only of these can never be satisfied.
fn required_refs(t: &TypeDesc) -> Vec<String> {
    t.fields.iter().filter(|f| f.default.is_none()).filter_map(|f| if let FieldType::Ref(n) = &f.ty { Some(n.clone()) } else { None }).collect()
}

/// Builds the schema document. Errors: an unknown type name, an unsatisfiable cycle of required members, a minimum above a
/// maximum, an empty enumeration, a default that breaks its own field, a bad `$id`.
pub fn generate(desc: &FormatDesc) -> Result<Value, String> {
    check_id(&desc.id)?;
    let mut known = std::collections::BTreeSet::new();
    for t in &desc.types {
        if !known.insert(t.name.clone()) {
            return Err(format!("type '{}' declared twice", t.name));
        }
        if t.name.is_empty() || t.name.contains(['/', '#', '~']) {
            return Err(format!("type name '{}' cannot be used in a reference", t.name));
        }
    }
    // a type that (through required members only) contains itself has no finite instance
    for t in desc.types.iter().chain(std::iter::once(&desc.root)) {
        let mut stack: Vec<String> = required_refs(t);
        let mut seen = std::collections::BTreeSet::new();
        while let Some(n) = stack.pop() {
            if n == t.name {
                return Err(format!("type '{}' requires itself (circular reference without an escape)", t.name));
            }
            if !seen.insert(n.clone()) {
                continue;
            }
            if let Some(u) = desc.types.iter().find(|u| u.name == n) {
                stack.extend(required_refs(u));
            }
        }
    }
    let mut root = match object_schema(&desc.root, desc.strict, &known)? {
        Value::Object(m) => m,
        _ => Map::new(),
    };
    let mut defs = Map::new();
    for t in &desc.types {
        defs.insert(t.name.clone(), object_schema(t, desc.strict, &known)?);
    }
    root.insert("$schema".into(), json!(DIALECT));
    root.insert("$id".into(), json!(desc.id));
    root.insert("title".into(), json!(desc.title));
    if !defs.is_empty() {
        root.insert("$defs".into(), Value::Object(defs));
    }
    Ok(Value::Object(root))
}

/// The schema as text: pretty-printed, UTF-8, trailing newline; the same description gives the same bytes.
pub fn to_text(schema: &Value) -> String {
    let mut s = serde_json::to_string_pretty(schema).unwrap_or_default();
    s.push('\n');
    s
}

/// Subset validator for the keywords [`generate`] emits (`type`, `enum`, `const`, `minimum`, `maximum`, `required`, `properties`,
/// `items`, `minItems`, `maxItems`, `additionalProperties: false`, `$ref` to `#/$defs/..`). `Err` carries the path of the first violation.
pub fn validate(schema: &Value, instance: &Value) -> Result<(), String> {
    check(schema, schema, instance, "$", 0)
}

fn type_ok(t: &str, v: &Value) -> bool {
    match t {
        "null" => v.is_null(),
        "boolean" => v.is_boolean(),
        "object" => v.is_object(),
        "array" => v.is_array(),
        "string" => v.is_string(),
        "number" => v.is_number(),
        "integer" => v.is_i64() || v.is_u64() || v.as_f64().is_some_and(|x| x.is_finite() && x.fract() == 0.0),
        _ => false,
    }
}

fn check(root: &Value, s: &Value, v: &Value, path: &str, depth: usize) -> Result<(), String> {
    if depth > MAX_DEPTH * 4 {
        return Err(format!("{path}: nested too deeply"));
    }
    let Some(o) = s.as_object() else {
        return if s == &Value::Bool(false) { Err(format!("{path}: nothing is allowed here")) } else { Ok(()) };
    };
    if let Some(r) = o.get("$ref").and_then(Value::as_str) {
        let name = r.strip_prefix("#/$defs/").ok_or_else(|| format!("{path}: unsupported $ref '{r}'"))?;
        let target = root.get("$defs").and_then(|d| d.get(name)).ok_or_else(|| format!("{path}: $ref '{r}' not found"))?;
        check(root, target, v, path, depth + 1)?;
    }
    match o.get("type") {
        Some(Value::String(t)) if !type_ok(t, v) => return Err(format!("{path}: expected {t}")),
        Some(Value::Array(ts)) if !ts.iter().filter_map(Value::as_str).any(|t| type_ok(t, v)) => return Err(format!("{path}: wrong type")),
        _ => {}
    }
    if let Some(e) = o.get("enum").and_then(Value::as_array) {
        if !e.contains(v) {
            return Err(format!("{path}: {v} is not one of the allowed values"));
        }
    }
    if let Some(c) = o.get("const") {
        if c != v {
            return Err(format!("{path}: must be {c}"));
        }
    }
    if let Some(x) = v.as_f64() {
        if o.get("minimum").and_then(Value::as_f64).is_some_and(|m| x < m) {
            return Err(format!("{path}: below the minimum"));
        }
        if o.get("maximum").and_then(Value::as_f64).is_some_and(|m| x > m) {
            return Err(format!("{path}: above the maximum"));
        }
    }
    if let Some(m) = v.as_object() {
        for r in o.get("required").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str) {
            if !m.contains_key(r) {
                return Err(format!("{path}: required property '{r}' is missing"));
            }
        }
        let props = o.get("properties").and_then(Value::as_object);
        for (k, val) in m {
            match props.and_then(|p| p.get(k)) {
                Some(ps) => check(root, ps, val, &format!("{path}.{k}"), depth + 1)?,
                None if o.get("additionalProperties") == Some(&Value::Bool(false)) => return Err(format!("{path}: property '{k}' is not allowed")),
                None => {}
            }
        }
    }
    if let Some(a) = v.as_array() {
        if o.get("minItems").and_then(Value::as_u64).is_some_and(|n| (a.len() as u64) < n) {
            return Err(format!("{path}: too few items"));
        }
        if o.get("maxItems").and_then(Value::as_u64).is_some_and(|n| (a.len() as u64) > n) {
            return Err(format!("{path}: too many items"));
        }
        if let Some(items) = o.get("items") {
            for (i, val) in a.iter().enumerate() {
                check(root, items, val, &format!("{path}[{i}]"), depth + 1)?;
            }
        }
    }
    Ok(())
}

/// `$id` of the published project schema.
pub const PROJECT_SCHEMA_ID: &str = "https://osatura.example/schema/project-v1.json";

/// The description of the project file (format version [`crate::project::FORMAT_VERSION`]).
pub fn project_format(strict: bool) -> FormatDesc {
    use FieldType::*;
    let f = Field::new;
    let obj_list = || List(Box::new(FreeObject));
    let camera = TypeDesc::new(
        "Camera",
        vec![
            f("position", Tuple(2)).optional().doc("Point of the stage at the screen centre; omitted = stage centre."),
            f("zoom", Real).default(json!(1)),
            f("rotation", Real).default(json!(0)).doc("Degrees."),
            f("focus", Real).default(json!(1)).doc("Depth of field: the parallax depth in focus."),
            f("blur", Real).default(json!(12)).doc("Depth of field: blur radius per unit of parallax distance."),
        ],
    );
    let bone = TypeDesc::new(
        "Bone",
        vec![
            f("name", Text),
            f("parent", Text).optional().doc("Parent bone name; parents are listed before their children."),
            f("length", Real),
            f("rest_offset", Tuple(2)).default(json!([0, 0])),
            f("rest_angle", Real).default(json!(0)).doc("Degrees."),
            f("angle_limits", Tuple(2)).optional(),
            f("scale_limits", Tuple(2)).optional(),
        ],
    );
    let ik = TypeDesc::new(
        "IkChain",
        vec![
            f("name", Text),
            f("bones", List(Box::new(Text))),
            f("target", Text),
            f("bend", Real).default(json!(1)),
            f("mix", Real).default(json!(1)),
            f("iterations", Int).default(json!(16)).range(Some(0.0), None),
            f("soft", Real).default(json!(0)),
            f("stretch", Real).default(json!(1)),
        ],
    );
    let filter = TypeDesc::new(
        "TrackFilter",
        vec![
            f("kind", Enum(["lowpass", "highpass", "bandpass", "notch", "allpass", "peaking", "lowshelf", "highshelf"].iter().map(|s| s.to_string()).collect())),
            f("freq", Real).range(Some(0.0), None).doc("Corner or centre frequency, Hz (> 0)."),
            f("q", Real).default(json!(0.7071067811865476)).doc("Quality factor."),
            f("gain_db", Real).default(json!(0)).doc("dB; peaking and shelves only."),
        ],
    );
    let sound = TypeDesc::new(
        "Sound",
        vec![
            f("name", Text),
            f("file", Text).doc("Path relative to the project file, or absolute."),
            f("start", Real).default(json!(0)).doc("Frame at which the sound starts."),
            f("gain", Real).default(json!(1)).doc("Linear gain 0..4."),
            f("trim_in", Real).default(json!(0)).doc("Seconds."),
            f("trim_out", Real).optional().doc("Seconds; omitted = to the end of the file."),
            f("fade_in", Real).default(json!(0)).doc("Seconds."),
            f("fade_out", Real).default(json!(0)).doc("Seconds."),
            f("pan", Real).optional().range(Some(-1.0), Some(1.0)).doc("Constant stereo pan, -1 (left) .. 1 (right); omitted = no pan stage."),
            f("filters", List(Box::new(Ref("TrackFilter".into())))).optional().doc("Biquad filters applied in order before the pan; omitted = none."),
        ],
    );
    let root = TypeDesc {
        name: "Project".into(),
        doc: Some("An OSATURA project file.".into()),
        fields: vec![
            f("format", Const(json!(crate::project::FORMAT_VERSION))).default(json!(crate::project::FORMAT_VERSION)).doc("File-format version; a missing value means 1."),
            f("name", Text),
            f("width", Real),
            f("height", Real),
            f("fps", Real),
            f("frames", Int).range(Some(0.0), None).doc("Number of frames."),
            f("background", Text).optional().doc("Hex colour (#rgb, #rrggbb or #rrggbbaa)."),
            f("color_space", Enum(vec!["srgb".into(), "linear".into()])).default(json!("srgb")),
            f("camera", Ref("Camera".into())).optional(),
            f("bones", List(Box::new(Ref("Bone".into())))).optional(),
            f("ik", List(Box::new(Ref("IkChain".into())))).optional(),
            f("constraints", obj_list()).optional(),
            f("springs", obj_list()).optional(),
            f("layers", obj_list()).optional(),
            f("channels", obj_list()).optional(),
            f("markers", obj_list()).optional(),
            f("comps", obj_list()).optional(),
            f("events", obj_list()).optional(),
            f("bone_vis", obj_list()).optional(),
            f("sounds", List(Box::new(Ref("Sound".into())))).optional(),
            f("skins", obj_list()).optional(),
            f("channel_meta", obj_list()).optional(),
            f("shots", obj_list()).optional(),
            f("clips", obj_list()).optional(),
            f("clip_baked", obj_list()).optional(),
            f("mesh_locks", obj_list()).optional(),
            f("glues", obj_list()).optional(),
            f("machines", obj_list()).optional(),
            f("bindings", obj_list()).optional(),
            f("containers", obj_list()).optional(),
            f("scrolls", obj_list()).optional(),
            f("lods", obj_list()).optional(),
        ],
    };
    FormatDesc { id: PROJECT_SCHEMA_ID.into(), title: "OSATURA project".into(), root, types: vec![bone, camera, ik, sound, filter], strict }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mini() -> FormatDesc {
        use FieldType::*;
        FormatDesc {
            id: "https://example.org/schema/mini.json".into(),
            title: "mini".into(),
            strict: false,
            root: TypeDesc::new("Root", vec![Field::new("versiune", Const(json!(3))), Field::new("puncte", List(Box::new(Ref("Punct".into()))))]),
            types: vec![TypeDesc::new("Punct", vec![Field::new("x", Real), Field::new("y", Real), Field::new("culoare", Enum(vec!["rosu".into(), "verde".into()])).default(json!("rosu"))])],
        }
    }

    #[test]
    fn header_and_defs() {
        let s = generate(&mini()).unwrap();
        assert_eq!(s["$schema"], DIALECT);
        let id = s["$id"].as_str().unwrap();
        assert!(id.contains("://") && !id.contains('#'));
        assert_eq!(s["$defs"].as_object().unwrap().len(), 1);
        assert_eq!(s["properties"]["puncte"]["items"]["$ref"], "#/$defs/Punct");
        assert_eq!(s["$defs"]["Punct"]["required"], json!(["x", "y"]));
    }

    #[test]
    fn enum_field_has_type_enum_and_default() {
        let s = generate(&mini()).unwrap();
        let c = &s["$defs"]["Punct"]["properties"]["culoare"];
        assert_eq!(c["type"], "string");
        assert_eq!(c["enum"], json!(["rosu", "verde"]));
        assert_eq!(c["default"], "rosu");
        assert_eq!(s["properties"]["versiune"]["const"], 3);
    }

    #[test]
    fn documents_validate_as_specified() {
        let s = generate(&mini()).unwrap();
        assert!(validate(&s, &json!({"versiune": 3, "puncte": [{"x": 1.5, "y": 2}]})).is_ok());
        let e = validate(&s, &json!({"versiune": 3, "puncte": [{"x": 1.5}]})).unwrap_err();
        assert!(e.contains("'y'"), "{e}");
        assert!(validate(&s, &json!({"versiune": 4, "puncte": []})).is_err());
        assert!(validate(&s, &json!({"versiune": 3, "puncte": [{"x": 1, "y": 2, "culoare": "albastru"}]})).is_err());
        assert!(validate(&s, &json!({"versiune": 3, "puncte": [{"x": 1, "y": 2, "culoare": "verde"}]})).is_ok());
    }

    #[test]
    fn output_is_deterministic_with_sorted_keys() {
        let a = to_text(&generate(&mini()).unwrap());
        let b = to_text(&generate(&mini()).unwrap());
        assert_eq!(a.as_bytes(), b.as_bytes());
        fn sorted(v: &Value) -> bool {
            match v {
                Value::Object(m) => m.keys().zip(m.keys().skip(1)).all(|(x, y)| x < y) && m.values().all(sorted),
                Value::Array(a) => a.iter().all(sorted),
                _ => true,
            }
        }
        let parsed: Value = serde_json::from_str(&a).unwrap();
        assert!(sorted(&parsed));
        // the text itself lists keys alphabetically: "$defs" < "$id" < "$schema" < "properties"
        let at = |k: &str| a.find(&format!("\n  \"{k}\"")).unwrap();
        assert!(at("$defs") < at("$id") && at("$id") < at("$schema") && at("$schema") < at("properties") && at("properties") < at("type"));
    }

    #[test]
    fn schema_only_uses_keywords_of_the_dialect() {
        let allowed = [
            "$schema",
            "$id",
            "$defs",
            "$ref",
            "type",
            "enum",
            "const",
            "minimum",
            "maximum",
            "required",
            "default",
            "description",
            "title",
            "properties",
            "items",
            "minItems",
            "maxItems",
            "additionalProperties",
        ];
        fn walk(v: &Value, allowed: &[&str], in_props: bool) {
            match v {
                Value::Object(m) => {
                    for (k, x) in m {
                        if !in_props {
                            assert!(allowed.contains(&k.as_str()), "keyword {k}");
                        }
                        // members of "properties" and "$defs" are names chosen by the format; "default"/"const"/"enum" hold data
                        let data = matches!(k.as_str(), "default" | "const" | "enum");
                        if !data {
                            walk(x, allowed, !in_props && matches!(k.as_str(), "properties" | "$defs"));
                        }
                    }
                }
                Value::Array(a) => a.iter().for_each(|x| walk(x, allowed, false)),
                _ => {}
            }
        }
        walk(&generate(&mini()).unwrap(), &allowed, false);
        walk(&generate(&project_format(true)).unwrap(), &allowed, false);
    }

    #[test]
    fn generation_errors() {
        use FieldType::*;
        let mut d = mini();
        d.types[0].fields[0] = Field::new("x", Real).range(Some(5.0), Some(3.0));
        assert!(generate(&d).unwrap_err().contains("minimum 5 is above maximum 3"));
        let mut d = mini();
        d.types[0].fields[2] = Field::new("culoare", Enum(vec![]));
        assert!(generate(&d).is_err());
        let mut d = mini();
        d.types[0].fields.push(Field::new("alt", Ref("Lipsa".into())));
        assert!(generate(&d).unwrap_err().contains("unknown type"));
        let mut d = mini();
        d.types[0].fields[2] = Field::new("culoare", Enum(vec!["a".into()])).default(json!("b"));
        assert!(generate(&d).unwrap_err().contains("default"));
        let mut d = mini();
        d.types[0].fields.push(Field::new("eu", Ref("Punct".into())));
        assert!(generate(&d).unwrap_err().contains("requires itself"));
        let mut d = mini();
        d.types[0].fields.push(Field::new("eu", List(Box::new(Ref("Punct".into())))));
        assert!(generate(&d).is_ok(), "a list of itself is fine (empty list)");
        let mut d = mini();
        d.id = "nu-e-uri".into();
        assert!(generate(&d).is_err());
        d.id = "https://x.org/s#frag".into();
        assert!(generate(&d).is_err());
        let mut d = mini();
        d.root.fields.clear();
        d.types.clear();
        assert!(generate(&d).is_ok(), "a format without fields is an empty object");
    }

    #[test]
    fn strict_mode_forbids_extra_properties() {
        let mut d = mini();
        d.strict = true;
        let s = generate(&d).unwrap();
        assert!(validate(&s, &json!({"versiune": 3, "puncte": [], "extra": 1})).is_err());
        let s = generate(&mini()).unwrap();
        assert!(validate(&s, &json!({"versiune": 3, "puncte": [], "extra": 1})).is_ok());
    }

    #[test]
    fn real_projects_validate_against_the_project_schema() {
        let s = generate(&project_format(false)).unwrap();
        let p = crate::demo::project();
        let v: Value = serde_json::from_str(&p.to_json_pretty()).unwrap();
        validate(&s, &v).unwrap();
        // every top-level field the writer can emit is described
        let props = s["properties"].as_object().unwrap();
        for k in v.as_object().unwrap().keys() {
            assert!(props.contains_key(k), "field {k} missing from the schema");
        }
        // sounds: old tracks (no pan, no filters) and tracks with the optional fields both validate; a bad pan does not
        let base = json!({"name": "x", "width": 1, "height": 1, "fps": 1, "frames": 1});
        let with = |snd: Value| {
            let mut v = base.clone();
            v["sounds"] = json!([snd]);
            v
        };
        assert!(validate(&s, &with(json!({"name": "a", "file": "a.wav", "start": 0, "gain": 1}))).is_ok());
        let new = json!({"name": "a", "file": "a.wav", "pan": -0.5, "filters": [{"kind": "lowpass", "freq": 800}, {"kind": "peaking", "freq": 1000, "q": 2, "gain_db": 3}]});
        assert!(validate(&s, &with(new)).is_ok());
        assert!(validate(&s, &with(json!({"name": "a", "file": "a.wav", "pan": 2}))).is_err());
        assert!(validate(&s, &with(json!({"name": "a", "file": "a.wav", "filters": [{"kind": "wobble", "freq": 1}]}))).is_err());
        assert!(validate(&s, &json!({"name": "x", "width": 1, "height": 1, "fps": 1})).is_err(), "frames is required");
        assert!(validate(&s, &json!({"format": 2, "name": "x", "width": 1, "height": 1, "fps": 1, "frames": 1})).is_err());
        assert!(validate(&s, &json!({"name": "x", "width": 1, "height": 1, "fps": 1, "frames": 1, "color_space": "cmyk"})).is_err());
    }
}
