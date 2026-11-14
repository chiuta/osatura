// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Matching (part of `mocap`; see the module docs in `mod.rs`).
use super::*;

// Matching

#[derive(Clone, Debug)]
pub struct BoneMatch {
    /// Canonical slot (`hips`, `spine.0`, `upperarm.l` ...) or `manual`.
    pub slot: String,
    pub bvh: usize,
    pub rig: usize,
    pub bvh_name: String,
    pub rig_name: String,
    pub confidence: f32,
    /// `name`, `topology` or `manual`.
    pub via: &'static str,
    /// Rig bone length / BVH bone length at rest.
    pub length_ratio: Option<f64>,
    pub note: String,
    pub tip: Tip,
}

#[derive(Clone, Debug, Default)]
pub struct MatchResult {
    pub matches: Vec<BoneMatch>,
    pub unmatched_bvh: Vec<String>,
    pub unmatched_rig: Vec<String>,
    pub warnings: Vec<String>,
}

/// Manual override: BVH joint name -> rig bone name, or `None` to exclude that joint.
pub type ManualMap = BTreeMap<String, Option<String>>;

pub fn parse_map(json: &str) -> Result<ManualMap, String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("bad mapping JSON: {e}"))?;
    let obj = v.as_object().ok_or("mapping must be a JSON object {\"bvh joint\": \"rig bone\"}")?;
    let obj = match (obj.len(), obj.get("map").or_else(|| obj.get("bones"))) {
        (1, Some(serde_json::Value::Object(inner))) => inner,
        _ => obj,
    };
    let mut out = ManualMap::new();
    for (k, val) in obj {
        match val {
            serde_json::Value::String(s) if !s.is_empty() => out.insert(k.clone(), Some(s.clone())),
            serde_json::Value::Null => out.insert(k.clone(), None),
            serde_json::Value::String(_) => out.insert(k.clone(), None),
            _ => return Err(format!("mapping for '{k}' must be a bone name or null")),
        };
    }
    Ok(out)
}

pub(super) fn median(v: &mut [f64]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    let n = v.len();
    Some(if n % 2 == 1 { v[n / 2] } else { 0.5 * (v[n / 2 - 1] + v[n / 2]) })
}

pub(super) fn find_joint(b: &Bvh, name: &str) -> Option<usize> {
    b.joint_index(name).or_else(|| {
        let l = name.to_lowercase();
        let hits: Vec<usize> = (0..b.joints.len()).filter(|&i| b.joints[i].name.to_lowercase() == l).collect();
        (hits.len() == 1).then(|| hits[0])
    })
}

/// Match BVH joints to rig bones. See the module docs for the strategy.
pub fn match_bones(bvh: &Bvh, p: &Project, manual: Option<&ManualMap>) -> Result<MatchResult, String> {
    if p.bones.is_empty() {
        return Err("project has no bones".into());
    }
    let mut sb = Skel::from_bvh(bvh);
    let mut sr = Skel::from_rig(p)?;
    let nb = name_slots(&sb);
    let nr = name_slots(&sr);
    let tb = topo_slots(&mut sb, nb.single.get("hips").map(|a| a.joint));
    let tr = topo_slots(&mut sr, nr.single.get("hips").map(|a| a.joint));
    let ab = merge_slots(nb, tb);
    let ar = merge_slots(nr, tr);

    let mut pairs: Vec<(String, Assign, Assign)> = Vec::new();
    let order: Vec<String> = {
        let mut o = vec!["hips".to_string()];
        o.push("spine".into());
        o.extend(["neck", "head"].iter().map(|s| s.to_string()));
        for side in ["l", "r"] {
            for r in ["clavicle", "upperarm", "forearm", "hand"] {
                o.push(format!("{r}.{side}"));
            }
        }
        for side in ["l", "r"] {
            for r in ["thigh", "shin", "foot", "toe"] {
                o.push(format!("{r}.{side}"));
            }
        }
        o
    };
    for key in &order {
        if key == "spine" {
            let (m, n) = (ab.spine.len(), ar.spine.len());
            if m == 0 || n == 0 {
                continue;
            }
            let k = m.min(n);
            let pick = |len: usize, i: usize| -> usize {
                if k == 1 {
                    0
                } else {
                    ((i as f64) * (len as f64 - 1.0) / (k as f64 - 1.0)).round() as usize
                }
            };
            for i in 0..k {
                let (mut a, mut r) = (ab.spine[pick(m, i)], ar.spine[pick(n, i)]);
                if m != n {
                    a.conf *= 0.85;
                    r.conf *= 0.85;
                }
                pairs.push((format!("spine.{i}"), a, r));
            }
        } else if let (Some(a), Some(r)) = (ab.single.get(key), ar.single.get(key)) {
            pairs.push((key.clone(), *a, *r));
        }
    }

    let mut matches: Vec<BoneMatch> = pairs
        .into_iter()
        .map(|(slot, a, r)| BoneMatch {
            slot,
            bvh: a.joint,
            rig: r.joint,
            bvh_name: bvh.joints[a.joint].name.clone(),
            rig_name: p.bones[r.joint].name.clone(),
            confidence: a.conf.min(r.conf),
            via: if a.via == Via::Name && r.via == Via::Name { "name" } else { "topology" },
            length_ratio: None,
            note: String::new(),
            tip: Tip::Extrap(a.joint),
        })
        .collect();

    // manual overrides
    let mut warnings = Vec::new();
    if let Some(map) = manual {
        let mut seen_rig: BTreeMap<usize, String> = BTreeMap::new();
        let mut errs = Vec::new();
        let mut ov: Vec<(usize, Option<usize>)> = Vec::new();
        for (bn, rn) in map {
            let Some(bj) = find_joint(bvh, bn) else {
                errs.push(format!("mapping: no BVH joint '{bn}'"));
                continue;
            };
            let rj = match rn {
                None => None,
                Some(rn) => match p.bones.iter().position(|b| &b.name == rn) {
                    Some(i) => {
                        if let Some(prev) = seen_rig.insert(i, bn.clone()) {
                            errs.push(format!("mapping: rig bone '{rn}' is the target of both '{prev}' and '{bn}'"));
                        }
                        Some(i)
                    }
                    None => {
                        errs.push(format!("mapping: no rig bone '{rn}'"));
                        continue;
                    }
                },
            };
            ov.push((bj, rj));
        }
        if !errs.is_empty() {
            return Err(errs.join("; "));
        }
        // a manual match keeps the canonical slot of the BVH joint (hips, spine.N, upperarm.l, ...): the slot decides the
        // tip, root travel and the "no hips match" warning, so `Hips=bone` must stay the hips
        let bvh_slot: BTreeMap<usize, String> = ab.single.iter().map(|(k, a)| (a.joint, k.clone())).collect();
        for (bj, rj) in ov {
            let inherited = matches.iter().find(|m| m.bvh == bj).map(|m| m.slot.clone()).filter(|sl| sl != "manual").or_else(|| bvh_slot.get(&bj).cloned());
            matches.retain(|m| m.bvh != bj && Some(m.rig) != rj);
            if let Some(rj) = rj {
                let slot = match inherited {
                    Some(sl) if !matches.iter().any(|m| m.slot == sl) => sl,
                    _ => "manual".to_string(),
                };
                matches.push(BoneMatch {
                    slot,
                    bvh: bj,
                    rig: rj,
                    bvh_name: bvh.joints[bj].name.clone(),
                    rig_name: p.bones[rj].name.clone(),
                    confidence: 1.0,
                    via: Via::Manual.label(),
                    length_ratio: None,
                    note: "manual".into(),
                    tip: Tip::Extrap(bj),
                });
            }
        }
    }

    // tips: next matched slot in the chain, else the default tip
    let reach = bvh.reach();
    let by_slot: BTreeMap<String, usize> = matches.iter().map(|m| (m.slot.clone(), m.bvh)).collect();
    let nspine = matches.iter().filter(|m| m.slot.starts_with("spine.")).count();
    for m in matches.iter_mut() {
        let next: Vec<String> = match m.slot.split_once('.') {
            _ if m.slot == "hips" => (0..nspine).map(|i| format!("spine.{i}")).chain(["neck".to_string(), "head".to_string()]).collect(),
            Some(("spine", i)) => {
                let i: usize = i.parse().unwrap_or(0);
                ((i + 1)..nspine).map(|k| format!("spine.{k}")).chain(["neck".to_string(), "head".to_string()]).collect()
            }
            _ if m.slot == "neck" => vec!["head".into()],
            Some(("clavicle", s)) => vec![format!("upperarm.{s}")],
            Some(("upperarm", s)) => vec![format!("forearm.{s}")],
            Some(("forearm", s)) => vec![format!("hand.{s}")],
            Some(("thigh", s)) => vec![format!("shin.{s}")],
            Some(("shin", s)) => vec![format!("foot.{s}")],
            Some(("foot", s)) => vec![format!("toe.{s}")],
            _ => vec![],
        };
        let mut tip = None;
        for k in next {
            if let Some(&j) = by_slot.get(&k) {
                if j != m.bvh && bvh.joints.len() > j {
                    let mut c = bvh.joints[j].parent;
                    let mut desc = false;
                    while let Some(x) = c {
                        if x == m.bvh {
                            desc = true;
                            break;
                        }
                        c = bvh.joints[x].parent;
                    }
                    if desc {
                        tip = Some(Tip::Joint(j));
                        break;
                    }
                }
            }
        }
        m.tip = tip.unwrap_or_else(|| bvh.default_tip_with(m.bvh, &reach));
    }

    // length ratios and sanity penalty
    let rest = bvh.pose(None);
    for m in matches.iter_mut() {
        if let Some(t) = bvh.tip_point(&rest, m.tip) {
            let bl = dist3(rest.pos[m.bvh], t);
            let rl = p.bones[m.rig].length as f64;
            if bl > 1e-9 && rl > 1e-9 {
                m.length_ratio = Some(rl / bl);
            }
        }
    }
    let mut limb: Vec<f64> = matches.iter().filter(|m| m.via != "manual" && ["upperarm", "forearm", "thigh", "shin"].iter().any(|r| m.slot.starts_with(r))).filter_map(|m| m.length_ratio).collect();
    if let Some(med) = median(&mut limb) {
        for m in matches.iter_mut() {
            if m.via == "manual" || !["upperarm", "forearm", "thigh", "shin"].iter().any(|r| m.slot.starts_with(r)) {
                continue;
            }
            if let Some(r) = m.length_ratio {
                let dev = (r / med).max(med / r);
                if dev > 3.0 {
                    m.confidence *= 0.5;
                    m.note = format!("limb length ratio is {dev:.1}x off the median");
                } else if dev > 2.0 {
                    m.confidence *= 0.7;
                    m.note = format!("limb length ratio is {dev:.1}x off the median");
                }
            }
        }
    }

    let mb: BTreeSet<usize> = matches.iter().map(|m| m.bvh).collect();
    let mr: BTreeSet<usize> = matches.iter().map(|m| m.rig).collect();
    let unmatched_bvh = (0..bvh.joints.len()).filter(|i| !mb.contains(i)).map(|i| bvh.joints[i].name.clone()).collect();
    let unmatched_rig = (0..p.bones.len()).filter(|i| !mr.contains(i)).map(|i| p.bones[i].name.clone()).collect();
    if !matches.iter().any(|m| m.slot == "hips") {
        warnings.push("no hips match: root motion will not be transferred".into());
    }
    for side in ["l", "r"] {
        for r in ["upperarm", "thigh"] {
            let k = format!("{r}.{side}");
            if !matches.iter().any(|m| m.slot == k) {
                warnings.push(format!("no match for {k}"));
            }
        }
    }
    if matches.is_empty() {
        warnings.push("no bones matched: supply a --map file".into());
    }
    Ok(MatchResult { matches, unmatched_bvh, unmatched_rig, warnings })
}

/// Plain-text match table.
pub fn format_table(r: &MatchResult) -> String {
    let mut s = String::new();
    s.push_str(&format!("{:<12} {:<26} {:<22} {:>5}  {:<9} {:>7}  {}\n", "slot", "bvh joint", "rig bone", "conf", "via", "len x", "note"));
    for m in &r.matches {
        s.push_str(&format!(
            "{:<12} {:<26} {:<22} {:>5.2}  {:<9} {:>7}  {}\n",
            m.slot,
            m.bvh_name,
            m.rig_name,
            m.confidence,
            m.via,
            m.length_ratio.map_or("-".to_string(), |v| format!("{v:.2}")),
            m.note
        ));
    }
    if !r.unmatched_bvh.is_empty() {
        s.push_str(&format!("unmatched BVH joints: {}\n", r.unmatched_bvh.join(", ")));
    }
    if !r.unmatched_rig.is_empty() {
        s.push_str(&format!("unmatched rig bones: {}\n", r.unmatched_rig.join(", ")));
    }
    for w in &r.warnings {
        s.push_str(&format!("warning: {w}\n"));
    }
    s
}
