//! E13 trajectory instrument (docs/anima-e13-protocol.md, frozen).
//! ANALYSIS-ONLY: imports anima_telemetry + std only (no anima_core);
//! opens run dirs read-only; never constructs an Environment.
//!
//! Checkpoints T0..T60 over the committed E12 run artifacts:
//!   T0  = rounds 51-60  (last 10 SEQ, [305000, 365000))
//!   Tk  = 10 rounds ending at REV round k, [5000+6000(50+k), 5000+6000(60+k))
//!         for k = 10..60 (T10 = [365000, 425000), ..., T60 = [665000, 725000))
//!
//! Per checkpoint (frozen E12 formulas):
//!   A-B/B-C/A-C pairwise-mean cosines (raw + L1), B-independence
//!   (0.60 all four), B-alignment argmin, windowed selectivity
//!   median, candidate-permanence events, in-window failures,
//!   internal rate mean/max from snapshots.
//! Also reports the in-round position of the first REV-B (round 61)
//! and the last REV-B (round 120) per seed.
//!
//! Usage: e13_trajectory <run-dir>

use std::collections::BTreeMap;

const N_IN: u32 = 24;
const T0_LO: u64 = 305_000;
const T0_HI: u64 = 365_000;
const BASE: u64 = 5_000;

fn main() {
    let dir = std::env::args().nth(1).expect("run dir");
    let tdir = std::path::Path::new(&dir).join("telemetry");
    let reader = anima_telemetry::TelemetryReader::open(&tdir).unwrap();
    let idx = reader.chunk_index();

    let mut stims: Vec<(String, String, u64, u64)> = Vec::new();
    let mut spikes: Vec<(u64, u32)> = Vec::new();
    let mut failures: Vec<u64> = Vec::new();
    let mut permanence: Vec<u64> = Vec::new();
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            match row.kind {
                3 => spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                5 => {
                    if let Ok(env) = row.envelope("e") {
                        if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } = env.payload {
                            stims.push((pattern_id, stage, env.t, env.t + 500));
                        }
                    }
                }
                6 => {
                    if let Ok(e) = row.envelope("v2") {
                        if let anima_telemetry::events::Payload::SynapseCreated { reason, .. } = e.payload {
                            if reason.trigger == "candidate-permanence" {
                                permanence.push(e.t);
                            }
                        }
                    }
                }
                17 => {
                    if let Ok(e) = row.envelope("v2") {
                        if let anima_telemetry::events::Payload::Failure { .. } = e.payload {
                            failures.push(e.t);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let snaps = anima_telemetry::recorder::read_snapshots(
        std::path::Path::new(&dir).join("snapshots.bin.zst").as_path(),
    )
    .expect("snapshots");

    // REV-B in-round positions (seed-dependent shuffle, E12 record).
    let mut b_s1: Vec<(u64, u32)> = stims
        .iter()
        .filter(|p| p.0 == "B" && p.1 == "S1")
        .map(|p| (p.2, ((p.2 - BASE) / 2000 % 3) as u32))
        .collect();
    b_s1.sort();
    let pos = |round: u64| {
        b_s1
            .iter()
            .find(|(t, _)| (t - BASE) / 6000 == round - 1)
            .map(|(_, p)| *p)
            .unwrap_or(99)
    };
    println!("first-REV-B round 61 position: {}", pos(61));
    println!("last-REV-B  round 120 position: {}", pos(120));

    // Checkpoint windows.
    let mut checkpoints: Vec<(String, u64, u64)> = vec![("T0".to_string(), T0_LO, T0_HI)];
    for k in (10..=60).step_by(10) {
        let lo = BASE + 6000 * (50 + k);
        let hi = BASE + 6000 * (60 + k);
        checkpoints.push((format!("T{k}"), lo, hi));
    }
    for (label, lo, hi) in checkpoints {
        let win: Vec<&(String, String, u64, u64)> = stims
            .iter()
            .filter(|p| p.1 == "S1" && p.2 >= lo && p.2 < hi)
            .collect();
        let per_pat = |pat: &str| win.iter().filter(|p| p.0 == pat).count();
        println!(
            "===== {label} ([{lo}, {hi})) presentations={} A={} B={} C={}",
            win.len(),
            per_pat("A"),
            per_pat("B"),
            per_pat("C")
        );
        if win.len() != 30 {
            println!("(window anomaly - not 30 presentations)");
            continue;
        }
        let vecs: Vec<(String, BTreeMap<u32, f64>)> = win
            .iter()
            .map(|p| {
                let mut v: BTreeMap<u32, f64> = BTreeMap::new();
                for &(t, n) in &spikes {
                    if t >= p.2 && t < p.3 && n >= N_IN {
                        *v.entry(n).or_insert(0.0) += 1.0;
                    }
                }
                (p.0.clone(), v)
            })
            .collect();
        let mut pats: Vec<String> = vecs.iter().map(|(p, _)| p.clone()).collect();
        pats.sort();
        pats.dedup();
        let pair_of = |a: &str, b: &str| -> (f64, f64) {
            let mut raw = Vec::new();
            let mut norm = Vec::new();
            for va in vecs.iter().filter(|(p, _)| p == a) {
                for vb in vecs.iter().filter(|(p, _)| p == b) {
                    raw.push(cos(&va.1, &vb.1, false));
                    norm.push(cos(&va.1, &vb.1, true));
                }
            }
            let m = |c: &[f64]| c.iter().sum::<f64>() / c.len().max(1) as f64;
            (m(&raw), m(&norm))
        };
        let (ab, abn) = pair_of("A", "B");
        let (bc, bcn) = pair_of("B", "C");
        let (ac, acn) = pair_of("A", "C");
        println!("A-B: raw {ab:.3}, L1 {abn:.3}");
        println!("B-C: raw {bc:.3}, L1 {bcn:.3}");
        println!("A-C: raw {ac:.3}, L1 {acn:.3}");
        let indep = ab < 0.60 && bc < 0.60 && abn < 0.60 && bcn < 0.60;
        println!("B-independence: {indep}");
        println!("B-alignment: {}", if ab <= bc { "A" } else { "C" });
        // windowed selectivity: per-neuron (best - 2nd)/best over patterns
        let mut per_neuron: BTreeMap<u32, Vec<f64>> = BTreeMap::new();
        for (pat, v) in &vecs {
            for (n, &c) in v {
                let e = per_neuron.entry(*n).or_insert_with(|| vec![0.0; pats.len()]);
                let pi = pats.iter().position(|p| p == pat).unwrap();
                e[pi] += c;
            }
        }
        let mut sels = Vec::new();
        for rates in per_neuron.values() {
            let mut s = rates.clone();
            s.sort_by(|a, b| b.partial_cmp(a).unwrap());
            let best = s[0];
            let second = s.get(1).copied().unwrap_or(0.0);
            if best > 1e-9 {
                sels.push((best - second) / best);
            }
        }
        sels.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let med = sels.get(sels.len() / 2).copied().unwrap_or(0.0);
        println!("selectivity median: {med:.3} (n={})", sels.len());
        let perm = permanence.iter().filter(|&&t| t >= lo && t < hi).count();
        println!("permanence events in window: {perm}");
        let wfail = failures.iter().filter(|&&t| t >= lo && t < hi).count();
        let rates: Vec<f32> = snaps
            .iter()
            .filter(|s| s.tick >= lo && s.tick < hi)
            .flat_map(|s| s.neurons.iter().filter(|n| n.class == "internal").filter_map(|n| n.rate_hz))
            .collect();
        let rmean = rates.iter().sum::<f32>() / rates.len().max(1) as f32;
        let rmax = rates.iter().cloned().fold(0.0f32, f32::max);
        println!("failures in window: {wfail}; internal rates mean={rmean:.1} max={rmax:.1} Hz");
    }
}

fn cos(a: &BTreeMap<u32, f64>, b: &BTreeMap<u32, f64>, norm_l1: bool) -> f64 {
    let keys: std::collections::BTreeSet<u32> = a.keys().chain(b.keys()).copied().collect();
    let mut dot = 0.0f64;
    let mut na = 0.0f64;
    let mut nb = 0.0f64;
    for k in keys {
        let x = a.get(&k).copied().unwrap_or(0.0);
        let y = b.get(&k).copied().unwrap_or(0.0);
        let (x, y) = if norm_l1 {
            let sx = a.values().sum::<f64>();
            let sy = b.values().sum::<f64>();
            (
                if sx > 0.0 { x / sx } else { 0.0 },
                if sy > 0.0 { y / sy } else { 0.0 },
            )
        } else {
            (x, y)
        };
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na.sqrt() * nb.sqrt())
    }
}