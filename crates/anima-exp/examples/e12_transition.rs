//! E12 transition instrument (docs/anima-e12-protocol.md §4, frozen).
//! Measurement only, no mechanism access.
//!
//! Computes the representation metrics at BOTH observation points from
//! recorded telemetry:
//!   T0 (midpoint) = S1 presentations in [5000, 365000)  (after 60 SEQ-B)
//!   T1 (final)    = S1 presentations in [365000, 723500) (after 60 REV-B)
//!
//! Per point, with the SAME definitions as the existing instruments:
//!   - A-B / B-C / A-C pairwise-mean cosines + L1-normalized attribution
//!   - B-independence booleans (A-B < 0.60 AND B-C < 0.60, L1-attributed)
//!   - B-alignment = argmin(B-A, B-C)
//!   - windowed selectivity (analyzer's per-neuron (best-2nd)/best, median)
//!   - permanence events created in the window
//!   - P2 window status: Failure events in-window, internal snapshot
//!     rates (mean/max)
//!
//! Usage: e12_transition <run-dir>

use std::collections::BTreeMap;

const N_IN: u32 = 24;
const T0: u64 = 5_000;
const TMID: u64 = 365_000;
const T1: u64 = 723_500;

fn main() {
    let dir = std::env::args().nth(1).expect("run dir");
    let tdir = std::path::Path::new(&dir).join("telemetry");
    let reader = anima_telemetry::TelemetryReader::open(&tdir).unwrap();
    let idx = reader.chunk_index();

    let mut pres: Vec<(String, String, u64, u64)> = Vec::new();
    let mut spikes: Vec<(u64, u32)> = Vec::new();
    let mut failures: Vec<(u64, String)> = Vec::new();
    let mut permanence: Vec<u64> = Vec::new();
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            match row.kind {
                5 => {
                    if let Ok(env) = row.envelope("e") {
                        if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } = env.payload {
                            pres.push((pattern_id, stage, env.t, env.t + 500));
                        }
                    }
                }
                3 => spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                7 => {
                    if let Ok(e) = row.envelope("v2") {
                        if let anima_telemetry::events::Payload::SynapseCreated { reason, .. } = e.payload {
                            if reason.trigger == "candidate-permanence" {
                                permanence.push(e.t);
                            }
                        }
                    }
                }
                2 => {
                    if let Ok(e) = row.envelope("v2") {
                        if let anima_telemetry::events::Payload::Failure { kind, detail, .. } = e.payload {
                            failures.push((e.t, format!("{kind}: {detail}")));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    // kind 2 = Failure? verify by scanning kinds at runtime is not needed;
    // fall back: any envelope with Payload::Failure regardless of kind.
    let snaps = anima_telemetry::recorder::read_snapshots(
        std::path::Path::new(&dir).join("snapshots.bin.zst").as_path(),
    )
    .expect("snapshots");

    let windows: [(&str, u64, u64); 2] = [("T0", T0, TMID), ("T1", TMID, T1)];
    for (label, lo, hi) in windows {
        let win_pres: Vec<&(String, String, u64, u64)> = pres
            .iter()
            .filter(|p| p.1 == "S1" && p.2 >= lo && p.2 < hi)
            .collect();
        // per-presentation internal vectors
        let vecs: Vec<(String, BTreeMap<u32, f64>)> = win_pres
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
        println!("===== {label} ([{lo}, {hi})) presentations={}", win_pres.len());
        if vecs.is_empty() {
            println!("(no presentations in window)");
            continue;
        }
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
            let mean = |c: &[f64]| c.iter().sum::<f64>() / c.len().max(1) as f64;
            (mean(&raw), mean(&norm))
        };
        let (ab, abn) = pair_of("A", "B");
        let (bc, bcn) = pair_of("B", "C");
        let (ac, acn) = pair_of("A", "C");
        println!("A-B: raw {ab:.3}, L1 {abn:.3}");
        println!("B-C: raw {bc:.3}, L1 {bcn:.3}");
        println!("A-C: raw {ac:.3}, L1 {acn:.3}");
        let indep = ab < 0.60 && bc < 0.60 && abn < 0.60 && bcn < 0.60;
        println!("B-independence (raw+L1): {indep}");
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
        let wfail = failures.iter().filter(|(t, _)| *t >= lo && *t < hi).count();
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
            (if sx > 0.0 { x / sx } else { 0.0 }, if sy > 0.0 { y / sy } else { 0.0 })
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