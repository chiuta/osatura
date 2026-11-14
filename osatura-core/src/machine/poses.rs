// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `machine` (see the module docs in `mod.rs`).
use super::*;

// ---------------------------------------------------------------------------------------------
// Poses
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pose {
    /// target, prop, value, coverage weight (0..1)
    pub(super) items: Vec<(String, Prop, f32, f32)>,
    pub(super) ix: HashMap<(String, Prop), usize>,
}

impl Pose {
    pub(super) fn add(&mut self, target: &str, prop: Prop, v: f32, w: f32) {
        match self.ix.get(&(target.to_string(), prop)) {
            Some(&i) => {
                self.items[i].2 = v;
                self.items[i].3 = w;
            }
            None => {
                self.ix.insert((target.to_string(), prop), self.items.len());
                self.items.push((target.to_string(), prop, v, w));
            }
        }
    }

    /// Value and coverage of one entry.
    pub fn get(&self, target: &str, prop: Prop) -> Option<(f32, f32)> {
        self.ix.get(&(target.to_string(), prop)).map(|&i| (self.items[i].2, self.items[i].3))
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Weighted mix; weights are expected to sum to 1.
    pub(super) fn mix(parts: &[(&Pose, f32)]) -> Pose {
        let mut out = Pose::default();
        let mut keys: Vec<(String, Prop)> = Vec::new();
        let mut seen: HashSet<(String, Prop)> = HashSet::new();
        for (p, w) in parts {
            if *w <= 0.0 {
                continue;
            }
            for (t, pr, ..) in &p.items {
                if seen.insert((t.clone(), *pr)) {
                    keys.push((t.clone(), *pr));
                }
            }
        }
        for (t, pr) in keys {
            let (mut num, mut cov) = (0.0f32, 0.0f32);
            for (p, w) in parts {
                if *w <= 0.0 {
                    continue;
                }
                if let Some((v, c)) = p.get(&t, pr) {
                    num += w * c * v;
                    cov += w * c;
                }
            }
            if cov > 1e-9 {
                out.add(&t, pr, num / cov, cov.min(1.0));
            }
        }
        out
    }

    pub(super) fn channels(&self) -> Vec<Channel> {
        self.items
            .iter()
            .map(|(t, p, v, w)| {
                let mut c = Channel::keys(t, *p, Track::with_keys(&[(0.0, *v, Ease::Linear)]));
                c.weight = *w;
                c
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------------------------

/// Weights of the sorted `thresholds` for input value `v`: the two neighbouring children share
/// the weight linearly; outside the range the end child has weight 1.
pub fn blend_weights(thresholds: &[f32], v: f32) -> Vec<f32> {
    let n = thresholds.len();
    let mut w = vec![0.0; n];
    if n == 0 {
        return w;
    }
    if v <= thresholds[0] {
        w[0] = 1.0;
    } else if v >= thresholds[n - 1] {
        w[n - 1] = 1.0;
    } else {
        for i in 0..n - 1 {
            if v >= thresholds[i] && v <= thresholds[i + 1] {
                let t = (v - thresholds[i]) / (thresholds[i + 1] - thresholds[i]);
                w[i] = 1.0 - t;
                w[i + 1] = t;
                break;
            }
        }
    }
    w
}

pub(super) fn source_leaves(p: &Project, src: &StateSource, mask: &[String]) -> Result<Vec<Leaf>, String> {
    match src {
        StateSource::Blend { children, .. } => children.iter().map(|c| resolve_leaf(p, &c.source, mask)).collect(),
        s => Ok(vec![resolve_leaf(p, s, mask)?]),
    }
}

/// Structural validation of every machine in `p.machines` (names, references, reachability,
/// transition cycles, layer disjointness). Property targets are checked by [`trial_errors`].
pub fn validate(p: &Project) -> Vec<String> {
    let mut errs = Vec::new();
    let mut names = HashSet::new();
    for m in &p.machines {
        if m.name.trim().is_empty() {
            errs.push("machine needs a name".into());
        } else if !names.insert(m.name.clone()) {
            errs.push(format!("duplicate machine name '{}'", m.name));
        }
        errs.extend(validate_machine(p, m));
    }
    errs.extend(crate::binding::validate(p));
    errs
}

pub fn validate_machine(p: &Project, m: &Machine) -> Vec<String> {
    let mut errs = Vec::new();
    let mn = &m.name;
    let mut inputs: HashMap<&str, InputKind> = HashMap::new();
    for i in &m.inputs {
        if i.name.trim().is_empty() {
            errs.push(format!("machine '{mn}': input needs a name"));
        } else if inputs.insert(&i.name, i.kind).is_some() {
            errs.push(format!("machine '{mn}': duplicate input '{}'", i.name));
        }
        if !i.default.is_finite() {
            errs.push(format!("machine '{mn}': input '{}' default must be finite", i.name));
        }
    }
    if m.layers.is_empty() {
        errs.push(format!("machine '{mn}': needs at least one layer"));
    }
    let mut lnames = HashSet::new();
    let mut layer_targets: Vec<(String, HashSet<(String, Prop)>)> = Vec::new();
    for l in &m.layers {
        let ln = format!("machine '{mn}' layer '{}'", l.name);
        if l.name.trim().is_empty() {
            errs.push(format!("machine '{mn}': layer needs a name"));
        } else if !lnames.insert(l.name.clone()) {
            errs.push(format!("{ln}: duplicate layer name"));
        }
        if l.mask.iter().any(|g| g.is_empty()) {
            errs.push(format!("{ln}: empty mask pattern"));
        }
        if l.states.is_empty() {
            errs.push(format!("{ln}: needs at least one state"));
            continue;
        }
        let mut states: HashMap<&str, usize> = HashMap::new();
        let mut targets: HashSet<(String, Prop)> = HashSet::new();
        for (si, s) in l.states.iter().enumerate() {
            let sn = format!("{ln} state '{}'", s.name);
            if s.name.trim().is_empty() || s.name == "*" {
                errs.push(format!("{ln}: state needs a name other than '*'"));
            } else if states.insert(&s.name, si).is_some() {
                errs.push(format!("{ln}: duplicate state name '{}'", s.name));
            }
            if !(s.speed.is_finite() && s.speed > 0.0) {
                errs.push(format!("{sn}: speed must be a finite number > 0"));
            }
            if !(s.blend_in.is_finite() && s.blend_in >= 0.0) {
                errs.push(format!("{sn}: blend_in must be a finite number >= 0"));
            }
            if let StateSource::Blend { input, children } = &s.source {
                match inputs.get(input.as_str()) {
                    None => errs.push(format!("{sn}: blend input '{input}' does not exist")),
                    Some(InputKind::Number) => {}
                    Some(_) => errs.push(format!("{sn}: blend input '{input}' must be a number input")),
                }
                if children.len() < 2 {
                    errs.push(format!("{sn}: a blend tree needs at least two children"));
                }
                if !children.iter().all(|c| c.threshold.is_finite()) || !children.windows(2).all(|w| w[0].threshold < w[1].threshold) {
                    errs.push(format!("{sn}: blend thresholds must be finite and strictly increasing"));
                }
            }
            match source_leaves(p, &s.source, &l.mask) {
                Ok(leaves) => {
                    for lf in leaves {
                        for (t, pr, ..) in lf.tracks {
                            targets.insert((t, pr));
                        }
                    }
                }
                Err(e) => errs.push(format!("{sn}: {e}")),
            }
        }
        if !states.contains_key(l.entry.as_str()) {
            errs.push(format!("{ln}: entry state '{}' does not exist", l.entry));
        }
        // transitions
        let mut edges_ok = true;
        for (ti, t) in l.transitions.iter().enumerate() {
            let tn = format!("{ln} transition {} ({} -> {})", ti + 1, t.from, t.to);
            if t.from != "*" && !states.contains_key(t.from.as_str()) {
                errs.push(format!("{tn}: unknown source state '{}'", t.from));
                edges_ok = false;
            }
            if !states.contains_key(t.to.as_str()) {
                errs.push(format!("{tn}: transition to missing state '{}'", t.to));
                edges_ok = false;
            }
            for c in &t.conditions {
                if !inputs.contains_key(c.input.as_str()) {
                    errs.push(format!("{tn}: condition on unknown input '{}'", c.input));
                }
                if !c.value.is_finite() {
                    errs.push(format!("{tn}: condition value must be finite"));
                }
            }
            if let Some(x) = t.exit_time {
                if !(x.is_finite() && (0.0..=1.0).contains(&x)) {
                    errs.push(format!("{tn}: exit_time must be within 0..1"));
                }
            }
            if let Some(d) = t.duration {
                if !(d.is_finite() && d >= 0.0) {
                    errs.push(format!("{tn}: duration must be a finite number >= 0"));
                }
            }
        }
        if edges_ok && states.contains_key(l.entry.as_str()) {
            // reachability
            let n = l.states.len();
            let mut seen = vec![false; n];
            let mut stack = vec![states[l.entry.as_str()]];
            seen[stack[0]] = true;
            while let Some(s) = stack.pop() {
                for t in &l.transitions {
                    let from_ok = t.from == "*" || states[t.from.as_str()] == s;
                    let to = states[t.to.as_str()];
                    if from_ok && !seen[to] {
                        seen[to] = true;
                        stack.push(to);
                    }
                }
            }
            for (i, s) in l.states.iter().enumerate() {
                if !seen[i] {
                    errs.push(format!("{ln}: state '{}' is unreachable", s.name));
                }
            }
            // cycles of unconditional transitions (they would fire on every step)
            let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
            for t in &l.transitions {
                if t.conditions.is_empty() && t.exit_time.is_none_or(|x| x <= 0.0) {
                    let to = states[t.to.as_str()];
                    if t.from == "*" {
                        for (f, a) in adj.iter_mut().enumerate() {
                            if f != to {
                                a.push(to);
                            }
                        }
                    } else {
                        adj[states[t.from.as_str()]].push(to);
                    }
                }
            }
            if let Some(s) = find_cycle(&adj) {
                errs.push(format!(
                    "{ln}: unconditional transitions form a cycle through state '{}' (add a condition or an exit_time; zero-duration cycles would loop forever)",
                    l.states[s].name
                ));
            }
        }
        layer_targets.push((l.name.clone(), targets));
    }
    for i in 0..layer_targets.len() {
        for j in i + 1..layer_targets.len() {
            if let Some((t, pr)) = layer_targets[i].1.intersection(&layer_targets[j].1).min_by(|a, b| a.0.cmp(&b.0)) {
                errs.push(format!(
                    "machine '{mn}': layers '{}' and '{}' both drive {t}.{pr:?}; layers need disjoint channel groups (use a mask)",
                    layer_targets[i].0, layer_targets[j].0
                ));
            }
        }
    }
    errs
}

pub(super) fn find_cycle(adj: &[Vec<usize>]) -> Option<usize> {
    // 0 = new, 1 = on stack, 2 = done
    let mut color = vec![0u8; adj.len()];
    fn dfs(u: usize, adj: &[Vec<usize>], color: &mut [u8]) -> Option<usize> {
        color[u] = 1;
        for &v in &adj[u] {
            if color[v] == 1 {
                return Some(v);
            }
            if color[v] == 0 {
                if let Some(c) = dfs(v, adj, color) {
                    return Some(c);
                }
            }
        }
        color[u] = 2;
        None
    }
    for s in 0..adj.len() {
        if color[s] == 0 {
            if let Some(c) = dfs(s, adj, &mut color) {
                return Some(c);
            }
        }
    }
    None
}

/// Compile the project once with a constant channel for every property the machines and bindings
/// can write, so unknown bones / layers / parameters are reported with the engine's own messages.
/// Returns nothing when there are no machines or bindings. Called by `Engine` after its own checks.
pub fn trial_errors(p: &Project, asset_dir: &std::path::Path) -> Vec<String> {
    if p.machines.is_empty() && p.bindings.is_empty() {
        return vec![];
    }
    let mut probe = p.clone();
    probe.machines.clear();
    probe.bindings.clear();
    let mut seen: HashSet<(String, Prop)> = HashSet::new();
    let mut add = |probe: &mut Project, t: &str, pr: Prop| {
        if seen.insert((t.to_string(), pr)) {
            probe.channels.push(Channel::keys(t, pr, Track::with_keys(&[(0.0, 0.0, Ease::Linear)])));
        }
    };
    for m in &p.machines {
        for l in &m.layers {
            for s in &l.states {
                if let Ok(leaves) = source_leaves(p, &s.source, &l.mask) {
                    for lf in leaves {
                        for (t, pr, ..) in lf.tracks {
                            add(&mut probe, &t, pr);
                        }
                    }
                }
            }
        }
    }
    for b in &p.bindings {
        if let crate::binding::Target::Prop { target, prop } = &b.target {
            add(&mut probe, target, *prop);
        }
    }
    match Engine::with_asset_dir(probe, asset_dir) {
        Ok(_) => vec![],
        Err(e) => e.into_iter().map(|m| format!("machine/binding targets: {m}")).collect(),
    }
}
