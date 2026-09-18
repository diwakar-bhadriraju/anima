//! E18 analysis instrument (docs/anima-e18-protocol.md, frozen).
//! ANALYSIS-ONLY: imports anima_telemetry + std only (no anima_core).
//!
//! Reports per window E/M/L/S2/S3:
//!   D = 1 - pairwise-mean cos(B-after-A, B-after-C) probe vectors
//!   (internal, n >= 24, 500 ms window; raw and L1 — identical
//!   expected); A-B / C-B alignment pair means; A/B/C epoch stats;
//!   permanent events; failures; snapshot max rate.
//! Schedule audit: antecedent bit sequence (A=0, C=1) per block,
//! lag-1 autocorrelation, per-40-window imbalance.
//! A-C sanity: pairwise mean cos(A epoch, C epoch) in S2.
//! Deterministic: pure function of the artifacts.
//!
//! Usage: e18_analysis <run-dir>

use std::collections::{BTreeMap, BTreeSet};

const N_IN: u32 = 24;
const TRIAL_MS: u64 = 2000;
const S1_TRIALS: u64 = 120;
const S2_TRIALS: u64 = 40;
const WINDOWS: [(&str, u64, u64); 6] = [
    ("E", 0, 20),
    ("M", 50, 70),
    ("L", 100, 120),
    ("S2", 120, 160),
    ("S3", 160, 200),
    ("ALL", 0, 200),
];

fn main() {
    let dir = std::env::args().nth(1).expect("run dir");
    let tdir = std::path::Path::new(&dir).join("telemetry");
    let reader = anima_telemetry::TelemetryReader::open(&tdir).unwrap();
    let idx = reader.chunk_index();

    let mut stims: Vec<(String, String, u64)> = Vec::new();
    let mut spikes: Vec<(u64, u32)> = Vec::new();
    let mut permanence: u64 = 0;
    let mut failures: u64 = 0;
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            match row.kind {
                3 => spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                5 => {
                    if let Ok(e) = row.envelope("e") {
                        if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } = e.payload {
                            stims.push((pattern_id, stage, e.t));
                        }
                    }
                }
                6 => {
                    if let Ok(e) = row.envelope("e") {
                        if let anima_telemetry::events::Payload::SynapseCreated { reason, .. } = e.payload {
                            if reason.trigger == "candidate-permanence" {
                                permanence += 1;
                            }
                        }
                    }
                }
                17 => {
                    if let Ok(e) = row.envelope("e") {
                        if let anima_telemetry::events::Payload::Failure { .. } = e.payload {
                            failures += 1;
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
    let max_rate = snaps
        .iter()
        .flat_map(|s| s.neurons.iter().filter(|n| n.class == "internal").filter_map(|n| n.rate_hz))
        .fold(0.0f32, f32::max);

    // Trial grid: trial k in S1/S2: antecedent at 5000+2000k, probe at
    // +1300; S3: probe at T_k + 2100 (amendment A-1).
    // Registered delivery-tick convention: telemetry timestamps sit at
    // env-scheduled + 1 (harness emits/delivers on the tick after the
    // env boundary). All windows use [scheduled + 1, scheduled + 501).
    let off: u64 = 1;
    let vec_of = |lo: u64, hi: u64| -> BTreeMap<u32, f64> {
        let mut v: BTreeMap<u32, f64> = BTreeMap::new();
        for &(t, n) in &spikes {
            if t >= lo && t < hi && n >= N_IN {
                *v.entry(n).or_insert(0.0) += 1.0;
            }
        }
        v
    };
    // A-1: S3 trials run at 2600 ms cadence (G1 = 1600, ITI = 0).
    let trial_base = |k: u64| -> u64 {
        if k >= S1_TRIALS + S2_TRIALS {
            5000 + TRIAL_MS * (S1_TRIALS + S2_TRIALS) + (k - S1_TRIALS - S2_TRIALS) * 2600
        } else {
            5000 + TRIAL_MS * k
        }
    };
    let trial_info = |k: u64| -> (Option<bool>, u64, u64) {
        // (antecedent_is_A: None if not found, antecedent_start, probe_start)
        let base = trial_base(k);
        let gap = if k >= S1_TRIALS + S2_TRIALS { 1600 } else { 800 };
        let probe = base + 500 + gap;
        let ant = stims
            .iter()
            .find(|(p, _, t)| *t == base + off && (p == "A" || p == "C"))
            .or_else(|| stims.iter().find(|(p, _, t)| *t == base && (p == "A" || p == "C")))
            .map(|(p, _, _)| p == "A");
        (ant, base, probe)
    };

    let mut seq: Vec<u32> = Vec::new();
    for k in 0..200u64 {
        let (ant, _, _) = trial_info(k);
        match ant {
            Some(true) => seq.push(0),
            Some(false) => seq.push(1),
            None => panic!("trial {k}: missing antecedent"),
        }
    }

    // Schedule audit.
    let lag1 = |v: &[u32]| -> f64 {
        let n = v.len() as f64;
        let mean = v.iter().sum::<u32>() as f64 / n;
        let num: f64 = (1..v.len()).map(|i| (v[i] as f64 - mean) * (v[i - 1] as f64 - mean)).sum();
        let den: f64 = v.iter().map(|&x| (x as f64 - mean).powi(2)).sum();
        if den == 0.0 { 0.0 } else { num / den }
    };
    println!("schedule audit: lag-1 autocorr (S1)={:.3} (all)={:.3}", lag1(&seq[..S1_TRIALS as usize]), lag1(&seq));
    for (label, start) in [("S1 1-40", 0u64), ("S1 41-80", 40), ("S1 81-120", 80), ("S2", 120), ("S3", 160)] {
        let v = &seq[start as usize..(start + 40) as usize];
        let a = v.iter().filter(|&&x| x == 0).count();
        println!("  balance {label}: A={a} C={}", 40 - a);
    }

    // Probe vectors + pairwise divergence per window.
    for (label, lo, hi) in WINDOWS {
        let ep_lo = |k: u64| trial_base(k) + off;
        let gap = if lo >= 160 { 1600 } else { 800 };
        let a_vecs: Vec<BTreeMap<u32, f64>> = (lo..hi)
            .filter(|&k| seq[k as usize] == 0)
            .map(|k| vec_of(ep_lo(k), ep_lo(k) + 500))
            .collect();
        let c_vecs: Vec<BTreeMap<u32, f64>> = (lo..hi)
            .filter(|&k| seq[k as usize] == 1)
            .map(|k| vec_of(ep_lo(k), ep_lo(k) + 500))
            .collect();
        let b_a: Vec<BTreeMap<u32, f64>> = (lo..hi)
            .filter(|&k| seq[k as usize] == 0)
            .map(|k| vec_of(ep_lo(k) + 500 + gap, ep_lo(k) + 500 + gap + 500))
            .collect();
        let b_c: Vec<BTreeMap<u32, f64>> = (lo..hi)
            .filter(|&k| seq[k as usize] == 1)
            .map(|k| vec_of(ep_lo(k) + 500 + gap, ep_lo(k) + 500 + gap + 500))
            .collect();
        let n = |v: &[BTreeMap<u32, f64>]| v.iter().map(|m| m.values().sum::<f64>() as u64).sum::<u64>() / v.len().max(1) as u64;
        let (ab, abl1) = pair_mean(&b_a, &a_vecs);
        let (bc, bcl1) = pair_mean(&b_c, &c_vecs);
        let (x, xl1) = pair_mean(&b_a, &b_c);
        let d = 1.0 - x;
        println!("[{label}] trials={} (A{} C{}) D=1-mean_cos(Ba,Bc)={d:.4} (raw {x:.4} L1 {xl1:.4}) | Ba-A={ab:.3} Bc-C={bc:.3} | spikes/epoch: ante {} probeA {} probeC {}",
            hi - lo, b_a.len(), b_c.len(), n(&a_vecs), n(&b_a), n(&b_c));
        let _ = (abl1, bcl1);
    }

    // A-C sanity in S2 (pair defined below at module level).
    let ac_vals = (120..160u64)
        .filter(|&k| seq[k as usize] == 0)
        .map(|k| vec_of(trial_base(k) + off, trial_base(k) + off + 500))
        .collect::<Vec<_>>();
    let cc_vals = (120..160u64)
        .filter(|&k| seq[k as usize] == 1)
        .map(|k| vec_of(trial_base(k) + off, trial_base(k) + off + 500))
        .collect::<Vec<_>>();
    let (ac, _acl1) = pair_mean(&ac_vals, &cc_vals);
    println!("A-C sanity (S2): mean cos = {:.4} (criterion < 0.60)", ac);
    println!("permanence events: {permanence}; failures: {failures}; snapshot max internal rate: {max_rate:.1} Hz");
}

fn pair_mean(x: &[BTreeMap<u32, f64>], y: &[BTreeMap<u32, f64>]) -> (f64, f64) {
    let mut raw = Vec::new();
    let mut l1 = Vec::new();
    for a in x {
        for b in y {
            raw.push(cos(a, b, false));
            l1.push(cos(a, b, true));
        }
    }
    let m = |c: &[f64]| c.iter().sum::<f64>() / c.len().max(1) as f64;
    (m(&raw), m(&l1))
}

fn cos(a: &BTreeMap<u32, f64>, b: &BTreeMap<u32, f64>, norm_l1: bool) -> f64 {
    let keys: BTreeSet<u32> = a.keys().chain(b.keys()).copied().collect();
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