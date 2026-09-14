//! ANIMA v3 analysis instrument (measurement only, no mechanism access;
//! docs/anima-v3-protocol.md §4/§8/§9):
//! - established structural changes (same definition as v2_analysis)
//! - RF snapshot at t = 715,000 ms: per-neuron channel-weight entropy,
//!   specialized classification over the REGISTERED v3 category sets
//!   A {0-7}, B {4-11}, C {8-15}, signatures {A}/{B}/{C}/{A,B}/{B,C}/BROAD,
//!   exclusive-evidence usage, per-channel participation, top-channel map
//! - stage mean internal rates: late-S1 / S2 / S3 (snapshot-based,
//!   same readout as read_rates)
//! - D-condition cosines: mean S1 activity vector of A/B/C vs mean S2
//!   activity vector of D (spike-count vectors from chunk telemetry,
//!   same row API as cross_cosine)
//! Usage: v3_analysis <run-dir>

use std::collections::BTreeMap;

const N_IN: usize = 24;
const CATS: [(&str, std::ops::Range<usize>); 3] = [
    ("A", 0..8),
    ("B", 4..12),
    ("C", 8..16),
];

fn main() {
    let run_dir = std::env::args().nth(1).expect("run dir");
    let tdir = std::path::Path::new(&run_dir).join("telemetry");

    // ---- established structural changes (v2 definition, verbatim) ----
    let reader = anima_telemetry::TelemetryReader::open(&tdir).unwrap();
    let idx = reader.chunk_index();
    let mut created: Vec<(u64, u64)> = Vec::new();
    let mut pruned: Vec<(u64, u64)> = Vec::new();
    let mut pres: Vec<(String, String, u64, u64)> = Vec::new(); // (pattern, stage, start, end)
    let mut spikes: Vec<(u64, u32)> = Vec::new();
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            match row.kind {
                5 => {
                    if let Ok(env) = row.envelope("e") {
                        if let anima_telemetry::events::Payload::StimulusPresented {
                            pattern_id,
                            stage,
                        } = env.payload
                        {
                            pres.push((pattern_id, stage, env.t, env.t + 500));
                        }
                    }
                }
                3 => spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                _ => {
                    if let Ok(e) = row.envelope("v2") {
                        use anima_telemetry::events::Payload;
                        match &e.payload {
                            Payload::SynapseCreated { syn, reason, .. } => {
                                if reason.trigger == "candidate-permanence" {
                                    created.push((e.t, syn.0 as u64));
                                }
                            }
                            Payload::SynapsePruned { syn, reason } => {
                                if reason.trigger == "competitive-prune"
                                    || reason.trigger == "budget-eviction"
                                {
                                    pruned.push((e.t, syn.0 as u64));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    let established: Vec<(u64, u64)> = created
        .iter()
        .filter(|&&(t, s)| {
            !pruned.iter().any(|&(pt, ps)| ps == s && pt > t && pt <= t + 10_000)
        })
        .copied()
        .collect();
    println!("== structural changes ==");
    println!("candidate-permanence events: {}", created.len());
    println!("established (alive at t+10s): {}", established.len());

    // ---- RF snapshot (v3 sets) ----
    let snaps = anima_telemetry::recorder::read_snapshots(
        std::path::Path::new(&run_dir).join("snapshots.bin.zst").as_path(),
    )
    .expect("snapshots");
    let snap = snaps
        .iter()
        .min_by_key(|s| s.tick.abs_diff(715_000))
        .expect("snapshots non-empty");
    println!("== RF snapshot (v3 sets) ==");
    println!("snapshot tick = {} (target 715000)", snap.tick);

    let internal_ids: Vec<u32> = snap
        .neurons
        .iter()
        .filter(|n| n.class == "internal")
        .map(|n| n.id)
        .collect();
    let mut per_neuron: BTreeMap<u32, Vec<f64>> = BTreeMap::new();
    for id in &internal_ids {
        per_neuron.insert(*id, vec![0.0; N_IN]);
    }
    for s in &snap.synapses {
        if (s.pre as usize) < N_IN {
            if let Some(v) = per_neuron.get_mut(&s.post) {
                v[s.pre as usize] = s.w.unwrap_or(0.0) as f64;
            }
        }
    }

    let mut h_vals = Vec::new();
    let mut specialized = 0usize;
    let mut signatures: BTreeMap<String, usize> = BTreeMap::new();
    let mut per_cat_presence = vec![0usize; 3]; // A B C
    let mut exclusive_evidence = 0usize;
    let mut top_count = vec![0usize; N_IN]; // per-channel top-pick histogram
    let mut mean_norm_w = vec![0.0f64; N_IN]; // mean over neurons of w_c / sum
    let mut top_map: BTreeMap<u32, u32> = BTreeMap::new();
    for (id, v) in &per_neuron {
        let sum: f64 = v.iter().sum();
        let mut h = 0.0f64;
        if sum > 0.0 {
            for &w in v.iter() {
                let p = w / sum;
                if p > 0.0 {
                    h -= p * p.log2();
                }
            }
        }
        h_vals.push(h);
        let max = v.iter().cloned().fold(0.0f64, f64::max);
        // Dominant set D = {c : w_c > 0.5 * max} (v2 definition).
        let d: Vec<usize> = v
            .iter()
            .enumerate()
            .filter(|&(_, &w)| w > 0.5 * max)
            .map(|(i, _)| i)
            .collect();
        if let Some(&t) = d.first() {
            top_map.insert(*id, t as u32);
            top_count[t] += 1;
        }
        if sum > 0.0 {
            for (c, &w) in v.iter().enumerate() {
                mean_norm_w[c] += w / sum;
            }
        }
        // Signature: categories whose set contains D (joint-type histogram;
        // per-category presence tracked separately).
        let mut sig: Vec<&str> = Vec::new();
        for (name, r) in CATS {
            if !d.is_empty() && d.iter().all(|&c| r.contains(&c)) {
                sig.push(name);
            }
        }
        if h <= 1.5 && !sig.is_empty() {
            specialized += 1;
            for name in &sig {
                let ci = CATS.iter().position(|(n, _)| n == name).unwrap();
                per_cat_presence[ci] += 1;
            }
            if d.iter().any(|&c| c < 4 || (12..16).contains(&c)) {
                exclusive_evidence += 1;
            }
            let joint = if sig.len() == 1 {
                sig[0].to_string()
            } else {
                format!("{}", sig.join("+"))
            };
            *signatures.entry(joint).or_insert(0) += 1;
        } else {
            *signatures.entry("BROAD".to_string()).or_insert(0) += 1;
        }
    }
    let n_neurons = internal_ids.len();
    let mean_h: f64 = h_vals.iter().sum::<f64>() / h_vals.len().max(1) as f64;
    println!("internal neurons: {n_neurons}");
    println!("mean H: {mean_h:.3}");
    println!("specialized: {specialized} / {n_neurons}");
    println!("specialized fraction: {:.3}", specialized as f64 / n_neurons.max(1) as f64);
    println!(
        "per-category presence [A,B,C]: {:?}",
        per_cat_presence
    );
    println!("signatures: {:?}", signatures);
    println!(
        "exclusive-evidence users among specialized: {exclusive_evidence} ({:.3})",
        exclusive_evidence as f64 / specialized.max(1) as f64
    );
    // Per-channel participation: top-pick fraction and mean normalized weight.
    let mut parts = Vec::new();
    for c in 0..N_IN {
        parts.push(format!(
            "{c}:{:.2}/{}",
            top_count[c] as f64 / n_neurons.max(1) as f64,
            (mean_norm_w[c] / n_neurons.max(1) as f64 * 100.0).round() / 100.0
        ));
    }
    println!("per-channel participation (top-frac / mean p_c): {}", parts.join(" "));
    println!("top-channel map: {:?}", top_map);

    // ---- stage mean internal rates (snapshot-based, read_rates readout) ----
    let late_start = {
        let mut s1: Vec<u64> = pres
            .iter()
            .filter(|p| p.1 == "S1")
            .map(|p| p.2)
            .collect();
        s1.sort_unstable();
        s1[s1.len() / 2]
    };
    println!("== stage mean internal rates ==");
    for (name, lo, hi) in [
        ("late-S1", late_start, 725_000u64),
        ("S2", 725_000, 785_000),
        ("S3", 785_000, 875_000),
    ] {
        let mut rates: Vec<f32> = Vec::new();
        for s in &snaps {
            if s.tick >= lo && s.tick < hi {
                rates.extend(
                    s.neurons
                        .iter()
                        .filter(|n| n.class == "internal")
                        .filter_map(|n| n.rate_hz),
                );
            }
        }
        let mean = rates.iter().sum::<f32>() / rates.len().max(1) as f32;
        let max = rates.iter().cloned().fold(0.0f32, f32::max);
        println!("{name}: mean={mean:.1} max={max:.1} Hz (n={})", rates.len() / 40);
    }

    // ---- D-condition cosines (mean S1 A/B/C vectors vs mean S2 D) ----
    pres.sort_by_key(|p| p.2);
    let s1: Vec<_> = pres.iter().filter(|p| p.1 == "S1").collect();
    let s2d: Vec<_> = pres.iter().filter(|p| p.1 == "S2" && p.0 == "D").collect();
    let mean_vec = |which: &[&(String, String, u64, u64)], pattern: &str| -> BTreeMap<u32, f64> {
        let mut m: BTreeMap<u32, f64> = BTreeMap::new();
        let mut n = 0usize;
        for p in which {
            if p.0 != pattern {
                continue;
            }
            if p.2 < late_start {
                continue;
            }
            n += 1;
            for &(t, id) in &spikes {
                if t >= p.2 && t < p.3 && id >= N_IN as u32 {
                    *m.entry(id).or_insert(0.0) += 1.0;
                }
            }
        }
        if n > 0 {
            for v in m.values_mut() {
                *v /= n as f64;
            }
        }
        m
    };
    let s1v: Vec<&(String, String, u64, u64)> = s1.iter().map(|p| *p).collect();
    let dvec = mean_vec(&s2d, "D");
    println!("== D-condition (S2) vs S1 categories ==");
    for cat in ["A", "B", "C"] {
        let cvec = mean_vec(&s1v, cat);
        let c = cosine_f64(&cvec, &dvec);
        match c {
            Some(v) => println!("cosine({cat}-mean, D-mean): {v:.3}"),
            None => println!("cosine({cat}-mean, D-mean): (degenerate)"),
        }
    }
    let s2_rate: Vec<f32> = snaps
        .iter()
        .filter(|s| s.tick >= 725_000 && s.tick < 785_000)
        .flat_map(|s| s.neurons.iter().filter(|n| n.class == "internal").filter_map(|n| n.rate_hz))
        .collect();
    let s2_mean = s2_rate.iter().sum::<f32>() / s2_rate.len().max(1) as f32;
    let s1l_mean = {
        let r: Vec<f32> = snaps
            .iter()
            .filter(|s| s.tick >= late_start && s.tick < 725_000)
            .flat_map(|s| s.neurons.iter().filter(|n| n.class == "internal").filter_map(|n| n.rate_hz))
            .collect();
        r.iter().sum::<f32>() / r.len().max(1) as f32
    };
    println!("S2/late-S1 mean-rate ratio: {:.3} (S2 {s2_mean:.1} Hz vs late-S1 {s1l_mean:.1} Hz)", s2_mean / s1l_mean.max(1e-6));
}

fn cosine_f64(a: &BTreeMap<u32, f64>, b: &BTreeMap<u32, f64>) -> Option<f64> {
    let keys: std::collections::BTreeSet<u32> = a.keys().chain(b.keys()).copied().collect();
    let dot: f64 = keys
        .iter()
        .map(|k| a.get(k).copied().unwrap_or(0.0) * b.get(k).copied().unwrap_or(0.0))
        .sum();
    let na: f64 = a.values().map(|x| x * x).sum::<f64>().sqrt();
    let nb: f64 = b.values().map(|x| x * x).sum::<f64>().sqrt();
    if na == 0.0 || nb == 0.0 {
        return None;
    }
    Some(dot / (na * nb))
}