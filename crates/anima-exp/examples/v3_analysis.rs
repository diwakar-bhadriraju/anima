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
    let args: Vec<String> = std::env::args().collect();
    let run_dir = args.get(1).expect("run dir").clone();
    // E7 (docs/anima-e7-protocol.md §6): optional trailing window args
    // (late_s1_lo late_s1_hi s2_lo s2_hi s3_lo s3_hi snapshot_tick).
    // All-or-none; absent ⇒ byte-identical v2/v3 defaults. The D-section
    // prints a fixed "no S2" line when the run has no S2 presentations.
    let (late_lo, late_hi, s2_lo, s2_hi, s3_lo, s3_hi, snap_target) = parse_window_args(&args);
    let has_s2 = s2_lo != s2_hi;
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
        .min_by_key(|s| s.tick.abs_diff(snap_target))
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
    // E7 §6: explicit arg overrides the event-driven second half; the
    // sentinel keeps the v2/v3 default path byte-identical.
    let late_lo_eff = if late_lo == u64::MAX { late_start } else { late_lo };
    println!("== stage mean internal rates ==");
    for (name, lo, hi) in [
        ("late-S1", late_lo_eff, late_hi),
        ("S2", s2_lo, s2_hi),
        ("S3", s3_lo, s3_hi),
    ] {
        if name == "S2" && !has_s2 {
            println!("S2: (no S2 in this curriculum; E7 registered layout)");
            continue;
        }
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

    // ---- E6 engagement readout (docs/anima-e6-protocol.md §3.4/§9) ----
    // φ is mechanism-internal; this RE-COMPUTES it deterministically from
    // the recorded input spike stream with the frozen EMA (α=1/25, init
    // 0.02, floor 0.001, W=100, one-window lag) — a measurement print,
    // no metric redefinition.
    {
        const E6_ALPHA: f32 = 1.0 / 25.0;
        const E6_INIT: f32 = 0.02;
        const E6_MIN: f32 = 0.001;
        const W: u64 = 100;
        let last_t = spikes.iter().map(|s| s.0).max().unwrap_or(0);
        let n_windows = (last_t / W) as usize + 1;
        let mut cnt: Vec<Vec<u32>> = vec![vec![0; N_IN]; n_windows];
        for &(t, n) in &spikes {
            if (n as usize) < N_IN {
                cnt[(t / W) as usize][n as usize] += 1;
            }
        }
        let mut phi: Vec<f32> = vec![E6_INIT; N_IN];
        for w in 0..n_windows {
            for i in 0..N_IN {
                phi[i] = ((1.0 - E6_ALPHA) * phi[i]
                    + E6_ALPHA * (cnt[w][i] as f32 / W as f32))
                    .max(E6_MIN);
            }
        }
        println!("== E6 φ readout (frozen EMA recomputed from input stream) ==");
        println!("final φ per channel: {}", phi.iter().map(|p| format!("{p:.4}")).collect::<Vec<_>>().join(" "));
        let excl: Vec<f32> = (0..4).chain(12..16).map(|c| phi[c]).collect();
        let shared: Vec<f32> = (4..12).map(|c| phi[c]).collect();
        let mean = |v: &[f32]| v.iter().sum::<f32>() / v.len().max(1) as f32;
        println!("mean φ exclusive {{0-3,12-15}}: {:.4} Hz-equiv, shared {{4-11}}: {:.4} (ratio {:.2})",
            mean(&excl) * 1000.0, mean(&shared) * 1000.0, mean(&shared) / mean(&excl).max(1e-6));
    }

    // ---- D-condition cosines (mean S1 A/B/C vectors vs mean S2 D) ----
    pres.sort_by_key(|p| p.2);
    let s1: Vec<_> = pres.iter().filter(|p| p.1 == "S1").collect();
    let s2d: Vec<_> = pres.iter().filter(|p| p.1 == "S2" && p.0 == "D").collect();
    if s2d.is_empty() {
        println!("== D-condition: (no S2-D presentations; E7 has no novelty condition, protocol §4) ==");
    } else {
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
    }
    let s2_rate: Vec<f32> = if has_s2 {
        snaps
            .iter()
            .filter(|s| s.tick >= s2_lo && s.tick < s2_hi)
            .flat_map(|s| s.neurons.iter().filter(|n| n.class == "internal").filter_map(|n| n.rate_hz))
            .collect()
    } else {
        Vec::new()
    };
    let s2_mean = s2_rate.iter().sum::<f32>() / s2_rate.len().max(1) as f32;
    let s1l_mean = {
        let r: Vec<f32> = snaps
            .iter()
            .filter(|s| s.tick >= late_lo_eff && s.tick < late_hi)
            .flat_map(|s| s.neurons.iter().filter(|n| n.class == "internal").filter_map(|n| n.rate_hz))
            .collect();
        r.iter().sum::<f32>() / r.len().max(1) as f32
    };
    if !has_s2 {
        println!("S2/late-S1 ratio: (no S2 in this curriculum)");
    } else {
        println!("S2/late-S1 mean-rate ratio: {:.3} (S2 {s2_mean:.1} Hz vs late-S1 {s1l_mean:.1} Hz)", s2_mean / s1l_mean.max(1e-6));
    }
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
/// E7 §6: parse optional stage-window args. All-or-none; absent returns
/// the v2/v3 defaults with late_lo = u64::MAX sentinel (event-defined
/// second half) — the byte-identical default path.
fn parse_window_args(args: &[String]) -> (u64, u64, u64, u64, u64, u64, u64) {
    if args.len() <= 2 {
        return (u64::MAX, 725_000, 725_000, 785_000, 785_000, 875_000, 715_000);
    }
    let nums: Vec<u64> = args[2..]
        .iter()
        .map(|a| a.parse().expect("window args must be integers"))
        .collect();
    assert_eq!(
        nums.len(),
        7,
        "window args: 7 integers (late_s1_lo late_s1_hi s2_lo s2_hi s3_lo s3_hi snapshot_tick)"
    );
    (nums[0], nums[1], nums[2], nums[3], nums[4], nums[5], nums[6])
}

#[cfg(test)]
mod tests {
    use super::parse_window_args;

    #[test]
    fn window_args_defaults_byte_identical() {
        let (lo, hi, s2l, s2h, s3l, s3h, snap) = parse_window_args(&["v3_analysis".into(), "dir".into()]);
        assert_eq!((lo, hi, s2l, s2h, s3l, s3h, snap), (u64::MAX, 725_000, 725_000, 785_000, 785_000, 875_000, 715_000));
    }

    #[test]
    fn window_args_explicit_values() {
        let a = ["x".into(), "dir".into(), "245000".into(), "485000".into(), "0".into(), "0".into(), "485000".into(), "530000".into(), "485000".into()];
        let (lo, hi, s2l, s2h, s3l, s3h, snap) = parse_window_args(&a);
        assert_eq!((lo, hi, s2l, s2h, s3l, s3h, snap), (245_000, 485_000, 0, 0, 485_000, 530_000, 485_000));
        // s2_lo == s2_hi => the registered "no S2" condition.
        assert_eq!(s2l, s2h);
    }
}
