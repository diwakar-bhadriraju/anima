//! E7 attribution instrument (docs/anima-e7-protocol.md §5 M4, frozen).
//! Measurement only, no mechanism access.
//!
//! Late-S1 cross-pattern cosine, computed exactly like the existing
//! cross_cosine instrument (per-presentation pairwise cosine, then mean)
//! in two modes:
//!   raw  — per-presentation internal spike-count vectors as fired;
//!   L1   — each per-presentation vector divided by its own L1 sum first.
//!
//! Separation that survives L1 normalization means the response PATTERN
//! differs (structure). Separation that collapses under L1 is carried by
//! total spike count / drive magnitude — the registered drive-threshold
//! effect (A′), which does NOT count as absence learning.
//!
//! Usage: e7_attribution <run-dir>

use std::collections::BTreeMap;

fn main() {
    let dir = std::env::args().nth(1).expect("run dir");
    let tdir = std::path::Path::new(&dir).join("telemetry");
    let reader = anima_telemetry::TelemetryReader::open(&tdir).unwrap();
    let idx = reader.chunk_index();

    let mut pres: Vec<(String, String, u64, u64)> = Vec::new(); // (pattern, stage, start, end)
    let mut spikes: Vec<(u64, u32)> = Vec::new();
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            match row.kind {
                5 => {
                    if let Some(env) = row.envelope("e").ok() {
                        if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } = env.payload {
                            pres.push((pattern_id, stage, env.t, env.t + 500));
                        }
                    }
                }
                3 => spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                _ => {}
            }
        }
    }
    pres.sort_by_key(|p| p.2);
    let s1: Vec<&(String, String, u64, u64)> = pres.iter().filter(|p| p.1 == "S1").collect();
    let late_start = s1[s1.len() / 2].2;
    let input_n = 24u32;

    // Per-presentation internal vectors: (neuron -> (raw, l1-normalized)).
    let vecs: Vec<(String, BTreeMap<u32, (f64, f64)>)> = s1
        .iter()
        .filter(|p| p.2 >= late_start)
        .map(|p| {
            let mut v: BTreeMap<u32, f64> = BTreeMap::new();
            for &(t, n) in &spikes {
                if t >= p.2 && t < p.3 && n >= input_n {
                    *v.entry(n).or_insert(0.0) += 1.0;
                }
            }
            let l1: f64 = v.values().sum();
            (p.0.clone(), v.into_iter().map(|(k, x)| (k, (x, if l1 > 0.0 { x / l1 } else { 0.0 }))).collect())
        })
        .collect();
    let mut pats: Vec<String> = vecs.iter().map(|(p, _)| p.clone()).collect();
    pats.sort();
    pats.dedup();
    if pats.len() < 2 {
        println!("(need >= 2 patterns in late S1)");
        return;
    }
    println!("== E7 attribution (late-S1 cross-pattern cosine, pairwise-mean) ==");
    for i in 0..pats.len() {
        for j in (i + 1)..pats.len() {
            let a = &pats[i];
            let b = &pats[j];
            let mut raw_cs = Vec::new();
            let mut norm_cs = Vec::new();
            for va in vecs.iter().filter(|(p, _)| p == a) {
                for vb in vecs.iter().filter(|(p, _)| p == b) {
                    raw_cs.push(cosine_maps(&va.1, &vb.1, false));
                    norm_cs.push(cosine_maps(&va.1, &vb.1, true));
                }
            }
            let mean = |c: &[f64]| c.iter().sum::<f64>() / c.len().max(1) as f64;
            let cr = mean(&raw_cs);
            let cn = mean(&norm_cs);
            if cr == 0.0 && raw_cs.is_empty() && norm_cs.is_empty() {
                println!("{a}-{b}: (no vector pairs)");
                continue;
            }
            println!(
                "{a}-{b}: raw {cr:.3}, L1-normalized {cn:.3}  (raw<0.60 && norm>=0.60 => A' drive-threshold, registered)"
            );
        }
    }
}

fn cosine_maps(a: &BTreeMap<u32, (f64, f64)>, b: &BTreeMap<u32, (f64, f64)>, mode_l1: bool) -> f64 {
    // Cosine over the union of neuron keys; missing coordinates are 0
    // (a silent neuron contributes nothing).
    let keys: std::collections::BTreeSet<u32> = a.keys().chain(b.keys()).copied().collect();
    let mut dot = 0.0f64;
    let mut na = 0.0f64;
    let mut nb = 0.0f64;
    for k in keys {
        let x = a.get(&k).map(|(r, l)| if mode_l1 { *l } else { *r }).unwrap_or(0.0);
        let y = b.get(&k).map(|(r, l)| if mode_l1 { *l } else { *r }).unwrap_or(0.0);
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